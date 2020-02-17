pub mod log {
    use crate::frontend::glyph::Glyph;
    use quicksilver::graphics::Color;
    use std::slice::Iter;

    pub struct GameLog {
        max_length: usize,
        lines: Vec<Vec<Glyph>>,
    }

    impl GameLog {
        pub fn with_length(length: usize) -> Self {
            GameLog {
                max_length: length,
                lines: Vec::with_capacity(length + 1),
            }
        }

        pub fn push(&mut self, message: &str, fg: Option<Color>, bg: Option<Color>) {
            let mut glyphs = vec![];
            for ch in message.chars() {
                glyphs.push(Glyph::from(ch, fg, bg));
            }
            self.push_glyphs(glyphs);
        }

        pub fn push_glyphs(&mut self, glyphs: Vec<Glyph>) {
            self.lines.push(glyphs);
            if self.lines.len() > self.max_length {
                self.lines.rotate_left(1);
                self.lines.pop();
            }
        }

        pub fn iter(&self) -> Iter<Vec<Glyph>> {
            self.lines.iter()
        }
    }
}
