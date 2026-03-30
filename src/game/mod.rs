use bevy::{log, platform::collections::HashMap, prelude::*};
use bevy_asset_loader::prelude::*;
use bevy_ecs_tilemap::prelude::*;

use crate::game::{
    map::{MapPlugin, TileKind},
    player::PlayerPlugin,
};

mod map;
mod player;

pub struct GamePlugin;

//// Resources ////
#[derive(AssetCollection, Resource)]
struct GameAssets {
    #[asset(texture_atlas_layout(tile_size_x = 16, tile_size_y = 16, columns = 12, rows = 11,))]
    layout: Handle<TextureAtlasLayout>,
    #[asset(path = "dungeon.png")]
    sprite: Handle<Image>,
}

#[derive(Resource)]
struct MapLayer(HashMap<IVec2, TileKind>);

#[derive(Resource)]
struct PlayerSpawn(pub TilePos);
//// End Resource ////

//// Componenta ////
#[derive(Debug, Component)]
struct Position(IVec2);
//// End Components ////

impl From<TilePos> for Position {
    fn from(val: TilePos) -> Self {
        Self(IVec2 {
            x: val.x as i32,
            y: val.y as i32,
        })
    }
}

impl From<&Position> for TilePos {
    fn from(value: &Position) -> Self {
        assert!(value.0.x >= 0 && value.0.y >= 0);
        Self {
            x: value.0.x as u32,
            y: value.0.y as u32,
        }
    }
}

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
            .add_systems(
                Update,
                (update_positions).run_if(in_state(GameStates::Play)),
            );
    }
}

fn update_positions(
    mut moved: Query<(&Position, &mut Transform), Changed<Position>>,
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
    for (grid_pos, mut transform) in &mut moved {
        // 3. Convert IVec2 to TilePos for the helper method
        // We handle the "negative" check here just in case, though your move logic should prevent it
        if grid_pos.0.x >= 0 && grid_pos.0.y >= 0 {
            let tile_pos: TilePos = grid_pos.into();

            // 4. Calculate world position using the crate's built-in math
            let world_pos =
                tile_pos.center_in_world(map_size, grid_size, tile_size, map_type, anchor);

            transform.translation.x = world_pos.x;
            transform.translation.y = world_pos.y;
            transform.translation.z = 1.0;
        }
    }
}
