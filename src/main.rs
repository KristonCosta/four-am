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
use crate::map::{SimpleMapBuilder, MapBuilder, Map, TileType, create_room, TilePos};
use std::panic;
use rand::Rng;
use crate::component::{Position, Name};
use crate::camera::{render_camera, get_screen_bounds};
use pathfinding::prelude::dijkstra;
use std::time::{Instant, Duration};
use pathfinding::directed::dijkstra::dijkstra_all;

pub mod font;
pub mod tileset;
pub mod glyph;
pub mod grid;
pub mod ui;
pub mod component;
pub mod map;

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

pub mod camera {
    use specs::{World, WorldExt};
    use crate::{TileContext, component, Focus};
    use crate::map::{Map, TileType};
    use crate::glyph::Glyph;
    use quicksilver::graphics::{Color, Graphics};
    use specs::join::Join;
    use quicksilver::geom::Rectangle;

    // from https://bfnightly.bracketproductions.com/rustbook/chapter_41.html
    pub fn render_camera(gfx: &mut Graphics, ecs: &World, ctx: &TileContext, region: Rectangle) {
        let map = ecs.fetch::<Map>();

        let (min_x, max_x, min_y, max_y) = get_screen_bounds(ecs, ctx);
        let (map_width, map_height) = map.size;

        for (y, ty) in (min_y..max_y).enumerate() {
            for (x, tx) in (min_x..max_x).enumerate() {
                if y < region.pos.y as usize || y > (region.pos.y + region.size.y) as usize || x < region.pos.x as usize || x > (region.pos.x + region.size.x) as usize {
                    continue;
                }
                let x = x + region.pos.x as usize;
                let y = y + region.pos.y as usize;
                if tx >= 0 && tx < map_width && ty >= 0 && ty < map_height {
                    let tile = map.tiles.get((tx + ty * map_width) as usize).expect(&format!("Couldn't find {} {}", tx, ty));
                    match tile {
                        TileType::Wall => {
                            ctx.draw(gfx,
                                     &Glyph::from('#', Some(Color::GREEN), None),
                                     (x as f32, y as f32));
                        },
                        TileType::Floor => {
                            ctx.draw(gfx,
                                     &Glyph::from('.',
                                                  Some(Color::from_rgba(128, 128, 128, 1.0)),
                                                  None),
                                     (x as f32, y as f32));
                        },
                    }
                } else {
                    ctx.draw(gfx,
                             &Glyph::from('-',
                                          Some( Color::WHITE),
                                          None),
                             (x as f32, y as f32));
                }

            }
        }

        let positions = ecs.read_storage::<component::Position>();
        let renderables = ecs.read_storage::<component::Renderable>();
        let mut data = (&positions, &renderables).join().collect::<Vec<_>>();
        data.sort_by(|&a, &b| b.1.glyph.render_order.cmp(&a.1.glyph.render_order) );
        for (pos, render) in data.iter() {
            let x = pos.x - min_x - region.pos.x as i32;
            let y = pos.y - min_y - region.pos.y as i32;
            if x >= region.pos.x as i32 && y >= region.pos.y as i32 && x < (region.pos.x + region.size.x) as i32 && y < (region.pos.y + region.size.y) as i32 {
                ctx.draw(gfx, &render.glyph, (x as f32, y as f32));
            }
        }
    }
    pub fn get_screen_bounds(ecs: &World, ctx: &TileContext) -> (i32, i32, i32, i32) {
        let focus = ecs.fetch::<Focus>();
        let (x_chars, y_chars) = ctx.grid.size;

        let center_x = (x_chars / 2);
        let center_y = (y_chars/2);

        let min_x = focus.x - center_x;
        let max_x = min_x + x_chars;

        let min_y = focus.y - center_y;
        let max_y = min_y + y_chars;
        (min_x, max_x, min_y, max_y)
    }
}

pub fn clear_pickable(ecs: &mut World) {
    let mut rm_entities = vec![];
    {
        let tiles = ecs.read_storage::<component::PickableTile>();
        let entities = ecs.entities();
        for (entity, tile) in (&entities, &tiles).join() {
            rm_entities.push(entity);
        }
    }
    for entity in rm_entities {
        ecs.delete_entity(entity);
    }
}

pub fn generate_pickable(ecs: &mut World, start: TilePos) {
    let mut result;
    {
        let map = ecs.fetch::<Map>();
        result = dijkstra_all(&start, |p| p.successors(&map, 5));
    }
    for (pos,_) in result.iter() {
        let glyph = Glyph {
            ch: '▒',
            foreground: Some(Color::from_rgba(233, 212, 96, 0.5)),
            background: None,
            render_order: 1,
        };
        ecs
            .create_entity()
            .with(component::Position { x: pos.0, y: pos.1 })
            .with(component::Renderable {
                glyph
            })
            .with(component::PickableTile)
            .build();
    }
}


pub struct State {
    ecs: World
}

pub fn move_player(ecs: &mut World, desired_pos: (i32, i32)) {
    let mut positions = ecs.write_storage::<component::Position>();
    let mut players = ecs.write_storage::<component::Player>();

    for (_player, pos) in (&mut players, &mut positions).join() {
        {
            let mut focus = ecs.write_resource::<Focus>();
            focus.x = desired_pos.0;
            focus.y = desired_pos.1;
        }
        pos.x = desired_pos.0;
        pos.y = desired_pos.1;
        break;
    }
}

pub fn try_move_player(delta_x: i32, delta_y: i32, ecs: &mut World) {
    let mut positions = ecs.write_storage::<component::Position>();
    let mut players = ecs.write_storage::<component::Player>();

    for (_player, pos) in (&mut players, &mut positions).join() {
        let desired_x = min(79, max(0, pos.x + delta_x));
        let desired_y = min(49, max(0, pos.y + delta_y));

        let map = ecs.fetch::<Map>();
        if map.blocked[map.coord_to_index(desired_x, desired_y)] {
            {
                let mut log = ecs.write_resource::<GameLog>();
                log.push(&format!("Ouch, you hit a wall!"), Some(Color::RED), None);
            }
        } else {
            {
                let mut log = ecs.write_resource::<GameLog>();
                log.push(&format!("Walkin' along~"), Some(Color::GREEN), None);
            }
            {
                let mut focus = ecs.write_resource::<Focus>();
                focus.x = desired_x;
                focus.y = desired_y;
            }
            pos.x = desired_x;
            pos.y = desired_y;        }
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

pub fn handle_click(gs: &mut State, ctx: &TileContext, region: Rectangle, pos: (i32, i32)) {
    let (min_x, max_x, min_y, max_y) = get_screen_bounds(&gs.ecs, ctx);
    let pos = (pos.0 + min_x + region.pos.x as i32, pos.1 + min_y + region.pos.y as i32);
    let mut was_teleported = false;
    let mut desired_pos = None;
    {
        let positions = gs.ecs.read_storage::<component::Position>();
        let tiles = gs.ecs.read_storage::<component::PickableTile>();

        for (position, _tile) in (&positions, &tiles).join() {
            println!("Checking {} {}", position.x, position.y);
            if position.x == pos.0 && position.y == pos.1 {
                println!("Teleporting to {} {}", pos.0, pos.1);
                desired_pos = Some(pos);

                break;
            }
        }
    }
    if let Some(desired_pos) = desired_pos {
        move_player(&mut gs.ecs, desired_pos);
        was_teleported = true;
    }

    {
        clear_pickable(&mut gs.ecs);
    }
    if was_teleported {
        return;
    }


    let start;
    {
        let focus = gs.ecs.fetch::<Focus>();
        start = TilePos(focus.x, focus.y, 0);
    }
    generate_pickable(&mut gs.ecs, start);

    let mut log = gs.ecs.write_resource::<GameLog>();
    let names = gs.ecs.read_storage::<Name>();
    let positions = gs.ecs.read_storage::<Position>();
    println!("Clicked on {} {}", pos.0, pos.1);
    for (name, position) in (&names, &positions).join() {
        println!("Player on {} {}", position.x, position.y);
        if position.x == pos.0 && position.y == pos.1 {
            log.push(&format!("You clicked on {}", name.name),
                     Some(Color::GREEN),
                     None);
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
    pub fn draw(&self, gfx: &mut Graphics, glyph: &Glyph, pos: (f32, f32)) {
        let rect = self.grid.rect(pos.0, pos.1);
        self.tileset.draw(gfx, &glyph, rect);
    }
}

async fn app(window: Window, mut gfx: Graphics, mut events: EventStream) -> Result<()> {
    let mut gs = State {
        ecs: World::new(),
    };

    let (map, position) = SimpleMapBuilder::build((80, 42), 10);
    // let (mut map, position) = (Map::new((10,10), 10), (5,2));
    // create_room(&mut map, &Rectangle::new((1, 1), (8, 8)));
    gs.ecs.insert(map);

    gs.ecs.register::<component::Position>();
    gs.ecs.register::<component::Renderable>();
    gs.ecs.register::<component::Player>();
    gs.ecs.register::<component::Name>();
    gs.ecs.register::<component::PickableTile>();
    gs.ecs
        .create_entity()
        .with(component::Position { x: position.0, y: position.1 })
        .with(component::Renderable {
            glyph: Glyph {
                ch: '@',
                foreground: Some(Color::YELLOW),
                background: None,
                render_order: 3
            }
        })
        .with(component::Player{})
        .with(component::Name{name: "Player".to_string() })
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

    let focus = Focus {
        x: position.0,
        y: position.1,
    };

    gs.ecs.insert(focus);
    let map_region = Rectangle::new((0.0, 0.0), (80.0, 42));
    let mut dirty = true;
    let mut duration = Duration::new(0, 0);
    loop {
        while let Some(event) = events.next_event().await {
            match event {
                Event::KeyboardInput {
                    key,
                    state
                } => {
                    handle_key(&mut gs, key, state);
                    dirty =true;
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
                            raw = (mouse.x, mouse.y);
                            pos = tile_ctx.grid.point_to_grid(mouse.x as f32, mouse.y as f32);
                        }

                        handle_click(&mut gs, &tile_ctx, map_region, pos);
                        dirty =true;
                    }
                },
                _ => (),
            }
        }
        if !dirty {
            continue;
        }
        dirty = false;
        gfx.clear(Color::BLACK);
        draw_box(&mut gfx, &tile_ctx, Rectangle::new((0.0, 43.0), (79.0, 6.0)), None, None);
        {
            let log = gs.ecs.fetch::<GameLog>();
            for (index, glyphs) in log.iter().enumerate() {
                print_glyphs(&mut gfx, &tile_ctx, &glyphs, (1, (44 + index) as i32));
            }
        }
        let positions = gs.ecs.read_storage::<component::Position>();
        let players = gs.ecs.read_storage::<component::Player>();

        for (_player, pos) in (&players, &positions).join() {
            render_camera(&mut gfx, &gs.ecs, &tile_ctx, map_region);
            break;
        }
        gfx.present(&window)?;
    }
}