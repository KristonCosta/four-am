use specs::Entity;
use quicksilver::geom::{Rectangle, Shape};
use std::cmp::{min, max};
use rand::{Rng, SeedableRng};

pub const MAPWIDTH : usize = 80;
pub const MAPHEIGHT : usize = 43;
pub const MAPCOUNT : usize = MAPHEIGHT * MAPWIDTH;

#[derive(Clone)]
pub enum TileType {
    Wall,
    Floor,
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
    pub fn new(depth: i32) -> Self {
        Map {
            tiles: vec![TileType::Wall; MAPCOUNT],
            size: (MAPWIDTH as i32, MAPHEIGHT as i32),
            revealed_tiles: vec![false; MAPCOUNT],
            visible_tiles: vec![false; MAPCOUNT],
            blocked: vec![true; MAPCOUNT],
            depth,
            tile_content: vec![Vec::new(); MAPCOUNT],
        }
    }
    pub fn coord_to_index(&self, x: i32, y: i32) -> usize {
        (y as usize * self.size.0 as usize) + x as usize
    }
}

pub(crate) trait MapBuilder {
    fn build(depth: i32) -> (Map, (i32, i32)) ;
}

pub struct SimpleMapBuilder {}

impl MapBuilder for SimpleMapBuilder {
    fn build(depth: i32) -> (Map, (i32, i32)) {
        let mut map = Map::new(depth);
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
        let mut rooms: Vec<Rectangle> = vec![];
        for i in 0..MAX_ROOMS {
            let w = rng.gen_range(MIN_SIZE, MAX_SIZE);
            let h = rng.gen_range(MIN_SIZE, MAX_SIZE);
            let x = rng.gen_range(1, map.size.0 - w - 1) - 1;
            let y = rng.gen_range(1, map.size.1 - h - 1) - 1;
            let new_room = Rectangle::new((x, y), (w, h));
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
        let center = rooms[0].center();
        (center.x as i32, center.y as i32)
    }
}

pub fn create_room(map: &mut Map, room: &Rectangle) {
    let (x_start, y_start) = (room.pos.x as i32, room.pos.y as i32);
    let (x_end, y_end) = ((room.pos.x + room.size.x) as i32, (room.pos.y + room.size.y) as i32);
    for x in x_start..=x_end {
        for y in y_start..=y_end {
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
