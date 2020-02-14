use quicksilver::lifecycle::{Key, ElementState, Event, Window};
use crate::geom::{Point, Rect};
use crate::client::camera::get_screen_bounds;
use crate::client::client::{MouseState, TileContext};
use crate::component;
use crate::component::{Name, Position};
use crate::resources::log::GameLog;
use quicksilver::graphics::Color;
use specs::{World, WorldExt};
use crate::server::gamestate::{try_move_player, move_player, clear_pickable};
use specs::join::Join;


pub fn handle_key(ecs: &mut World, key: Key, state: ElementState) {
    if state == ElementState::Pressed {
        match key {
            Key::W => try_move_player(0, -1, ecs),
            Key::A => try_move_player(-1, 0, ecs),
            Key::S => try_move_player(0, 1, ecs),
            Key::D => try_move_player(1, 0, ecs),
            _ => {}
        }
    }
}


pub fn handle_click(ecs: &mut World, map_region: &Rect, tile_ctx: &TileContext, point: impl Into<Point>) {
    let point = point.into();
    let (min_x, _max_x, min_y, _max_y) = get_screen_bounds(&ecs, &tile_ctx);
    let point: Point = (
        point.x + min_x + map_region.origin.x,
        point.y + min_y + map_region.origin.y,
    )
        .into();
    let mut log = ecs.write_resource::<GameLog>();
    let names = ecs.read_storage::<Name>();
    let positions = ecs.read_storage::<Position>();
    for (name, position) in (&names, &positions).join() {
        if position.x == point.x && position.y == point.y {
            log.push(
                &format!("You clicked on {}", name.name),
                Some(Color::GREEN),
                None,
            );
        }
    }
}