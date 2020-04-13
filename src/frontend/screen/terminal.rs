use crate::frontend::{client::RenderContext, glyph::Glyph};
use crate::geom::{Point, Rect, Vector};
use std::cmp::min;

pub struct Terminal {
    glyphs: Vec<Option<Glyph>>,
    pub(crate) region: Rect,
    contain_region: Rect,
    pub num_layers: u8,
    pub min_layer: u8,
}

impl Terminal {
    pub fn new(dimensions: impl Into<Vector>) -> Self {
        let dimensions = dimensions.into();
        let num_layers: u8 = 3;
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

    pub fn draw_layer(&mut self, position: impl Into<Vector>, glyph: &Glyph, min_offset: u8) {
        let position = position.into();
        if min_offset > self.num_layers - 1 {
            println!("Warning, tried drawing on an upper layer that doesn't exist.");
        }
        let layer = min(min_offset + self.min_layer, self.min_layer + self.num_layers - 1);
        if self.contain_region.contains(position.to_tuple().into()) {
            let index = self.convert_to_index(position.x, position.y, layer);
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

    pub fn blit(&mut self, terminal: &mut Terminal) {
        let intersection = self.region.intersection(&terminal.region).unwrap();
        let offset = intersection.origin - terminal.region.origin;
        for layer in terminal.min_layer..terminal.min_layer + terminal.num_layers {
            for (index_y, y) in (offset.y..offset.y + intersection.size.height).enumerate() {
                for (index_x, x) in (offset.x..offset.x + intersection.size.width).enumerate() {
                    let local_index = self.convert_to_index(intersection.origin.x + index_x as i32, intersection.origin.y + index_y as i32, layer);
                    let other_index = terminal.convert_to_index(x, y, layer);
                    self.glyphs[local_index] = terminal.glyphs[other_index].take();
                }
            }
        }
    }

    pub fn subterminal(&self, origin: impl Into<Point>, dimensions: impl Into<Vector>) -> Terminal {
        let mut term = Terminal::new(dimensions);
        term.region.origin = origin.into();
        term.min_layer = self.min_layer;
        term.num_layers = self.num_layers;
        term
    }
}
