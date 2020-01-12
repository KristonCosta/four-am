use quicksilver::geom::Rectangle;
use quicksilver::graphics::{Color, Graphics};
use crate::grid::Grid;
use crate::TileContext;
use crate::glyph::Glyph;


pub fn draw_box(gfx: &mut Graphics,
                ctx: &TileContext,
                rect: Rectangle,
                fg: Option<Color>,
                bg: Option<Color>) {
    let top_left = Glyph::from('╔', fg, bg);
    let top_right = Glyph::from('╗', fg, bg);
    let bottom_left = Glyph::from('╚', fg, bg);
    let bottom_right = Glyph::from('╝', fg, bg);
    let vertical = Glyph::from('║', fg, bg);
    let horizontal = Glyph::from('═', fg, bg);

    ctx.draw(gfx, &top_left, (rect.pos.x, rect.pos.y));
    ctx.draw(gfx, &top_right, (rect.pos.x + rect.size.x, rect.pos.y));
    ctx.draw(gfx, &bottom_left, (rect.pos.x, rect.pos.y + rect.size.y));
    ctx.draw(gfx, &bottom_right, (rect.pos.x + rect.size.x, rect.pos.y + rect.size.y));
    let (x_start, x_end) = (rect.pos.x as i32, (rect.pos.x + rect.size.x) as i32);
    let (y_start, y_end) = (rect.pos.y as i32, (rect.pos.y + rect.size.y) as i32);
    for x in (x_start + 1)..x_end {
        ctx.draw(gfx, &horizontal, (x as f32, y_start as f32));
        ctx.draw(gfx, &horizontal, (x as f32, y_end as f32));
    }
    for y in (y_start + 1)..y_end {
        ctx.draw(gfx, &vertical, (x_start as f32, y as f32));
        ctx.draw(gfx, &vertical, (x_end as f32, y as f32));
    }
}

pub fn print(gfx: &mut Graphics,
             ctx: &TileContext,
             text: &str,
             pos: (i32, i32),
             fg: Option<Color>,
             bg: Option<Color>) {
    for (index, ch) in text.chars().enumerate() {
        let ch = Glyph::from(ch, fg, bg);
        ctx.draw(gfx, &ch, (pos.0 as f32 + index as f32, pos.1 as f32));
    }
}

pub fn print_glyphs(gfx: &mut Graphics,
                    ctx: &TileContext,
                    glyphs: &Vec<Glyph>,
                    pos: (i32, i32)) {
    for (index, glyph) in glyphs.iter().enumerate() {
        ctx.draw(gfx, &glyph, (pos.0 as f32 + index as f32, pos.1 as f32));
    }
}

