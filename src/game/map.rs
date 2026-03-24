use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

use crate::game::{GameAssets, GameStates};

pub(super) struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(TilemapPlugin)
            .add_systems(OnEnter(GameStates::Next), spawn_map);
    }
}

fn spawn_map(mut commands: Commands, game_assets: Res<GameAssets>) {
    //spawn a basic floor with walls map
    let map_size = TilemapSize { x: 32, y: 16 };

    let tilemap_entity = commands.spawn_empty().id();
    let mut tile_storage = TileStorage::empty(map_size);

    for x in 0..map_size.x {
        for y in 0..map_size.y {
            let t_idx = if x == 0 || x == map_size.x - 1 || y == 0 || y == map_size.y - 1 {
                GameAssets::tile_index(1, 2)
            } else {
                GameAssets::tile_index(4, 0)
            };
            let tile_pos = TilePos { x, y };
            let tile_entity = commands
                .spawn(TileBundle {
                    position: tile_pos,
                    texture_index: TileTextureIndex(t_idx),
                    tilemap_id: TilemapId(tilemap_entity),
                    ..Default::default()
                })
                .id();
            tile_storage.set(&tile_pos, tile_entity);
        }
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
}
