#[macro_use]
extern crate specs_derive;
use quicksilver::{
    geom::{Vector, Rectangle},
    graphics::{Color, Graphics},
    lifecycle::{run, EventStream, Settings, Window, Event},
    Result,
};
use specs::{World, WorldExt, Builder};
use quicksilver::lifecycle::{Key, ElementState};
use std::cmp::{min, max};
use crate::glyph::Glyph;
use specs::prelude::*;
use crate::grid::Grid;
use crate::tileset::Tileset;
use crate::ui::{draw_box, print, print_glyphs};
use std::slice::Iter;

pub mod font;
pub mod tileset;
pub mod glyph;
pub mod grid;
pub mod ui;
pub mod component;

fn main() {
    run(
        Settings {
            size: Vector::new(800.0, 600.0).into(),
            title: "Whoa",
            ..Settings::default()
        },
        app,
    );
}

pub struct State {
    ecs: World
}

pub fn try_move_player(delta_x: i32, delta_y: i32, ecs: &mut World) {
    let mut positions = ecs.write_storage::<component::Position>();
    let mut players = ecs.write_storage::<component::Player>();

    for (_player, pos) in (&mut players, &mut positions).join() {
        pos.x = min(79, max(0, pos.x + delta_x));
        pos.y = min(49, max(0, pos.y + delta_y));
    }
}

pub fn handle_key(gs: &mut State, key: Key, state: ElementState) {
    if state == ElementState::Pressed {
        {
            let mut log = gs.ecs.write_resource::<GameLog>();
            log.push(&format!("Test message: {:?}", key), Some(Color::RED), None);
        }
        match key {
            Key::W => try_move_player(0, -1, &mut gs.ecs),
            Key::A => try_move_player(-1, 0, &mut gs.ecs),
            Key::S => try_move_player(0, 1, &mut gs.ecs),
            Key::D => try_move_player(1, 0, &mut gs.ecs),
            _ => {}
        }
    }
}

pub fn handle_click(gs: &mut State, raw: (i32, i32), pos: (i32, i32)) {
    {
        let mut log = gs.ecs.write_resource::<GameLog>();
        log.push(&format!("Clicked on tile: {} {}", pos.0, pos.1), Some(Color::RED), None);
        log.push(&format!("Raw on tile: {} {}", raw.0, raw.1), Some(Color::GREEN), None);
    }

}


pub fn make_char(state: &mut State, ch: char, pos: (i32, i32)) {
    state.ecs
        .create_entity()
        .with(component::Position { x: pos.0, y: pos.1 })
        .with(component::Renderable {
            glyph: Glyph {
                ch: ch,
                foreground: Some(Color::YELLOW),
                background: None,
            }
        })
        .build();
}

pub struct GameLog {
    max_length: usize,
    lines: Vec<Vec<Glyph>>
}

impl GameLog {
    pub fn with_length(length: usize) -> Self {
        GameLog {
            max_length: length,
            lines: Vec::with_capacity(length + 1)
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

pub struct TileContext {
    grid: Grid,
    tileset: Tileset,
}

impl TileContext {
    pub fn draw(&self, gfx: &mut Graphics, glyph: &Glyph, pos: (f32, f32)) {
        let rect = self.grid.rect(pos.0, pos.1);
        self.tileset.draw(gfx, &glyph, rect);
    }
}

async fn app(window: Window, mut gfx: Graphics, mut events: EventStream) -> Result<()> {
    let mut gs = State {
        ecs: World::new(),
    };
    gs.ecs.register::<component::Position>();
    gs.ecs.register::<component::Renderable>();
    gs.ecs.register::<component::Player>();
    gs.ecs
        .create_entity()
        .with(component::Position { x: 20, y: 20 })
        .with(component::Renderable {
            glyph: Glyph {
                ch: '@',
                foreground: Some(Color::YELLOW),
                background: None,
            }
        })
        .with(component::Player{})
        .build();

    let tileset = tileset::Tileset::from_font(&gfx, "Px437_Wyse700b-2y.ttf", 16.0/8.0).await?;
    let grid = grid::Grid::from_screen_size((80.0, 50.0), (800.0, 600.0));

    let tile_ctx = TileContext {
        tileset,
        grid
    };

    let mut log = GameLog::with_length(5);
    log.push("Hello, world!", Some(Color::GREEN), None);

    gs.ecs.insert(log);

    let mouse = MouseState {
        x: 0,
        y: 0,
    };

    gs.ecs.insert(mouse);

    loop {
        while let Some(event) = events.next_event().await {
            match event {
                Event::KeyboardInput {
                    key,
                    state
                } => {
                    handle_key(&mut gs, key, state)
                },
                Event::MouseMoved {
                    pointer,
                    position
                } => {
                    let scale = window.scale_factor();
                    let mut mouse = gs.ecs.write_resource::<MouseState>();
                    mouse.x = position.x as i32 / scale as i32;
                    mouse.y = position.y as i32 / scale as i32;
                },
                Event::MouseInput {
                    pointer,
                    state,
                    button,
                } => {
                    if state == ElementState::Pressed {
                        let mut pos = (0, 0);
                        let mut raw = (0, 0);
                        {
                            let mut mouse = gs.ecs.fetch::<MouseState>();
                            println!("{:?} {:?}", mouse.x, mouse.y);
                            raw = (mouse.x, mouse.y);
                            pos = tile_ctx.grid.point_to_grid(mouse.x as f32, mouse.y as f32);
                        }

                        handle_click(&mut gs, raw, pos);
                    }
                },
                _ => (),
            }
        }
        gfx.clear(Color::BLACK);

        let positions = gs.ecs.read_storage::<component::Position>();
        let renderables = gs.ecs.read_storage::<component::Renderable>();

        for (pos, render) in (&positions, &renderables).join() {
            tile_ctx.draw(&mut gfx, &render.glyph, (pos.x as f32, pos.y as f32));
        }
        draw_box(&mut gfx, &tile_ctx, Rectangle::new((0.0, 43.0), (79.0, 6.0)), None, None);
        draw_box(&mut gfx, &tile_ctx, Rectangle::new((10.0, 10.0), (6.0, 6.0)), None, None);
        draw_box(&mut gfx, &tile_ctx, Rectangle::new((20.0, 30.0), (10.0, 2.0)), None, None);
        let log = gs.ecs.fetch::<GameLog>();
        for (index, glyphs) in log.iter().enumerate() {
            print_glyphs(&mut gfx, &tile_ctx, &glyphs, (1, (44 + index) as i32));
        }


        gfx.present(&window)?;
    }
}