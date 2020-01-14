use specs::Entity;
use std::cmp::{min, max};
use rand::{Rng, SeedableRng};
use crate::geom::Rect;

pub const MAPWIDTH : usize = 80;
pub const MAPHEIGHT : usize = 43;
pub const MAPCOUNT : usize = MAPHEIGHT * MAPWIDTH;

#[derive(Clone)]
pub enum TileType {
    Wall,
    Floor,
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TilePos(pub i32, pub i32, pub i32);

impl TilePos {
    fn push_if_under_cost(&self,
                          map: &Map,
                          succ: &mut Vec<(TilePos, i32)>,
                          offset: (i32, i32),
                          max_cost: i32) {
        if !map.blocked[map.coord_to_index(self.0 + offset.0, self.1 + offset.1)] {
            let agg_cost = self.2 + 1;
            if agg_cost <= max_cost {
                succ.push((TilePos(self.0 + offset.0, self.1 + offset.1, agg_cost), 1));
            }
        }
    }

    pub fn successors(&self, map: &Map, max_cost: i32) -> Vec<(TilePos, i32)> {
        let (width, height) = map.size;
        let mut succ = vec![];

        if self.1 < height - 1 {
            self.push_if_under_cost(map, &mut succ, (0, 1), max_cost)
        }
        if self.1 > 0 {
            self.push_if_under_cost(map, &mut succ, (0, -1), max_cost)
        }
        if self.0 < width - 1 {
            self.push_if_under_cost(map, &mut succ, (1, 0), max_cost)
        }
        if self.0 > 0 {
            self.push_if_under_cost(map, &mut succ, (-1, 0), max_cost)
        }
        succ
    }
}

#[derive(Default)]
pub struct Map {
    pub tiles: Vec<TileType>,
    pub size: (i32, i32),
    pub revealed_tiles: Vec<bool>,
    pub visible_tiles: Vec<bool>,
    pub blocked: Vec<bool>,
    pub depth: i32,
    pub tile_content: Vec<Vec<Entity>>
}

// Base taken from https://bfnightly.bracketproductions.com/rustbook/chapter_23.html
impl Map {
    pub fn new(size: (i32, i32), depth: i32) -> Self {
        let (width, height) = (size.0, size.1);
        let total = (width * height) as usize;
        Map {
            tiles: vec![TileType::Wall; total],
            size: (width as i32, height as i32),
            revealed_tiles: vec![false; total],
            visible_tiles: vec![false; total],
            blocked: vec![true; total],
            depth,
            tile_content: vec![Vec::new(); total],
        }
    }
    pub fn coord_to_index(&self, x: i32, y: i32) -> usize {
        ((y as usize) * self.size.0 as usize) + x as usize
    }
}

pub(crate) trait MapBuilder {
    fn build(size: (i32, i32), depth: i32) -> (Map, (i32, i32)) ;
}

pub struct SimpleMapBuilder {}

impl MapBuilder for SimpleMapBuilder {
    fn build(size: (i32, i32), depth: i32) -> (Map, (i32, i32)) {
        let mut map = Map::new(size, depth);
        let start_pos = SimpleMapBuilder::rooms_and_corridors(&mut map);
        (map, start_pos)
    }

}

impl SimpleMapBuilder {
    pub fn rooms_and_corridors(map: &mut Map) -> (i32, i32) {
        let mut rng = rand::thread_rng();

        const MAX_ROOMS : i32 = 30;
        const MIN_SIZE : i32 = 6;
        const MAX_SIZE : i32 = 10;
        let mut rooms: Vec<Rect> = vec![];
        for i in 0..MAX_ROOMS {
            let w = rng.gen_range(MIN_SIZE, MAX_SIZE);
            let h = rng.gen_range(MIN_SIZE, MAX_SIZE);
            let x = rng.gen_range(1, map.size.0 - w - 1);
            let y = rng.gen_range(1, map.size.1 - h - 1);
            let new_room = Rect::new((x, y).into(), (w, h).into());
            let mut ok = true;
            for other_room in rooms.iter() {
                if new_room.intersects(other_room) {ok = false; break}
            }

            if ok {
                create_room(map, &new_room);
                if !rooms.is_empty() {
                    let center = new_room.center();
                    let prev = rooms[rooms.len() - 1].center();
                    if rng.gen_range(0, 2) == 1 {
                        dig_horizontal(map, prev.x as i32, center.x as i32, prev.y as i32);
                        dig_vertical(map, prev.y as i32, center.y as i32, center.x as i32);
                    } else {
                        dig_horizontal(map, prev.x as i32, center.x as i32, center.y as i32);
                        dig_vertical(map, prev.y as i32, center.y as i32, prev.x as i32);
                    }
                }
                rooms.push(new_room);
            }

        }
        let center = rooms[0].center();
        (center.x as i32, center.y as i32)
    }
}

pub fn create_room(map: &mut Map, room: &Rect) {
    let (x_start, y_start) = room.origin.to_tuple();
    let (x_end, y_end) = ((room.origin.x + room.size.width), (room.origin.y + room.size.height));
    for x in x_start..x_end {
        for y in y_start..y_end {
            let index = map.coord_to_index(x, y);
            map.tiles[index] = TileType::Floor;
            map.blocked[index] = false;
        }
    }
}

pub fn dig_horizontal(map: &mut Map, start: i32, end: i32, y: i32) {
    for x in min(start, end)..=max(start, end) {
        let index = map.coord_to_index(x, y);
        map.tiles[index] = TileType::Floor;
        map.blocked[index] = false;
    }
}

pub fn dig_vertical(map: &mut Map, start: i32, end: i32, x: i32) {
    for y in min(start, end)..=max(start, end) {
        let index = map.coord_to_index(x, y);
        map.tiles[index] = TileType::Floor;
        map.blocked[index] = false;
    }
}
