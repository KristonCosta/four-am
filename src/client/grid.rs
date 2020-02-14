use crate::geom::{Point, Rect, Vector};

pub struct Grid {
    pub width_multi: i32,
    pub height_multi: i32,
    pub size: Vector,
}

impl Grid {
    pub fn from_screen_size(grid_size: impl Into<Vector>, screen_size: impl Into<Vector>) -> Self {
        let grid = grid_size.into();
        let screen = screen_size.into();
        Grid {
            width_multi: (screen.x / grid.x),
            height_multi: (screen.y / grid.y),
            size: (grid.x, grid.y).into(),
        }
    }

    pub fn from_tile_size(tile_size: impl Into<Vector>, screen_size: impl Into<Vector>) -> Self {
        let tile = tile_size.into();
        let screen = screen_size.into();
        Grid {
            width_multi: tile.x as i32,
            height_multi: tile.y as i32,
            size: (screen.x / tile.x, screen.y / tile.y).into(),
        }
    }

    pub fn rect(&self, point: impl Into<Point>) -> Rect {
        let point = point.into();
        Rect::new(
            (self.width_multi * point.x, self.height_multi * point.y).into(),
            (self.width_multi, self.height_multi).into(),
        )
    }

    pub fn point_to_grid(&self, point: impl Into<Point>) -> Point {
        let point = point.into();
        (point.x / self.width_multi, point.y / self.height_multi).into()
    }
}
