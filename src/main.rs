#[macro_use]
extern crate specs_derive;
use crate::camera::{get_screen_bounds, render_camera};
use crate::component::{register_components, Name, Position};
use crate::geom::{Point, Rect, Vector};
use crate::glyph::Glyph;
use crate::grid::Grid;
use crate::map::{
    create_room, Map, MapBuilder, RoomMapBuilder, SimpleMapBuilder, TilePos, TileType,
};
use crate::tileset::Tileset;
use crate::ui::{draw_box, print, print_glyphs};
use pathfinding::directed::dijkstra::dijkstra_all;
use pathfinding::prelude::dijkstra;
use quicksilver::graphics::Color;
use quicksilver::lifecycle::{ElementState, Key};
use quicksilver::{
    graphics::Graphics,
    lifecycle::{run, Event, EventStream, Settings, Window},
    Result,
};
use rand::Rng;
use specs::prelude::*;
use specs::{Builder, World, WorldExt};
use std::cmp::{max, min};
use std::panic;
use std::slice::Iter;
use std::time::{Duration, Instant};
use crate::gamestate::{handle_event, RunState, GameState};

pub mod gamestate;
pub mod camera;
pub mod common;
pub mod component;
pub mod font;
pub mod glyph;
pub mod grid;
pub mod map;
pub mod tileset;
pub mod ui;
pub mod fov;
pub mod geom;
pub mod color;
pub mod error;

fn main() {
    run(
        Settings {
            size: quicksilver::geom::Vector::new(800.0, 600.0).into(),
            title: "Whoa",
            ..Settings::default()
        },
        app,
    );
}




pub struct GameLog {
    max_length: usize,
    lines: Vec<Vec<Glyph>>,
}

impl GameLog {
    pub fn with_length(length: usize) -> Self {
        GameLog {
            max_length: length,
            lines: Vec::with_capacity(length + 1),
        }
    }

    pub fn push(&mut self, message: &str, fg: Option<Color>, bg: Option<Color>) {
        let mut glyphs = vec![];
        for ch in message.chars() {
            glyphs.push(Glyph::from(ch, fg, bg));
        }
        self.push_glyphs(glyphs);
    }

    pub fn push_glyphs(&mut self, glyphs: Vec<Glyph>) {
        self.lines.push(glyphs);
        if self.lines.len() > self.max_length {
            self.lines.rotate_left(1);
            self.lines.pop();
        }
    }

    pub fn iter(&self) -> Iter<Vec<Glyph>> {
        self.lines.iter()
    }
}

pub struct MouseState {
    pub x: i32,
    pub y: i32,
}

pub struct Focus {
    pub x: i32,
    pub y: i32,
}

pub struct TileContext {
    grid: Grid,
    tileset: Tileset,
}

impl TileContext {
    pub fn draw(&self, gfx: &mut Graphics, glyph: &Glyph, pos: impl Into<Point>) {
        let rect = self.grid.rect(pos);
        self.tileset.draw(gfx, &glyph, rect);
    }
}

pub mod ai {
    use crate::component::{
        ActiveTurn,
        Monster,
        TurnState,
        Name,
        Position,
    };
    use specs::prelude::*;
    use std::cmp::{min, max};
    use quicksilver::graphics::Color;
    use rand::Rng;

    pub struct MonsterAi;

    impl<'a> System<'a> for MonsterAi {
        type SystemData = (
            ReadExpect<'a, crate::map::Map>,
            WriteExpect<'a, crate::GameLog>,
            ReadStorage<'a, Monster>,
            WriteStorage<'a, ActiveTurn>,
            WriteStorage<'a, Position>,
            ReadStorage<'a, Name>,
        );

        fn run(&mut self, data: Self::SystemData) {
            let (mut map,
                 mut log,
                 monsters,
                 mut turns,
                 mut positions,
                 names,) = data;

            for (monster, mut turn, mut pos, name) in (&monsters, &mut turns, &mut positions, &names).join() {
                let mut rng = rand::thread_rng();
                let delta_x = rng.gen_range(-1, 2);
                let delta_y = rng.gen_range(-1, 2);
                let desired_x = min(map.size.0, max(0, pos.x + delta_x));
                let desired_y = min(map.size.1, max(0, pos.y + delta_y));

                if map.blocked[map.coord_to_index(desired_x, desired_y)] {
                    log.push(&format!("Da {} hit a wall!", name.name), Some(Color::RED), None);
                } else {
                    pos.x = desired_x;
                    pos.y = desired_y;
                    turn.state = TurnState::DONE;

                }
            }
        }
    }
}

pub mod turn_system {

    use crate::component::{
        ActiveTurn,
        Priority,
        TurnState,
    };
    use specs::prelude::*;

    pub struct PendingMoves {
        list: Vec<Entity>
    }
    
    impl PendingMoves {
        pub fn new() -> Self {
            Self {
                list: vec![]
            }
        }
    }

    pub struct TurnSystem;

    impl<'a> System<'a> for TurnSystem {
        type SystemData = (
            Entities<'a>,
            WriteExpect<'a, crate::GameLog>,
            WriteExpect<'a, PendingMoves>,
            ReadStorage<'a, Priority>,
            WriteStorage<'a, ActiveTurn>,
        );

        fn run(&mut self, data: Self::SystemData) {
            let (entities,
                mut log,
                mut pending_moves,
                priorities,
                mut active) = data;

            let mut finished = vec![];
            let mut active_entity = false;
            for (entity, mut turn) in (&entities, &mut active).join() {
                active_entity = true;
                match turn.state {
                    TurnState::DONE => finished.push(entity.clone()),
                    _ => (),
                }
            }

            if finished.is_empty() && active_entity {
                return
            }

            for entity in finished {
                active.remove(entity);
            }

            if pending_moves.list.is_empty() {
                let mut priority_tuple = vec![];
                for (entity, priority) in (&entities, &priorities).join() {
                    priority_tuple.push((priority.value, entity));
                }
                priority_tuple.sort_by_key(|k| k.0);
                pending_moves.list = priority_tuple.iter().map(|v| v.1).collect();
            }

            let next_turn = pending_moves.list.pop().expect("No entites could take a turn");
            active.insert(next_turn, ActiveTurn{
                state: TurnState::PENDING
            });
        }
    }

}

async fn app(window: Window, mut gfx: Graphics, mut events: EventStream) -> Result<()> {
    let mut ecs = World::new();
    register_components(&mut ecs);

    let (map, position) = RoomMapBuilder::build((80, 42), 10);
    ecs.insert(map);

    ecs.create_entity()
        .with(component::Position {
            x: position.0,
            y: position.1 + 15,
        })
        .with(component::Renderable {
            glyph: Glyph {
                ch: 'C',
                foreground: Some(color::TAN),
                background: None,
                render_order: 3,
            },
        })
        .with(component::Name {
            name: "Giant Centipede".to_string(),
        })
        .with(component::Priority {
            value: 1,
        })
        .with(component::Monster)
        .build();

    ecs.create_entity()
        .with(component::Position {
            x: position.0,
            y: position.1,
        })
        .with(component::Renderable {
            glyph: Glyph {
                ch: '@',
                foreground: Some(Color::YELLOW),
                background: None,
                render_order: 3,
            },
        })
        .with(component::Player {})
        .with(component::Name {
            name: "Player".to_string(),
        })
        .with(component::FieldOfView {
            visible_tiles: vec![],
            range: 8,
        })
        .with(component::Priority {
            value: 1,
        })
        .build();

    let tileset = tileset::Tileset::from_font(&gfx, "Px437_Wyse700b-2y.ttf", 16.0 / 8.0)
        .await
        .expect("oof");
    let grid = grid::Grid::from_screen_size((80, 50), (800, 600));

    let tile_ctx = TileContext { tileset, grid };

    let mut log = GameLog::with_length(5);
    log.push("Hello, world!", Some(Color::GREEN), None);

    let mouse = MouseState { x: 0, y: 0 };

    let focus = Focus {
        x: position.0,
        y: position.1,
    };

    let turn = turn_system::PendingMoves::new();
    ecs.insert(turn);
    ecs.insert(log);
    ecs.insert(mouse);
    ecs.insert(focus);
    let map_region = Rect::new((0, 0).into(), (80, 42).into());
    let mut gs = GameState {
        ecs,
        runstate: RunState::Running,
        tile_ctx,
        map_region
    };
    let mut duration = Duration::new(0, 0);

    loop {
        let mut ms = ai::MonsterAi;
        let mut ts = turn_system::TurnSystem;
        ms.run_now(&gs.ecs);
        ts.run_now(&gs.ecs);
        gs.ecs.maintain();
        while let Some(event) = events.next_event().await {
            handle_event(&window, &mut gs, event);
        }
        gfx.clear(Color::BLACK);
        draw_box(
            &mut gfx,
            &gs.tile_ctx,
            Rect::new((0, 43).into(), (79, 6).into()),
            None,
            None,
        );
        {
            let log = gs.ecs.fetch::<GameLog>();
            for (index, glyphs) in log.iter().enumerate() {
                print_glyphs(&mut gfx, &gs.tile_ctx, &glyphs, (1, (44 + index) as i32));
            }
        }

        render_camera(&mut gs, &mut gfx);

        gfx.present(&window)?;
    }
}
