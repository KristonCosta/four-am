use crate::frontend::client::RenderContext;
use crate::frontend::glyph::Glyph;

use super::{client::{UIMode, LayoutManager, Interactable}, screen::terminal::Terminal};
use crate::geom::{Point, Rect};
use crate::{
    resources::log::GameLog, client::network_client::NetworkClient,
};
use legion::prelude::*;
use quicksilver::{lifecycle::Key, graphics::Color};
use super::client::{UIWidget, UITransition};
use std::char::from_digit;

const VERSION: &str = "1.1.3";

pub fn key_to_char(key: Key) -> char {
    match key {
        Key::A => 'a',
        Key::B => 'b',
        Key::C => 'c',
        Key::D => 'd',
        Key::E => 'e',
        Key::F => 'f',
        Key::G => 'g',
        Key::H => 'h',
        Key::I => 'i',
        Key::J => 'j',
        Key::K => 'k',
        Key::L => 'l',
        Key::M => 'm',
        Key::N => 'n',
        Key::O => 'o',
        Key::P => 'p',
        Key::Q => 'q',
        Key::R => 'r',
        Key::S => 's',
        Key::T => 't',
        Key::U => 'u',
        Key::V => 'v',
        Key::W => 'w',
        Key::X => 'x',
        Key::Y => 'y',
        Key::Z => 'z',
        _ => '_'
    }
}

pub trait UIElement {
    fn render(&self, terminal: &mut Terminal);
}

pub struct MessageWidget {
    message: String
}

impl UIWidget for MessageWidget {}

impl Interactable for MessageWidget {
    fn handle_key(&mut self, key: quicksilver::lifecycle::Key, client: &mut NetworkClient) -> UITransition {
        UITransition::Exit
    }
}

impl UIElement for MessageWidget {
    fn render(&self, terminal: &mut Terminal) {
        let mut region = terminal.region.clone();
        region.origin = (0, 0).into();
        draw_box_filled(terminal, region, None, Some(Color::BLACK));
        print(terminal, self.message.as_str(), (1, 1), None, Some(Color::BLACK))
     }
}
#[derive(Clone)]
pub struct InventoryEntry {
    entity: Entity,
    display_name: String,
}

impl InventoryEntry {
    pub fn new(entity: Entity, display_name: String) -> Self {
        InventoryEntry {
            display_name,
            entity
        }
    }
}
pub struct InventoryWidget {
    contents: Vec<InventoryEntry>
}

pub struct DisplayCaseWidget {
    case: Entity,
    contents: Vec<InventoryEntry>,
    player_inventory: Vec<InventoryEntry>,
}

impl DisplayCaseWidget {
    pub fn new(case: Entity, contents: Vec<InventoryEntry>, player_inventory: Vec<InventoryEntry>) -> Self {
        DisplayCaseWidget {
            case,
            contents,
            player_inventory
        }
    }
}

pub fn entity_enum(entites: &Vec<InventoryEntry>) -> Vec<(char, InventoryEntry)> {
    let mut res = vec![];
    for (index, entity) in entites.clone().drain(..).enumerate() {
        res.push(((b'a' + index as u8) as char, entity));
    }
    res
}

pub fn get_entity(entities: &Vec<InventoryEntry>, c: char) -> Option<InventoryEntry> {
    let index = ((c as u8) - b'a') as usize;
    entities.get(index).map(|e| e.clone())
}


impl UIWidget for DisplayCaseWidget {}

impl Interactable for DisplayCaseWidget {
    fn handle_key(&mut self, key: quicksilver::lifecycle::Key, client: &mut NetworkClient) -> UITransition {
        match key {
            Key::T => {
                if !self.contents.is_empty() {
                    client.try_player_take(self.case);
                    UITransition::Switch(Box::new(MessageWidget{
                    message: format!("You took the {:?}",  self.contents.first().unwrap().display_name)
                }))
                } else {
                    UITransition::Exit
                }
            }
            key => {
                if self.contents.is_empty() {
                    let key = key_to_char(key);
                    if let Some(choice) = get_entity(&self.player_inventory, key) {
                        client.try_player_put(self.case, choice.entity);
                        return UITransition::Switch(Box::new(MessageWidget{
                            message: format!("You put the {:?}",choice.display_name)
                        }))
                    }
                }
                UITransition::Exit
            }
        }
    }
}

impl UIElement for DisplayCaseWidget {
    fn render(&self, terminal: &mut Terminal) {
        let mut region = terminal.region.clone();
        region.origin = (0, 0).into();
        draw_box_filled(terminal, region, None, Some(Color::BLACK));
        if !self.contents.is_empty() {
            print(terminal, &format!("Press T to take the"), (1, 1), None, Some(Color::BLACK));
            print(terminal, &format!("'{:?}'", self.contents.first().unwrap().display_name), (1, 2), None, Some(Color::BLACK))
        } else {

            print(terminal, &format!("To put press:"), (1, 1), None, Some(Color::BLACK));
            for (index, (c, entry)) in entity_enum(&self.player_inventory).iter().enumerate() {
                print(terminal, &format!("[{}] {}", c, entry.display_name), (1, (2 + index) as i32), None, Some(Color::BLACK));
            }
        }

     }
}

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
        UIMode::Overlay(inner) => {
            let mut menu_terminal = overlay.subterminal((main.region.size.width / 2 - 15, main.region.size.height / 2 - 10), (30, 20));
            let rect = menu_terminal.region.clone();
            inner.render(&mut menu_terminal);

            overlay.blit(&mut menu_terminal);
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

pub fn draw_box_filled(terminal: &mut Terminal, rect: Rect, fg: Option<Color>, bg: Option<Color>) {
    let width = rect.size.width - 1;
    let height = rect.size.height - 1;
    let (x_start, x_end) = (rect.origin.x, (rect.origin.x + width));
    let (y_start, y_end) = (rect.origin.y, (rect.origin.y + height));
    let glyph = Glyph::from(' ', fg, bg);
    for x in x_start..x_end {
        for y in y_start..y_end {
            terminal.draw((x, y), &glyph);
        }
    }
    draw_box(terminal, rect, fg, bg)
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
