use crate::frontend::client::{RenderContext};
use crate::frontend::glyph::Glyph;

use crate::geom::{Point, Rect};
use quicksilver::graphics::{Color};
use crate::color::{RED, BLACK};
use legion::prelude::*;
use crate::component::{Health, Player, Name};

const VERSION: &str = "1.1.1";

pub fn draw_ui(render_context: &mut RenderContext, world: &World) {
    draw_box(
         render_context,
        Rect::new((0, 0).into(), (49, 45).into()),
        None,
        None,
    );
    draw_box(
        render_context,
        Rect::new((0, 0).into(), (79, 59).into()),
        None,
        None,
    );
    draw_box(
        render_context,
        Rect::new((0, 45).into(), (79, 14).into()),
        None,
        None,
    );
    draw_box(
        render_context,
        Rect::new((49, 0).into(), (30, 8).into()),
        None,
        None,
    );
    let query = <Read<Health>>::query().filter(tag::<Player>());

    for health in query.iter(world) {
        let health_string = format!("Health: {}/{}", health.current, health.max);
        print(render_context, health_string.as_str(), (50, 1), None, None);
        draw_bar_horizontal(render_context, (64, 1).into(), 14, health.current as u32, health.max, RED, BLACK);
        break;
    }

    let mut name: Option<Name> = None;
    let mut health: Option<Health> = None;
    if let Some(targeted) = render_context.targeted_entity {
        name = world.get_component::<Name>(targeted).map(|x| (*x).clone());
        health = world.get_component::<Health>(targeted).map(|x| *x);
    }

    if let Some(name) = name {
        print(render_context, name.name.as_str(), (50, 9), None, None);
    }
    if let Some(health) = health {
        let health_string = format!("Health: {}/{}", health.current, health.max);
        print(render_context, health_string.as_str(), (50, 10), None, None);
        draw_bar_horizontal(render_context, (64, 10).into(), 14, health.current as u32, health.max, RED, BLACK);
    }

    print(render_context, VERSION, (1, 46), None, None);
}

pub fn draw_bar_horizontal(
    render_context: &mut RenderContext,
    position: Point,
    width: u32,
    current_value: u32,
    max_value: u32,
    fg: Color,
    bg: Color) {
    let fill = ((current_value as f32) / (max_value as f32) * (width as f32)) as u32;
    let filled_glyph = Glyph::from('█', Some(fg), Some(bg));
    let empty_glyph = Glyph::from('░', Some(fg), Some(bg));
    for i in 0..width {
        if i > fill {
            render_context.draw(&empty_glyph, (position.x + i as i32, position.y));
        } else {
            render_context.draw(&filled_glyph, (position.x + i as i32, position.y));
        }
    }
}

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
