use specs::prelude::*;
use quicksilver::graphics::Color;
use crate::glyph;

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
    pub name: String
}
