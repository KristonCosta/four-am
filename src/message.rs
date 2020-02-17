use crate::geom::Point;
use quicksilver::graphics::Color;

pub enum Message {
    GameEvent(String, Option<Color>, Option<Color>),
}
