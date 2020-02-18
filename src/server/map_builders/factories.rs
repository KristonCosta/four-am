use crate::server::map_builders::{BuiltMap, MapBuilder};
use rand::prelude::ThreadRng;
use crate::geom::Vector;
use crate::server::map_builders::basic_builders::SimpleMapBuilder;

pub fn random_builder(size: Vector, depth: i32, rng: &mut ThreadRng) -> BuiltMap {
    let mut builder = MapBuilder::new(size, depth, SimpleMapBuilder);
    builder.keep_history();
    builder.build(rng)
}