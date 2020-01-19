use crate::geom::Rect;
use crate::glyph::Glyph;
use crate::map::{Map, TileType};
use crate::{component, Focus, GameState, TileContext};
use quicksilver::graphics::{Color, Graphics};
use specs::join::Join;
use specs::{World, WorldExt};

// from https://bfnightly.bracketproductions.com/rustbook/chapter_41.html
pub fn render_camera(
    GameState {
        ecs,
        runstate,
        tile_ctx,
        map_region,
    }: &GameState,
    gfx: &mut Graphics,
) {
    let map = ecs.fetch::<Map>();

    let (min_x, max_x, min_y, max_y) = get_screen_bounds(&ecs, tile_ctx);
    let (map_width, map_height) = map.size;

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
                let tile = map
                    .tiles
                    .get((tx + ty * map_width) as usize)
                    .expect(&format!("Couldn't find {} {}", tx, ty));
                match tile {
                    TileType::Wall => {
                        tile_ctx.draw(gfx, &Glyph::from('#', Some(Color::GREEN), None), (x, y));
                    }
                    TileType::Floor => {
                        tile_ctx.draw(
                            gfx,
                            &Glyph::from('.', Some(Color::from_rgba(128, 128, 128, 1.0)), None),
                            (x, y),
                        );
                    }
                }
            } else {
                tile_ctx.draw(gfx, &Glyph::from('-', Some(Color::WHITE), None), (x, y));
            }
        }
    }

    let positions = ecs.read_storage::<component::Position>();
    let renderables = ecs.read_storage::<component::Renderable>();
    let mut data = (&positions, &renderables).join().collect::<Vec<_>>();
    data.sort_by(|&a, &b| b.1.glyph.render_order.cmp(&a.1.glyph.render_order));
    for (pos, render) in data.iter() {
        let x = pos.x - min_x - map_region.origin.x;
        let y = pos.y - min_y - map_region.origin.y;
        if x >= map_region.origin.x
            && y >= map_region.origin.y
            && x < (map_region.origin.x + map_region.size.width)
            && y < (map_region.origin.y + map_region.size.height)
        {
            tile_ctx.draw(gfx, &render.glyph, (x, y));
        }
    }
}

pub fn get_screen_bounds(ecs: &World, ctx: &TileContext) -> (i32, i32, i32, i32) {
    let focus = ecs.fetch::<Focus>();
    let (x_chars, y_chars) = ctx.grid.size.to_tuple();

    let center_x = (x_chars / 2);
    let center_y = (y_chars / 2);

    let min_x = focus.x - center_x;
    let max_x = min_x + x_chars;

    let min_y = focus.y - center_y;
    let max_y = min_y + y_chars;
    (min_x, max_x, min_y, max_y)
}
