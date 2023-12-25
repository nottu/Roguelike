#![warn(clippy::all, clippy::pedantic, clippy::nursery)]

use gui::GameLog;
use player::Player;
use rltk::prelude::*;
use specs::{
    prelude::*,
    saveload::{SimpleMarker, SimpleMarkerAllocator},
};

mod clean_up_systems;
mod components;
mod gui;
mod inventory_system;
mod map;
mod player;
mod save_load_systems;
mod spawner;
mod state;
mod systems;

use components::{
    AreaOfEffect, BlockedTile, CombatStats, Confusion, Consumable, FilePersistent, InBackpack,
    InflictsDamage, Item, Monster, Name, Position, ProvidesHealing, Ranged, Renderable,
    SufferDamage, ToDelete, Viewshed, WantsToDropItem, WantsToMelee, WantsToPickUp, WantsToUseItem,
};
use spawner::spawn_player;
use state::State;

fn main() -> BError {
    let context = {
        let mut ctx = RltkBuilder::simple80x50()
            .with_title("Roguelike Tutorial")
            .build()?;
        ctx.with_post_scanlines(true);
        ctx
    };

    let mut gs = State::new();
    register_components(&mut gs.ecs);
    // Insert GameLog
    let mut game_log = GameLog::new();
    game_log.log("Welcome to Rusty Roguelike".to_string());
    gs.ecs.insert(game_log);

    // put player in any position, it will get set on map creation
    let player_entity = spawn_player(&mut gs.ecs, Position { x: 0, y: 0 });
    gs.ecs.insert(player_entity);
    let rng = RandomNumberGenerator::new();
    gs.ecs.insert(rng);

    let map = gs
        .spawn_map_with_enemies(1)
        .expect("Failed building initial map");
    gs.ecs.insert(map);

    main_loop(context, gs)
}

fn register_components(ecs: &mut World) {
    ecs.register::<Position>();
    ecs.register::<Renderable>();
    ecs.register::<Player>();
    ecs.register::<Viewshed>();
    ecs.register::<Monster>();
    ecs.register::<Name>();
    ecs.register::<BlockedTile>();
    ecs.register::<CombatStats>();
    ecs.register::<WantsToMelee>();
    ecs.register::<SufferDamage>();

    ecs.register::<ProvidesHealing>();
    ecs.register::<Item>();
    ecs.register::<Consumable>();

    ecs.register::<Ranged>();
    ecs.register::<InflictsDamage>();
    ecs.register::<AreaOfEffect>();
    ecs.register::<Confusion>();

    ecs.register::<InBackpack>();
    ecs.register::<WantsToPickUp>();
    ecs.register::<WantsToUseItem>();
    ecs.register::<WantsToDropItem>();

    ecs.register::<SimpleMarker<FilePersistent>>();
    ecs.insert(SimpleMarkerAllocator::<FilePersistent>::default());
    ecs.register::<save_load_systems::SerializationHelper>();

    ecs.register::<ToDelete>();
}
