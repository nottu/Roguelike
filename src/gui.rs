use rltk::prelude::*;
use specs::prelude::*;

use crate::{
    components::{CombatStats, InBackpack, Name, Position, Viewshed},
    map::Map,
    player::{fetch_player_entity, Player},
};

pub struct GameLog {
    pub entries: Vec<String>,
}

impl GameLog {
    pub const fn new() -> Self {
        Self { entries: vec![] }
    }
    pub fn log(&mut self, log_message: String) {
        self.entries.push(log_message);
    }
}

// todo: make more ECS-y...
// all functions can be part of a system use AppState to determine which
// systems get to run

// todo: fix hardcoded `x`, `y` values
pub fn draw_ui(ecs: &World, ctx: &mut Rltk) {
    ctx.draw_box(0, 43, 79, 6, RGB::named(WHITE), RGB::named(BLACK));

    let combat_stats = ecs.read_storage::<CombatStats>();
    let players = ecs.read_storage::<Player>();
    let map = ecs.fetch::<Map>();

    if let Some((_player, stats)) = (&players, &combat_stats).join().next() {
        draw_player_stats(stats, ctx);
    }
    if let Some(logs) = ecs.try_fetch::<GameLog>() {
        draw_logs(&logs.entries, ctx);
    }

    {
        let depth = format!("Depth: {}", map.depth);
        ctx.print_color(2, 43, RGB::named(YELLOW), RGB::named(BLACK), depth);
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

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ItemMenuResult {
    Cancel,
    NoResponse,
    Selected(Entity),
}

pub fn show_inventory(ecs: &World, ctx: &mut Rltk) -> ItemMenuResult {
    let player_entity = fetch_player_entity(ecs);
    show_inventory_menu(ecs, ctx, player_entity, "Inventory")
}

pub fn drop_item_menu(ecs: &World, ctx: &mut Rltk) -> ItemMenuResult {
    let player_entity = fetch_player_entity(ecs);
    show_inventory_menu(ecs, ctx, player_entity, "Drop Which Item?")
}

fn show_inventory_menu(
    ecs: &World,
    ctx: &mut Rltk,
    player_entity: Entity,
    menu_text: &str,
) -> ItemMenuResult {
    let names = ecs.read_storage::<Name>();
    let backpack = ecs.read_storage::<InBackpack>();
    let entities = ecs.entities();

    let inventory: Vec<(&str, Entity)> = (&backpack, &names, &entities)
        .join()
        .filter(|&(bk, _name, _item_entity)| bk.owner == player_entity)
        .map(|(_bk, name, item_entity)| (name.name.as_str(), item_entity))
        .collect();

    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    let y = 25 - (inventory.len() / 2) as i32;
    ctx.draw_box(
        15,
        y - 2,
        31,
        inventory.len() + 3,
        RGB::named(WHITE),
        RGB::named(BLACK),
    );
    ctx.print_color(18, y - 2, RGB::named(YELLOW), RGB::named(BLACK), menu_text);
    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    ctx.print_color(
        18,
        y + inventory.len() as i32 + 1,
        RGB::named(YELLOW),
        RGB::named(BLACK),
        "ESCAPE to cancel",
    );

    for (idx, (item_name, _item_ent)) in inventory.iter().enumerate() {
        #[allow(clippy::cast_possible_wrap, clippy::cast_possible_truncation)]
        let idx = idx as i32;
        ctx.set(
            17,
            y + idx,
            RGB::named(WHITE),
            RGB::named(BLACK),
            rltk::to_cp437('('),
        );
        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
        ctx.set(
            18,
            y + idx,
            RGB::named(WHITE),
            RGB::named(BLACK),
            97 + idx as FontCharType,
        );
        ctx.set(
            19,
            y + idx,
            RGB::named(WHITE),
            RGB::named(BLACK),
            rltk::to_cp437(')'),
        );

        ctx.print(21, y + idx, item_name.to_owned());
    }
    ctx.key.map_or(ItemMenuResult::NoResponse, |key| match key {
        VirtualKeyCode::Escape => ItemMenuResult::Cancel,
        letter => {
            let selection = rltk::letter_to_option(letter);
            #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
            if selection < 0 || selection >= inventory.len() as i32 {
                ItemMenuResult::NoResponse
            } else {
                #[allow(clippy::cast_sign_loss)]
                let item_entity = inventory[selection as usize].1;
                ItemMenuResult::Selected(item_entity)
            }
        }
    })
}

fn get_in_range_points(ecs: &World, entity: Entity, range: i32) -> Option<Vec<Point>> {
    let player_pos: Point = ecs.read_storage::<Position>().get(entity).unwrap().into();

    let viewshed = ecs.read_storage::<Viewshed>();
    #[allow(clippy::cast_precision_loss)]
    viewshed.get(entity).map(|viewshed| {
        viewshed
            .visible_tiles
            .iter()
            .filter(|&point| {
                rltk::DistanceAlg::Pythagoras.distance2d(*point, player_pos) <= range as f32
            })
            .map(std::borrow::ToOwned::to_owned)
            .collect()
    })
}

pub fn ranged_target(ecs: &mut World, ctx: &mut Rltk, range: i32) -> ItemMenuResult {
    let player_entity = fetch_player_entity(ecs);
    let Some(in_range_points) = get_in_range_points(ecs, player_entity, range) else {
        return ItemMenuResult::Cancel;
    };

    ctx.print_color(
        5,
        0,
        RGB::named(YELLOW),
        RGB::named(BLACK),
        "Select Target:",
    );
    for Point { x, y } in &in_range_points {
        ctx.set_bg(*x, *y, RGB::named(BLUE));
    }

    let mouse_pos = {
        let (x, y) = ctx.mouse_pos();
        Point { x, y }
    };

    if ctx.left_click {
        if in_range_points
            .iter()
            .any(|&in_range| in_range.x == mouse_pos.x && in_range.y == mouse_pos.y)
        {
            // valid target
            let click_position_entity = ecs.create_entity().with(Position::from(mouse_pos)).build();
            ItemMenuResult::Selected(click_position_entity)
        } else {
            ItemMenuResult::Cancel
        }
    } else {
        ItemMenuResult::NoResponse
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum MainMenuOption {
    New,
    Load,
    Continue,
    Save,
    Quit,
}

impl MainMenuOption {
    const fn as_str(self) -> &'static str {
        match self {
            Self::New => "New Game",
            Self::Load => "Load Game",
            Self::Save => "Save Game",
            Self::Continue => "Continue",
            Self::Quit => "Quit",
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct MainMenuItem {
    item: MainMenuOption,
    hovered: bool,
}

impl MainMenuItem {
    pub const NEW: Self = Self {
        item: MainMenuOption::New,
        hovered: false,
    };
    pub const LOAD: Self = Self {
        item: MainMenuOption::Load,
        hovered: false,
    };
    pub const QUIT: Self = Self {
        item: MainMenuOption::Quit,
        hovered: false,
    };
    pub const SAVE: Self = Self {
        item: MainMenuOption::Save,
        hovered: false,
    };
    pub const CONTINUE: Self = Self {
        item: MainMenuOption::Continue,
        hovered: false,
    };
}

// todo: on LoadGame, draw menu to select saved game
// on save game... draw manu to select save slot?
pub fn draw_main_menu(menu_items: &mut [MainMenuItem], ctx: &mut Rltk) -> Option<MainMenuOption> {
    ctx.print_color_centered(
        15,
        RGB::named(YELLOW),
        RGB::named(BLACK),
        "Rust Roguelike Tutorial",
    );
    let mut selection_idx = menu_items
        .iter()
        .enumerate()
        .find(|(_idx, menu_item)| menu_item.hovered)
        .map_or(0, |(idx, _item)| idx);
    menu_items[selection_idx].hovered = true;
    for (idx, menu_item) in menu_items.iter().enumerate() {
        ctx.print_color_centered(
            24 + 2 * idx,
            if menu_item.hovered {
                RGB::named(MAGENTA)
            } else {
                RGB::named(WHITE)
            },
            RGB::named(BLACK),
            menu_item.item.as_str(),
        );
    }
    // handle item selection
    menu_items[selection_idx].hovered = false;
    selection_idx = ctx.key.map_or(selection_idx, |key| match key {
        VirtualKeyCode::Up => selection_idx.checked_sub(1).unwrap_or(menu_items.len() - 1),
        VirtualKeyCode::Down => selection_idx + 1,
        _ => selection_idx,
    }) % menu_items.len();
    menu_items[selection_idx].hovered = true;

    // handle item choosing...
    ctx.key.and_then(|key| match key {
        VirtualKeyCode::Escape => Some(MainMenuOption::Quit),
        VirtualKeyCode::Return => menu_items.get(selection_idx).map(|item| item.item),
        _ => None,
    })
}
