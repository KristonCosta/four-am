use crate::frontend::client::RenderContext;
use crate::frontend::glyph::Glyph;

use super::{client::{UIMode, LayoutManager}, screen::terminal::Terminal};
use crate::geom::{Point, Rect};
use crate::{
    resources::log::GameLog,
};
use legion::prelude::*;
use quicksilver::graphics::Color;

const VERSION: &str = "1.1.3";

pub fn draw_ui(
    layout: &mut LayoutManager,
    _: &World,
    _: &RenderContext,
    game_log: &GameLog,
    mode: &UIMode,
) {
    let LayoutManager {
        main,
        map,
        log,
        player,
        status,
        overlay,
    } = layout;
    draw_box(overlay, main.region, None, Some(Color::BLACK));
    draw_box(overlay, log.region, None, Some(Color::BLACK));
    draw_box(overlay, map.region, None, Some(Color::BLACK));
    draw_box(overlay, player.region, None, Some(Color::BLACK));

    for (index, glyphs) in game_log.iter().rev().enumerate() {
        if index as i32 >= log.region.size.height - 1 {
            break;
        }
        print_glyphs(log, &glyphs, (1, log.region.size.height - index as i32 - 2));
    }

    match mode {
        UIMode::Interact => {
            print(status, "INTERACTIVE", (1, 1), Some(Color::RED), None);
        }, 
        _ => {}
    }
    print(log, VERSION, (1, 1), None, None);
}

pub fn draw_bar_horizontal(
    terminal: &mut Terminal,
    position: Point,
    width: u32,
    current_value: u32,
    max_value: u32,
    fg: Color,
    bg: Color,
) {
    let fill = ((current_value as f32) / (max_value as f32) * (width as f32)) as u32;
    let filled_glyph = Glyph::from('█', Some(fg), Some(bg));
    let empty_glyph = Glyph::from('░', Some(fg), Some(bg));
    for i in 0..width {
        if i > fill {
            terminal.draw((position.x + i as i32, position.y), &empty_glyph);
        } else {
            terminal.draw((position.x + i as i32, position.y), &filled_glyph);
        }
    }
}

pub fn draw_box(terminal: &mut Terminal, rect: Rect, fg: Option<Color>, bg: Option<Color>) {
    let top_left = Glyph::from('╔', fg, bg);
    let top_right = Glyph::from('╗', fg, bg);
    let bottom_left = Glyph::from('╚', fg, bg);
    let bottom_right = Glyph::from('╝', fg, bg);
    let vertical = Glyph::from('║', fg, bg);
    let horizontal = Glyph::from('═', fg, bg);

    let width = rect.size.width - 1;
    let height = rect.size.height - 1;

    terminal.draw((rect.origin.x, rect.origin.y), &top_left);
    terminal.draw((rect.origin.x + width, rect.origin.y), &top_right);
    terminal.draw((rect.origin.x, rect.origin.y + height), &bottom_left);
    terminal.draw(
        (rect.origin.x + width, rect.origin.y + height),
        &bottom_right,
    );
    let (x_start, x_end) = (rect.origin.x, (rect.origin.x + width));
    let (y_start, y_end) = (rect.origin.y, (rect.origin.y + height));
    for x in (x_start + 1)..x_end {
        terminal.draw((x, y_start), &horizontal);
        terminal.draw((x, y_end), &horizontal);
    }
    for y in (y_start + 1)..y_end {
        terminal.draw((x_start, y), &vertical);
        terminal.draw((x_end, y), &vertical);
    }
}

pub fn print(
    terminal: &mut Terminal,
    text: &str,
    pos: impl Into<Point>,
    fg: Option<Color>,
    bg: Option<Color>,
) {
    let pos = pos.into();
    for (index, ch) in text.chars().enumerate() {
        let ch = Glyph::from(ch, fg, bg);
        terminal.draw((pos.x + index as i32, pos.y), &ch);
    }
}

pub fn print_glyphs(terminal: &mut Terminal, glyphs: &Vec<Glyph>, pos: impl Into<Point>) {
    let pos = pos.into();
    for (index, glyph) in glyphs.iter().enumerate() {
        terminal.draw((pos.x + index as i32, pos.y), &glyph);
    }
}
