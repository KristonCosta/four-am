use specs::{World, WorldExt};
use crate::{TileContext, component, Focus};
use crate::map::{Map, TileType};
use crate::glyph::Glyph;
use quicksilver::graphics::{Color, Graphics};
use specs::join::Join;
use crate::geom::Rect;

// from https://bfnightly.bracketproductions.com/rustbook/chapter_41.html
pub fn render_camera(gfx: &mut Graphics, ecs: &World, ctx: &TileContext, region: Rect) {
    let map = ecs.fetch::<Map>();

    let (min_x, max_x, min_y, max_y) = get_screen_bounds(ecs, ctx);
    let (map_width, map_height) = map.size;

    for (y, ty) in (min_y..max_y).enumerate() {
        for (x, tx) in (min_x..max_x).enumerate() {
            let y = y as i32;
            let x = x as i32;
            if y < region.origin.y
                || y > (region.origin.y + region.size.height)
                || x < region.origin.x
                || x > (region.origin.x + region.size.width) {
                continue;
            }
            let x = x + region.origin.x;
            let y = y + region.origin.y;
            if tx >= 0 && tx < map_width && ty >= 0 && ty < map_height {
                let tile = map.tiles.get((tx + ty * map_width) as usize).expect(&format!("Couldn't find {} {}", tx, ty));
                match tile {
                    TileType::Wall => {
                        ctx.draw(gfx,
                                 &Glyph::from('#', Some(Color::GREEN), None),
                                 (x, y));
                    },
                    TileType::Floor => {
                        ctx.draw(gfx,
                                 &Glyph::from('.',
                                              Some(Color::from_rgba(128, 128, 128, 1.0)),
                                              None),
                                 (x, y));
                    },
                }
            } else {
                ctx.draw(gfx,
                         &Glyph::from('-',
                                      Some( Color::WHITE),
                                      None),
                         (x, y));
            }

        }
    }

    let positions = ecs.read_storage::<component::Position>();
    let renderables = ecs.read_storage::<component::Renderable>();
    let mut data = (&positions, &renderables).join().collect::<Vec<_>>();
    data.sort_by(|&a, &b| b.1.glyph.render_order.cmp(&a.1.glyph.render_order) );
    for (pos, render) in data.iter() {
        let x = pos.x - min_x - region.origin.x;
        let y = pos.y - min_y - region.origin.y;
        if x >= region.origin.x && y >= region.origin.y && x < (region.origin.x + region.size.width) && y < (region.origin.y + region.size.height) {
            ctx.draw(gfx, &render.glyph, (x, y));
        }
    }
}

pub fn get_screen_bounds(ecs: &World, ctx: &TileContext) -> (i32, i32, i32, i32) {
    let focus = ecs.fetch::<Focus>();
    let (x_chars, y_chars) = ctx.grid.size.to_tuple();

    let center_x = (x_chars / 2);
    let center_y = (y_chars/2);

    let min_x = focus.x - center_x;
    let max_x = min_x + x_chars;

    let min_y = focus.y - center_y;
    let max_y = min_y + y_chars;
    (min_x, max_x, min_y, max_y)
}
