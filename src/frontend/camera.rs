use crate::component;
use crate::frontend::client::{Client, TileContext};
use crate::frontend::glyph::Glyph;
use crate::geom::{Point, Rect, Vector};
use crate::server::map::{Map, TileType};
use legion::prelude::*;
use quicksilver::graphics::{Color, Graphics};

// from https://bfnightly.bracketproductions.com/rustbook/chapter_41.html
pub fn render_camera(client: &mut Client) {
    let (min_x, max_x, min_y, max_y) = get_screen_bounds(client);
    let (map_width, map_height) = client.resources().get::<Map>().unwrap().size;
    let map_region = client.map_region();

    for (y, ty) in (min_y..max_y).enumerate() {
        for (x, tx) in (min_x..max_x).enumerate() {
            let y = y as i32;
            let x = x as i32;

            if y < map_region.origin.y
                || y > (map_region.origin.y + map_region.size.height)
                || x < map_region.origin.x
                || x > (map_region.origin.x + map_region.size.width)
            {
                continue;
            }
            let x = x + map_region.origin.x;
            let y = y + map_region.origin.y;
            if tx >= 0 && tx < map_width && ty >= 0 && ty < map_height {
                let map = client.network_client.resources().get::<Map>().unwrap();
                let tile = map
                    .tiles
                    .get((tx + ty * map_width) as usize)
                    .expect(&format!("Couldn't find {} {}", tx, ty));

                match tile {
                    TileType::Wall => {
                        client
                            .render_context
                            .draw(&Glyph::from('#', Some(Color::GREEN), None), (x, y));
                    }
                    TileType::Floor => {
                        client.render_context.draw(
                            &Glyph::from('.', Some(Color::from_rgba(128, 128, 128, 1.0)), None),
                            (x, y),
                        );
                    }
                }
            } else {
                client
                    .render_context
                    .draw(&Glyph::from('-', Some(Color::WHITE), None), (x, y));
            }
        }
    }

    let mut query = <(Read<component::Position>, Read<component::Renderable>)>::query();
    let world = client.network_client.world();
    let mut data = query.iter(world).collect::<Vec<_>>();
    data.sort_by(|a, b| b.1.glyph.render_order.cmp(&a.1.glyph.render_order));
    for (pos, render) in data.iter() {
        let x = pos.x - min_x - map_region.origin.x;
        let y = pos.y - min_y - map_region.origin.y;
        if x >= map_region.origin.x
            && y >= map_region.origin.y
            && x < (map_region.origin.x + map_region.size.width)
            && y < (map_region.origin.y + map_region.size.height)
        {
            client.render_context.draw(&render.glyph, (x, y));
        }
    }
}

pub fn get_screen_bounds(client: &Client) -> (i32, i32, i32, i32) {
    let focus = client.focus();
    let (x_chars, y_chars) = client.tile_context().grid.size.to_tuple();

    let center_x = x_chars / 2;
    let center_y = y_chars / 2;

    let min_x = focus.x - center_x;
    let max_x = min_x + x_chars;

    let min_y = focus.y - center_y;
    let max_y = min_y + y_chars;
    (min_x, max_x, min_y, max_y)
}
