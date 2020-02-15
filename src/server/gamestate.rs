use specs::{World, WorldExt, Builder};
use crate::geom::{Rect, Point};
use crate::component;
use quicksilver::graphics::Color;
use crate::client::glyph::Glyph;
use crate::server::map::{Map, TilePos};
use std::cmp::{max, min};
use pathfinding::prelude::dijkstra_all;
use crate::resources::log::GameLog;
use crate::component::{Killed, TurnState};
use crate::client::client::Focus;
use specs::join::Join;
use crate::server::server::MessageQueue;
use crate::message::Message;

pub struct GameState {
    pub(crate) ecs: World,
    pub(crate) runstate: RunState,
    pub(crate) map_region: Rect,
}


#[derive(PartialEq, Copy, Clone)]
pub enum RunState {
    Paused,
    Running,
}


pub fn clear_pickable(ecs: &mut World) {
    let mut rm_entities = vec![];
    {
        let tiles = ecs.read_storage::<component::PickableTile>();
        let entities = ecs.entities();
        for (entity, _tile) in (&entities, &tiles).join() {
            rm_entities.push(entity);
        }
    }
    for entity in rm_entities {
        ecs.delete_entity(entity);
    }
}

pub fn generate_pickable(ecs: &mut World, start: TilePos) {
    let result;
    {
        let map = ecs.fetch::<Map>();
        result = dijkstra_all(&start, |p| p.successors(&map, 5));
    }
    for (pos, _) in result.iter() {
        let glyph = Glyph {
            ch: '▒',
            foreground: Some(Color::from_rgba(233, 212, 96, 0.5)),
            background: None,
            render_order: 1,
        };
        ecs.create_entity()
            .with(component::Position { x: pos.0, y: pos.1 })
            .with(component::Renderable { glyph })
            .with(component::PickableTile)
            .build();
    }
}

pub fn try_move_player(delta_x: i32, delta_y: i32, ecs: &mut World) {
    let mut message_queue = ecs.write_resource::<MessageQueue>();
    let mut positions = ecs.write_storage::<component::Position>();
    let mut players = ecs.write_storage::<component::Player>();
    let mut active = ecs.write_storage::<component::ActiveTurn>();
    let mut killed = ecs.write_storage::<component::Killed>();
    let names = ecs.read_storage::<component::Name>();
    for (_player, pos, turn) in (&mut players, &mut positions, &mut active).join() {
        let map = ecs.fetch::<Map>();
        let desired_x = min(map.size.0, max(0, pos.x + delta_x));
        let desired_y = min(map.size.1, max(0, pos.y + delta_y));

        let coord = map.coord_to_index(desired_x, desired_y);
        if map.blocked[coord] {
            message_queue.push(Message::GameEvent(format!("Ouch, you hit a wall!"), Some(Color::RED), None));
        } else if let Some(entity) = map.tile_content[coord] {
            let name = names.get(entity);
            if let Some(name) = name {
                message_queue.push(Message::GameEvent(format!("Ouch, you killed {}", name.name), Some(Color::RED), None));
            }
            killed.insert(entity, Killed).expect("failed to insert killed");
        } else {
            {
                let mut focus = ecs.write_resource::<Focus>();
                focus.x = desired_x;
                focus.y = desired_y;
            }
            pos.x = desired_x;
            pos.y = desired_y;
            turn.state = TurnState::DONE;
        }
    }
}

pub fn move_player(ecs: &mut World, desired_pos: impl Into<Point>) {
    let desired_pos = desired_pos.into();
    let mut positions = ecs.write_storage::<component::Position>();
    let mut players = ecs.write_storage::<component::Player>();
    let mut active = ecs.write_storage::<component::ActiveTurn>();
    for (_player, pos, active) in (&mut players, &mut positions, &mut active).join() {
        {
            let mut focus = ecs.write_resource::<Focus>();
            focus.x = desired_pos.x;
            focus.y = desired_pos.y;
        }
        pos.x = desired_pos.x;
        pos.y = desired_pos.y;
        active.state = TurnState::DONE;
        break;
    }
}
