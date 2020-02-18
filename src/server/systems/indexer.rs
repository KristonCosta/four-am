use legion::prelude::*;
use crate::component::*;
use crate::map::Map;

pub fn map_indexer() -> Box<dyn Schedulable> {
    SystemBuilder::new("map_indexer")
        .write_resource::<Map>()
        .with_query(<(Read<Position>, Read<TileBlocker>)>::query())
        .build(move |_, mut world, (map), query| {
            let map: &mut Map = map;
            map.refresh_blocked();
            map.refresh_content();
            for (entity, (position, _)) in query.iter_entities(&mut world) {
                let index = map.coord_to_index(position.x, position.y);
                map.tile_content[index] = Some(entity.clone());
            }
        })
}
