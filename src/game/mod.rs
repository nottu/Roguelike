use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

use crate::game::{assets::AssetsPlugin, map::MapPlugin, player::PlayerPlugin};

pub use bevy_ecs_tilemap::prelude::TilePos;
pub mod assets;
pub mod map;
pub mod player;

const ANIM_FPS: f32 = 6.0;
const TILE_SIZE: f32 = 16.0;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.init_state::<GameState>()
            .add_plugins((AssetsPlugin, MapPlugin, PlayerPlugin))
            .add_systems(
                Update,
                (sync_transforms, sync_facing, animate_sprites)
                    .chain()
                    .run_if(in_state(GameState::InGame)),
            );
    }
}

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum GameState {
    #[default]
    Loading,
    InGame,
}

#[derive(Debug, Component, PartialEq, Eq, Default)]
pub enum FacingDir {
    #[default]
    Right,
    Left,
}

#[derive(Component)]
struct SpriteAnimation {
    frames: &'static [usize],
    current: usize,
    timer: Timer,
}

impl SpriteAnimation {
    fn new(frames: &'static [usize]) -> Self {
        Self {
            frames,
            current: 0,
            timer: Timer::from_seconds(1.0 / ANIM_FPS, TimerMode::Repeating),
        }
    }
}

fn sync_facing(mut query: Query<(&FacingDir, &mut Sprite), Changed<FacingDir>>) {
    for (facing, mut sprite) in &mut query {
        sprite.flip_x = *facing == FacingDir::Left;
    }
}

fn sync_transforms(
    mut query: Query<(&TilePos, &mut Transform), Or<(Changed<TilePos>, Added<TilePos>)>>,
    tilemap_q: Query<(
        &TilemapSize,
        &TilemapGridSize,
        &TilemapTileSize,
        &TilemapType,
        &TilemapAnchor,
    )>,
) {
    let Ok((map_size, grid_size, tile_size, map_type, anchor)) = tilemap_q.single() else {
        info!("sync_transforms: no tilemap found");
        return;
    };

    for (tile_pos, mut transform) in &mut query {
        let world_pos = tile_pos.center_in_world(map_size, grid_size, tile_size, map_type, anchor);
        let z = transform.translation.z;
        info!(
            "Moving player at {:?} to world pos {:?}",
            tile_pos, world_pos
        );
        transform.translation = world_pos.extend(z);
    }
}

fn animate_sprites(time: Res<Time>, mut query: Query<(&mut SpriteAnimation, &mut Sprite)>) {
    for (mut anim, mut sprite) in &mut query {
        anim.timer.tick(time.delta());
        if anim.timer.just_finished() {
            anim.current = (anim.current + 1) % anim.frames.len();
            if let Some(atlas) = &mut sprite.texture_atlas {
                atlas.index = anim.frames[anim.current];
            }
        }
    }
}
