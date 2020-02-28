use crate::geom::Point;
use crate::map::Map;
use std::collections::HashSet;

pub fn calculate_fov(pos: Point, radius: u32, map: &Map) -> HashSet<Point> {
    let mut set = HashSet::new();
    for i in 0..361 {
        calculate_fov_for_direction(
            pos,
            radius,
            ((i as f32).to_radians().cos(), (i as f32).to_radians().sin()),
            map,
            &mut set,
        );
    }
    set
}

fn calculate_fov_for_direction(
    pos: Point,
    radius: u32,
    direction: (f32, f32),
    map: &Map,
    set: &mut HashSet<Point>,
) {
    let mut ox = (pos.x as f32) + 0.5;
    let mut oy = (pos.y as f32) + 0.5;
    for _i in 0..radius {
        set.insert((ox as i32, oy as i32).into());
        let coords = map.coord_to_index(ox as i32, oy as i32);
        if map.blocked[coords] {
            return;
        }
        ox += direction.0;
        oy += direction.1;
    }
}
