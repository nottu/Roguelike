use rltk::prelude::*;
use std::cmp::{max, min};

pub const MAP_WIDTH: usize = 80;
pub const MAP_HEIGHT: usize = 50;
pub const MAP_SIZE: usize = MAP_WIDTH * MAP_HEIGHT;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TileType {
    Wall,
    Floor,
}

impl TileType {
    fn fg(&self) -> RGB {
        match self {
            TileType::Floor => RGB::from_f32(0.5, 0.5, 0.5),
            TileType::Wall => RGB::from_f32(0.0, 1.0, 0.0),
        }
    }
    fn bg(&self) -> RGB {
        match self {
            TileType::Floor => RGB::from_f32(0.0, 0.0, 0.0),
            TileType::Wall => RGB::from_f32(0.0, 0.0, 0.0),
        }
    }
    fn font_char(&self) -> FontCharType {
        match self {
            TileType::Floor => rltk::to_cp437('.'),
            TileType::Wall => rltk::to_cp437('#'),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Rect {
    pub x1: i32,
    pub x2: i32,
    pub y1: i32,
    pub y2: i32,
}

impl Rect {
    pub fn new(x: i32, y: i32, w: i32, h: i32) -> Self {
        Self {
            x1: x,
            x2: x + w,
            y1: y,
            y2: y + h,
        }
    }

    pub fn intersect(&self, other: &Self) -> bool {
        self.x1 <= other.x2 && self.x2 >= other.x1 && self.y1 <= other.y2 && self.y2 >= other.y1
    }

    pub fn center(&self) -> (i32, i32) {
        ((self.x1 + self.x2) / 2, (self.y1 + self.y2) / 2)
    }
}

pub struct Map {
    pub tiles: Vec<TileType>,
    pub rooms: Vec<Rect>,
    pub width: i32,
    pub height: i32,
    pub revealed_tiles: Vec<bool>,
    pub visible_tiles: Vec<bool>,
}

impl Map {
    pub fn xy_idx(&self, x: i32, y: i32) -> usize {
        (y * self.width + x) as usize
    }
    pub fn new_map_rooms_and_corridors() -> Self {
        let mut map = Self {
            tiles: vec![TileType::Wall; MAP_SIZE],
            rooms: vec![],
            width: MAP_WIDTH as i32,
            height: MAP_HEIGHT as i32,
            revealed_tiles: vec![false; MAP_SIZE],
            visible_tiles: vec![false; MAP_SIZE],
        };

        const MAX_ROOMS: i32 = 30;

        let mut rng = RandomNumberGenerator::new();

        // generate and draw rooms
        for _ in 0..MAX_ROOMS {
            let new_room = Self::new_random_room(&mut rng);
            let intersects = map
                .rooms
                .iter()
                .any(|other_room| new_room.intersect(other_room));
            if intersects {
                continue;
            }
            map.apply_room_to_map(&new_room);
            map.rooms.push(new_room);
        }

        // draw tunnels
        for idx in 1..map.rooms.len() {
            let (new_x, new_y) = map.rooms[idx].center();
            let (prev_x, prev_y) = map.rooms[idx - 1].center();
            if rng.range(0, 2) == 1 {
                map.apply_horizontal_tunnel(prev_x, new_x, prev_y);
                map.apply_vertical_tunnel(prev_y, new_y, new_x);
            } else {
                map.apply_vertical_tunnel(prev_y, new_y, prev_x);
                map.apply_horizontal_tunnel(prev_x, new_x, new_y);
            }
        }

        map
    }

    fn new_random_room(rng: &mut RandomNumberGenerator) -> Rect {
        const MIN_SIZE: i32 = 6;
        const MAX_SIZE: i32 = 10;
        let w = rng.range(MIN_SIZE, MAX_SIZE);
        let h = rng.range(MIN_SIZE, MAX_SIZE);
        let x = rng.roll_dice(1, MAP_WIDTH as i32 - w - 1) - 1;
        let y = rng.roll_dice(1, MAP_HEIGHT as i32 - h - 1) - 1;
        Rect::new(x, y, w, h)
    }

    fn apply_room_to_map(&mut self, room: &Rect) {
        for y in (room.y1 + 1)..=room.y2 {
            for x in (room.x1 + 1)..=room.x2 {
                let idx = self.xy_idx(x, y);
                self.tiles[idx] = TileType::Floor;
            }
        }
    }

    fn apply_horizontal_tunnel(&mut self, x1: i32, x2: i32, y: i32) {
        for x in min(x1, x2)..=max(x1, x2) {
            let idx = self.xy_idx(x, y);
            if idx > 0 && idx < MAP_SIZE {
                self.tiles[idx] = TileType::Floor;
            }
        }
    }

    fn apply_vertical_tunnel(&mut self, y1: i32, y2: i32, x: i32) {
        for y in min(y1, y2)..=max(y1, y2) {
            let idx = self.xy_idx(x, y);
            if idx > 0 && idx < MAP_SIZE {
                self.tiles[idx] = TileType::Floor;
            }
        }
    }

    pub fn draw(&self, ctx: &mut Rltk) {
        let mut y = 0;
        let mut x = 0;
        for (idx, tile) in self.tiles.iter().enumerate() {
            if self.revealed_tiles[idx] {
                let fg = if self.visible_tiles[idx] {
                    tile.fg()
                } else {
                    tile.fg().to_greyscale()
                };
                ctx.set(x, y, fg, tile.bg(), tile.font_char());
            }
            // Move the coordinates
            x += 1;
            if x > 79 {
                x = 0;
                y += 1;
            }
        }
    }
}

impl Algorithm2D for Map {
    fn dimensions(&self) -> Point {
        Point {
            x: self.width,
            y: self.height,
        }
    }
}

impl BaseMap for Map {
    fn is_opaque(&self, idx: usize) -> bool {
        self.tiles[idx] == TileType::Wall
    }
}
