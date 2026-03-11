use bevy::prelude::*;

use crate::game::{assets::AssetsPlugin, player::PlayerPlugin};

pub mod assets;
pub mod player;

const ANIM_FPS: f32 = 6.0;
const TILE_SIZE: f32 = 16.0;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.init_state::<GameState>()
            .add_plugins((AssetsPlugin, PlayerPlugin))
            .add_systems(
                Update,
                (sync_transforms, animate_sprites).run_if(in_state(GameState::InGame)),
            );
    }
}

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum GameState {
    #[default]
    Loading,
    InGame,
}

#[derive(Debug, Component)]
pub struct Position {
    pub x: i32,
    pub y: i32,
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

fn sync_transforms(mut query: Query<(&Position, &mut Transform), Changed<Position>>) {
    for (pos, mut transform) in &mut query {
        transform.translation.x = pos.x as f32 * TILE_SIZE;
        transform.translation.y = pos.y as f32 * TILE_SIZE;
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
