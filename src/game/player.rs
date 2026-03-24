use bevy::prelude::*;

use crate::game::{GameAssets, GameStates};

pub(super) struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameStates::Next), spawn_player);
    }
}

#[derive(Debug, Component)]
pub struct Player;

fn spawn_player(mut commands: Commands, game_assets: Res<GameAssets>) {
    commands.spawn((
        Player,
        // ensure we render above the map...
        Transform::from_xyz(0.0, 0.0, 1.0),
        Sprite {
            image: game_assets.sprite.clone(),
            texture_atlas: Some(TextureAtlas {
                layout: game_assets.layout.clone(),
                index: GameAssets::tile_index(7, 1) as usize,
            }),
            ..default()
        },
    ));
}
