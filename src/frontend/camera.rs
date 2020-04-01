use crate::component;
use crate::frontend::client::{Client};
use crate::frontend::glyph::Glyph;

use crate::map::{Map, TileType};
use legion::prelude::*;
use quicksilver::graphics::{Color};

// from https://bfnightly.bracketproductions.com/rustbook/chapter_41.html
pub fn render_camera(client: &mut Client) {
    let (min_x, max_x, min_y, max_y) = client.map_region().get_screen_bounds();
    let (map_width, map_height) = client.resources().get::<Map>().unwrap().size.to_tuple();

    for (y, ty) in (min_y..max_y).enumerate() {
        for (x, tx) in (min_x..max_x).enumerate() {
            let x = x as i32 + client.map_region().region.origin.x;
            let y = y as i32 + client.map_region().region.origin.y;
            if tx >= 0 && tx < map_width && ty >= 0 && ty < map_height {
                let map = client.network_client.resources().get::<Map>().unwrap();
                let index = map.point_to_index((tx, ty).into());
                if !map.revealed_tiles[index] {
                    continue;
                }
                let tile = map
                    .tiles
                    .get((tx + ty * map_width) as usize)
                    .expect(&format!("Couldn't find {} {}", tx, ty));

                let glyph = match tile {
                    TileType::Wall => {
                        Glyph::from('#', Some(Color::GREEN), None)
                    }
                    TileType::Floor => {
                        Glyph::from('.', Some(Color::from_rgba(128, 128, 128, 1.0)), None)

                    }
                    TileType::Digging => {
                        Glyph::from('>', Some(Color::from_rgba(128, 20, 20, 1.0)), None)
                    }
                };
                let glyph = if map.visible_tiles[index] {
                    glyph
                } else {
                    glyph.greyscale()
                };
                client
                    .render_context
                    .draw(&glyph, (x, y))
            } else {
                client
                    .render_context
                    .draw(&Glyph::from('-', Some(Color::WHITE), None), (x, y));
            }
        }
    }

    let query = <(Read<component::Position>, Read<component::Renderable>)>::query();
    let world = client.network_client.world();
    let mut data = query.iter(world).collect::<Vec<_>>();
    data.sort_by(|a, b| b.1.glyph.render_order.cmp(&a.1.glyph.render_order));
    let map = client.network_client.resources().get::<Map>().unwrap();
    for (pos, render) in data.iter() {
        let (x, y) = client.map_region().project((pos.x, pos.y).into()).to_tuple();
        if x >= client.map_region().region.origin.x
            && y >= client.map_region().region.origin.y
            && x < (client.map_region().region.origin.x + client.map_region().region.size.width)
            && y < (client.map_region().region.origin.y + client.map_region().region.size.height)
        {
            let index = map.point_to_index((pos.x, pos.y).into());
            if map.visible_tiles[index] {
                client.render_context.draw(&render.glyph, (x, y));
            }
        }
    }
}


