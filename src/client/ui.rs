use crate::geom::{Point, Rect};
use crate::client::glyph::Glyph;
use quicksilver::graphics::{Color, Graphics};
use crate::client::client::TileContext;

pub fn draw_box(
    gfx: &mut Graphics,
    ctx: &TileContext,
    rect: Rect,
    fg: Option<Color>,
    bg: Option<Color>,
) {
    let top_left = Glyph::from('╔', fg, bg);
    let top_right = Glyph::from('╗', fg, bg);
    let bottom_left = Glyph::from('╚', fg, bg);
    let bottom_right = Glyph::from('╝', fg, bg);
    let vertical = Glyph::from('║', fg, bg);
    let horizontal = Glyph::from('═', fg, bg);

    ctx.draw(gfx, &top_left, (rect.origin.x, rect.origin.y));
    ctx.draw(
        gfx,
        &top_right,
        (rect.origin.x + rect.size.width, rect.origin.y),
    );
    ctx.draw(
        gfx,
        &bottom_left,
        (rect.origin.x, rect.origin.y + rect.size.height),
    );
    ctx.draw(
        gfx,
        &bottom_right,
        (
            rect.origin.x + rect.size.width,
            rect.origin.y + rect.size.height,
        ),
    );
    let (x_start, x_end) = (rect.origin.x, (rect.origin.x + rect.size.width));
    let (y_start, y_end) = (rect.origin.y, (rect.origin.y + rect.size.height));
    for x in (x_start + 1)..x_end {
        ctx.draw(gfx, &horizontal, (x, y_start));
        ctx.draw(gfx, &horizontal, (x, y_end));
    }
    for y in (y_start + 1)..y_end {
        ctx.draw(gfx, &vertical, (x_start, y));
        ctx.draw(gfx, &vertical, (x_end, y));
    }
}

pub fn print(
    gfx: &mut Graphics,
    ctx: &TileContext,
    text: &str,
    pos: impl Into<Point>,
    fg: Option<Color>,
    bg: Option<Color>,
) {
    let pos = pos.into();
    for (index, ch) in text.chars().enumerate() {
        let ch = Glyph::from(ch, fg, bg);
        ctx.draw(gfx, &ch, (pos.x + index as i32, pos.y));
    }
}

pub fn print_glyphs(
    gfx: &mut Graphics,
    ctx: &TileContext,
    glyphs: &Vec<Glyph>,
    pos: impl Into<Point>,
) {
    let pos = pos.into();
    for (index, glyph) in glyphs.iter().enumerate() {
        ctx.draw(gfx, &glyph, (pos.x + index as i32, pos.y));
    }
}
