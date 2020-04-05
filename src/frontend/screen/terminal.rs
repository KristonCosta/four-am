use crate::frontend::{client::RenderContext, glyph::Glyph};
use crate::geom::{Point, Rect, Vector};

pub struct Terminal {
    glyphs: Vec<Option<Glyph>>,
    pub(crate) region: Rect,
    contain_region: Rect,
    num_layers: u8,
    pub min_layer: u8,
}

impl Terminal {
    pub fn new(dimensions: impl Into<Vector>) -> Self {
        let dimensions = dimensions.into();
        let num_layers: u8 = 2;
        Terminal {
            glyphs: vec![None; ((dimensions.x) * (dimensions.y)) as usize * num_layers as usize],
            region: Rect::new((0, 0).into(), dimensions.into()),
            contain_region: Rect::new((0, 0).into(), dimensions.into()),
            num_layers,
            min_layer: 0,
        }
    }

    fn convert_to_index(&self, x: i32, y: i32, layer: u8) -> usize {
        (x + y * self.region.size.width
            + (layer - self.min_layer) as i32 * (self.region.size.width * self.region.size.height))
            as usize
    }

    pub fn draw(&mut self, position: impl Into<Vector>, glyph: &Glyph) {
        let position = position.into();
    
        if self.contain_region.contains(position.to_tuple().into()) {
            let index = self.convert_to_index(position.x, position.y, self.min_layer);
            self.glyphs[index] = Some(glyph.clone());
        }
    }

    pub fn render(&self, context: &mut RenderContext) {
        for layer in self.min_layer..self.min_layer + self.num_layers {
            let layer_offset = layer as i32 * (self.region.size.width * self.region.size.height);
            for y in 0..self.region.size.height {
                let y_offset = y * self.region.size.width;
                for x in 0..self.region.size.width {
                    let index = x + y_offset + layer_offset;
                    if let Some(ref glyph) = self.glyphs[index as usize] {
                        context.draw(glyph, (x + self.region.origin.x, y + self.region.origin.y));
                    }
                }
            }
        }
    }

    pub fn blit(&mut self, terminal: &Terminal) {
        let intersection = self.region.intersection(&terminal.region).unwrap();
        let offset = intersection.origin - terminal.region.origin;
        let x_width = intersection.size.width as usize;
        for layer in terminal.min_layer..terminal.num_layers {
            for (index, y) in (offset.y..offset.y + intersection.size.height).enumerate() {
                let start = terminal.convert_to_index(offset.x, y, layer);
                let end = start + x_width;
                let new_glyphs = terminal.glyphs[start..end].iter().cloned();

                let start = self.convert_to_index(
                    intersection.origin.x,
                    intersection.origin.y + index as i32,
                    layer,
                );
                let end = start + x_width;

                self.glyphs.splice(start..end, new_glyphs);
            }
        }
    }

    pub fn subterminal(&self, origin: impl Into<Point>, dimensions: impl Into<Vector>) -> Terminal {
        let mut term = Terminal::new(dimensions);
        term.region.origin = origin.into();
        term
    }
}
