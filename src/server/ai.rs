use crate::component::{ActiveTurn, Monster, TurnState, Name, Position, Killed};
use specs::prelude::*;
use std::cmp::{min, max};
use quicksilver::graphics::Color;
use rand::Rng;
use crate::resources::log::GameLog;

pub struct MonsterAi;

impl<'a> System<'a> for MonsterAi {
    type SystemData = (
        ReadExpect<'a, crate::server::map::Map>,
        WriteExpect<'a, GameLog>,
        ReadStorage<'a, Monster>,
        WriteStorage<'a, ActiveTurn>,
        WriteStorage<'a, Position>,
        ReadStorage<'a, Name>,
        WriteStorage<'a, Killed>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (map,
             mut log,
             monsters,
             mut turns,
             mut positions,
             names,
             mut killed) = data;
        let mut to_kill = vec![];
        for (_monster, mut turn, mut pos, name) in (&monsters, &mut turns, &mut positions, &names).join() {
            let mut rng = rand::thread_rng();
            let delta_x = rng.gen_range(-1, 2);
            let delta_y = rng.gen_range(-1, 2);
            if delta_x + delta_y == 0 {
                continue
            }
            let desired_x = min(map.size.0, max(0, pos.x + delta_x));
            let desired_y = min(map.size.1, max(0, pos.y + delta_y));

            let coord = map.coord_to_index(desired_x, desired_y);
            if map.blocked[coord] {
                log.push(&format!("Da {} hit a wall!", name.name), Some(Color::RED), None);
            } else if let Some(entity) = map.tile_content[coord] {
                let other_name = names.get(entity);
                if let Some(other_name) = other_name {
                    log.push(&format!("Ouch, {} killed {}", name.name, other_name.name), Some(Color::RED), None);
                }
                to_kill.push(entity);
            } else {
                pos.x = desired_x;
                pos.y = desired_y;
                turn.state = TurnState::DONE;
            }
        }
        for entity in to_kill.drain(..) {
            killed.insert(entity, Killed).expect("failed to insert killed");
        }
    }
}
