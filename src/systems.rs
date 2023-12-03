use rltk::prelude::*;
use specs::prelude::*;

use crate::{
    components::{
        CombatStats, Confusion, Monster, Name, Position, SufferDamage, Viewshed, WantsToMelee,
    },
    gui::GameLog,
    map::Map,
    player::Player,
    state::RunState,
};

//
#[derive(Debug)]
pub struct VisibilitySystem;

impl<'a> System<'a> for VisibilitySystem {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, Player>,
        WriteExpect<'a, Map>,
        WriteStorage<'a, Viewshed>,
    );
    fn run(&mut self, data: Self::SystemData) {
        let (entities, positions, players, mut map, mut viewsheds) = data;
        for (entity, viewshed, pos) in (&entities, &mut viewsheds, &positions).join() {
            if !viewshed.dirty {
                continue;
            }
            viewshed.visible_tiles.clear();
            viewshed.visible_tiles = field_of_view(pos.into(), viewshed.range, &*map);
            viewshed
                .visible_tiles
                .retain(|p| p.x >= 0 && p.x < map.width && p.y >= 0 && p.y < map.height);

            // if entity is a player, mark tile as revealed
            if let Some(_p) = players.get(entity) {
                for t in &mut map.visible_tiles {
                    *t = false;
                }
                for vis in &viewshed.visible_tiles {
                    let idx = map.xy_idx(vis.x, vis.y);
                    map.revealed_tiles[idx] = true;
                    map.visible_tiles[idx] = true;
                }
            }
            viewshed.dirty = false;
        }
    }
}
//

//
#[derive(Debug)]
pub struct MonsterAI;

impl<'a> System<'a> for MonsterAI {
    type SystemData = (
        ReadExpect<'a, RunState>,
        WriteExpect<'a, Map>,
        WriteStorage<'a, Viewshed>,
        WriteStorage<'a, Position>,
        ReadStorage<'a, Monster>,
        ReadStorage<'a, Player>,
        ReadStorage<'a, Name>,
        WriteStorage<'a, WantsToMelee>,
        Entities<'a>,
        WriteStorage<'a, Confusion>,
    );
    fn run(&mut self, data: Self::SystemData) {
        let (
            runstate,
            mut map,
            mut viewsheds,
            mut positions,
            monsters,
            players,
            names,
            mut wants_to_melee,
            entities,
            mut confused,
        ) = data;
        if *runstate != RunState::MonsterTurn {
            return;
        }
        let Some((player_entity, player_pos)) = (&entities, &positions, &players)
            .join()
            .map(|(ent, pos, _)| (ent, Point::new(pos.x, pos.y)))
            .next()
        else {
            return;
        };

        for (viewshed, _, name, monster_pos, monster_entity) in
            (&mut viewsheds, &monsters, &names, &mut positions, &entities).join()
        {
            if let Some(confusion) = confused.get_mut(monster_entity) {
                println!(
                    "Monster {} confused for {} more turns",
                    name.name, confusion.turns
                );
                confusion.turns -= 1;
                if confusion.turns == 0 {
                    confused.remove(monster_entity);
                }
                continue;
            }
            if !viewshed.visible_tiles.contains(&player_pos) {
                continue;
            }

            let distance = rltk::DistanceAlg::Pythagoras.distance2d(
                Point::new(monster_pos.x, monster_pos.y),
                Point::new(player_pos.x, player_pos.y),
            );

            if distance < 1.5 {
                let fail_message = format!("Failed to add wants_to_melee for {}", name.name);
                wants_to_melee
                    .insert(
                        monster_entity,
                        WantsToMelee {
                            target: player_entity,
                        },
                    )
                    .expect(&fail_message);
                return;
            }

            // path towards player
            let path = {
                let monster_idx = map.xy_idx(monster_pos.x, monster_pos.y);
                let player_idx = map.xy_idx(player_pos.x, player_pos.y);

                rltk::a_star_search(monster_idx, player_idx, &*map)
            };
            if path.success && path.steps.len() > 1 {
                let idx = map.xy_idx(monster_pos.x, monster_pos.y);
                map.blocked[idx] = false;
                #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
                let path_steps = path.steps[1] as i32;
                monster_pos.x = path_steps % map.width;
                monster_pos.y = path_steps / map.width;
                let idx = map.xy_idx(monster_pos.x, monster_pos.y);
                map.blocked[idx] = true;
                viewshed.dirty = true;
            }
        }
    }
}
//

//
#[derive(Debug)]
pub struct MeleeCombatSystem;
impl<'a> System<'a> for MeleeCombatSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, WantsToMelee>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, CombatStats>,
        WriteStorage<'a, SufferDamage>,
        WriteExpect<'a, GameLog>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut wants_melee, names, combat_stats, mut inflict_damage, mut game_log) =
            data;

        for (_entity, melee, name, stats) in (&entities, &wants_melee, &names, &combat_stats).join()
        {
            let name = name.name.as_str();
            if stats.hp <= 0 {
                continue;
            }
            let Some(target_stats) = combat_stats.get(melee.target) else {
                console::log("Could not find melee target with combat stats!");
                continue;
            };
            if target_stats.hp <= 0 {
                continue;
            }
            let target_name = names
                .get(melee.target)
                .map_or("UNNAMED", |named| named.name.as_str());

            let damage = stats.power - target_stats.defense;

            match damage {
                1.. => {
                    SufferDamage::new_damage(&mut inflict_damage, melee.target, damage);
                    game_log
                        .entries
                        .push(format!("{name} hits {target_name}, for {damage} hp"));
                }
                _ => {
                    game_log
                        .entries
                        .push(format!("{name} is unable to hurt {target_name}"));
                }
            }
        }

        wants_melee.clear();
    }
}

//
#[derive(Debug)]
pub struct DamageSystem;
impl<'a> System<'a> for DamageSystem {
    type SystemData = (
        WriteStorage<'a, CombatStats>,
        WriteStorage<'a, SufferDamage>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut stats, mut suffered_damage) = data;
        for (stats, suffered_damage) in (&mut stats, &mut suffered_damage).join() {
            stats.hp -= suffered_damage.amount.iter().sum::<i32>();
        }
        suffered_damage.clear();
    }
}
