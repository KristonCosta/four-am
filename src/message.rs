use quicksilver::graphics::Color;
use crate::geom::Point;

pub enum Message {
    GameEvent(String, Option<Color>, Option<Color>),
}