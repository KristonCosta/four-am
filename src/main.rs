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

pub mod component {
    use specs::prelude::*;
    use quicksilver::graphics::Color;
    use crate::glyph;

    #[derive(Component)]
    pub struct Position {
        pub x: i32,
        pub y: i32,
    }

    #[derive(Component)]
    pub struct Renderable {
        pub glyph: glyph::Glyph,
    }

    #[derive(Component)]
    pub struct Player;

}

fn main() {
    run(
        Settings {
            size: Vector::new(800.0, 600.0).into(),
            title: "RGB Triangle Example",
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
        match key {
            Key::W => try_move_player(0, -1, &mut gs.ecs),
            Key::A => try_move_player(-1, 0, &mut gs.ecs),
            Key::S => try_move_player(0, 1, &mut gs.ecs),
            Key::D => try_move_player(1, 0, &mut gs.ecs),
            _ => {}
        }
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

pub mod ui {
    use quicksilver::geom::Rectangle;
    use quicksilver::graphics::{Color, Graphics};
    use crate::grid::Grid;
    use crate::TileContext;
    use crate::glyph::Glyph;


    pub fn draw_box(gfx: &mut Graphics,
                    ctx: &TileContext,
                    rect: Rectangle,
                    fg: Option<Color>,
                    bg: Option<Color>) {
        let top_left = Glyph::from('╔', fg, bg);
        let top_right = Glyph::from('╗', fg, bg);
        let bottom_left = Glyph::from('╚', fg, bg);
        let bottom_right = Glyph::from('╝', fg, bg);
        let vertical = Glyph::from('║', fg, bg);
        let horizontal = Glyph::from('═', fg, bg);

        ctx.draw(gfx, &top_left, (rect.pos.x, rect.pos.y));
        ctx.draw(gfx, &top_right, (rect.pos.x + rect.size.x, rect.pos.y));
        ctx.draw(gfx, &bottom_left, (rect.pos.x, rect.pos.y + rect.size.y));
        ctx.draw(gfx, &bottom_right, (rect.pos.x + rect.size.x, rect.pos.y + rect.size.y));
        let (x_start, x_end) = (rect.pos.x as i32, (rect.pos.x + rect.size.x) as i32);
        let (y_start, y_end) = (rect.pos.y as i32, (rect.pos.y + rect.size.y) as i32);
        for x in (x_start + 1)..x_end {
            ctx.draw(gfx, &horizontal, (x as f32, y_start as f32));
            ctx.draw(gfx, &horizontal, (x as f32, y_end as f32));
        }
        for y in (y_start + 1)..y_end {
            ctx.draw(gfx, &vertical, (x_start as f32, y as f32));
            ctx.draw(gfx, &vertical, (x_end as f32, y as f32));
        }
    }

    pub fn print(gfx: &mut Graphics,
                 ctx: &TileContext,
                 text: &str,
                 pos: (i32, i32),
                 fg: Option<Color>,
                 bg: Option<Color>) {
        for (index, ch) in text.chars().enumerate() {
            let ch = Glyph::from(ch, fg, bg);
            ctx.draw(gfx, &ch, (pos.0 as f32 + index as f32, pos.1 as f32));
        }
    }

    pub fn print_glyphs(gfx: &mut Graphics,
                        ctx: &TileContext,
                        glyphs: &Vec<Glyph>,
                        pos: (i32, i32)) {
        for (index, glyph) in glyphs.iter().enumerate() {
            ctx.draw(gfx, &glyph, (pos.0 as f32 + index as f32, pos.1 as f32));
        }
    }

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
    loop {
        while let Some(event) = events.next_event().await {
            match event {
                Event::KeyboardInput {
                    key,
                    state
                } => {
                    if (state == ElementState::Pressed) {
                        log.push(&format!("Test message: {:?}", key), Some(Color::RED), None);
                    }


                    handle_key(&mut gs, key, state)
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

        for (index, glyphs) in log.iter().enumerate() {
            print_glyphs(&mut gfx, &tile_ctx, &glyphs, (1, (44 + index) as i32));
        }


        gfx.present(&window)?;
    }
}