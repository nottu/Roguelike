use rltk::prelude::*;
use specs::prelude::*;

use crate::{
    components::{
        BlockedTile, CombatStats, InBackpack, Item, Monster, Name, Position, Potion, Renderable,
        Viewshed,
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
        .with(Potion { heal_amount: 8 })
        .with(Name {
            name: "Health Potion".to_string(),
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

impl TryFrom<i32> for EnemyType {
    type Error = String;
    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::Orc),
            2 => Ok(Self::Goblin),
            n => Err(format!("Unknown Enemy Type {n}")),
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

pub fn spawn_random_monster(ecs: &mut World, position: Position) -> Result<Entity, String> {
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
    // same but for potions
    {
        let potion_points: Vec<(i32, i32)> = {
            let mut rng = ecs.fetch_mut::<RandomNumberGenerator>();
            let num_items = rng.roll_dice(1, MAX_ITEMS) - 1;
            generate_random_room_positions(room, num_items, &mut rng)
        };

        for (x, y) in potion_points {
            let _enemy_entity = spawn_potion(ecs, Position { x, y });
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

//

fn spawn_potion(ecs: &mut World, position: Position) -> Entity {
    ecs.create_entity()
        .with(position)
        .with(Renderable {
            glyph: rltk::to_cp437('¡'),
            fg: RGB::named(MAGENTA),
            bg: RGB::named(BLACK),
            render_order: 2,
        })
        .with(Item)
        .with(Potion { heal_amount: 8 })
        .with(Name {
            name: "Health Potion".to_string(),
        })
        .build()
}
