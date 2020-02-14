#[macro_use]
extern crate specs_derive;
use crate::camera::render_camera;
use crate::component::{register_components, Position, Killed};
use crate::geom::{Point, Rect, Vector};
use crate::glyph::Glyph;
use crate::grid::Grid;
use crate::map::{
    MapBuilder, RoomMapBuilder
};
use crate::tileset::Tileset;
use crate::ui::{draw_box, print_glyphs};
use quicksilver::graphics::Color;
use quicksilver::{
    graphics::Graphics,
    lifecycle::{run, EventStream, Settings, Window},
    Result,
};
use rand::Rng;
use specs::prelude::*;
use specs::{Builder, World, WorldExt};
use std::slice::Iter;
use instant::Instant;
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
pub mod ai;
pub mod turn_system;

fn main() {
    run(
        Settings {
            size: quicksilver::geom::Vector::new(800.0, 600.0).into(),
            title: "Whoa",
            vsync: true,
            ..Settings::default()
        },
        app,
    );
}

type FP = f32;
const MS_PER_UPDATE: FP = 0.1;

#[derive(Debug)]
pub struct TimeStep {
    last_time:   Instant,
    delta_time:  FP,
    frame_count: u32,
    frame_time:  FP,
}

impl TimeStep {
    // https://gitlab.com/flukejones/diir-doom/blob/master/game/src/main.rs
    // Grabbed this from here
    pub fn new() -> TimeStep {
        TimeStep {
            last_time:   Instant::now(),
            delta_time:  0.0,
            frame_count: 0,
            frame_time:  0.0,
        }
    }

    pub fn delta(&mut self) -> FP {
        let current_time = Instant::now();
        let delta = current_time.duration_since(self.last_time).as_micros()
            as FP
            * 0.001;
        self.last_time = current_time;
        self.delta_time = delta;
        delta
    }

    // provides the framerate in FPS
    pub fn frame_rate(&mut self) -> Option<u32> {
        self.frame_count += 1;
        self.frame_time += self.delta_time;
        let tmp;
        // per second
        if self.frame_time >= 1000.0 {
            tmp = self.frame_count;
            self.frame_count = 0;
            self.frame_time = 0.0;
            return Some(tmp);
        }
        None
    }
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

fn register_resources(ecs: &mut World) {
    let mut log = GameLog::with_length(5);
    log.push("Hello, world!", Some(Color::GREEN), None);

    let mouse = MouseState { x: 0, y: 0 };

    let turn = turn_system::PendingMoves::new();
    ecs.insert(turn);
    ecs.insert(log);
    ecs.insert(mouse);

}

fn generate_centipede(ecs: &mut World, i :u32) {
    let mut rng = rand::thread_rng();
    let position_x = rng.gen_range(10, 50);
    let position_y = rng.gen_range(10, 20);
    ecs.create_entity()
        .with(component::Position {
            x: position_x,
            y: position_y,
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
            name: format!("Giant Centipede {}", i),
        })
        .with(component::Priority {
            value: 1,
        })
        .with(component::TileBlocker)
        .with(component::Monster)
        .build();
}

pub fn generate_blood(ecs: &mut World, pos: Vector) {
    ecs.create_entity()
        .with(component::Position {
            x: pos.x,
            y: pos.y,
        })
        .with(component::Renderable {
            glyph: Glyph {
                ch: '%',
                foreground: Some(color::RED),
                background: None,
                render_order: 100
            }
        })
        .build();
}

pub fn sweep(ecs: &mut World) {
    let mut killed = vec![];
    {
        let combat_stats = ecs.read_storage::<Killed>();
        let positions = ecs.read_storage::<Position>();
        let entities = ecs.entities();
        for (entity, _stats, position) in (&entities, &combat_stats, &positions).join() {
            killed.push((entity, (position.x, position.y)));
        }
    }

    for (entity, position) in killed {
        ecs.delete_entity(entity).expect("Failed to delete entity");
        generate_blood(ecs, (position.0, position.1).into());
    }
}

async fn app(window: Window, mut gfx: Graphics, mut events: EventStream) -> Result<()> {
    println!("Starting.");
    let x = 80;
    let y = 50;
    let mut ecs = World::new();
    register_components(&mut ecs);
    register_resources(&mut ecs);
    let (map, position) = RoomMapBuilder::build((60, 30), 10);
    ecs.insert(map);
    let focus = Focus {
        x: position.0,
        y: position.1,
    };
    ecs.insert(focus);

    for i in 1..100 {
        generate_centipede(&mut ecs, i);
    }

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
        .with(component::Priority{
            value: 100
        })
        .with(component::TileBlocker)
        .build();

    let tileset = tileset::Tileset::from_font(&gfx, "Px437_Wyse700b-2y.ttf", 16.0 / 8.0)
        .await
        .expect("oof");
    let grid = grid::Grid::from_screen_size((x, y), (800, 600));

    let tile_ctx = TileContext { tileset, grid };
    let map_region = Rect::new((0, 0).into(), (x, y - 8).into());
    let mut gs = GameState {
        ecs,
        runstate: RunState::Running,
        tile_ctx,
        map_region
    };
    let mut timestep = TimeStep::new();
    let mut lag: f32 = 0.0;
    let mut turns = 0;
    loop {
        while let Some(event) = events.next_event().await {
            handle_event(&window, &mut gs, event);
        }
        let mut ms = ai::MonsterAi;
        let mut ts = turn_system::TurnSystem;
        let mut indexer = map::MapIndexer;
        lag += timestep.delta();
        while lag >= MS_PER_UPDATE {
            turns += 1;
            indexer.run_now(&gs.ecs);
            ms.run_now(&gs.ecs);
            ts.run_now(&gs.ecs);
            sweep(&mut gs.ecs);
            gs.ecs.maintain();
            lag -= MS_PER_UPDATE;

        }
        if let Some(fps) = timestep.frame_rate() {
            println!("FPS {}", fps);
            println!("TPS {}", turns);
            turns = 0;
        }
        gfx.clear(Color::BLACK);
        draw_box(
            &mut gfx,
            &gs.tile_ctx,
            Rect::new((0, y - 7).into(), (x - 1, 6).into()),
            None,
            None,
        );
        {
            let log = gs.ecs.fetch::<GameLog>();
            for (index, glyphs) in log.iter().enumerate() {
                print_glyphs(&mut gfx, &gs.tile_ctx, &glyphs, (1, (y - 6 + index as i32) as i32));
            }
        }

        render_camera(&mut gs, &mut gfx);

        gfx.present(&window)?;
    }
}
