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
use crate::map::{SimpleMapBuilder, MapBuilder, Map, TileType};
use std::panic;
pub mod font;
pub mod tileset;
pub mod glyph;
pub mod grid;
pub mod ui;
pub mod component;

fn main() {
    panic::set_hook(Box::new(console_error_panic_hook::hook));
    run(
        Settings {
            size: Vector::new(800.0, 600.0).into(),
            title: "Whoa",
            ..Settings::default()
        },
        app,
    );
}


pub mod map {
    use specs::Entity;
    use quicksilver::geom::{Rectangle, Shape};
    use std::cmp::{min, max};

    pub const MAPWIDTH : usize = 80;
    pub const MAPHEIGHT : usize = 43;
    pub const MAPCOUNT : usize = MAPHEIGHT * MAPWIDTH;

    #[derive(Clone)]
    pub enum TileType {
        Wall,
        Floor,
    }

    #[derive(Default)]
    pub struct Map {
        pub tiles: Vec<TileType>,
        pub size: (i32, i32),
        pub revealed_tiles: Vec<bool>,
        pub visible_tiles: Vec<bool>,
        pub blocked: Vec<bool>,
        pub depth: i32,
        pub tile_content: Vec<Vec<Entity>>
    }

    // Base taken from https://bfnightly.bracketproductions.com/rustbook/chapter_23.html
    impl Map {
        pub fn new(depth: i32) -> Self {
            Map {
                tiles: vec![TileType::Wall; MAPCOUNT],
                size: (MAPWIDTH as i32, MAPHEIGHT as i32),
                revealed_tiles: vec![false; MAPCOUNT],
                visible_tiles: vec![false; MAPCOUNT],
                blocked: vec![true; MAPCOUNT],
                depth,
                tile_content: vec![Vec::new(); MAPCOUNT],
            }
        }
        pub fn coord_to_index(&self, x: i32, y: i32) -> usize {
            (y as usize * self.size.0 as usize) + x as usize
        }
    }

    pub(crate) trait MapBuilder {
        fn build(depth: i32) -> (Map, (i32, i32)) ;
    }

    pub struct SimpleMapBuilder {}

    impl MapBuilder for SimpleMapBuilder {
        fn build(depth: i32) -> (Map, (i32, i32)) {
            let mut map = Map::new(depth);
            let start_pos = SimpleMapBuilder::rooms_and_corridors(&mut map);
            (map, start_pos)
        }

    }
    impl SimpleMapBuilder {
        pub fn rooms_and_corridors(map: &mut Map) -> (i32, i32) {
            // let mut rng = rand::thread_rng();

            const MAX_ROOMS : i32 = 30;
            const MIN_SIZE : i32 = 6;
            const MAX_SIZE : i32 = 10;
            let mut rooms: Vec<Rectangle> = vec![];
            for i in 0..MAX_ROOMS {
                let w = 10; // rng.gen_range(MIN_SIZE, MAX_SIZE);
                let h = 10; // rng.gen_range(MIN_SIZE, MAX_SIZE);
                let x = 10; // rng.gen_range(1, map.size.0 - w - 1) - 1;
                let y = 10;// rng.gen_range(1, map.size.1 - h - 1) - 1;
                let new_room = Rectangle::new((x, y), (w, h));
                create_room(map, &new_room);
              /*  if !rooms.is_empty() {
                    let center = new_room.center();
                    let prev = rooms[rooms.len() - 1].center();
                    if rng.gen_range(0, 2) == 1 {
                        dig_horizontal(map, prev.x as i32, center.x as i32, prev.y as i32);
                        dig_vertical(map, prev.y as i32, center.y as i32, center.x as i32);
                    } else {
                        dig_horizontal(map, prev.x as i32, center.x as i32, center.y as i32);
                        dig_vertical(map, prev.y as i32, center.y as i32, prev.x as i32);
                    }
                }*/
                rooms.push(new_room);
            }
            let center = rooms[0].center();
            (center.x as i32, center.y as i32)
        }
    }

    pub fn create_room(map: &mut Map, room: &Rectangle) {
        let (x_start, y_start) = (room.pos.x as i32, room.pos.y as i32);
        let (x_end, y_end) = ((room.pos.x + room.size.x) as i32, (room.pos.y + room.size.y) as i32);
        for x in x_start..=x_end {
            for y in y_start..=y_end {
                let index = map.coord_to_index(x, y);
                map.tiles[index] = TileType::Floor;
                map.blocked[index] = false;
            }
        }
    }

    pub fn dig_horizontal(map: &mut Map, start: i32, end: i32, y: i32) {
        for x in min(start, end)..=max(start, end) {
            let index = map.coord_to_index(x, y);
            map.tiles[index] = TileType::Floor;
            map.blocked[index] = false;
        }
    }

    pub fn dig_vertical(map: &mut Map, start: i32, end: i32, x: i32) {
        for y in min(start, end)..=max(start, end) {
            let index = map.coord_to_index(x, y);
            map.tiles[index] = TileType::Floor;
            map.blocked[index] = false;
        }
    }
}


pub struct State {
    ecs: World
}

pub fn try_move_player(delta_x: i32, delta_y: i32, ecs: &mut World) {
    let mut positions = ecs.write_storage::<component::Position>();
    let mut players = ecs.write_storage::<component::Player>();

    for (_player, pos) in (&mut players, &mut positions).join() {
        let desired_x = min(79, max(0, pos.x + delta_x));
        let desired_y = min(49, max(0, pos.y + delta_y));
        println!("{} {}", desired_x, desired_y);

        let map = ecs.fetch::<Map>();
        if map.blocked[map.coord_to_index(desired_x, desired_y)] {
            {
                let mut log = ecs.write_resource::<GameLog>();
                log.push(&format!("Ouch, you hit a wall!"), Some(Color::RED), None);
            }
        } else {
            pos.x = desired_x;
            pos.y = desired_y;
        }
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

    let (map, position) = SimpleMapBuilder::build(10);

    gs.ecs.insert(map);

    gs.ecs.register::<component::Position>();
    gs.ecs.register::<component::Renderable>();
    gs.ecs.register::<component::Player>();
    gs.ecs
        .create_entity()
        .with(component::Position { x: position.0, y: position.1 })
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
        let offset = 44;
        let mut map = gs.ecs.fetch::<Map>();
        for x in 0..map::MAPWIDTH {
            for y in 0..map::MAPHEIGHT {
                let tile = map.tiles.get(x + y * map::MAPWIDTH).unwrap();
                match tile {
                    TileType::Wall => {
                        tile_ctx.draw(&mut gfx,
                                      &Glyph::from('#', Some(Color::GREEN), None),
                                      (x as f32, y as f32));
                    },
                    TileType::Floor => {
                        tile_ctx.draw(&mut gfx,
                                      &Glyph::from('.',
                                                   Some(Color::from_rgba(128, 128, 128, 1.0)),
                                                   None),
                                      (x as f32, y as f32));
                    },
                }
            }
        }



        let positions = gs.ecs.read_storage::<component::Position>();
        let renderables = gs.ecs.read_storage::<component::Renderable>();

        for (pos, render) in (&positions, &renderables).join() {
            tile_ctx.draw(&mut gfx, &render.glyph, (pos.x as f32, pos.y as f32));
        }

        draw_box(&mut gfx, &tile_ctx, Rectangle::new((0.0, 43.0), (79.0, 6.0)), None, None);
        let log = gs.ecs.fetch::<GameLog>();
        for (index, glyphs) in log.iter().enumerate() {
            print_glyphs(&mut gfx, &tile_ctx, &glyphs, (1, (44 + index) as i32));
        }


        gfx.present(&window)?;
    }
}