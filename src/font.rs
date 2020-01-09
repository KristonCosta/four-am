use quicksilver::{
    graphics::{Graphics, Image},
    Result,
    load_file
};
use rusttype::{Font as RTFont, Scale, point};
use std::path::Path;
use quicksilver::graphics::{PixelFormat};

pub enum Char {
    Smile,
    SmileFilled,
}

impl Char {
    pub fn as_char(&self) -> char {
        use Char::*;

        match *self {
            Smile => '☺',
            SmileFilled => '☻',
        }
    }}

pub struct Font {
    data: RTFont<'static>
}

fn round_pow_2(n: i32) -> i32 {
    (2.0 as f32).powi(((n as f32).log2() + 1.0) as i32) as i32
}

impl Font {
    pub async fn load(path: impl AsRef<Path>) -> Result<Font> {
        let file_contents = load_file(path).await.unwrap();
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
