use rltk::console;
use specs::prelude::*;

use crate::{
    components::{
        AreaOfEffect, CombatStats, Confusion, Consumable, InBackpack, InflictsDamage, Name,
        Position, ProvidesHealing, SufferDamage, WantsToDropItem, WantsToPickUp, WantsToUseItem,
    },
    gui::GameLog,
    map::Map,
    player::Player,
};

#[derive(Debug)]
pub struct ItemCollectionSystem;

impl<'a> System<'a> for ItemCollectionSystem {
    type SystemData = (
        Entities<'a>,
        WriteExpect<'a, GameLog>,
        WriteStorage<'a, WantsToPickUp>,
        WriteStorage<'a, Position>,
        ReadStorage<'a, Name>,
        WriteStorage<'a, InBackpack>,
        ReadStorage<'a, Player>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            entities,
            mut game_log,
            mut wants_to_pickup,
            mut positions,
            names,
            mut backpack,
            players,
        ) = data;

        let Some((player_entity, _player)) = (&entities, &players).join().next() else {
            panic!("Failed to find Player entity!");
        };

        for (picking_entity, pickup) in (&entities, &wants_to_pickup).join() {
            positions.remove(pickup.item);
            backpack
                .insert(
                    pickup.item,
                    InBackpack {
                        owner: picking_entity,
                    },
                )
                .expect("Failed to pickup item");

            if picking_entity == player_entity {
                let name = names
                    .get(pickup.item)
                    .map_or("Unnamed item", |name| name.name.as_str());
                game_log.log(format!("You picked up the {name}"));
            } else {
                game_log.log("??".to_string());
            }
        }
        wants_to_pickup.clear();
    }
}

#[derive(Debug)]
pub struct ItemUseSystem;
// todo: Split Item Use system
impl<'a> System<'a> for ItemUseSystem {
    type SystemData = (
        Entities<'a>,
        WriteExpect<'a, GameLog>,
        WriteStorage<'a, WantsToUseItem>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, ProvidesHealing>,
        ReadStorage<'a, Player>,
        WriteStorage<'a, CombatStats>,
        ReadStorage<'a, Consumable>,
        ReadStorage<'a, InflictsDamage>,
        WriteStorage<'a, Confusion>,
        ReadStorage<'a, Position>,
        ReadExpect<'a, Map>,
        WriteStorage<'a, SufferDamage>,
        ReadStorage<'a, AreaOfEffect>,
    );
    fn run(&mut self, data: Self::SystemData) {
        let (
            entities,
            mut game_log,
            mut wants_to_use,
            names,
            healing_items,
            players,
            mut combat_stats,
            consumables,
            inflict_damage,
            mut confused,
            positions,
            map,
            mut suffer_damage,
            area_of_effect,
        ) = data;
        // println!("{}", entities.join().count());
        let Some((player_entity, _player, _player_position)) =
            (&entities, &players, &positions).join().next()
        else {
            panic!("Failed to find Player entity!");
        };
        for (entity, use_item) in (&entities, &wants_to_use).join() {
            let affected_entities =
                get_affected_entities(&area_of_effect, &positions, &combat_stats, &map, use_item);
            let effect_data = &ItemEffectData {
                used_item: use_item.item,
                affected_entities: &affected_entities,
                player_entity,
                using_entity: entity,
            };
            // using a healing item..
            run_healing_effects(
                &healing_items,
                &names,
                &mut combat_stats,
                effect_data,
                &mut game_log,
            );

            run_damage_effects(
                &inflict_damage,
                &names,
                &mut suffer_damage,
                effect_data,
                &mut game_log,
            );

            run_confuse_effects(&mut confused, &names, effect_data, &mut game_log);

            // remove one time use items aka: consumables
            if consumables.get(use_item.item).is_some() {
                // println!("Removing Consumable");
                entities.delete(use_item.item).expect("Failed to delete");
            }
            // remove target item if it doesn't have combat stats,
            // this is because when we create the `WantsToUse` Component
            // we set it's target entity as the user (player) when not Ranged
            // and when Ranged the target entity is a new enitity with Position
            if combat_stats.get(use_item.target).is_none() {
                entities
                    .delete(use_item.target)
                    .expect("Failed to delete target posistion entity");
            }
        }
        wants_to_use.clear();
    }
}

/// Handy helper struct that has data regarding who to apply effects to
struct ItemEffectData<'a> {
    used_item: Entity,
    affected_entities: &'a [Entity],
    player_entity: Entity,
    using_entity: Entity,
}

fn run_healing_effects(
    healing_items: &ReadStorage<ProvidesHealing>,
    names: &ReadStorage<Name>,
    combat_stats: &mut WriteStorage<CombatStats>,
    effect_data: &ItemEffectData,
    game_log: &mut WriteExpect<GameLog>,
) {
    let Some(healing_item) = healing_items.get(effect_data.used_item) else {
        return;
    };
    let item_name = names
        .get(effect_data.used_item)
        .map_or("Unnamed Potion", |name| name.name.as_str());

    println!(
        "Using Healing on {} entities",
        effect_data.affected_entities.len()
    );
    for affected_entity in effect_data.affected_entities {
        let Some(stats) = combat_stats.get_mut(*affected_entity) else {
            continue;
        };
        let entity_name = names
            .get(*affected_entity)
            .map_or("Unnamed Entity", |name| name.name.as_str());
        let heal_ammout = stats.max_hp.min(stats.hp + healing_item.heal_amount) - stats.hp;
        stats.hp += heal_ammout;
        if effect_data.using_entity == effect_data.player_entity {
            game_log.log(format!(
                "You consume the {item_name} to heal {entity_name} for {heal_ammout}"
            ));
        }
    }
}

fn run_damage_effects(
    inflict_damage: &ReadStorage<InflictsDamage>,
    names: &ReadStorage<Name>,
    suffer_damage: &mut WriteStorage<SufferDamage>,
    effect_data: &ItemEffectData,
    game_log: &mut WriteExpect<GameLog>,
) {
    // using a damaging item
    let Some(damage_item) = inflict_damage.get(effect_data.used_item) else {
        return;
    };
    let item_name = names
        .get(effect_data.used_item)
        .map_or("Unnamed Damaging Item", |name| name.name.as_str());

    // println!("Using Damage Item");
    for affected_entity in effect_data.affected_entities {
        if *affected_entity == effect_data.using_entity {
            println!("Don't harm caster");
            continue;
        }
        let entity_name = names
            .get(*affected_entity)
            .map_or("Unnamed Entity", |name| name.name.as_str());
        SufferDamage::new_damage(suffer_damage, *affected_entity, damage_item.damage);
        if effect_data.using_entity == effect_data.player_entity {
            game_log.log(format!(
                "You use {item_name} on {entity_name} inflicting {} hp",
                damage_item.damage
            ));
        }
    }
}

fn run_confuse_effects(
    confused: &mut WriteStorage<Confusion>,
    names: &ReadStorage<Name>,
    effect_data: &ItemEffectData,
    game_log: &mut WriteExpect<GameLog>,
) {
    let Some(confuse_item) = confused.get(effect_data.used_item) else {
        return;
    };
    let confuse_item = confuse_item.to_owned();
    let item_name = names
        .get(effect_data.used_item)
        .map_or("Unnamed Damaging Item", |name| name.name.as_str());

    // println!("Using Damage Item");
    for affected_entity in effect_data.affected_entities {
        if *affected_entity == effect_data.using_entity {
            // println!("Don't harm caster");
            continue;
        }
        let entity_name = names
            .get(*affected_entity)
            .map_or("Unnamed Entity", |name| name.name.as_str());
        confused.insert(*affected_entity, confuse_item).expect("h");
        if effect_data.using_entity == effect_data.player_entity {
            game_log.log(format!(
                "You use {item_name} on {entity_name} confusing them fo {} turns",
                confuse_item.turns
            ));
        }
    }
}

fn get_affected_entities(
    area_of_effect: &ReadStorage<AreaOfEffect>,
    positions: &ReadStorage<Position>,
    // since caller has a write storage we use write storage
    // we need to check if we can cast WriteStorage to ReadStorage
    combat_stats: &WriteStorage<CombatStats>,
    map: &Map,
    use_item: &WantsToUseItem,
) -> Vec<Entity> {
    let effect_position = positions.get(use_item.target).unwrap();
    // We are not preventing friendly fire!
    area_of_effect.get(use_item.item).map_or_else(
        || {
            let idx = map.xy_idx(effect_position.x, effect_position.y);
            map.tile_content[idx]
                .iter()
                .filter(|&e| combat_stats.contains(*e))
                .map(std::borrow::ToOwned::to_owned)
                .collect()
        },
        |aoe| {
            rltk::field_of_view(effect_position.into(), aoe.radius, map)
                .into_iter()
                .filter(|p| p.x > 0 && p.x < map.width - 1 && p.y > 0 && p.y < map.height - 1)
                .flat_map(|p| {
                    let idx = map.xy_idx(p.x, p.y);
                    map.tile_content[idx]
                        .iter()
                        .filter(|&e| combat_stats.contains(*e))
                        .map(std::borrow::ToOwned::to_owned)
                })
                .collect()
        },
    )
}

#[derive(Debug)]
pub struct ItemDropSystem;

impl<'a> System<'a> for ItemDropSystem {
    type SystemData = (
        Entities<'a>,
        WriteExpect<'a, GameLog>,
        WriteStorage<'a, WantsToDropItem>,
        ReadStorage<'a, Name>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, InBackpack>,
        ReadStorage<'a, Player>,
    );
    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut game_log, mut wants_drop, names, mut positions, mut backpack, players) =
            data;

        let Some((player_entity, _player)) = (&entities, &players).join().next() else {
            panic!("Failed to find Player entity!");
        };
        for (entity, to_drop) in (&entities, &wants_drop).join() {
            // println!("Droping something");
            let Some(dropper_pos) = positions.get(entity).map(std::borrow::ToOwned::to_owned)
            else {
                console::log("Cannot get dropper position");
                return;
            };
            positions
                .insert(to_drop.item, dropper_pos)
                .expect("Failed to drop at position");
            backpack.remove(to_drop.item);

            if entity == player_entity {
                let item_name = names
                    .get(to_drop.item)
                    .map_or("Unnamed Item", |name| name.name.as_str());
                game_log.log(format!("You droped the {item_name}"));
            }
        }
        wants_drop.clear();
    }
}
