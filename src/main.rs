use rltk::{GameState, Point, Rltk, RGB};
use specs::prelude::*;

mod components;
pub use components::{Monster, Name, Player, Position, Renderable, Viewshed};
mod map;
pub use map::*;
mod player;
use player::*;
mod rect;
pub use rect::Rect;
mod visibility_system;
use visibility_system::VisibilitySystem;
mod monster_ai_system;
use monster_ai_system::MonsterAI;

pub const MIN_X: i32 = 0;
pub const MAX_X: i32 = 79;
pub const MIN_Y: i32 = 0;
pub const MAX_Y: i32 = 49;

pub const MAP_WIDTH: i32 = MAX_X + 1;
pub const MAP_HEIGHT: i32 = MAX_Y + 1;

#[derive(PartialEq, Copy, Clone)]
pub enum RunState {
    Paused,
    Running,
}

struct State {
    entity_component_system: World,
    run_state: RunState,
}

impl State {
    fn run_systems(&mut self) {
        let mut vis = VisibilitySystem {};
        vis.run_now(&self.entity_component_system);

        let mut mob = MonsterAI {};
        mob.run_now(&self.entity_component_system);

        self.entity_component_system.maintain();
    }
}

impl GameState for State {
    fn tick(&mut self, context: &mut Rltk) {
        context.cls();

        if self.run_state == RunState::Running {
            self.run_systems();
            self.run_state = RunState::Paused;
        } else {
            self.run_state = player_input(self, context);
        }

        draw_map(&self.entity_component_system, context);

        let positions = self.entity_component_system.read_storage::<Position>();
        let renderables = self.entity_component_system.read_storage::<Renderable>();
        let map = self.entity_component_system.fetch::<Map>();

        for (position, render) in (&positions, &renderables).join() {
            let index = map.xy_idx(position.x, position.y);
            if map.visible_tiles[index] {
                context.set(position.x, position.y, render.fg, render.bg, render.glyph);
            }
        }
    }
}

fn main() -> rltk::BError {
    use rltk::RltkBuilder;

    let context = RltkBuilder::simple80x50()
        .with_fitscreen(true)
        .with_title("Roguelike Tutorial")
        .build()?;
    let mut gs = State {
        entity_component_system: World::new(),
        run_state: RunState::Running,
    };

    gs.entity_component_system.register::<Position>();
    gs.entity_component_system.register::<Renderable>();
    gs.entity_component_system.register::<Player>();
    gs.entity_component_system.register::<Viewshed>();
    gs.entity_component_system.register::<Monster>();
    gs.entity_component_system.register::<Name>();

    let map = Map::new_map_rooms_and_corridors();
    let mut rng = rltk::RandomNumberGenerator::new();
    for (i, room) in map.rooms.iter().skip(1).enumerate() {
        let (x, y) = room.center();

        let glyph: rltk::FontCharType;
        let name: String;
        let roll = rng.roll_dice(1, 2);
        match roll {
            1 => {
                glyph = rltk::to_cp437('g');
                name = "Goblin".to_string();
            }
            _ => {
                glyph = rltk::to_cp437('o');
                name = "Orc".to_string();
            }
        }

        gs.entity_component_system
            .create_entity()
            .with(Position { x, y })
            .with(Renderable {
                glyph,
                fg: RGB::named(rltk::RED),
                bg: RGB::named(rltk::BLACK),
            })
            .with(Viewshed {
                visible_tiles: Vec::new(),
                range: 8,
                dirty: true,
            })
            .with(Monster {})
            .with(Name {
                name: format!("{} #{}", name, i),
            })
            .build();
    }

    let (player_x, player_y) = map.rooms[0].center();
    gs.entity_component_system.insert(map);
    gs.entity_component_system
        .insert(Point::new(player_x, player_y));

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
        .with(Name {
            name: "Player".to_string(),
        })
        .with(Viewshed {
            visible_tiles: Vec::new(),
            range: 8,
            dirty: true,
        })
        .build();

    rltk::main_loop(context, gs)
}
