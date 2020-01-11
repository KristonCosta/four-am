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
    pub async fn from_font(gfx: &Graphics, path: &str) -> Result<Tileset> {
        let lines = r#"╦╩═╬╧╨╤╥╙╘╒╓╫╪┘╠┌█▄▌▐▀αßΓπΣσµτΦδ∞φ╟╚╔║╗╝╣╢╖
*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQ⌠⌡≥
RSTUVWXYZ[\]^_`abcdefghijklmnopqrstuvwxy÷≈
z{|} ~⌂ÇüéâäàåçêëèïîìÄÅÉæÆôöòûùÿÖÜ¢£¥₧ƒáí°∙
óúñÑªº¿⌐¬½¼¡«»░▒▓│┤╡╕╜╛┐└┴┬├─┼╞·√±≤ⁿε∩≡ΘΩ
"☺☻♥♦♣♠•◘○◙♂♀♪♫☼►◄↕‼¶§▬↨↑↓→←∟↔▲▼!#$%&'()²■"#;
        let font = Font::load(path).await.expect("failed to load font");
        let size = 40.0;
        let font_image = font.render(&gfx, &lines, size).expect("failed to load font image");
        let tile_size_px = Vector::new(size, size);
        let mut map = HashMap::new();
        for (line_num, glyphs) in lines.lines().enumerate() {
            for (index, glyph) in glyphs.chars().enumerate() {
                let pos = (index as i32 * tile_size_px.x as i32, (line_num * tile_size_px.y as usize) as i32);
                let rect = Rectangle::new(pos, tile_size_px);
                map.insert(glyph, rect);
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
        let r = Rectangle::new((rect.pos.x + 1.0, rect.pos.y + 1.0), (rect.size.x - 2.0, rect.size.y - 2.0));
        gfx.draw_subimage(image, r, pos);
    }
}
