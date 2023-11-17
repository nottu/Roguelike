use rltk::console;
use specs::prelude::*;

use crate::{
    components::{
        CombatStats, InBackpack, Name, Position, Potion, WantsToDrinkPotion, WantsToPickUp,
    },
    gui::GameLog,
    player::Player,
};

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

        //todo: remove unwrap?
        let (player_entity, _player) = (&entities, &players).join().next().unwrap();

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

pub struct PotionSystem;

impl<'a> System<'a> for PotionSystem {
    type SystemData = (
        Entities<'a>,
        WriteExpect<'a, GameLog>,
        WriteStorage<'a, WantsToDrinkPotion>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, Potion>,
        ReadStorage<'a, Player>,
        WriteStorage<'a, CombatStats>,
    );
    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut game_log, mut wants_drink, names, potions, players, mut combat_stats) =
            data;

        let player_entity = (&entities, &players)
            .join()
            .map(|(ent, _player)| ent)
            .next()
            .unwrap();
        for (entity, drink, stats) in (&entities, &wants_drink, &mut combat_stats).join() {
            let potion = potions.get(drink.potion);
            match potion {
                None => console::log(format!("Entity {:?} is not a potion!", drink.potion)),
                Some(potion) => {
                    let heal_ammout = stats.max_hp.min(stats.hp + potion.heal_amount) - stats.hp;
                    stats.hp += heal_ammout;
                    let potion_name = names
                        .get(drink.potion)
                        .map_or("Unnamed Potion", |name| name.name.as_str());
                    if entity == player_entity {
                        game_log.log(format!(
                            "You drink the {potion_name} and heal {heal_ammout}"
                        ));
                    }
                    entities.delete(drink.potion).expect("Failed to delete");
                }
            }
        }
        wants_drink.clear();
    }
}
