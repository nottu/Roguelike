use std::cmp;

use rltk::prelude::*;
use specs::{Entities, Join, ReadStorage, System, World, WorldExt, WriteExpect, WriteStorage};

use crate::{
    components::*,
    map::{Map, TileType},
    State,
};

fn move_player(delta_x: i32, delta_y: i32, ecs: &mut World) {
    let mut positions = ecs.write_storage::<Position>();
    let players = ecs.read_storage::<Player>();
    let mut viewsheds = ecs.write_storage::<Viewshed>();

    let map = ecs.fetch::<Map>();
    for (_player, pos, viewshed) in (&players, &mut positions, &mut viewsheds).join() {
        let new_x = cmp::min(79, cmp::max(0, pos.x + delta_x));
        let new_y = cmp::min(49, cmp::max(0, pos.y + delta_y));
        if map.tiles[map.xy_idx(new_x, new_y)] != TileType::Wall {
            pos.x = new_x;
            pos.y = new_y;

            viewshed.dirty = true;
        }
    }
}

pub fn player_input(gs: &mut State, ctx: &mut Rltk) {
    let Some(pressed_key) = ctx.key else {
        return;
    };

    let (delta_x, delta_y) = match pressed_key {
        rltk::VirtualKeyCode::Left => (-1, 0),
        rltk::VirtualKeyCode::Right => (1, 0),
        rltk::VirtualKeyCode::Up => (0, -1),
        rltk::VirtualKeyCode::Down => (0, 1),
        _ => (0, 0),
    };
    move_player(delta_x, delta_y, &mut gs.ecs)
}

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
                for t in map.visible_tiles.iter_mut() {
                    *t = false
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
