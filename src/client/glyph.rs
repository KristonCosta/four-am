use quicksilver::graphics::Color;

#[derive(Debug)]
pub struct Glyph {
    pub ch: char,
    pub foreground: Option<Color>,
    pub background: Option<Color>,
    pub render_order: i32,
}

impl Glyph {
    pub fn from(ch: char, foreground: Option<Color>, background: Option<Color>) -> Self {
        Glyph {
            ch,
            foreground,
            background,
            render_order: 0,
        }
    }
}
