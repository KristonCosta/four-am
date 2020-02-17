use crate::frontend::client::{RenderContext, TileContext};
use crate::frontend::glyph::Glyph;

use crate::geom::{Point, Rect};
use quicksilver::graphics::{Color, Graphics};

pub fn draw_box(
    render_context: &mut RenderContext,
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

    render_context.draw(&top_left, (rect.origin.x, rect.origin.y));
    render_context.draw(&top_right, (rect.origin.x + rect.size.width, rect.origin.y));
    render_context.draw(
        &bottom_left,
        (rect.origin.x, rect.origin.y + rect.size.height),
    );
    render_context.draw(
        &bottom_right,
        (
            rect.origin.x + rect.size.width,
            rect.origin.y + rect.size.height,
        ),
    );
    let (x_start, x_end) = (rect.origin.x, (rect.origin.x + rect.size.width));
    let (y_start, y_end) = (rect.origin.y, (rect.origin.y + rect.size.height));
    for x in (x_start + 1)..x_end {
        render_context.draw(&horizontal, (x, y_start));
        render_context.draw(&horizontal, (x, y_end));
    }
    for y in (y_start + 1)..y_end {
        render_context.draw(&vertical, (x_start, y));
        render_context.draw(&vertical, (x_end, y));
    }
}

pub fn print(
    render_context: &mut RenderContext,
    text: &str,
    pos: impl Into<Point>,
    fg: Option<Color>,
    bg: Option<Color>,
) {
    let pos = pos.into();
    for (index, ch) in text.chars().enumerate() {
        let ch = Glyph::from(ch, fg, bg);
        render_context.draw(&ch, (pos.x + index as i32, pos.y));
    }
}

pub fn print_glyphs(
    render_context: &mut RenderContext,
    glyphs: &Vec<Glyph>,
    pos: impl Into<Point>,
) {
    let pos = pos.into();
    for (index, glyph) in glyphs.iter().enumerate() {
        render_context.draw(&glyph, (pos.x + index as i32, pos.y));
    }
}
