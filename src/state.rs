use crate::{
    components::{CombatStats, Name, Position, Renderable},
    gui::{draw_ui, show_inventory, GameLog, ItemMenuResult},
    inventory_system::{ItemCollectionSystem, PotionSystem},
    map::Map,
    player::{self, Player},
    systems::{DamageSystem, MeleeCombatSystem, MonsterAI, VisibilitySystem},
};
use rltk::prelude::*;
use specs::prelude::*;

pub struct State {
    pub ecs: World,
}

impl State {
    fn run_systems(&mut self) {
        // visibility system
        {
            let mut vis = VisibilitySystem;
            vis.run_now(&self.ecs);
        }
        // monster ai
        {
            let mut monster_ai = MonsterAI;
            monster_ai.run_now(&self.ecs);
        }
        // melee combat
        {
            let mut melee_system = MeleeCombatSystem;
            melee_system.run_now(&self.ecs);
        }
        // damage system
        {
            let mut damage_system = DamageSystem;
            damage_system.run_now(&self.ecs);
        }
        // item collection
        {
            let mut item_collection_system = ItemCollectionSystem;
            item_collection_system.run_now(&self.ecs);
        }
        // potion system
        {
            let mut potion_system = PotionSystem;
            potion_system.run_now(&self.ecs);
        }
        self.ecs.maintain();
    }

    fn remove_dead(&mut self) {
        let dead: Vec<(Entity, (i32, i32))> = {
            let combat_stats = self.ecs.read_storage::<CombatStats>();
            let entities = self.ecs.entities();
            let positions = self.ecs.read_storage::<Position>();
            let players = self.ecs.read_storage::<Player>();
            let mut game_log = self.ecs.fetch_mut::<GameLog>();
            let names = self.ecs.read_storage::<Name>();
            (&entities, &combat_stats, &positions)
                .join()
                .filter_map(|(entity, stats, pos)| {
                    if stats.hp < 1 {
                        Some((entity, (pos.x, pos.y)))
                    } else {
                        None
                    }
                })
                .filter(|&(entity, _)| {
                    if let Some(_player) = players.get(entity) {
                        game_log.log("You are dead!".into());
                        false
                    } else {
                        let victim_name = names
                            .get(entity)
                            .map_or("Unnamed Victim", |victim_name| victim_name.name.as_str());
                        game_log.log(format!("{victim_name} is dead"));
                        true
                    }
                })
                .collect()
        };

        for (victim, _position) in &dead {
            self.ecs
                .delete_entity(*victim)
                .expect("Unable to delete dead entity");
        }

        let mut map = self.ecs.fetch_mut::<Map>();
        for (_victim, (x, y)) in dead {
            let idx = map.xy_idx(x, y);
            map.blocked[idx] = false;
        }
    }

    pub fn new() -> Self {
        Self { ecs: World::new() }
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        // rendering system
        ctx.cls();
        self.remove_dead();
        // render map
        {
            let map = self.ecs.fetch::<Map>();

            map.draw(ctx);
            ctx.print(1, 1, &format!("{}", ctx.fps));
            let positions = self.ecs.read_storage::<Position>();
            let renderables = self.ecs.read_storage::<Renderable>();
            for (pos, render) in (&positions, &renderables).join() {
                let idx = map.xy_idx(pos.x, pos.y);
                if map.visible_tiles[idx] {
                    ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph);
                }
            }
        }
        // run state machine
        {
            let runstate = *self.ecs.fetch::<RunState>();
            let new_runstate = match runstate {
                RunState::MonsterTurn | RunState::PreRun => {
                    self.run_systems();
                    RunState::AwaitingInput
                }
                RunState::AwaitingInput => player::input(self, ctx),
                RunState::PlayerTurn => {
                    self.run_systems();
                    RunState::MonsterTurn
                }
                RunState::ShowInventory => match show_inventory(&self.ecs, ctx) {
                    ItemMenuResult::NoResponse => RunState::ShowInventory,
                    ItemMenuResult::Cancel => RunState::AwaitingInput,
                    ItemMenuResult::Selected(_selected_entity) => {
                        // let names = self.ecs.read_storage::<Name>();
                        // let item_name = names
                        //     .get(selected_entity)
                        //     .map(|name| name.name.as_str())
                        //     .unwrap_or("Unnamed item");
                        // self.ecs
                        //     .fetch_mut::<GameLog>()
                        //     .log(format!("Trying to use {item_name}"));
                        RunState::PlayerTurn
                    }
                },
            };
            *self.ecs.write_resource::<RunState>() = new_runstate;
        }
        draw_ui(&self.ecs, ctx);
    }
}

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum RunState {
    AwaitingInput,
    PreRun,
    PlayerTurn,
    MonsterTurn,
    ShowInventory,
}
