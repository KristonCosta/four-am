use crate::component::{
    ActiveTurn,
    Monster,
    TurnState,
    Name,
    Position,
};
use specs::prelude::*;
use std::cmp::{min, max};
use quicksilver::graphics::Color;
use rand::Rng;

pub struct MonsterAi;

impl<'a> System<'a> for MonsterAi {
    type SystemData = (
        ReadExpect<'a, crate::map::Map>,
        WriteExpect<'a, crate::GameLog>,
        ReadStorage<'a, Monster>,
        WriteStorage<'a, ActiveTurn>,
        WriteStorage<'a, Position>,
        ReadStorage<'a, Name>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut map,
             mut log,
             monsters,
             mut turns,
             mut positions,
             names,) = data;

        for (monster, mut turn, mut pos, name) in (&monsters, &mut turns, &mut positions, &names).join() {
            let mut rng = rand::thread_rng();
            let delta_x = rng.gen_range(-1, 2);
            let delta_y = rng.gen_range(-1, 2);
            let desired_x = min(map.size.0, max(0, pos.x + delta_x));
            let desired_y = min(map.size.1, max(0, pos.y + delta_y));

            if map.blocked[map.coord_to_index(desired_x, desired_y)] {
                log.push(&format!("Da {} hit a wall!", name.name), Some(Color::RED), None);
            } else {
                pos.x = desired_x;
                pos.y = desired_y;
                turn.state = TurnState::DONE;

            }
        }
    }
}
