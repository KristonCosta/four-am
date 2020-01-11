use std::path::Path;
use rusttype::{Font as RTFont, Scale, point, PositionedGlyph, Point};
use quicksilver::{Result, load_file};
use quicksilver::graphics::{Graphics, Image, PixelFormat};
use quicksilver::geom::{Vector, Rectangle};

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

    pub fn render(&self, gfx: &Graphics, text: &str, size: f32, ratio: f32) -> Result<(Image, Vec<Rectangle>)> {
        // Most of this is either from, or inspired by, quicksilver
        let scale = Scale::uniform(40.0);
        let line_count = text.lines().count();
        let glyphs_per_line = text
            .lines()
            .map(|text| {
                //Avoid clipping
                let offset = point(0.0, self.data.v_metrics(scale).ascent);
                let glyphs = self.data.layout(text.trim(), scale, offset)
                    .collect::<Vec<PositionedGlyph>>();
                let width = glyphs.iter().rev()
                    .map(|g|
                        g.position().x as f32 + g.unpositioned().h_metrics().advance_width)
                    .next().unwrap_or(0.0).ceil() as usize;
                (glyphs, width)
            })
            .collect::<Vec<_>>();
        let base_width = *glyphs_per_line.iter().map(|(_, width)| width).max().unwrap_or(&0);
        let max_width = round_pow_2(base_width as i32) as usize;
        let height = round_pow_2((size * (line_count as f32)).ceil() as i32) as usize;
        let mut imgbuf = image::ImageBuffer::new(max_width as u32, height as u32);
        let mut rect_vec = vec![];
        for (line_index, (glyphs, width)) in glyphs_per_line.iter().enumerate() {
            for glyph in glyphs {
                if let Some(bounding_box) = glyph.pixel_bounding_box() {
                    let mut origin = glyph.position();
                    origin.x = origin.x.round();
                    origin.y = origin.y.round();
                    let glyph_size = glyph.scale();
                    rect_vec.push( Rectangle::new(
                        Vector::new(origin.x + 1.0, (line_index * size as usize) as u32 + 1),
                        Vector::new(glyph_size.x / ratio - 2.0, glyph_size.y - 2.0)
                    ));
                    glyph.draw(|_x, _y, v| {
                        let x = _x + std::cmp::max(0, bounding_box.min.x) as u32;
                        let y = _y + std::cmp::max(0, bounding_box.min.y) as u32;
                        if x < *width as u32 && y < size as u32 {
                            let pixel = imgbuf.get_pixel_mut(x, y + (line_index * size as usize) as u32);
                            *pixel = image::Rgba([255, 255, 255, (255.0 * v) as u8]);
                        }
                    });
                }
            }
        }
        let img = Image::from_raw(gfx,
                        &imgbuf.into_raw(),
                        max_width as u32,
                        height as u32,
                        PixelFormat::RGBA).unwrap();
        Ok((img, rect_vec))
    }
}

fn round_pow_2(n: i32) -> i32 {
    (2.0 as f32).powi(((n as f32).log2() + 1.0) as i32) as i32
}
