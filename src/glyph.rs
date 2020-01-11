use quicksilver::graphics::Color;

#[derive(Debug)]
pub struct Glyph {
    pub ch: char,
    pub foreground: Option<Color>,
    pub background: Option<Color>,
}
