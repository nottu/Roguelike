use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

pub struct GamePlugin;

#[derive(AssetCollection, Resource)]
struct GameAssets {
    #[asset(texture_atlas_layout(tile_size_x = 16, tile_size_y = 16, columns = 12, rows = 11,))]
    layout: Handle<TextureAtlasLayout>,
    #[asset(path = "dungeon.png")]
    sprite: Handle<Image>,
}

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum GameStates {
    #[default]
    AssetLoading,
    Next,
}

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameStates>()
            .add_loading_state(
                LoadingState::new(GameStates::AssetLoading)
                    .continue_to_state(GameStates::Next)
                    .load_collection::<GameAssets>(),
            )
            .add_systems(OnEnter(GameStates::Next), spawn_player);
    }
}

#[derive(Debug, Component)]
pub struct Player;

fn spawn_player(mut commands: Commands, game_assets: Res<GameAssets>) {
    commands.spawn((
        Player,
        Sprite {
            image: game_assets.sprite.clone(),
            texture_atlas: Some(TextureAtlas {
                layout: game_assets.layout.clone(),
                index: 85,
            }),
            ..default()
        },
    ));
}
