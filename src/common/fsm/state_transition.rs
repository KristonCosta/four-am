use super::state::State;

pub enum StateTransition<Entity> {
    None,
    Push(Box<dyn State<Entity = Entity> + 'static>),
    Pop(),
    Switch(Box<dyn State<Entity = Entity> + 'static>),
    Exit(),
}
