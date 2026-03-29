use bevy::{log, prelude::*};
use bevy_asset_loader::prelude::*;
use bevy_ecs_tilemap::prelude::*;

use crate::game::{map::MapPlugin, player::PlayerPlugin};

mod map;
mod player;

pub struct GamePlugin;

#[derive(AssetCollection, Resource)]
pub(crate) struct GameAssets {
    #[asset(texture_atlas_layout(tile_size_x = 16, tile_size_y = 16, columns = 12, rows = 11,))]
    layout: Handle<TextureAtlasLayout>,
    #[asset(path = "dungeon.png")]
    sprite: Handle<Image>,
}

#[derive(Debug, Component, PartialEq, Eq, Clone, Copy)]
pub struct Rigid;

#[derive(Resource)]
pub(super) struct PlayerSpawn(pub TilePos);

impl From<TilePos> for PlayerSpawn {
    fn from(value: TilePos) -> Self {
        Self(value)
    }
}

impl GameAssets {
    const fn tile_index(row: u32, col: u32) -> u32 {
        const TILE_COLS: u32 = 12;
        const TILE_ROWS: u32 = 11;
        let idx = row * TILE_COLS + col;
        assert!(idx < TILE_COLS * TILE_ROWS);
        idx
    }
}

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
pub(crate) enum GameStates {
    #[default]
    AssetLoading,
    MapLoading,
    Play,
}

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((MapPlugin, PlayerPlugin))
            .init_state::<GameStates>()
            .add_loading_state(
                LoadingState::new(GameStates::AssetLoading)
                    .continue_to_state(GameStates::MapLoading)
                    .load_collection::<GameAssets>(),
            )
            .add_systems(Update, update_positions.run_if(in_state(GameStates::Play)));
    }
}

fn update_positions(
    mut moved: Query<(&TilePos, &mut Transform), Or<(Changed<TilePos>, Added<TilePos>)>>,
    tilemap_q: Query<(
        &TilemapSize,
        &TilemapGridSize,
        &TilemapTileSize,
        &TilemapType,
        &TilemapAnchor,
    )>,
) {
    if moved.is_empty() {
        return;
    }
    let Ok((map_size, grid_size, tile_size, map_type, anchor)) = tilemap_q.single() else {
        log::warn!("No map loaded yet");
        return;
    };
    for (tile_pos, mut transform) in &mut moved {
        let world_pos = tile_pos.center_in_world(map_size, grid_size, tile_size, map_type, anchor);
        transform.translation.x = world_pos.x;
        transform.translation.y = world_pos.y;
    }
}
