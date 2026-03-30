use bevy::log;
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

use crate::game::MapLayer;
use crate::game::PlayerSpawn;
use crate::game::Position;
use crate::game::TileKind;
use crate::game::{GameAssets, GameStates};

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
        Position::from(spawn_position.0),
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
    mut player_query: Query<(&ActionState<PlayerAction>, &mut Position), With<Player>>,
    map_layer: Res<MapLayer>,
) {
    let Ok((action, mut position)) = player_query.single_mut() else {
        log::warn!("No player");
        return;
    };

    let Some(action) = action.get_just_pressed().first().copied() else {
        return;
    };

    log::info!("PlayerAction: {action:?}");

    let pos_dif = match action {
        PlayerAction::MoveUp => IVec2 { x: 0, y: 1 },
        PlayerAction::MoveDown => IVec2 { x: 0, y: -1 },
        PlayerAction::MoveLeft => IVec2 { x: -1, y: 0 },
        PlayerAction::MoveRight => IVec2 { x: 1, y: 0 },
    };
    let target_pos = position.0 + pos_dif;

    let Some(tile_kind) = map_layer.0.get(&target_pos) else {
        log::warn!("Can't move: {target_pos} not registerd in MapLayer");
        return;
    };
    match tile_kind {
        TileKind::Floor => {
            position.0 = target_pos;
        }
        TileKind::Wall => {
            log::info!("WALL");
        }
    }
}
