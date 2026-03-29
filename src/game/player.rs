use bevy::log;
use bevy::prelude::*;
use bevy_ecs_tilemap::tiles::TilePos;
use bevy_ecs_tilemap::tiles::TileStorage;
use leafwing_input_manager::prelude::*;

use crate::game::PlayerSpawn;
use crate::game::{GameAssets, GameStates, Rigid};

pub(super) struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InputManagerPlugin::<PlayerAction>::default())
            .add_systems(OnEnter(GameStates::Play), spawn_player)
            .add_systems(Update, player_input.run_if(in_state(GameStates::Play)));
    }
}

#[derive(Debug, Component)]
pub struct Player;

fn spawn_player(
    mut commands: Commands,
    game_assets: Res<GameAssets>,
    spawn_position: Res<PlayerSpawn>,
) {
    commands.spawn((
        Player,
        Rigid,
        spawn_position.0,
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
        player_input_map(),
    ));
}

#[derive(Debug, Actionlike, PartialEq, Eq, Clone, Copy, Hash, Reflect)]
pub enum PlayerAction {
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
}

pub fn player_input_map() -> InputMap<PlayerAction> {
    InputMap::new([
        (PlayerAction::MoveUp, KeyCode::ArrowUp),
        (PlayerAction::MoveDown, KeyCode::ArrowDown),
        (PlayerAction::MoveLeft, KeyCode::ArrowLeft),
        (PlayerAction::MoveRight, KeyCode::ArrowRight),
    ])
    .with(PlayerAction::MoveUp, KeyCode::KeyW)
    .with(PlayerAction::MoveDown, KeyCode::KeyS)
    .with(PlayerAction::MoveLeft, KeyCode::KeyA)
    .with(PlayerAction::MoveRight, KeyCode::KeyD)
}

pub fn player_input(
    mut player_query: Query<(&ActionState<PlayerAction>, &mut TilePos), With<Player>>,
    tile_storage_q: Query<&TileStorage>,
    rigid_q: Query<(), With<Rigid>>,
) {
    let Ok((action, mut pos)) = player_query.single_mut() else {
        log::warn!("No player");
        return;
    };
    let Ok(storage) = tile_storage_q.single() else {
        log::warn!("Map not loaded");
        return;
    };

    let Some(action) = action.get_just_pressed().first().copied() else {
        return;
    };

    log::info!("PlayerAction: {action:?}");

    let target_pos = match action {
        PlayerAction::MoveUp => TilePos {
            x: pos.x,
            y: pos.y + 1,
        },
        PlayerAction::MoveDown => TilePos {
            x: pos.x,
            y: pos.y.saturating_sub(1),
        },
        PlayerAction::MoveLeft => TilePos {
            x: pos.x.saturating_sub(1),
            y: pos.y,
        },
        PlayerAction::MoveRight => TilePos {
            x: pos.x + 1,
            y: pos.y,
        },
    };

    let Some(entity) = storage.checked_get(&target_pos) else {
        log::info!("Move target out of bounds");
        return; // out of bounds
    };
    if rigid_q.get(entity).is_ok() {
        log::info!("Movement blocked!");
        return; // blocked by rigid tile
    }

    *pos = target_pos;
}
