use rltk::{GameState, Rltk, RGB};
use specs::prelude::*;

mod components;
pub use components::{Player, Position, Renderable, Viewshed};
mod map;
pub use map::*;
mod player;
use player::*;
mod rect;
pub use rect::Rect;
mod visibility_system;
use visibility_system::VisibilitySystem;

pub const MIN_X: i32 = 0;
pub const MAX_X: i32 = 79;
pub const MIN_Y: i32 = 0;
pub const MAX_Y: i32 = 49;

pub const MAP_WIDTH: i32 = MAX_X + 1;
pub const MAP_HEIGHT: i32 = MAX_Y + 1;

struct State {
    entity_component_system: World,
}

impl State {
    fn run_systems(&mut self) {
        let mut vis = VisibilitySystem {};
        vis.run_now(&self.entity_component_system);
        self.entity_component_system.maintain();
    }
}

impl GameState for State {
    fn tick(&mut self, context: &mut Rltk) {
        context.cls();

        player_input(self, context);
        self.run_systems();

        draw_map(&self.entity_component_system, context);

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
    gs.entity_component_system.register::<Viewshed>();

    let map = Map::new_map_rooms_and_corridors();
    let (player_x, player_y) = map.rooms[0].center();
    gs.entity_component_system.insert(map);

    gs.entity_component_system
        .create_entity()
        .with(Position {
            x: player_x,
            y: player_y,
        })
        .with(Renderable {
            glyph: rltk::to_cp437('@'),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::BLACK),
        })
        .with(Player {})
        .with(Viewshed {
            visible_tiles: Vec::new(),
            range: 8,
        })
        .build();

    rltk::main_loop(context, gs)
}
