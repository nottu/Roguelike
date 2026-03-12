use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

use crate::game::assets::SpriteAssets;
use crate::game::map::TileKind;
use crate::game::*;

// Hero 0 body animations (cols 0-2, various rows)
const IDLE_FRAMES: &[usize] = &[0, 1, 2];

#[derive(Actionlike, PartialEq, Eq, Hash, Clone, Copy, Debug, Reflect)]
pub enum PlayerAction {
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
    //
    Attack,
}

impl PlayerAction {
    fn direction(&self) -> Option<IVec2> {
        match self {
            Self::MoveRight => Some(IVec2::X),
            Self::MoveLeft => Some(IVec2::NEG_X),
            Self::MoveUp => Some(IVec2::Y),
            Self::MoveDown => Some(IVec2::NEG_Y),
            _ => None,
        }
    }
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
    tilemap_q: Query<(&TileStorage, &TilemapSize)>,
    tile_kind_q: Query<&TileKind>,
) {
    let Ok((action, mut pos, mut facing)) = query.single_mut() else {
        return;
    };
    let Ok((storage, map_size)) = tilemap_q.single() else {
        return;
    };

    let Some(delta) = action
        .get_just_pressed()
        .iter()
        .find_map(PlayerAction::direction)
    else {
        return;
    };

    if delta.x > 0 {
        *facing = FacingDir::Right;
    } else if delta.x < 0 {
        *facing = FacingDir::Left;
    }

    let dest = TilePos {
        x: pos
            .x
            .saturating_add_signed(delta.x)
            .min(map_size.x.saturating_sub(1)),
        y: pos
            .y
            .saturating_add_signed(delta.y)
            .min(map_size.y.saturating_sub(1)),
    };

    let walkable = storage
        .get(&dest)
        .and_then(|e| tile_kind_q.get(e).ok())
        .is_some_and(|kind| *kind != TileKind::Wall);

    if walkable {
        pos.x = dest.x;
        pos.y = dest.y;
    }
}
