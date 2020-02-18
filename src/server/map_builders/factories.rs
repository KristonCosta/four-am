use crate::server::map_builders::{BuiltMap, MapBuilder};
use rand::prelude::ThreadRng;
use crate::geom::Vector;
use crate::server::map_builders::basic_builders::SimpleMapBuilder;
use crate::server::map_builders::drunkard::DrunkardsWalkBuilder;

pub fn random_builder(size: Vector, depth: i32, rng: &mut ThreadRng) -> BuiltMap {
    let mut builder = MapBuilder::new(size, depth, SimpleMapBuilder);
    builder.keep_history();
    builder.build(rng)
}

pub fn drunk_builder(size: Vector, depth: i32, rng: &mut ThreadRng) -> BuiltMap {
    let mut builder = MapBuilder::new(size, depth, DrunkardsWalkBuilder{
        lifetime: 400,
        floor_percent: 0.6,
        brush_size: 1
    });
    builder.keep_history();
    builder.build(rng)
}