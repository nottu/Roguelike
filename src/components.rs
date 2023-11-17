use rltk::prelude::*;
use specs::prelude::*;
use specs_derive::Component;

#[derive(Debug, Component, PartialEq, Eq, Clone, Copy)]
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

#[derive(Debug, Component)]
pub struct Renderable {
    pub glyph: FontCharType,
    pub fg: RGB,
    pub bg: RGB,
    pub render_order: i32,
}

#[derive(Debug, Component)]
pub struct LeftMover;

#[derive(Debug, Component)]
pub struct Viewshed {
    pub visible_tiles: Vec<Point>,
    pub range: i32,
    pub dirty: bool,
}

#[derive(Debug, Component)]
pub struct Monster;

#[derive(Debug, Component)]
pub struct Name {
    pub name: String,
}

#[derive(Debug, Component)]
pub struct BlockedTile;

#[derive(Debug, Component)]
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

#[derive(Debug, Component)]
pub struct Item;

#[derive(Debug, Component)]
pub struct Potion {
    pub heal_amount: i32,
}

#[derive(Debug, Component)]
pub struct InBackpack {
    pub owner: Entity,
}

#[derive(Debug, Component)]
pub struct WantsToPickUp {
    pub item: Entity,
}

#[derive(Debug, Component)]
pub struct WantsToDrinkPotion {
    pub potion: Entity,
}

#[derive(Debug, Component)]
pub struct WantsToDropItem {
    pub item: Entity,
}
