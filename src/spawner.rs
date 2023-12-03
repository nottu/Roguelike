use rltk::prelude::*;
use specs::prelude::*;

use crate::{
    components::{
        AreaOfEffect, BlockedTile, CombatStats, Confusion, Consumable, InBackpack, InflictsDamage,
        Item, Monster, Name, Position, ProvidesHealing, Ranged, Renderable, Viewshed,
    },
    map::Rect,
    player::Player,
};

pub fn spawn_player(ecs: &mut World, position: Position) -> Entity {
    let player_entity = ecs
        .create_entity()
        .with(position)
        .with(Renderable {
            glyph: rltk::to_cp437('@'),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::BLACK),
            render_order: 0,
        })
        .with(Player)
        .with(Viewshed {
            visible_tiles: vec![],
            range: 8,
            dirty: true,
        })
        .with(CombatStats {
            max_hp: 30,
            hp: 30,
            defense: 2,
            power: 5,
        })
        .with(Name {
            name: "Player".to_string(),
        })
        .build();

    // spawn player with a simple potion
    // can we make this a potion that refreshes
    // after some ammount of time/turns?
    ecs.create_entity()
        .with(InBackpack {
            owner: player_entity,
        })
        .with(Renderable {
            glyph: rltk::to_cp437('¡'),
            fg: RGB::named(MAGENTA),
            bg: RGB::named(BLACK),
            render_order: 2,
        })
        .with(Item)
        .with(Consumable)
        .with(ProvidesHealing { heal_amount: 4 })
        .with(Name {
            name: "Health Potion".to_string(),
        })
        .build();

    magic_missile_scroll_builder(ecs)
        .with(InBackpack {
            owner: player_entity,
        })
        .build();
    fireball_scroll_builder(ecs)
        .with(InBackpack {
            owner: player_entity,
        })
        .build();

    fireball_scroll_builder(ecs).with(position).build();
    confusion_scroll_builder(ecs)
        .with(InBackpack {
            owner: player_entity,
        })
        .build();
    player_entity
}

#[derive(Debug, PartialEq)]
enum EnemyType {
    Goblin,
    Orc,
}

impl EnemyType {
    fn renderable(&self) -> Renderable {
        let glyph = match self {
            Self::Orc => rltk::to_cp437('o'),
            Self::Goblin => rltk::to_cp437('g'),
        };
        Renderable {
            glyph,
            fg: RGB::named(rltk::RED),
            bg: RGB::named(rltk::BLACK),
            render_order: 1,
        }
    }
    fn name(&self) -> String {
        match self {
            Self::Orc => "Orc",
            Self::Goblin => "Goblin",
        }
        .to_string()
    }
}
pub struct UnknownEnemyType;
impl TryFrom<i32> for EnemyType {
    type Error = UnknownEnemyType;
    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::Orc),
            2 => Ok(Self::Goblin),
            _ => Err(UnknownEnemyType),
        }
    }
}

fn spawn_enemy(ecs: &mut World, position: Position, enemy_type: &EnemyType) -> Entity {
    ecs.create_entity()
        .with(position)
        .with(enemy_type.renderable())
        .with(Viewshed {
            visible_tiles: Vec::new(),
            range: 8,
            dirty: true,
        })
        .with(Monster)
        .with(Name {
            name: enemy_type.name(),
        })
        .with(BlockedTile)
        .with(CombatStats {
            max_hp: 16,
            hp: 16,
            defense: 1,
            power: 4,
        })
        .build()
}

pub fn spawn_random_monster(
    ecs: &mut World,
    position: Position,
) -> Result<Entity, UnknownEnemyType> {
    let roll: i32 = ecs.fetch_mut::<RandomNumberGenerator>().roll_dice(1, 2);
    let enemy_type = EnemyType::try_from(roll)?;
    Ok(spawn_enemy(ecs, position, &enemy_type))
}

const MAX_MONSTERS: i32 = 4;
const MAX_ITEMS: i32 = 2;

/// Spawns from 0 up to `MAX_MONSTERS` per room
/// Also spawns from 0 up to `MAX_ITEMS`
pub fn spawn_room(ecs: &mut World, room: &Rect) {
    // generate num_moster number of Positions
    {
        let monster_points: Vec<(i32, i32)> = {
            let mut rng = ecs.fetch_mut::<RandomNumberGenerator>();
            let num_monsters = rng.roll_dice(1, MAX_MONSTERS) - 1;
            generate_random_room_positions(room, num_monsters, &mut rng)
        };

        for (x, y) in monster_points {
            let _enemy_entity = spawn_random_monster(ecs, Position { x, y });
        }
    }
    // same but for items
    {
        let item_points: Vec<(i32, i32)> = {
            let mut rng = ecs.fetch_mut::<RandomNumberGenerator>();
            let num_items = rng.roll_dice(1, MAX_ITEMS) - 1;
            generate_random_room_positions(room, num_items, &mut rng)
                .into_iter()
                .collect()
        };

        let item_types: Vec<ItemType> = (0..item_points.len())
            .map(|_| ItemType::random_item(ecs))
            .collect();

        for ((x, y), item_type) in item_points.into_iter().zip(item_types) {
            item_type.builder(ecs).with(Position { x, y }).build();
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum ItemType {
    HealingPotion,
    MagicMissileScroll,
    FireBallScroll,
    ConfusionScroll,
}
impl<'a> ItemType {
    fn builder(&'a self, ecs: &'a mut World) -> EntityBuilder {
        match self {
            Self::HealingPotion => healing_potion_builder(ecs),
            Self::MagicMissileScroll => magic_missile_scroll_builder(ecs),
            Self::FireBallScroll => fireball_scroll_builder(ecs),
            Self::ConfusionScroll => confusion_scroll_builder(ecs),
        }
    }
    fn random_item(ecs: &'a World) -> Self {
        let mut rng = ecs.fetch_mut::<RandomNumberGenerator>();
        match rng.roll_dice(1, 3) {
            1 => Self::HealingPotion,
            2 => Self::FireBallScroll,
            3 => Self::MagicMissileScroll,
            4 => Self::ConfusionScroll,
            n => unimplemented!("scroll type {n} not implemented!"),
        }
    }
}

fn generate_random_room_positions(
    room: &Rect,
    num_positions: i32,
    rng: &mut RandomNumberGenerator,
) -> Vec<(i32, i32)> {
    let mut positions: Vec<(i32, i32)> = Vec::new();
    for _i in 0..num_positions {
        let mut added = false;
        while !added {
            // generate random (x, y pair)
            let x = room.x1 + rng.roll_dice(1, room.width());
            let y = room.y1 + rng.roll_dice(1, room.height());
            if !positions.contains(&(x, y)) {
                positions.push((x, y));
                added = true;
            }
        }
    }
    positions
}

/// Return an `EntityBuilder` with components to describe a healing potion
/// that can be composed with more `Components`
/// like a `Position` to be rendered on the map or a `InBag` to be in an
/// entity's inventory
fn healing_potion_builder(ecs: &mut World) -> EntityBuilder {
    ecs.create_entity()
        .with(Renderable {
            glyph: rltk::to_cp437('¡'),
            fg: RGB::named(MAGENTA),
            bg: RGB::named(BLACK),
            render_order: 2,
        })
        .with(Item)
        .with(Consumable)
        .with(ProvidesHealing { heal_amount: 8 })
        .with(Name {
            name: "Health Potion".to_string(),
        })
}

/// Return an `EntityBuilder` with components to describe a magic missle scroll
/// that can be composed with more `Components`
/// like a `Position` to be rendered on the map or a `InBag` to be in an
/// entity's inventory
fn magic_missile_scroll_builder(ecs: &mut World) -> EntityBuilder {
    ecs.create_entity()
        .with(Renderable {
            glyph: rltk::to_cp437(')'),
            fg: RGB::named(CYAN),
            bg: RGB::named(BLACK),
            render_order: 2,
        })
        .with(Name {
            name: "Magic Missile Scroll".to_string(),
        })
        .with(Item)
        .with(Consumable)
        .with(Ranged { range: 6 })
        .with(InflictsDamage { damage: 8 })
}

/// Return an `EntityBuilder` with components to describe a fireball scroll
/// that can be composed with more `Components`
/// like a `Position` to be rendered on the map or a `InBag` to be in an
/// entity's inventory
fn fireball_scroll_builder(ecs: &mut World) -> EntityBuilder {
    ecs.create_entity()
        .with(Renderable {
            glyph: rltk::to_cp437(')'),
            fg: RGB::named(ORANGE),
            bg: RGB::named(BLACK),
            render_order: 2,
        })
        .with(Name {
            name: "Fireball Scroll".to_string(),
        })
        .with(Item)
        .with(Consumable)
        .with(Ranged { range: 6 })
        .with(InflictsDamage { damage: 20 })
        .with(AreaOfEffect { radius: 3 })
}

/// Return an `EntityBuilder` with components to describe a confussion scroll
/// that can be composed with more `Components`
/// like a `Position` to be rendered on the map or a `InBag` to be in an
/// entity's inventory
fn confusion_scroll_builder(ecs: &mut World) -> EntityBuilder {
    ecs.create_entity()
        .with(Renderable {
            glyph: rltk::to_cp437(')'),
            fg: RGB::named(PINK),
            bg: RGB::named(BLACK),
            render_order: 2,
        })
        .with(Name {
            name: "Confusion Scroll".to_string(),
        })
        .with(Item)
        .with(Consumable)
        .with(Ranged { range: 6 })
        .with(Confusion { turns: 4 })
}
