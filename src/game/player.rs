use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

use crate::game::assets::SpriteAssets;
use crate::game::*;

// Hero 0 body animations (cols 0-2, various rows)
const IDLE_FRAMES: &[usize] = &[0, 1, 2];

#[derive(Actionlike, PartialEq, Eq, Hash, Clone, Copy, Debug, Reflect)]
pub enum PlayerAction {
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
}

#[derive(Debug, Component)]
pub struct Player;

fn player_input_map() -> InputMap<PlayerAction> {
    InputMap::default()
        .with_one_to_many(PlayerAction::MoveUp, [KeyCode::ArrowUp, KeyCode::KeyW])
        .with_one_to_many(PlayerAction::MoveDown, [KeyCode::ArrowDown, KeyCode::KeyS])
        .with_one_to_many(PlayerAction::MoveLeft, [KeyCode::ArrowLeft, KeyCode::KeyA])
        .with_one_to_many(
            PlayerAction::MoveRight,
            [KeyCode::ArrowRight, KeyCode::KeyD],
        )
}

fn spawn_player(mut commands: Commands, assets: Res<SpriteAssets>) {
    commands.spawn((
        Player,
        TilePos { x: 5, y: 5 },
        Sprite::from_atlas_image(
            assets.heroes.clone(),
            TextureAtlas {
                layout: assets.heroes_layout.clone(),
                index: IDLE_FRAMES[0],
            },
        ),
        Transform::from_xyz(0.0, 0.0, 10.0),
        SpriteAnimation::new(IDLE_FRAMES),
        player_input_map(),
        ActionState::<PlayerAction>::default(),
        FacingDir::default(),
    ));
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InputManagerPlugin::<PlayerAction>::default())
            .add_systems(OnEnter(GameState::InGame), spawn_player)
            .add_systems(Update, move_player);
    }
}

pub fn move_player(
    mut query: Query<(&ActionState<PlayerAction>, &mut TilePos, &mut FacingDir), With<Player>>,
) {
    let Ok((action, mut pos, mut facing)) = query.single_mut() else {
        return;
    };

    let dx: i32 = if action.just_pressed(&PlayerAction::MoveRight) {
        1
    } else if action.just_pressed(&PlayerAction::MoveLeft) {
        -1
    } else {
        0
    };

    let dy: i32 = if action.just_pressed(&PlayerAction::MoveUp) {
        1
    } else if action.just_pressed(&PlayerAction::MoveDown) {
        -1
    } else {
        0
    };

    if dx == 1 {
        *facing = FacingDir::Right;
    } else if dx == -1 {
        *facing = FacingDir::Left;
    }

    if dx != 0 || dy != 0 {
        if dx > 0 {
            pos.x = pos.x.saturating_add(dx as u32);
        } else {
            pos.x = pos.x.saturating_sub(dx.unsigned_abs());
        }
        if dy > 0 {
            pos.y = pos.y.saturating_add(dy as u32);
        } else {
            pos.y = pos.y.saturating_sub(dy.unsigned_abs());
        }
    }
}
