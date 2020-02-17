use crate::geom::Rect;

pub struct GameState {
    pub(crate) runstate: RunState,
    pub(crate) map_region: Rect,
}

#[derive(PartialEq, Copy, Clone)]
pub enum RunState {
    Paused,
    Running,
}