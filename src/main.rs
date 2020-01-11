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
    match key {
        Key::W => try_move_player(0, -1, &mut gs.ecs),
        Key::A => try_move_player(-1, 0, &mut gs.ecs),
        Key::S => try_move_player(0, 1, &mut gs.ecs),
        Key::D => try_move_player(1, 0, &mut gs.ecs),
        _ => {}
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


async fn app(window: Window, mut gfx: Graphics, mut events: EventStream) -> Result<()> {

    let mut gs = State {
        ecs: World::new(),
    };

    gs.ecs.register::<component::Position>();
    gs.ecs.register::<component::Renderable>();
    gs.ecs.register::<component::Player>();
    make_char(&mut gs, '╔', (10, 10));
    make_char(&mut gs, '╗', (12, 10));
    make_char(&mut gs, '╦', (11, 10));
    make_char(&mut gs, '║', (10, 11));
    make_char(&mut gs, '╝', (12, 11));
    make_char(&mut gs, '╚', (11, 11));

    let tileset = tileset::Tileset::from_font(&gfx, "Px437_Wyse700b-2y.ttf", 16.0/8.0).await?;

    let grid = grid::Grid::from_tile_size((10.0, 20.0));

    loop {
        while let Some(event) = events.next_event().await {
            match event {
                Event::KeyboardInput {
                    key,
                    state
                } => handle_key(&mut gs, key, state),
                _ => (),
            }
        }
        gfx.clear(Color::BLACK);

        let positions = gs.ecs.read_storage::<component::Position>();
        let renderables = gs.ecs.read_storage::<component::Renderable>();

        for (pos, render) in (&positions, &renderables).join() {
            tileset.draw(&mut gfx, &render.glyph, grid.rect(pos.x, pos.y));
        }

        gfx.present(&window)?;
    }
}