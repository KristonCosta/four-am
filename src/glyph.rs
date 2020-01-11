use quicksilver::graphics::Color;

#[derive(Debug)]
pub struct Glyph {
    pub ch: char,
    pub foreground: Option<Color>,
    pub background: Option<Color>,
}

impl Glyph {
    pub fn from(ch: char, foreground: Option<Color>, background: Option<Color>) -> Self {
        Glyph {
            ch,
            foreground,
            background
        }
    }
}