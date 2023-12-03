use rltk::prelude::*;
use specs::prelude::*;
use std::cmp::{max, min};

use crate::components::Position;

pub const MAP_WIDTH: usize = 80;
pub const MAP_HEIGHT: usize = 43;
pub const MAP_SIZE: usize = MAP_WIDTH * MAP_HEIGHT;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TileType {
    Wall,
    Floor,
}

impl TileType {
    fn fg(self) -> RGB {
        match self {
            Self::Floor => RGB::from_f32(0.0, 0.5, 0.5),
            Self::Wall => RGB::from_f32(0.0, 1.0, 0.0),
        }
    }
    #[allow(clippy::unused_self)]
    fn bg(self) -> RGB {
        RGB::from_f32(0.0, 0.0, 0.0)
    }
    fn font_char(self) -> FontCharType {
        match self {
            Self::Floor => rltk::to_cp437('.'),
            Self::Wall => rltk::to_cp437('#'),
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
    pub const fn new(x: i32, y: i32, w: i32, h: i32) -> Self {
        Self {
            x1: x,
            x2: x + w,
            y1: y,
            y2: y + h,
        }
    }

    pub const fn intersect(&self, other: &Self) -> bool {
        self.x1 <= other.x2 && self.x2 >= other.x1 && self.y1 <= other.y2 && self.y2 >= other.y1
    }

    pub const fn center(&self) -> (i32, i32) {
        ((self.x1 + self.x2) / 2, (self.y1 + self.y2) / 2)
    }

    pub const fn width(&self) -> i32 {
        self.x2 - self.x1
    }

    pub const fn height(self) -> i32 {
        self.y2 - self.y1
    }
}

pub struct Map {
    pub tiles: Vec<TileType>,
    pub rooms: Vec<Rect>,
    pub width: i32,
    pub height: i32,
    pub revealed_tiles: Vec<bool>,
    pub visible_tiles: Vec<bool>,
    pub blocked: Vec<bool>,
    pub tile_content: Vec<Vec<Entity>>,
}

impl Map {
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
    #[allow(clippy::cast_sign_loss)]
    pub const fn xy_idx(&self, x: i32, y: i32) -> usize {
        (y * self.width + x) as usize
    }

    pub fn new_map_rooms_and_corridors(rng: &mut RandomNumberGenerator) -> Self {
        const MAX_ROOMS: i32 = 30;
        #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
        let mut map = Self {
            tiles: vec![TileType::Wall; MAP_SIZE],
            rooms: vec![],
            width: MAP_WIDTH as i32,
            height: MAP_HEIGHT as i32,
            revealed_tiles: vec![false; MAP_SIZE],
            visible_tiles: vec![false; MAP_SIZE],
            blocked: vec![false; MAP_SIZE],
            tile_content: vec![vec![]; MAP_SIZE],
        };

        // generate and draw rooms
        for _ in 0..MAX_ROOMS {
            let new_room = Self::new_random_room(rng);
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

        // mark walls as blocked
        map.populate_blocked();

        map
    }

    fn new_random_room(rng: &mut RandomNumberGenerator) -> Rect {
        const MIN_SIZE: i32 = 6;
        const MAX_SIZE: i32 = 10;
        let w = rng.range(MIN_SIZE, MAX_SIZE);
        let h = rng.range(MIN_SIZE, MAX_SIZE);
        #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
        let x = rng.roll_dice(1, MAP_WIDTH as i32 - w - 1) - 1;
        #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
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

    fn is_exit_valid(&self, x: i32, y: i32) -> bool {
        if x < 1 || x > self.width - 1 || y < 1 || y > self.height - 1 {
            return false;
        }
        let idx = self.xy_idx(x, y);
        !self.blocked[idx]
    }

    pub fn populate_blocked(&mut self) {
        for (idx, tile) in self.tiles.iter_mut().enumerate() {
            self.blocked[idx] = *tile == TileType::Wall;
        }
    }

    fn reset_tiles(&mut self) {
        for content in &mut self.tile_content {
            content.clear();
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

    fn get_pathing_distance(&self, idx1: usize, idx2: usize) -> f32 {
        #[allow(clippy::cast_sign_loss)]
        let w = self.width as usize;
        let p1 = Point::new(idx1 % w, idx1 / w);
        let p2 = Point::new(idx2 % w, idx2 / w);

        rltk::DistanceAlg::Pythagoras.distance2d(p1, p2)
    }

    fn get_available_exits(&self, idx: usize) -> SmallVec<[(usize, f32); 10]> {
        #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
        let idx = idx as i32;
        let x = idx % self.width;
        let y = idx / self.width;

        [
            // Cardinal directions
            (x - 1, y, 1.0),
            (x + 1, y, 1.0),
            (x, y - 1, 1.0),
            (x, y + 1, 1.0),
            // Diagonals
            // (x - 1, y - 1, 1.45),
            // (x + 1, y + 1, 1.45),
            // (x + 1, y - 1, 1.45),
            // (x - 1, y + 1, 1.45),
        ]
        .into_iter()
        .filter(|&(x, y, _)| self.is_exit_valid(x, y))
        .map(|(x, y, dist)| (self.xy_idx(x, y), dist))
        .collect()
    }
}

pub struct PositionUpdateSystem;

impl<'a> System<'a> for PositionUpdateSystem {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, Position>,
        WriteExpect<'a, Map>,
    );
    fn run(&mut self, data: Self::SystemData) {
        let (entities, positions, mut map) = data;
        map.reset_tiles();
        for (entity, position) in (&entities, &positions).join() {
            let idx = map.xy_idx(position.x, position.y);
            map.tile_content[idx].push(entity);
        }
    }
}
