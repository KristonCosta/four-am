use std::collections::HashMap;
use quicksilver::{
    graphics::{Image, Graphics},
    geom::{Rectangle, Vector},
    Result
};
use crate::font::Font;
use crate::glyph::Glyph;

pub struct Tileset {
    image: Image,
    map: HashMap<char, Rectangle>,
}

impl Tileset {
    pub async fn from_font(gfx: &Graphics, path: &str, ratio: f32) -> Result<Tileset> {
        let lines = r#"╦╩═╬╧╨╤╥╙╘╒╓╫╪┘╠┌█▄▌▐▀αßΓπΣσµτΦδ∞φ╟╚╔║╗╝╣╢╖
*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQ⌠⌡≥
RSTUVWXYZ[\]^_`abcdefghijklmnopqrstuvwxy÷≈
z{|}~⌂ÇüéâäàåçêëèïîìÄÅÉæÆôöòûùÿÖÜ¢£¥₧ƒáí°∙
óúñÑªº¿⌐¬½¼¡«»░▒▓│┤╡╕╜╛┐└┴┬├─┼╞·√±≤ⁿε∩≡ΘΩ
"☺☻♥♦♣♠•◘○◙♂♀♪♫☼►◄↕‼¶§▬↨↑↓→←∟↔▲▼!#$%&'()²■"#;
        let font = Font::load(path).await.expect("failed to load font");
        let size = 40.0;
        let tile_size_px: Vector = (size / ratio, size).into();
        let (font_image, mut width_vec) = font.render(&gfx,
                                                      &lines,
                                                      size,
                                                      ratio).expect("failed to load font image");
        let mut map = HashMap::new();
        let glyph_index = 0;
        width_vec.reverse();
        for glyphs in lines.lines() {
            for glyph in glyphs.chars() {
                map.insert(glyph, width_vec.pop().unwrap());
            }
        }
        Ok(
            Tileset {
                image: font_image,
                map,
            }
        )
    }
    pub fn draw(&self, gfx: &mut Graphics, glyph: &Glyph, pos: Rectangle) {
        let image = &self.image;
        let rect = self.map.get(&glyph.ch).unwrap();
        if let Some(background) = &glyph.background {
            gfx.fill_rect(&pos, *background);
        }
        if let Some(foreground) = &glyph.foreground {
            gfx.draw_subimage_tinted(image, *rect, pos, *foreground);
        } else {
            gfx.draw_subimage(image, *rect, pos);
        }
    }

    pub fn draw_char(&self, gfx: &mut Graphics, glyph: char, pos: Rectangle) {
        let image = &self.image;
        let rect = self.map.get(&glyph).unwrap();
        gfx.draw_subimage(image, *rect, pos);
    }
}
