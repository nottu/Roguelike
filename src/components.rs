use rltk::prelude::*;
use serde::{Deserialize, Serialize};
#[allow(deprecated)]
use specs::{prelude::*, saveload::ConvertSaveload, saveload::Marker, Entity};
use specs_derive::{Component, ConvertSaveload};

#[derive(Debug, Component, PartialEq, Eq, Clone, Copy, ConvertSaveload)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

impl From<&Position> for Point {
    fn from(point: &Position) -> Self {
        Self {
            x: point.x,
            y: point.y,
        }
    }
}

impl From<Point> for Position {
    fn from(point: Point) -> Self {
        Self {
            x: point.x,
            y: point.y,
        }
    }
}

#[derive(Debug, Component, Clone, Copy, ConvertSaveload)]
pub struct Renderable {
    pub glyph: FontCharType,
    pub fg: RGB,
    pub bg: RGB,
    pub render_order: i32,
}

/// Player and Enemies

#[derive(Debug, Component, Clone, ConvertSaveload)]
pub struct Viewshed {
    pub visible_tiles: Vec<Point>,
    pub range: i32,
    pub dirty: bool,
}

#[derive(Debug, Component, Serialize, Deserialize, Clone, Copy)]
pub struct Monster;

#[derive(Debug, Component, ConvertSaveload, Clone)]
pub struct Name {
    pub name: String,
}

#[derive(Debug, Component, Serialize, Deserialize, Clone, Copy)]
pub struct BlockedTile;

#[derive(Debug, Component, ConvertSaveload, Clone, Copy)]
pub struct CombatStats {
    pub max_hp: i32,
    pub hp: i32,
    pub defense: i32,
    pub power: i32,
}

#[derive(Debug, Component)]
pub struct WantsToMelee {
    pub target: Entity,
}

#[derive(Debug, Component)]
pub struct SufferDamage {
    pub amount: Vec<i32>,
}

impl SufferDamage {
    pub fn new_damage(store: &mut WriteStorage<Self>, victim: Entity, amount: i32) {
        match store.get_mut(victim) {
            Some(suffering) => {
                suffering.amount.push(amount);
            }
            None => {
                store
                    .insert(
                        victim,
                        Self {
                            amount: vec![amount],
                        },
                    )
                    .expect("Unable to insert damage");
            }
        }
    }
}

/// Items and Inventory

#[derive(Debug, Component, Clone, Copy, Serialize, Deserialize)]
pub struct Item;

#[derive(Debug, Component, ConvertSaveload, Clone, Copy)]
pub struct ProvidesHealing {
    pub heal_amount: i32,
}

#[derive(Debug, Component, Serialize, Deserialize, Clone, Copy)]
pub struct Consumable;

#[derive(Debug, Component, ConvertSaveload, Clone, Copy)]
pub struct InBackpack {
    pub owner: Entity,
}

#[derive(Debug, Component)]
pub struct WantsToPickUp {
    pub item: Entity,
}

#[derive(Debug, Component)]
pub struct WantsToUseItem {
    pub item: Entity,
    pub target: Entity,
}

#[derive(Debug, Component)]
pub struct WantsToDropItem {
    pub item: Entity,
}

#[derive(Debug, Component, ConvertSaveload, Clone, Copy)]
pub struct Ranged {
    pub range: i32,
}

#[derive(Debug, Component, ConvertSaveload, Clone, Copy)]
pub struct InflictsDamage {
    pub damage: i32,
}

#[derive(Debug, Component, ConvertSaveload, Clone, Copy)]
pub struct AreaOfEffect {
    pub radius: i32,
}

#[derive(Debug, Component, Clone, Copy, ConvertSaveload)]
pub struct Confusion {
    pub turns: i32,
}

#[derive(Debug, Component, Clone, Copy)]
pub struct ToDelete;

#[derive(Debug)]
pub struct FilePersistent;
