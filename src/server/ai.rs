use crate::component::{ActiveTurn, Killed, Monster, Name, Position, TurnState};
use crate::message::Message;
use crate::server::map::Map;
use crate::server::server::MessageQueue;
use legion::prelude::*;
use quicksilver::graphics::Color;
use rand::Rng;
use std::cmp::{max, min};

pub fn monster_ai() -> Box<dyn Schedulable> {
    SystemBuilder::new("monster_ai")
        .read_component::<Name>()
        .write_resource::<MessageQueue>()
        .write_resource::<Map>()
        .with_query(<(
            Read<Monster>,
            Read<Name>,
            Write<ActiveTurn>,
            Write<Position>,
        )>::query())
        .build(move |command_buffer, mut world, (queue, map), query| {
            let map: &mut Map = map;
            let mut killed = vec![];
            for (entity, (_, name, mut turn, mut pos)) in query.iter_entities_mut(&mut world) {
                let mut rng = rand::thread_rng();
                let delta_x = rng.gen_range(-1, 2);
                let delta_y = rng.gen_range(-1, 2);
                if delta_x + delta_y == 0 {
                    continue;
                }
                let desired_x = min(map.size.0, max(0, pos.x + delta_x));
                let desired_y = min(map.size.1, max(0, pos.y + delta_y));

                let coord = map.coord_to_index(desired_x, desired_y);
                if map.blocked[coord] {
                    queue.push(Message::GameEvent(
                        format!("Da {} hit a wall!", name.name),
                        Some(Color::RED),
                        None,
                    ));
                } else if let Some(other_entity) = map.tile_content[coord] {
                    killed.push((entity, other_entity));
                    command_buffer.add_component(other_entity, Killed);
                } else {
                    pos.x = desired_x;
                    pos.y = desired_y;
                    turn.state = TurnState::DONE;
                }
            }
            for (entity, other_entity) in killed {
                let name = world.get_component::<Name>(entity).unwrap();
                let other_name = world.get_component::<Name>(entity).unwrap();
                queue.push(Message::GameEvent(
                    format!("Ouch, {} killed {}", name.name, other_name.name),
                    Some(Color::RED),
                    None,
                ));
            }
        })
}
