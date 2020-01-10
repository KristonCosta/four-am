// Example 3: The RGB Triangle
// Open a window, and draw the standard GPU triangle
use quicksilver::{geom::Vector, graphics::{Color, Element, Graphics, Mesh, Vertex}, lifecycle::{run, EventStream, Settings, Window}, Result, load_file};
use rusttype::{Font as RTFont, Scale, point};
use quicksilver::geom::Rectangle;
use quicksilver::graphics::{Image, PixelFormat};

fn main() {
    run(
        Settings {
            size: Vector::new(800.0, 600.0).into(),
            title: "RGB Triangle Example",
            ..Settings::default()
        },
        app,
    );
}


pub mod font {
    use std::path::Path;
    use rusttype::{Font as RTFont, Scale, point};
    use quicksilver::{Result, load_file};
    use quicksilver::graphics::{Graphics, Image, PixelFormat};

    pub struct Font {
        pub(crate) data: RTFont<'static>
    }

    impl Font {
        pub async fn load(path: impl AsRef<Path>) -> Result<Font> {
            let file_contents = load_file(path).await?;
            let font = RTFont::from_bytes(file_contents).unwrap();
            Ok(
                Font {
                    data: font
                }
            )
        }

        pub fn render(&self, gfx: &Graphics, text: &str, size: f32) -> Result<Image> {
            let scale = Scale::uniform(size);
            let offset = point(0.0, self.data.v_metrics(scale).ascent);
            let glyphs: Vec<_> = self.data.layout(text, scale, offset).collect();
            println!("{}", text);
            let base_width = glyphs.iter().rev()
                .map(|g|
                    g.position().x as f32 + g.unpositioned().h_metrics().advance_width)
                .next().unwrap_or(0.0).ceil() as usize;
            let width = round_pow_2(base_width as i32) as usize;
            let height = round_pow_2(size.ceil() as i32) as usize;
            let mut imgbuf = image::ImageBuffer::new(width as u32, height as u32);
            for glyph in glyphs {
                if let Some(bounding_box) = glyph.pixel_bounding_box() {
                    glyph.draw(|_x, _y, v| {
                        let x = _x + bounding_box.min.x as u32;
                        let y = _y + bounding_box.min.y as u32;
                        let pixel = imgbuf.get_pixel_mut(x, y);
                        *pixel = image::Rgba([255, 255, 255, (255.0 * v) as u8]);
                    });
                }
            }
            Image::from_raw(gfx,
                            &imgbuf.into_raw(),
                            width as u32,
                            height as u32,
                            PixelFormat::RGBA)
        }
    }

    fn round_pow_2(n: i32) -> i32 {
        (2.0 as f32).powi(((n as f32).log2() + 1.0) as i32) as i32
    }
}

pub mod tileset {
    use std::collections::HashMap;
    use quicksilver::{
        graphics::{Image, Graphics},
        geom::{Rectangle, Vector},
        Result
    };
    use crate::font::Font;

    pub struct Tileset {
        image: Image,
        map: HashMap<char, Rectangle>,
    }

    impl Tileset {
        pub async fn from_font(gfx: &Graphics, path: &str) -> Result<Tileset> {
            let glyphs = "ab";
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
                Tileset {
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
}


async fn app(window: Window, mut gfx: Graphics, mut events: EventStream) -> Result<()> {
    // Clear the screen to a blank, black color
    // let mut font = font::Font::load("square.ttf").await?;
    // let img = font.render(&gfx, "12", 100.0).unwrap();
    let tileset = tileset::Tileset::from_font(&gfx, "square.ttf").await?;
    gfx.clear(Color::BLACK);
    // Paint a triangle with red, green, and blue vertices, blending the colors for the pixels in-between
    // Define the 3 vertices and move them inside a Vec
    let rect = Rectangle::new(Vector::new(350.0, 100.0), Vector::new(100.0, 100.0));
    gfx.fill_rect(&rect, Color::RED);
    tileset.draw(&mut gfx, 'a', Rectangle::new((0.0, 0.0), (200.0, 100.0)));
    // Send the data to be drawn
    gfx.present(&window)?;
    loop {
        while let Some(_) = events.next_event().await {}
    }
}