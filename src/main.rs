#![warn(clippy::all, clippy::pedantic)]

use gui::GameLog;
use player::Player;
use rltk::prelude::*;
use specs::prelude::*;

mod components;
mod gui;
mod inventory_system;
mod map;
mod player;
mod spawner;
mod state;
mod systems;

use components::{BlockedTile, CombatStats, InBackpack, Item, Monster, Name, Position, Potion, Renderable, SufferDamage, Viewshed, WantsToDrinkPotion, WantsToMelee, WantsToPickUp};
use map::Map;
use spawner::{spawn_player, spawn_room};
use state::{RunState, State};

fn main() -> rltk::BError {
    let context = {
        let mut ctx = RltkBuilder::simple80x50()
            .with_title("Rougelike Tutorial")
            .build()?;
        ctx.with_post_scanlines(true);
        ctx
    };

    let mut gs = State::new();
    register_components(&mut gs.ecs);

    let mut rng = rltk::RandomNumberGenerator::new();
    let map = Map::new_map_rooms_and_corridors(&mut rng);
    gs.ecs.insert(rng);

    let Some((x, y)) = map.rooms.first().map(map::Rect::center) else {
        return Err("NO FIRST ROOM".into());
    };
    // Create and keep track of player entity
    let player_entity = spawn_player(&mut gs.ecs, Position { x, y });
    gs.ecs.insert(player_entity);

    for room in map.rooms.iter().skip(1) {
        spawn_room(&mut gs.ecs, room);
    }

    gs.ecs.insert(map);
    gs.ecs.insert(RunState::PreRun);

    // Insert GameLog
    let mut game_log = GameLog::new();
    game_log.log("Welcome to Rusty Roguelike".to_string());
    gs.ecs.insert(game_log);
    //

    rltk::main_loop(context, gs)
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

    ecs.register::<Potion>();
    ecs.register::<Item>();
    ecs.register::<InBackpack>();
    ecs.register::<WantsToPickUp>();
    ecs.register::<WantsToDrinkPotion>();
}
