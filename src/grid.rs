use quicksilver::{
    geom::{Rectangle, Vector}
};

pub struct Grid {
    width_multi: i32,
    height_multi: i32
}

impl Grid {
    pub fn from_screen_size(grid_size: impl Into<Vector>, screen_size: impl Into<Vector>) -> Self {
        let grid = grid_size.into();
        let screen = screen_size.into();
        Grid {
            width_multi: (screen.x / grid.x) as i32,
            height_multi: (screen.y / grid.y) as i32,
        }
    }

    pub fn from_tile_size(tile_size: impl Into<Vector>) -> Self {
        let tile = tile_size.into();
        Grid {
            width_multi: tile.x as i32,
            height_multi: tile.y as i32,
        }
    }

    pub fn rect(&self, x: f32, y: f32) -> Rectangle {
        Rectangle::new(
            (self.width_multi * x as i32, self.height_multi * y as i32),
            (self.width_multi, self.height_multi)
        )
    }

    pub fn point_to_grid(&self, x: f32, y: f32) -> (i32, i32) {
        ((x / self.width_multi as f32) as i32, (y / self.height_multi as f32) as i32)
    }
}
