use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

use crate::game::{GameState, map};

#[derive(Debug, PartialEq, Eq)]
pub enum TileKind {
    Wall,
    Floor,
}

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(TilemapPlugin)
            .add_systems(OnEnter(GameState::InGame), spawn_tilemap);
    }
}

fn generate_map(width: u32, height: u32) -> Vec<Vec<TileKind>> {
    // For now, simple: walls on edges, floor inside
    (0..height)
        .map(|y| {
            (0..width)
                .map(|x| {
                    if x == 0 || y == 0 || x == width - 1 || y == height - 1 {
                        TileKind::Wall
                    } else {
                        TileKind::Floor
                    }
                })
                .collect()
        })
        .collect()
}

pub fn spawn_tilemap(mut commands: Commands, asset_server: Res<AssetServer>) {
    let texture_handle: Handle<Image> = asset_server.load("sprites/dungeon.png");

    let map_size = TilemapSize { x: 30, y: 20 };
    let tile_size = TilemapTileSize::new(16.0, 16.0);

    // Create a tilemap entity a little early.
    // We want this entity early because we need to tell each tile which tilemap entity
    // it is associated with. This is done with the TilemapId component on each tile.
    // Eventually, we will insert the `TilemapBundle` bundle on the entity, which
    // will contain various necessary components, such as `TileStorage`.
    let tilemap_entity = commands.spawn_empty().id();

    // To begin creating the map we will need a `TileStorage` component.
    // This component is a grid of tile entities and is used to help keep track of individual
    // tiles in the world. If you have multiple layers of tiles you would have a tilemap entity
    // per layer, each with their own `TileStorage` component.
    let mut tile_storage = TileStorage::empty(map_size);

    let map_data = generate_map(map_size.x, map_size.y);
    // Spawn the elements of the tilemap.
    for x in 0..map_size.x {
        for y in 0..map_size.y {
            let tile_pos = TilePos { x, y };
            let tile_entity = commands
                .spawn(TileBundle {
                    position: tile_pos,
                    tilemap_id: TilemapId(tilemap_entity),
                    texture_index: match map_data[y as usize][x as usize] {
                        TileKind::Wall => TileTextureIndex(0),
                        TileKind::Floor => TileTextureIndex(6),
                    }, // index into your spritesheet
                    ..default()
                })
                .id();
            tile_storage.set(&tile_pos, tile_entity);
        }
    }
    commands.entity(tilemap_entity).insert(TilemapBundle {
        grid_size: tile_size.into(),
        map_type: TilemapType::Square,
        size: map_size,
        storage: tile_storage,
        texture: TilemapTexture::Single(texture_handle),
        tile_size,
        anchor: TilemapAnchor::Center,
        ..default()
    });
}
