use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

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
    Next,
}

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((MapPlugin, PlayerPlugin))
            .init_state::<GameStates>()
            .add_loading_state(
                LoadingState::new(GameStates::AssetLoading)
                    .continue_to_state(GameStates::Next)
                    .load_collection::<GameAssets>(),
            );
    }
}
