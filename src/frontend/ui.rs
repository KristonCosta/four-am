use crate::frontend::client::RenderContext;
use crate::frontend::glyph::Glyph;

use super::{client::LayoutManager, screen::terminal::Terminal};
use crate::color::{BLACK, RED};
use crate::geom::{Point, Rect};
use crate::{
    component::{Health, Name, Player},
    resources::log::GameLog,
};
use legion::prelude::*;
use quicksilver::graphics::Color;

const VERSION: &str = "1.1.2";

pub fn draw_ui(
    layout: &mut LayoutManager,
    world: &World,
    render_context: &RenderContext,
    game_log: &GameLog,
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

    let query = <Read<Health>>::query().filter(tag::<Player>());

    for health in query.iter(world) {
        let health_string = format!("Health: {}/{}", health.current, health.max);
        print(player, health_string.as_str(), (1, 1), None, None);
        draw_bar_horizontal(
            player,
            (15, 1).into(),
            14,
            health.current as u32,
            health.max,
            RED,
            BLACK,
        );
        break;
    }

    let mut name: Option<Name> = None;
    let mut health: Option<Health> = None;
    if let Some(targeted) = render_context.targeted_entity {
        name = world.get_component::<Name>(targeted).map(|x| (*x).clone());
        health = world.get_component::<Health>(targeted).map(|x| *x);
    }

    if let Some(name) = name {
        print(status, name.name.as_str(), (1, 9), None, None);
    }
    if let Some(health) = health {
        let health_string = format!("Health: {}/{}", health.current, health.max);
        print(status, health_string.as_str(), (1, 10), None, None);
        draw_bar_horizontal(
            status,
            (15, 10).into(),
            14,
            health.current as u32,
            health.max,
            RED,
            BLACK,
        );
    }

    for (index, glyphs) in game_log.iter().rev().enumerate() {
        if index as i32 >= log.region.size.height - 1 {
            break;
        }
        print_glyphs(log, &glyphs, (1, log.region.size.height - index as i32 - 2));
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
