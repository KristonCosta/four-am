use crate::error::Result;
use crate::font::Font;
use crate::geom::{Rect, To, Vector};
use crate::glyph::Glyph;
use quicksilver::graphics::{Graphics, Image};
use std::collections::HashMap;

pub struct Tileset {
    image: Image,
    map: HashMap<char, Rect>,
}

static SUPPORTED_CHARS: &str = r#"╦╩═╬╧╨╤╥╙╘╒╓╫╪┘╠┌█▄▌▐▀αßΓπΣσµτΦδ∞φ╟╚╔║╗╝╣╢╖
*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQ⌠⌡≥
RSTUVWXYZ[\]^_`abcdefghijklmnopqrstuvwxy÷≈
z{|}~⌂ÇüéâäàåçêëèïîìÄÅÉæÆôöòûùÿÖÜ¢£¥₧ƒáí°∙
óúñÑªº¿⌐¬½¼¡«»░▒▓│┤╡╕╜╛┐└┴┬├─┼╞·√±≤ⁿε∩≡ΘΩ
"☺☻♥♦♣♠•◘○◙♂♀♪♫☼►◄↕‼¶§▬↨↑↓→←∟↔▲▼!#$%&'()²■"#;

impl Tileset {
    pub async fn from_font(gfx: &Graphics, path: &str, ratio: f32) -> Result<Tileset> {
        let font = Font::load(path).await?;
        let size = 40;
        let (font_image, mut width_vec) = font.render(&gfx, SUPPORTED_CHARS, size, ratio)?;
        let mut map = HashMap::new();
        width_vec.reverse();
        for glyphs in SUPPORTED_CHARS.lines() {
            for glyph in glyphs.chars() {
                map.insert(glyph, width_vec.pop().unwrap());
            }
        }
        Ok(Tileset {
            image: font_image,
            map,
        })
    }
    pub fn draw(&self, gfx: &mut Graphics, glyph: &Glyph, region: Rect) {
        let image = &self.image;
        let region = region.to();
        if let Some(background) = &glyph.background {
            gfx.fill_rect(&region, *background);
        }
        if glyph.ch == ' ' {
            return;
        }
        let rect = self.map[&glyph.ch];
        if let Some(foreground) = &glyph.foreground {
            gfx.draw_subimage_tinted(image, rect.to(), region, *foreground);
        } else {
            gfx.draw_subimage(image, rect.to(), region);
        }
    }

    pub fn draw_char(&self, gfx: &mut Graphics, glyph: char, region: Rect) {
        let image = &self.image;
        let rect = self.map[&glyph];
        gfx.draw_subimage(image, rect.to(), region.to());
    }
}
