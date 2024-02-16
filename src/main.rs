use rltk::{GameState, Rltk, RGB};
use specs::prelude::*;

mod components;
pub use components::{Player, Position, Renderable};
mod map;
pub use map::*;
mod player;
use player::*;
mod rect;
// pub use rect::Rect;

pub const MIN_X: i32 = 0;
pub const MAX_X: i32 = 79;
pub const MIN_Y: i32 = 0;
pub const MAX_Y: i32 = 49;

pub const MAP_WIDTH: usize = (MAX_X as usize) + 1;
pub const MAP_HEIGHT: usize = (MAX_Y as usize) + 1;

struct State {
    entity_component_system: World,
}

impl State {
    fn run_systems(&mut self) {
        self.entity_component_system.maintain();
    }
}

impl GameState for State {
    fn tick(&mut self, context: &mut Rltk) {
        context.cls();

        player_input(self, context);
        self.run_systems();

        let map = self.entity_component_system.fetch::<Vec<TileType>>();
        draw_map(&map, context);

        let positions = self.entity_component_system.read_storage::<Position>();
        let renderables = self.entity_component_system.read_storage::<Renderable>();

        for (position, render) in (&positions, &renderables).join() {
            context.set(position.x, position.y, render.fg, render.bg, render.glyph);
        }
    }
}

fn main() -> rltk::BError {
    use rltk::RltkBuilder;

    let context = RltkBuilder::simple80x50()
        .with_title("Roguelike Tutorial")
        .build()?;
    let mut gs = State {
        entity_component_system: World::new(),
    };
    gs.entity_component_system.register::<Position>();
    gs.entity_component_system.register::<Renderable>();
    gs.entity_component_system.register::<Player>();

    gs.entity_component_system.insert(new_map());

    gs.entity_component_system
        .create_entity()
        .with(Position { x: 40, y: 25 })
        .with(Renderable {
            glyph: rltk::to_cp437('@'),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::BLACK),
        })
        .with(Player {})
        .build();

    rltk::main_loop(context, gs)
}
