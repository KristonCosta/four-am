use crate::component::{
    ActiveTurn,
    Priority,
    TurnState,
};
use specs::prelude::*;
use crate::resources::log::GameLog;

pub struct PendingMoves {
    list: Vec<Entity>
}

impl PendingMoves {
    pub fn new() -> Self {
        Self {
            list: vec![]
        }
    }
}

pub struct TurnSystem;

impl<'a> System<'a> for TurnSystem {
    type SystemData = (
        Entities<'a>,
        WriteExpect<'a, PendingMoves>,
        ReadStorage<'a, Priority>,
        WriteStorage<'a, ActiveTurn>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities,
            mut pending_moves,
            priorities,
            mut active) = data;

        let mut finished = vec![];
        let mut active_entity = false;
        for (entity, turn) in (&entities, &mut active).join() {
            active_entity = true;
            match turn.state {
                TurnState::DONE => finished.push(entity.clone()),
                _ => (),
            }
        }

        if finished.is_empty() && active_entity {
            return
        }

        for entity in finished {
            active.remove(entity);
        }

        if pending_moves.list.is_empty() {
            let mut priority_tuple = vec![];
            for (entity, priority) in (&entities, &priorities).join() {
                priority_tuple.push((priority.value, entity));
            }
            priority_tuple.sort_by_key(|k| k.0);
            pending_moves.list = priority_tuple.iter().map(|v| v.1).collect();
        }

        let next_turn = pending_moves.list.pop().expect("No entites could take a turn");
        active.insert(next_turn, ActiveTurn{
            state: TurnState::PENDING
        });
    }
}

