use crate::error::Result;
use crate::geom::{Point, Rect, Size};
use quicksilver::graphics::{Graphics, Image, PixelFormat};
use quicksilver::load_file;
use rusttype::{point, Font as RTFont, PositionedGlyph, Scale};
use std::path::Path;

pub struct Font {
    pub(crate) data: RTFont<'static>,
}

impl Font {
    pub async fn load(path: impl AsRef<Path>) -> Result<Font> {
        let file_contents = load_file(path).await?;
        let font = RTFont::from_bytes(file_contents)?;
        Ok(Font { data: font })
    }

    pub fn render(
        &self,
        gfx: &Graphics,
        text: &str,
        size: usize,
        ratio: f32,
    ) -> Result<(Image, Vec<Rect>)> {
        // Most of this is either from, or inspired by, quicksilver
        let scale = Scale::uniform(size as f32);
        let line_count = text.lines().count();
        let glyphs_per_line = text
            .lines()
            .map(|text| {
                let offset = point(0.0, self.data.v_metrics(scale).ascent);
                let glyphs = self
                    .data
                    .layout(text.trim(), scale, offset)
                    .collect::<Vec<PositionedGlyph>>();
                let width = glyphs
                    .iter()
                    .rev()
                    .map(|g| g.position().x as f32 + g.unpositioned().h_metrics().advance_width)
                    .next()
                    .unwrap_or(0.0)
                    .ceil() as usize;
                (glyphs, width)
            })
            .collect::<Vec<_>>();
        let base_width = *glyphs_per_line
            .iter()
            .map(|(_, width)| width)
            .max()
            .unwrap_or(&0);
        let max_width = round_pow_2(base_width);
        let height = round_pow_2(size * line_count);
        let mut imgbuf = image::ImageBuffer::new(max_width as u32, height as u32);
        let mut rect_vec = Vec::with_capacity(max_width * height);
        for (line_index, (glyphs, width)) in glyphs_per_line.iter().enumerate() {
            for glyph in glyphs {
                if let Some(bounding_box) = glyph.pixel_bounding_box() {
                    let mut origin = glyph.position();
                    origin.x = origin.x.round();
                    origin.y = origin.y.round();
                    let glyph_size = glyph.scale();
                    rect_vec.push(Rect::new(
                        Point::new(origin.x as i32 + 1, ((line_index * size) + 1) as i32),
                        Size::new((glyph_size.x / ratio) as i32 - 2, glyph_size.y as i32 - 2),
                    ));
                    glyph.draw(|_x, _y, v| {
                        let x = _x + std::cmp::max(0, bounding_box.min.x) as u32;
                        let y = _y + std::cmp::max(0, bounding_box.min.y) as u32;
                        if x < *width as u32 && y < size as u32 {
                            let pixel = imgbuf.get_pixel_mut(x, y + (line_index * size) as u32);
                            *pixel = image::Rgba([255, 255, 255, (255.0 * v) as u8]);
                        }
                    });
                }
            }
        }
        let img = Image::from_raw(
            gfx,
            Some(&imgbuf.into_raw()),
            max_width as u32,
            height as u32,
            PixelFormat::RGBA,
        )
        .expect("failed to create image");
        Ok((img, rect_vec))
    }
}

fn round_pow_2(n: usize) -> usize {
    (2.0 as f32).powi(((n as f32).log2() + 1.0) as i32) as usize
}
