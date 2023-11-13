use rltk::prelude::*;
use specs::prelude::*;

mod components;
mod map;
mod systems;

use components::*;
use map::*;
use systems::*;

struct State {
    ecs: World,
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        self.run_systems();
        player_input(self, ctx);

        ctx.cls();
        {
            self.ecs.fetch::<Map>().draw(ctx);
        }

        let positions = self.ecs.read_storage::<Position>();
        let renderables = self.ecs.read_storage::<Renderable>();

        for (pos, render) in (&positions, &renderables).join() {
            ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph);
        }
    }
}

impl State {
    fn run_systems(&mut self) {
        let mut vis = VisibilitySystem;
        vis.run_now(&self.ecs);
        self.ecs.maintain();
    }
}

fn main() -> rltk::BError {
    let context = RltkBuilder::simple80x50()
        .with_title("Rougelike Tutorial")
        .build()?;

    let mut gs = State { ecs: World::new() };
    gs.ecs.register::<Position>();
    gs.ecs.register::<Renderable>();
    gs.ecs.register::<Player>();
    gs.ecs.register::<Viewshed>();

    let map = Map::new_map_rooms_and_corridors();
    let Some((x, y)) = map.rooms.first().map(|room| room.center()) else {
        println!("NO FIRST ROOM");
        return Ok(());
    };

    gs.ecs.insert(map);
    gs.ecs
        .create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: rltk::to_cp437('@'),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::BLACK),
        })
        .with(Player)
        .with(Viewshed {
            visible_tiles: vec![],
            range: 8,
            dirty: true,
        })
        .build();

    rltk::main_loop(context, gs)
}
