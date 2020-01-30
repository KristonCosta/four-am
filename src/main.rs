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
pub mod ai;
pub mod turn_system;

fn main() {
    run(
        Settings {
            size: quicksilver::geom::Vector::new(800.0, 600.0).into(),
            title: "Whoa",
            vsync: false,
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
    let position_x = rng.gen_range(10, 70);
    let position_y = rng.gen_range(10, 30);
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
        .with(component::Monster)
        .build();
}

async fn app(window: Window, mut gfx: Graphics, mut events: EventStream) -> Result<()> {
    let mut ecs = World::new();
    register_components(&mut ecs);
    register_resources(&mut ecs);
    let (map, position) = RoomMapBuilder::build((80, 42), 10);
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
    let map_region = Rect::new((0, 0).into(), (80, 42).into());
    let mut gs = GameState {
        ecs,
        runstate: RunState::Running,
        tile_ctx,
        map_region
    };

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
