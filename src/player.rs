use rltk::prelude::*;
use specs::prelude::*;
use specs_derive::Component;
use std::cmp;

use crate::{
    components::{CombatStats, Item, Name, Position, Viewshed, WantsToMelee, WantsToPickUp},
    gui::GameLog,
    map::Map,
    state::{RunState, State},
};

#[derive(Debug, Component)]
pub struct Player;

fn move_player(delta_x: i32, delta_y: i32, ecs: &mut World) {
    let mut positions = ecs.write_storage::<Position>();
    let players = ecs.read_storage::<Player>();
    let entities = ecs.entities();
    let mut viewsheds = ecs.write_storage::<Viewshed>();
    let combat_stats = ecs.read_storage::<CombatStats>();
    let mut wants_to_melee = ecs.write_storage::<WantsToMelee>();
    let map = ecs.fetch::<Map>();

    for (_player, pos, viewshed, player_entity) in
        (&players, &mut positions, &mut viewsheds, &entities).join()
    {
        let new_x = cmp::min(79, cmp::max(0, pos.x + delta_x));
        let new_y = cmp::min(49, cmp::max(0, pos.y + delta_y));
        let destination_idx = map.xy_idx(new_x, new_y);

        if !map.blocked[destination_idx] {
            pos.x = new_x;
            pos.y = new_y;

            viewshed.dirty = true;
        } else if let Some(potential_target) = map.tile_content[destination_idx] {
            match combat_stats.get(potential_target) {
                Some(_t) => {
                    wants_to_melee
                        .insert(
                            player_entity,
                            WantsToMelee {
                                target: potential_target,
                            },
                        )
                        .expect("Failed to insert WantsToMelee");
                }
                None => console::log("cant attack"),
            }
        }
    }
}

fn get_item(ecs: &mut World) {
    let players = ecs.read_storage::<Player>();
    let positions = ecs.read_storage::<Position>();
    let entities = ecs.entities();
    let items = ecs.read_storage::<Item>();
    let names = ecs.read_storage::<Name>();
    let mut game_log = ecs.fetch_mut::<GameLog>();

    let Some((_player, player_entity, player_pos)) =
        (&players, &entities, &positions).join().next()
    else {
        console::log("unable to fetch player");
        return;
    };

    let target_item = (&entities, &items, &positions, &names)
        .join()
        .filter(|&(_ent, _item, item_positon, _item_name)| {
            player_pos.x == item_positon.x && player_pos.y == item_positon.y
        })
        .map(|(item_entity, _item, _pos, item_name)| (item_entity, item_name.name.as_str()))
        .next();

    match target_item {
        None => game_log.log("There is nothing to pick up".to_string()),
        Some((item_entity, item_name)) => {
            let mut pickup = ecs.write_storage::<WantsToPickUp>();
            pickup
                .insert(player_entity, WantsToPickUp { item: item_entity })
                .expect("Failed to write to WantsToPickUp");
            game_log.log(format!("WantsToPickUp {item_name}"));
        }
    }
}

pub fn input(gs: &mut State, ctx: &mut Rltk) -> RunState {
    let Some(pressed_key) = ctx.key else {
        return RunState::AwaitingInput;
    };

    match pressed_key {
        rltk::VirtualKeyCode::Left => move_player(-1, 0, &mut gs.ecs),
        rltk::VirtualKeyCode::Right => move_player(1, 0, &mut gs.ecs),
        rltk::VirtualKeyCode::Up => move_player(0, -1, &mut gs.ecs),
        rltk::VirtualKeyCode::Down => move_player(0, 1, &mut gs.ecs),
        rltk::VirtualKeyCode::G => get_item(&mut gs.ecs),
        rltk::VirtualKeyCode::I => return RunState::ShowInventory,
        _ => return RunState::AwaitingInput,
    };
    RunState::PlayerTurn
}
