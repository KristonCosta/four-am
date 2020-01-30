use crate::geom;
use crate::glyph;
use quicksilver::graphics::Color;
use specs::prelude::*;

#[derive(Component)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

#[derive(Component)]
pub struct Renderable {
    pub glyph: glyph::Glyph,
}

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct Name {
    pub name: String,
}

#[derive(Component)]
pub struct PickableTile;

#[derive(Component)]
pub struct FieldOfView {
    pub visible_tiles: Vec<geom::Point>,
    pub range: i32,
}

pub enum TurnState {
    PENDING,
    ACTIVE,
    DONE
}

#[derive(Component)]
pub struct ActiveTurn {
    pub state: TurnState
}

#[derive(Component)]
pub struct Monster;

#[derive(Component)]
pub struct Priority {
    pub value: u8
}

pub fn register_components(ecs: &mut World) {
    ecs.register::<Position>();
    ecs.register::<Renderable>();
    ecs.register::<Player>();
    ecs.register::<Name>();
    ecs.register::<PickableTile>();
    ecs.register::<FieldOfView>();
    ecs.register::<Monster>();
    ecs.register::<ActiveTurn>();
    ecs.register::<Priority>();
}
