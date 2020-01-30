use euclid::default::{
    Point2D as EuclidPoint2D, Rect as EuclidRect, Size2D as EuclidSize2D,
    Vector2D as EuclidVector2D,
};
use quicksilver::geom;

pub type Rect = EuclidRect<i32>;
pub type Point = EuclidPoint2D<i32>;
pub type Vector = EuclidVector2D<i32>;
pub type Size = EuclidSize2D<i32>;

pub trait To<T>: Sized {
    fn to(self) -> T;
}

impl To<geom::Rectangle> for Rect {
    fn to(self) -> geom::Rectangle {
        geom::Rectangle::new(
            geom::Vector::new(self.origin.x, self.origin.y),
            geom::Vector::new(self.size.width, self.size.height),
        )
    }
}
