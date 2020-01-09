use quicksilver::{
    geom::{Vector},
    lifecycle::{run, Settings, Window, EventStream},
    graphics::{Graphics, Image},
    Result,
};
use quicksilver::geom::Rectangle;

mod font;
use font::{Font, Char};
use std::collections::HashMap;
use quicksilver::graphics::Color;
extern crate console_error_panic_hook;
use std::panic;

struct Game;

struct Tileset {
    image: Image,
    map: HashMap<char, Rectangle>,
}

impl Tileset {
    pub async fn from_font(gfx: &Graphics, path: &str) -> Result<Tileset> {
        let glyphs = format!("a{}{}", Char::Smile.as_char(), Char::SmileFilled.as_char());
        let font = Font::load(path).await.expect("failed to load font");
        let font_image = font.render(&gfx, &glyphs, 100.0).expect("failed to load font image");
        let tile_size_px = Vector::new(100, 100);
        let mut map = HashMap::new();
        for (index, glyph) in glyphs.chars().enumerate() {
            let pos = (index as i32 * tile_size_px.x as i32, 0);
            let rect = Rectangle::new(pos, tile_size_px);
            map.insert(glyph, rect);
        }
        Ok(
            Tileset{
                image: font_image,
                map,
            }
        )
    }
    pub fn draw(&self, gfx: &mut Graphics, glyph: char, pos: Rectangle) {
        let image = &self.image;
        let rect = self.map.get(&glyph).unwrap();
        gfx.draw_subimage(image, *rect, pos);
    }
}

async fn core(window: Window, mut gfx: Graphics, mut events: EventStream) -> Result<()> {
    let font_name = "Px437_PhoenixEGA_8x8.ttf";
    // let tileset = Tileset::from_font(&gfx, font_name).await.unwrap();
    gfx.clear(Color::WHITE);
    // Paint a blue square with a red outline in the center of our screen
    // It should have a top-left of (350, 100) and a size of (150, 100)
    let rect = Rectangle::new(Vector::new(350.0, 100.0), Vector::new(100.0, 100.0));
    gfx.fill_rect(&rect, Color::BLUE);
    gfx.stroke_rect(&rect, Color::RED);
    // Send the data to be drawn
    gfx.present(&window)?;

    // tileset.draw(&mut gfx, Char::Smile.as_char(), Rectangle::new((10,10), (10, 10)));

    loop {
        while let Some(_) = events.next_event().await {}
    }
}


fn main() {
    panic::set_hook(Box::new(console_error_panic_hook::hook));
    run(Settings {
        size: Vector::new(800.0, 600.0).into(),
        title: "Four AM",
        ..Settings::default()
    }, core);
}