use crate::component::{FieldOfView, Position, Player};
use crate::geom::Point;
use crate::map::Map;
use crate::server::fov::calculate_fov;
use legion::prelude::*;

pub fn vision_system() -> Box<dyn Schedulable> {
    SystemBuilder::new("vision_system")
        .write_resource::<Map>()
        .with_query(<(Read<Position>, Write<FieldOfView>)>::query())
        .build(move |_, mut world, map, query| {
            let map: &mut Map = map;
            for (entity, (position, mut fov)) in query.iter_entities_mut(&mut world) {
                let position = Point::new(position.x, position.y);
                if position == fov.previous_position {
                    continue
                }
                fov.visible_tiles.clear();
                let mut visible_tiles = calculate_fov(position, fov.range, map);
                visible_tiles.retain(|&tile| {
                    tile.x >= 0 && tile.x < map.size.x && tile.y >= 0 && tile.y < map.size.y
                });
                fov.visible_tiles = visible_tiles.drain().collect::<Vec<Point>>();
                fov.previous_position = position;
                if let Some(_) = world.get_tag::<Player>(entity) {
                    map.clear_visible();
                    for tile in fov.visible_tiles.iter() {
                        map.set_visible(*tile);
                        map.set_revealed(*tile);
                    }
                }
                // world.
            }
        })
}
