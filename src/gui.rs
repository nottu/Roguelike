use rltk::prelude::*;
use specs::prelude::*;

use crate::{components::{CombatStats, InBackpack, Name, Position, WantsToDrinkPotion}, map::Map, player::Player};

pub struct GameLog {
    pub entries: Vec<String>,
}

impl GameLog {
    pub fn new() -> Self {
        Self { entries: vec![] }
    }
    pub fn log(&mut self, log_message: String) {
        self.entries.push(log_message);
    }
}

pub fn draw_ui(ecs: &World, ctx: &mut Rltk) {
    ctx.draw_box(0, 43, 79, 6, RGB::named(WHITE), RGB::named(BLACK));

    let combat_stats = ecs.read_storage::<CombatStats>();
    let players = ecs.read_storage::<Player>();

    if let Some((_player, stats)) = (&players, &combat_stats).join().next() {
        draw_player_stats(stats, ctx);
    }
    if let Some(logs) = ecs.try_fetch::<GameLog>() {
        draw_logs(&logs.entries, ctx);
    }

    // draw mouse
    {
        let (x, y) = ctx.mouse_pos();
        ctx.set_bg(x, y, RGB::named(MAGENTA));
    }
    draw_tooltips(ecs, ctx);
}

fn draw_player_stats(stats: &CombatStats, ctx: &mut Rltk) {
    let health = format!("HP: {} / {}", stats.hp, stats.max_hp);
    ctx.print_color(12, 43, RGB::named(YELLOW), RGB::named(BLACK), health);
    ctx.draw_bar_horizontal(
        28,
        43,
        51,
        stats.hp,
        stats.max_hp,
        RGB::named(RED),
        RGB::named(BLACK),
    );
}

fn draw_logs(logs: &[String], ctx: &mut Rltk) {
    let mut y = 44;
    for log in logs.iter().rev() {
        if y >= 49 {
            break;
        }
        ctx.print(2, y, log);
        y += 1;
    }
}

fn draw_tooltips(ecs: &World, ctx: &mut Rltk) {
    let map = ecs.fetch::<Map>();
    let names = ecs.read_storage::<Name>();
    let positions = ecs.read_storage::<Position>();

    let mouse_pos = {
        let (x, y) = ctx.mouse_pos();
        Point { x, y }
    };

    let mut tooltip = Vec::<&str>::new();

    for (name, pos) in (&names, &positions).join() {
        let idx = map.xy_idx(mouse_pos.x, mouse_pos.y);
        if mouse_pos.x == pos.x && mouse_pos.y == pos.y && map.visible_tiles[idx] {
            tooltip.push(&name.name);
        }
    }

    if tooltip.is_empty() {
        return;
    }
    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    let width = 3 + tooltip.iter().map(|s| s.len()).max().unwrap_or_default() as i32;
    if mouse_pos.x > 40 {
        let arrow_pos = Point::new(mouse_pos.x - 2, mouse_pos.y);
        let left_x = mouse_pos.x - width;
        let mut y = mouse_pos.y;
        for s in tooltip {
            ctx.print_color(left_x, y, RGB::named(WHITE), RGB::named(GREEN), s);
            #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
            let padding = width - s.len() as i32 - 1;
            for i in 0..padding {
                ctx.print_color(arrow_pos.x - i, y, RGB::named(WHITE), RGB::named(GRAY), " ");
            }
            y += 1;
        }
        ctx.print_color(
            arrow_pos.x,
            arrow_pos.y,
            RGB::named(WHITE),
            RGB::named(GREY),
            "->",
        );
    } else {
        let arrow_pos = Point::new(mouse_pos.x + 1, mouse_pos.y);
        let left_x = mouse_pos.x + 4;
        let mut y = mouse_pos.y;
        for s in &tooltip {
            ctx.print_color(left_x, y, RGB::named(WHITE), RGB::named(GREY), s);
            #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
            let padding = width - s.len() as i32 - 1;
            for i in 0..padding {
                ctx.print_color(
                    arrow_pos.x + 1 + i,
                    y,
                    RGB::named(WHITE),
                    RGB::named(GREY),
                    " ",
                );
            }
            y += 1;
        }
        ctx.print_color(
            arrow_pos.x,
            arrow_pos.y,
            RGB::named(WHITE),
            RGB::named(GREY),
            "<-",
        );
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ItemMenuResult {
    Cancel,
    NoResponse,
    Selected(Entity),
}

pub fn show_inventory(ecs: &World, ctx: &mut Rltk) -> ItemMenuResult {
    let players = ecs.read_storage::<Player>();
    let names = ecs.read_storage::<Name>();
    let backpack = ecs.read_storage::<InBackpack>();
    let entities = ecs.entities();

    let Some(player_entity) = (&entities, &players)
        .join()
        .map(|(entity, _p)| entity)
        .next()
    else {
        return ItemMenuResult::Cancel;
    };

    let inventory: Vec<(&str, Entity)> = (&backpack, &names, &entities)
        .join()
        .filter(|(bk, _name, _item_entity)| bk.owner == player_entity)
        .map(|(_bk, name, item_entity)| (name.name.as_str(), item_entity))
        .collect();

    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    let mut y = 25 - (inventory.len() / 2) as i32;
    ctx.draw_box(
        15,
        y - 2,
        31,
        inventory.len() + 3,
        RGB::named(WHITE),
        RGB::named(BLACK),
    );
    ctx.print_color(
        18,
        y - 2,
        RGB::named(YELLOW),
        RGB::named(BLACK),
        "Inventory",
    );
    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    ctx.print_color(
        18,
        y + inventory.len() as i32 + 1,
        RGB::named(YELLOW),
        RGB::named(BLACK),
        "ESCAPE to cancel",
    );

    for (idx, (item_name, _item_ent)) in inventory.iter().enumerate() {
        ctx.set(
            17,
            y,
            RGB::named(WHITE),
            RGB::named(BLACK),
            rltk::to_cp437('('),
        );
        #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
        ctx.set(
            18,
            y,
            RGB::named(WHITE),
            RGB::named(BLACK),
            97 + idx as FontCharType,
        );
        ctx.set(
            19,
            y,
            RGB::named(WHITE),
            RGB::named(BLACK),
            rltk::to_cp437(')'),
        );

        ctx.print(21, y, item_name.to_owned());
        y += 1;
    }

    match ctx.key {
        None => ItemMenuResult::NoResponse,
        Some(key) => match key {
            VirtualKeyCode::Escape => ItemMenuResult::Cancel,
            letter => {
                let selection = rltk::letter_to_option(letter);
                #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
                if selection < 0 || selection >= inventory.len() as i32 {
                    ItemMenuResult::NoResponse
                } else {
                    #[allow(clippy::cast_sign_loss)]
                    let potion_entity = inventory[selection as usize].1;
                    let want_drink = WantsToDrinkPotion {
                        potion: potion_entity,
                    };
                    ecs.write_storage::<WantsToDrinkPotion>()
                        .insert(player_entity, want_drink)
                        .expect("Failed to WantsToDrinkPotion");
                    ecs.fetch_mut::<GameLog>()
                        .log("Tyring to dringk potion".to_string());
                    ItemMenuResult::Selected(potion_entity)
                }
            }
        },
    }
}
