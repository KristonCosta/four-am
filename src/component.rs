use crate::frontend::glyph::Glyph;
use crate::geom;
use legion::prelude::World;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Renderable {
    pub glyph: Glyph,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Player;

#[derive(Clone, Debug, PartialEq)]
pub struct Name {
    pub name: String,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PickableTile;

#[derive(Clone, Debug, PartialEq)]
pub struct FieldOfView {
    pub visible_tiles: Vec<geom::Point>,
    pub range: i32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TurnState {
    PENDING,
    ACTIVE,
    DONE,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ActiveTurn {
    pub state: TurnState,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Monster;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TileBlocker;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Killed;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Priority {
    pub value: u8,
}
