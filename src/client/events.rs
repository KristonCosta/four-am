use quicksilver::lifecycle::{Key, ElementState, Event, Window};
use crate::geom::{Point, Rect};
use crate::client::camera::get_screen_bounds;
use crate::client::client::{MouseState, TileContext};
use crate::component;
use crate::component::{Name, Position};
use crate::resources::log::GameLog;
use quicksilver::graphics::Color;
use specs::{World, WorldExt};
use crate::server::gamestate::{try_move_player, move_player, clear_pickable};
use specs::join::Join;

