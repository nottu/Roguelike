use std::ops::{Index, IndexMut};

use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use rand::prelude::*;

use crate::game::{GameAssets, GameStates, PlayerSpawn, Rigid};

pub(super) struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(TilemapPlugin)
            .add_systems(OnEnter(GameStates::MapLoading), spawn_map);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum TileKind {
    Wall,
    Floor,
}

impl TileKind {
    fn get_tile_idx(&self) -> u32 {
        match self {
            Self::Floor => GameAssets::tile_index(4, 0),
            Self::Wall => GameAssets::tile_index(1, 2),
        }
    }
}

fn spawn_map(
    mut commands: Commands,
    game_assets: Res<GameAssets>,
    mut next_state: ResMut<NextState<GameStates>>,
) {
    let (map, player_spawn) = Map::generate_random();
    commands.insert_resource(player_spawn);
    let map_size = map.size();

    let tilemap_entity = commands.spawn_empty().id();
    let mut tile_storage = TileStorage::empty(map_size);

    for (tile_pos, tile_kind) in map.iter() {
        let tile_entity = commands
            .spawn(TileBundle {
                position: tile_pos,
                texture_index: TileTextureIndex(tile_kind.get_tile_idx()),
                tilemap_id: TilemapId(tilemap_entity),
                ..Default::default()
            })
            .id();
        if tile_kind == TileKind::Wall {
            commands.entity(tile_entity).insert(Rigid);
        }
        tile_storage.set(&tile_pos, tile_entity);
    }

    let tile_size = TilemapTileSize { x: 16.0, y: 16.0 };
    let grid_size = tile_size.into();
    let map_type = TilemapType::default();

    commands.entity(tilemap_entity).insert(TilemapBundle {
        grid_size,
        map_type,
        size: map_size,
        storage: tile_storage,
        texture: TilemapTexture::Single(game_assets.sprite.clone()),
        tile_size,
        anchor: TilemapAnchor::Center,
        ..Default::default()
    });
    next_state.set(GameStates::Play);
}

#[derive(Debug, Clone, Copy)]
struct Rect {
    pub x1: u32,
    pub y1: u32,
    pub x2: u32,
    pub y2: u32,
}

impl Rect {
    pub fn new(x: u32, y: u32, w: u32, h: u32) -> Rect {
        Rect {
            x1: x,
            y1: y,
            x2: x + w,
            y2: y + h,
        }
    }

    fn center(&self) -> TilePos {
        TilePos::new((self.x1 + self.x2) / 2, (self.y1 + self.y2) / 2)
    }

    fn intersect(&self, other: &Rect) -> bool {
        self.x1 <= other.x2 && self.x2 >= other.x1 && self.y1 <= other.y2 && self.y2 >= other.y1
    }
}

struct Map {
    size: TilemapSize,
    tile_kinds: Vec<TileKind>,
}

impl Map {
    fn new(width: u32, height: u32) -> Self {
        Map {
            size: TilemapSize {
                x: width,
                y: height,
            },
            tile_kinds: vec![TileKind::Wall; (width * height) as usize],
        }
    }

    fn size(&self) -> TilemapSize {
        self.size
    }

    fn tile_idx(&self, pos: TilePos) -> usize {
        debug_assert!(
            pos.x < self.size.x && pos.y < self.size.y,
            "tile position ({}, {}) out of bounds for map size ({}, {})",
            pos.x,
            pos.y,
            self.size.x,
            self.size.y,
        );
        (self.size.x * pos.y + pos.x) as usize
    }

    fn carve_room(&mut self, room: Rect) {
        for x in room.x1..room.x2 {
            for y in room.y1..room.y2 {
                self[TilePos { x, y }] = TileKind::Floor;
            }
        }
    }

    fn iter(&self) -> impl Iterator<Item = (TilePos, TileKind)> {
        (0..self.size.y)
            .flat_map(move |y| (0..self.size.x).map(move |x| TilePos { x, y }))
            .map(|pos| (pos, self[pos]))
    }

    fn generate_random() -> (Self, PlayerSpawn) {
        let mut map = Self::new(80, 50);
        const MAX_ROOMS: usize = 30;
        const MIN_SIZE: u32 = 6;
        const MAX_SIZE: u32 = 10;

        let mut rooms = Vec::with_capacity(MAX_ROOMS);

        let mut rng = rand::rng();
        for _ in 0..MAX_ROOMS {
            let w = rng.random_range(MIN_SIZE..MAX_SIZE);
            let h = rng.random_range(MIN_SIZE..MAX_SIZE);

            let x = rng.random_range(1..map.size.x - w);
            let y = rng.random_range(1..map.size.y - h);

            let new_room = Rect::new(x, y, w, h);
            if rooms.iter().any(|o| new_room.intersect(o)) {
                continue;
            }
            map.carve_room(new_room);
            if let Some(prev_room) = rooms.last() {
                let center = new_room.center();
                let prev_center = prev_room.center();

                if rng.random_bool(0.5) {
                    map.apply_horizontal_tunnel(center, prev_center);
                    map.apply_vertical_tunnel(prev_center, center);
                } else {
                    map.apply_vertical_tunnel(center, prev_center);
                    map.apply_horizontal_tunnel(prev_center, center);
                }
            }
            rooms.push(new_room);
        }

        (map, rooms.first().expect("expected a room").center().into())
    }

    fn apply_horizontal_tunnel(&mut self, start: TilePos, end: TilePos) {
        for x in u32::min(start.x, end.x)..=u32::max(start.x, end.x) {
            self[TilePos { x, y: start.y }] = TileKind::Floor;
        }
    }
    fn apply_vertical_tunnel(&mut self, start: TilePos, end: TilePos) {
        for y in u32::min(start.y, end.y)..=u32::max(start.y, end.y) {
            self[TilePos { x: start.x, y }] = TileKind::Floor;
        }
    }
}

impl Index<TilePos> for Map {
    type Output = TileKind;

    fn index(&self, pos: TilePos) -> &TileKind {
        &self.tile_kinds[self.tile_idx(pos)]
    }
}

impl IndexMut<TilePos> for Map {
    fn index_mut(&mut self, pos: TilePos) -> &mut TileKind {
        let idx = self.tile_idx(pos);
        &mut self.tile_kinds[idx]
    }
}
