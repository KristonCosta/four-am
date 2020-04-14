use crate::component;
use crate::frontend::glyph::Glyph;

use super::screen::terminal::Terminal;
use crate::{
    client::network_client::NetworkClient,
    geom::{Point, Vector},
    map::{Map, TileType},
};
use legion::prelude::*;
use quicksilver::graphics::Color;

pub struct Camera {
    dimensions: Vector,
    focus: Point,
}

impl Camera {
    pub fn new(dimensions: impl Into<Vector>, focus: impl Into<Point>) -> Self {
        Camera {
            dimensions: dimensions.into(),
            focus: focus.into(),
        }
    }

    pub fn set_dimensions(&mut self, dimensions: Vector) {
        self.dimensions = dimensions
    }

    pub fn focus(&self) -> Point {
        self.focus
    }

    pub fn set_focus(&mut self, focus: impl Into<Point>) {
        self.focus = focus.into();
    }

    pub fn move_focus(&mut self, delta: impl Into<Vector>) {
        let delta = delta.into();
        self.focus.x += delta.x;
        self.focus.y += delta.y;
    }

    pub fn project(&self, point: Point) -> Point {
        let (min_x, _, min_y, _) = self.get_screen_bounds();
        let x = point.x - min_x;
        let y = point.y - min_y;
        (x, y).into()
    }

    pub fn unproject(&self, point: Point) -> Point {
        let (min_x, _, min_y, _) = self.get_screen_bounds();
        let x = point.x + min_x;
        let y = point.y + min_y;
        (x, y).into()
    }

    pub fn get_screen_bounds(&self) -> (i32, i32, i32, i32) {
        let focus = self.focus;
        let (x_chars, y_chars) = self.dimensions.to_tuple();

        let center_x = x_chars / 2;
        let center_y = y_chars / 2;

        let min_x = focus.x - center_x;
        let max_x = min_x + x_chars;

        let min_y = focus.y - center_y;
        let max_y = min_y + y_chars;
        (min_x, max_x, min_y, max_y)
    }

    pub fn render(&self, client: &NetworkClient, terminal: &mut Terminal) {
        let (min_x, max_x, min_y, max_y) = self.get_screen_bounds();
        let (map_width, map_height) = client.resources().get::<Map>().unwrap().size.to_tuple();

        for (y, ty) in (min_y..max_y).enumerate() {
            for (x, tx) in (min_x..max_x).enumerate() {
                let x = x as i32;
                let y = y as i32;
                if tx >= 0 && tx < map_width && ty >= 0 && ty < map_height {
                    let map = client.resources().get::<Map>().unwrap();
                    let tile = map
                        .tiles
                        .get((tx + ty * map_width) as usize)
                        .expect(&format!("Couldn't find {} {}", tx, ty));

                    let glyph = match tile {
                        TileType::Wall => Glyph::from('#', Some(Color::GREEN), None),
                        TileType::Floor => {
                            Glyph::from('.', Some(Color::from_rgba(128, 128, 128, 1.0)), None)
                        }
                        TileType::Digging => {
                            Glyph::from('>', Some(Color::from_rgba(128, 20, 20, 1.0)), None)
                        }
                    };
                    terminal.draw((x, y), &glyph);
                } else {
                    let glyph = Glyph::from('-', Some(Color::WHITE), None);
                    terminal.draw((x, y), &glyph);
                }
            }
        }

        let query = <(Read<component::Position>, Read<component::Renderable>)>::query();
        let world = client.world();
        let mut data = query.iter(world).collect::<Vec<_>>();
        data.sort_by(|a, b| b.1.glyph.render_order.cmp(&a.1.glyph.render_order));
        for (pos, render) in data.iter() {
            let (x, y) = self.project((pos.x, pos.y).into()).to_tuple();
            if x >= 0 && y >= 0 && x < (self.dimensions.x) && y < (self.dimensions.y) {
                terminal.draw((x, y), &render.glyph);
            }
        }

        let query = <(Read<component::Position>, Read<component::Inventory>)>::query().filter(tag::<component::DisplayCabinet>());

        let world = client.world();
        let data = query.iter(world).collect::<Vec<_>>();
        for (pos, inv) in data.iter() {
            let (x, y) = self.project((pos.x, pos.y).into()).to_tuple();
            if x >= 0 && y >= 0 && x < (self.dimensions.x) && y < (self.dimensions.y) {
                let contents: &Vec<Entity> = &inv.as_ref().contents;
                if !contents.is_empty()  {
                    let renderable = world.get_component::<component::Renderable>(*contents.first().unwrap());
                    if let Some(renderable) = renderable {
                        terminal.draw_layer((x, y), &renderable.glyph, 1);
                    }
                }
            }
        }
    }
}
