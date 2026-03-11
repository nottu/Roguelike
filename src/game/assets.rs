use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

use crate::game::GameState;

#[allow(unused)]
#[derive(AssetCollection, Resource)]
pub struct SpriteAssets {
    // --- Dungeon tiles (12 x 25) ---
    #[asset(texture_atlas_layout(tile_size_x = 16, tile_size_y = 16, columns = 12, rows = 25))]
    pub dungeon_layout: Handle<TextureAtlasLayout>,
    #[asset(image(sampler(filter = nearest)))]
    #[asset(path = "sprites/dungeon.png")]
    pub dungeon: Handle<Image>,

    // --- Cave tiles (10 x 19) ---
    #[asset(texture_atlas_layout(tile_size_x = 16, tile_size_y = 16, columns = 10, rows = 19))]
    pub cave_layout: Handle<TextureAtlasLayout>,
    #[asset(image(sampler(filter = nearest)))]
    #[asset(path = "sprites/cave.png")]
    pub cave: Handle<Image>,

    // --- Water tiles (18 x 27) ---
    #[asset(texture_atlas_layout(tile_size_x = 16, tile_size_y = 16, columns = 18, rows = 27))]
    pub water_layout: Handle<TextureAtlasLayout>,
    #[asset(image(sampler(filter = nearest)))]
    #[asset(path = "sprites/water.png")]
    pub water: Handle<Image>,

    // --- Heroes (23 x 88) ---
    #[asset(texture_atlas_layout(tile_size_x = 16, tile_size_y = 16, columns = 23, rows = 88))]
    pub heroes_layout: Handle<TextureAtlasLayout>,
    #[asset(image(sampler(filter = nearest)))]
    #[asset(path = "sprites/heroes.png")]
    pub heroes: Handle<Image>,

    // --- Monsters (11 x 8) ---
    #[asset(texture_atlas_layout(tile_size_x = 16, tile_size_y = 16, columns = 11, rows = 8))]
    pub monsters_layout: Handle<TextureAtlasLayout>,
    #[asset(image(sampler(filter = nearest)))]
    #[asset(path = "sprites/monsters.png")]
    pub monsters: Handle<Image>,

    // --- Items (23 x 38) ---
    #[asset(texture_atlas_layout(tile_size_x = 16, tile_size_y = 16, columns = 23, rows = 38))]
    pub items_layout: Handle<TextureAtlasLayout>,
    #[asset(image(sampler(filter = nearest)))]
    #[asset(path = "sprites/items.png")]
    pub items: Handle<Image>,

    // --- Particles (2 x 3) ---
    #[asset(texture_atlas_layout(tile_size_x = 16, tile_size_y = 16, columns = 2, rows = 3))]
    pub particles_layout: Handle<TextureAtlasLayout>,
    #[asset(image(sampler(filter = nearest)))]
    #[asset(path = "sprites/particles.png")]
    pub particles: Handle<Image>,
}

pub struct AssetsPlugin;

impl Plugin for AssetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_loading_state(
            LoadingState::new(GameState::Loading)
                .continue_to_state(GameState::InGame)
                .load_collection::<SpriteAssets>(),
        );
    }
}
