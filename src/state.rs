use crate::{
    clean_up_systems::CleanUpSystem,
    components::{
        CombatStats, InBackpack, Name, Position, Ranged, Renderable, ToDelete, Viewshed,
        WantsToDropItem, WantsToUseItem,
    },
    gui::{
        self, draw_ui, drop_item_menu, ranged_target, show_inventory, GameLog, ItemMenuResult,
        MainMenuItem,
    },
    inventory_system::{ItemCollectionSystem, ItemDropSystem, ItemUseSystem},
    map::{self, Map},
    player::{self, fetch_player_entity, Player},
    save_load_systems::{delete_save, load_game, save_game},
    spawner,
    systems::{DamageSystem, MeleeCombatSystem, MonsterAI, VisibilitySystem},
};
use rltk::prelude::*;
use specs::prelude::*;

pub struct State {
    pub ecs: World,
}

impl State {
    pub fn new() -> Self {
        let mut ecs = World::new();
        ecs.insert(RunState::PreRun);
        ecs.insert(AppState::new_main_menu());
        Self { ecs }
    }

    fn run_systems(&mut self) {
        // remove marked entities
        {
            let mut clean_up_system = CleanUpSystem;
            clean_up_system.run_now(&self.ecs);
        }
        // visibility system
        {
            let mut system = VisibilitySystem;
            //dbg!((&system, self.ecs.read_storage::<Player>().count()));
            system.run_now(&self.ecs);
            // self.ecs.maintain();
        }
        // monster ai
        {
            let mut system = MonsterAI;
            //dbg!((&system, self.ecs.read_storage::<Player>().count()));
            system.run_now(&self.ecs);
            // self.ecs.maintain();
        }
        // melee combat
        {
            let mut system = MeleeCombatSystem;
            //dbg!((&system, self.ecs.read_storage::<Player>().count()));
            system.run_now(&self.ecs);
            // self.ecs.maintain();
        }
        // damage system
        {
            let mut system = DamageSystem;
            //dbg!((&system, self.ecs.read_storage::<Player>().count()));
            system.run_now(&self.ecs);
            // self.ecs.maintain();
        }
        // item collection
        {
            let mut system = ItemCollectionSystem;
            //dbg!((&system, self.ecs.read_storage::<Player>().count()));
            system.run_now(&self.ecs);
            // self.ecs.maintain();
        }
        // potion system
        {
            let mut system = ItemUseSystem;
            //dbg!((&system, self.ecs.read_storage::<Player>().count()));
            system.run_now(&self.ecs);
            // self.ecs.maintain();
        }
        // item droping
        {
            let mut system = ItemDropSystem;
            //dbg!((&system, self.ecs.read_storage::<Player>().count()));
            system.run_now(&self.ecs);
            // self.ecs.maintain();
        }
        // map position system, what entity is in each tile
        {
            let mut system = map::PositionUpdateSystem;
            system.run_now(&self.ecs);
        }
        self.ecs.maintain();
    }

    fn remove_dead(&mut self) {
        let get_dead = || {
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
        let dead: Vec<(Entity, (i32, i32))> = get_dead();

        // remove dead entities
        for (victim, _position) in &dead {
            self.ecs
                .delete_entity(*victim)
                .expect("Unable to delete dead entity");
        }

        // remove blockers from map
        let mut map = self.ecs.fetch_mut::<Map>();
        for (_victim, (x, y)) in dead {
            let idx = map.xy_idx(x, y);
            map.blocked[idx] = false;
        }
    }

    fn render_map(&mut self, ctx: &mut Rltk) {
        let map = self.ecs.fetch::<Map>();

        map.draw(ctx);
        let positions = self.ecs.read_storage::<Position>();
        let renderables = self.ecs.read_storage::<Renderable>();
        let mut render_data: Vec<_> = (&positions, &renderables).join().collect();
        render_data.sort_by_key(|(_p, renderable)| -renderable.render_order);
        for (pos, render) in render_data {
            let idx = map.xy_idx(pos.x, pos.y);
            if map.visible_tiles[idx] {
                ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph);
            }
        }
    }

    fn render_debug_info(&mut self, ctx: &mut Rltk) {
        let runstate = *self.ecs.fetch::<RunState>();
        ctx.print(1, 1, &format!("FPS: {}", ctx.fps));
        ctx.print(1, 2, &format!("RunState: {runstate:?}"));
    }

    // state machine logic...
    fn next_game_state(&mut self, ctx: &mut Rltk) -> RunState {
        let runstate = *self.ecs.fetch::<RunState>();
        match runstate {
            // todo: `PreRun` state should init map & player
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
                ItemMenuResult::Selected(item) => {
                    let ranged_target = self.ecs.read_storage::<Ranged>();
                    if let Some(ranged_item) = ranged_target.get(item) {
                        RunState::ShowTargeting {
                            range: ranged_item.range,
                            item,
                        }
                    } else {
                        let player_entity = fetch_player_entity(&self.ecs);
                        self.ecs
                            .write_storage::<WantsToUseItem>()
                            .insert(
                                player_entity,
                                WantsToUseItem {
                                    item,
                                    target: player_entity,
                                },
                            )
                            .expect("Failed to use item");
                        RunState::PlayerTurn
                    }
                }
            },
            RunState::ShowDropItem => match drop_item_menu(&self.ecs, ctx) {
                ItemMenuResult::NoResponse => RunState::ShowDropItem,
                ItemMenuResult::Cancel => RunState::AwaitingInput,
                ItemMenuResult::Selected(item) => {
                    let player_entity = fetch_player_entity(&self.ecs);
                    self.ecs
                        .write_storage::<WantsToDropItem>()
                        .insert(player_entity, WantsToDropItem { item })
                        .expect("Failed to WantsToDropItem");
                    RunState::PlayerTurn
                }
            },
            RunState::ShowTargeting { range, item } => {
                // needs mut reference since it creates new entity at click position.
                match ranged_target(&mut self.ecs, ctx, range) {
                    ItemMenuResult::NoResponse => RunState::ShowTargeting { range, item },
                    ItemMenuResult::Cancel => RunState::AwaitingInput,
                    // the target entity has a position attached
                    ItemMenuResult::Selected(target) => {
                        let player_entity = fetch_player_entity(&self.ecs);
                        self.ecs
                            .write_storage::<WantsToUseItem>()
                            .insert(player_entity, WantsToUseItem { item, target })
                            .expect("Failed to use targeted item");
                        RunState::PlayerTurn
                    }
                }
            }
            RunState::ShowMenu => {
                *self.ecs.fetch_mut::<AppState>() = AppState::new_pause_menu();
                RunState::PreRun
            }
            RunState::NextLevel => {
                self.ecs
                    .fetch_mut::<GameLog>()
                    .log("Loading Next Level".to_string());
                self.mark_entities_to_remove_on_level_change();
                self.load_new_level();
                RunState::PreRun
            }
        }
    }

    // level change logic
    fn mark_entities_to_remove_on_level_change(&mut self) {
        let entities = self.ecs.entities();
        let backpack = self.ecs.read_storage::<InBackpack>();
        let player = player::fetch_player_entity(&self.ecs);
        let mut positions = self.ecs.write_storage::<Position>();
        let mut to_delete = self.ecs.write_storage::<ToDelete>();

        entities
            .join()
            .filter(|&entity| {
                let in_player_backpack = backpack
                    .get(entity)
                    .map(|bp| bp.owner == player)
                    .unwrap_or_default();
                !(entity == player || in_player_backpack)
            })
            .for_each(|to_delete_entity| {
                to_delete
                    .insert(to_delete_entity, ToDelete)
                    .expect("Failed marking entity to delete");
                positions.remove(to_delete_entity);
            });
    }

    fn load_new_level(&mut self) {
        let curr_depth = self.ecs.read_resource::<Map>().depth;
        let new_map = self
            .spawn_map_with_enemies(curr_depth + 1)
            .expect("Failed to load new level");
        *self.ecs.write_resource::<Map>() = new_map;
    }
    // todo: find a better place for this
    pub fn spawn_map_with_enemies(&mut self, depth: i32) -> Result<Map, String> {
        let map = {
            let mut rng = self.ecs.fetch_mut::<RandomNumberGenerator>();
            Map::new_map_rooms_and_corridors(&mut rng, depth)
        };

        // should Player positioning be done as part of the pre-run state?
        if let Some((x, y)) = map.rooms.first().map(map::Rect::center) {
            self.ecs
                .fetch_mut::<GameLog>()
                .log(format!("You descend to level {depth}"));
            let mut positions = self.ecs.write_storage::<Position>();
            let player = self.ecs.read_storage::<Player>();
            let mut view_shed = self.ecs.write_storage::<Viewshed>();
            let mut stats = self.ecs.write_storage::<CombatStats>();
            (&player, &mut positions, &mut view_shed, &mut stats)
                .join()
                .for_each(|(_player, position, view_shed, stats)| {
                    //only one player
                    position.x = x;
                    position.y = y;
                    view_shed.dirty = true;
                    stats.hp = stats.hp.max(stats.max_hp / 2);
                });
        } else {
            return Err("NO FIRST ROOM".into());
        };

        for room in map.rooms.iter().skip(1) {
            spawner::spawn_room(&mut self.ecs, room);
        }
        Ok(map)
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        // rendering system
        ctx.cls();
        // probably not very efficient..
        // need to find a better way that does not require a clone!
        let prev_state = (*self.ecs.fetch::<AppState>()).clone();
        *self.ecs.fetch_mut::<AppState>() = match prev_state {
            AppState::MainMenu { mut items } => {
                // draw menu or something...
                if let Some(selection) = gui::draw_main_menu(&mut items, ctx) {
                    match selection {
                        gui::MainMenuOption::New | gui::MainMenuOption::Continue => {
                            // do something first?
                            AppState::InGame
                        }
                        gui::MainMenuOption::Quit => AppState::Quit,
                        gui::MainMenuOption::Load => AppState::Loading,
                        gui::MainMenuOption::Save => AppState::Saving,
                    }
                } else {
                    AppState::MainMenu { items }
                }
            }
            AppState::InGame => {
                self.remove_dead();

                self.render_map(ctx);
                // run state machine
                *self.ecs.fetch_mut::<RunState>() = self.next_game_state(ctx);

                draw_ui(&self.ecs, ctx);
                AppState::InGame
            }
            AppState::Saving => {
                save_game(&mut self.ecs);
                AppState::InGame
            }
            AppState::Loading => {
                load_game(&mut self.ecs);
                let _deleted = delete_save();
                AppState::InGame
            }
            AppState::Quit => {
                ctx.quit();
                AppState::Quit
            }
        };
        self.render_debug_info(ctx);
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
    ShowDropItem,
    ShowTargeting { range: i32, item: Entity },
    ShowMenu,
    NextLevel,
}

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum AppState {
    MainMenu { items: Vec<MainMenuItem> },
    InGame,
    Saving,
    Loading,
    Quit,
}

impl AppState {
    fn new_main_menu() -> Self {
        Self::MainMenu {
            items: vec![MainMenuItem::NEW, MainMenuItem::LOAD, MainMenuItem::QUIT],
        }
    }
    fn new_pause_menu() -> Self {
        Self::MainMenu {
            items: vec![
                MainMenuItem::CONTINUE,
                MainMenuItem::SAVE,
                MainMenuItem::LOAD,
                MainMenuItem::QUIT,
            ],
        }
    }
}
