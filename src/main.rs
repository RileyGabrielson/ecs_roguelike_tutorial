use rltk::{GameState, Point, Rltk};
use specs::prelude::*;

mod components;
pub use components::{
    AreaOfEffect, BlocksTile, CombatStats, Confusion, Consumable, InInventory, InflictsDamage,
    Item, Monster, Name, Player, Position, ProvidesHealing, Ranged, Renderable, SufferDamage,
    Viewshed, WantsToDropItem, WantsToMelee, WantsToPickupItem, WantsToUseItem,
};
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
mod map_indexing_system;
use map_indexing_system::MapIndexingSystem;
mod melee_combat_system;
use melee_combat_system::MeleeCombatSystem;
mod damage_system;
use damage_system::DamageSystem;
mod game_log;
pub use game_log::GameLog;
mod gui;
mod inventory_system;
use inventory_system::{ItemCollectionSystem, ItemDropSystem, ItemUseSystem};
mod menu;
mod spawner;

pub const MAP_WIDTH: i32 = 80;
pub const MAP_HEIGHT: i32 = 43;

pub const MIN_X: i32 = 0;
pub const MAX_X: i32 = MAP_WIDTH - 1;
pub const MIN_Y: i32 = 0;
pub const MAX_Y: i32 = MAP_HEIGHT - 1;

#[derive(PartialEq, Copy, Clone)]
pub enum RunState {
    AwaitingInput,
    PreRun,
    PlayerTurn,
    MonsterTurn,
    ShowInventory,
    ShowDropItem,
    ShowTargeting {
        range: i32,
        item: Entity,
    },
    MainMenu {
        menu_selection: menu::MainMenuSelection,
    },
}

struct State {
    entity_component_system: World,
}

impl State {
    fn run_systems(&mut self) {
        let mut visibility = VisibilitySystem {};
        visibility.run_now(&self.entity_component_system);

        let mut monster_ai = MonsterAI {};
        monster_ai.run_now(&self.entity_component_system);

        let mut map_indexing = MapIndexingSystem {};
        map_indexing.run_now(&self.entity_component_system);

        let mut melee_combat = MeleeCombatSystem {};
        melee_combat.run_now(&self.entity_component_system);

        let mut damage = DamageSystem {};
        damage.run_now(&self.entity_component_system);

        let mut item_collection = ItemCollectionSystem {};
        item_collection.run_now(&self.entity_component_system);

        let mut item_drop = ItemDropSystem {};
        item_drop.run_now(&self.entity_component_system);

        let mut drink_potions = ItemUseSystem {};
        drink_potions.run_now(&self.entity_component_system);

        self.entity_component_system.maintain();
    }
}

impl GameState for State {
    fn tick(&mut self, context: &mut Rltk) {
        let mut run_state = *self.entity_component_system.fetch::<RunState>();
        context.cls();

        match run_state {
            RunState::MainMenu { .. } => {}
            _ => {
                draw_map(&self.entity_component_system, context);

                {
                    let positions = self.entity_component_system.read_storage::<Position>();
                    let renderables = self.entity_component_system.read_storage::<Renderable>();
                    let map = self.entity_component_system.fetch::<Map>();

                    let mut data = (&positions, &renderables).join().collect::<Vec<_>>();
                    data.sort_by(|&a, &b| b.1.layer.cmp(&a.1.layer));
                    for (pos, render) in data.iter() {
                        let idx = map.xy_idx(pos.x, pos.y);
                        if map.visible_tiles[idx] {
                            context.set(pos.x, pos.y, render.fg, render.bg, render.glyph)
                        }
                    }

                    gui::draw_ui(&self.entity_component_system, context);
                }
                {
                    damage_system::delete_the_dead(&mut self.entity_component_system);
                    gui::draw_ui(&self.entity_component_system, context);
                }
            }
        }

        // {
        //     draw_map(&self.entity_component_system, context);

        //     let positions = self.entity_component_system.read_storage::<Position>();
        //     let renderables = self.entity_component_system.read_storage::<Renderable>();
        //     let map = self.entity_component_system.fetch::<Map>();

        //     let mut layers: Vec<Vec<(&Position, &Renderable)>> = vec![Vec::new(); 10 as usize];

        //     for (position, render) in (&positions, &renderables).join() {
        //         layers[render.layer as usize].push((position, render));
        //     }

        //     for layer in layers {
        //         for (position, render) in layer {
        //             let index = map.xy_idx(position.x, position.y);
        //             if map.visible_tiles[index] {
        //                 context.set(position.x, position.y, render.fg, render.bg, render.glyph);
        //             }
        //         }
        //     }
        // }

        match run_state {
            RunState::MainMenu { .. } => {
                let result = menu::main_menu(&mut self.entity_component_system, context);
                match result {
                    menu::MainMenuResult::NoSelection { selected } => {
                        run_state = RunState::MainMenu {
                            menu_selection: selected,
                        }
                    }
                    menu::MainMenuResult::Selected { selected } => match selected {
                        menu::MainMenuSelection::NewGame => run_state = RunState::PreRun,
                        menu::MainMenuSelection::LoadGame => run_state = RunState::PreRun,
                        menu::MainMenuSelection::Quit => {
                            ::std::process::exit(0);
                        }
                    },
                }
            }
            RunState::PreRun => {
                self.run_systems();
                run_state = RunState::AwaitingInput;
            }
            RunState::AwaitingInput => {
                run_state = player_input(self, context);
            }
            RunState::PlayerTurn => {
                self.run_systems();
                run_state = RunState::MonsterTurn;
            }
            RunState::MonsterTurn => {
                self.run_systems();
                run_state = RunState::AwaitingInput;
            }
            RunState::ShowTargeting { range, item } => {
                let (item_menu_result, target_position) =
                    gui::ranged_target(&mut self.entity_component_system, context, range);
                match item_menu_result {
                    gui::ItemMenuResult::Cancel => run_state = RunState::AwaitingInput,
                    gui::ItemMenuResult::NoResponse => {}
                    gui::ItemMenuResult::Selected => {
                        let mut intent = self
                            .entity_component_system
                            .write_storage::<WantsToUseItem>();
                        intent
                            .insert(
                                *self.entity_component_system.fetch::<Entity>(),
                                WantsToUseItem {
                                    item,
                                    target: target_position,
                                },
                            )
                            .expect("Unable to insert intent");
                        run_state = RunState::PlayerTurn;
                    }
                }
            }
            RunState::ShowDropItem => {
                let (menu_state, entity_result) =
                    gui::drop_item_menu(&mut self.entity_component_system, context);
                match menu_state {
                    gui::ItemMenuResult::Cancel => run_state = RunState::AwaitingInput,
                    gui::ItemMenuResult::NoResponse => {}
                    gui::ItemMenuResult::Selected => {
                        let item_entity = entity_result.unwrap();
                        let mut intent = self
                            .entity_component_system
                            .write_storage::<WantsToDropItem>();
                        intent
                            .insert(
                                *self.entity_component_system.fetch::<Entity>(),
                                WantsToDropItem { item: item_entity },
                            )
                            .expect("Unable to insert intent");

                        run_state = RunState::PlayerTurn;
                    }
                }
            }
            RunState::ShowInventory => {
                let (menu_state, entity_result) =
                    gui::show_inventory(&mut self.entity_component_system, context);
                match menu_state {
                    gui::ItemMenuResult::Cancel => run_state = RunState::AwaitingInput,
                    gui::ItemMenuResult::NoResponse => {}
                    gui::ItemMenuResult::Selected => {
                        let item_entity = entity_result.unwrap();

                        let is_ranged = self.entity_component_system.read_storage::<Ranged>();
                        let is_item_ranged = is_ranged.get(item_entity);
                        if let Some(is_item_ranged) = is_item_ranged {
                            run_state = RunState::ShowTargeting {
                                range: is_item_ranged.range,
                                item: item_entity,
                            };
                        } else {
                            let mut intent = self
                                .entity_component_system
                                .write_storage::<WantsToUseItem>();
                            intent
                                .insert(
                                    *self.entity_component_system.fetch::<Entity>(),
                                    WantsToUseItem {
                                        item: item_entity,
                                        target: None,
                                    },
                                )
                                .expect("Unable to insert intent");
                            run_state = RunState::PlayerTurn;
                        }
                    }
                }
            }
        }

        {
            let mut runwriter = self.entity_component_system.write_resource::<RunState>();
            *runwriter = run_state;
        }
    }
}

fn main() -> rltk::BError {
    use rltk::RltkBuilder;

    let mut context = RltkBuilder::simple80x50()
        .with_fitscreen(true)
        .with_title("Roguelike Tutorial")
        .build()?;
    context.with_post_scanlines(true);

    let mut gs = State {
        entity_component_system: World::new(),
    };

    gs.entity_component_system.register::<Position>();
    gs.entity_component_system.register::<Renderable>();
    gs.entity_component_system.register::<Player>();
    gs.entity_component_system.register::<Viewshed>();
    gs.entity_component_system.register::<Monster>();
    gs.entity_component_system.register::<Name>();
    gs.entity_component_system.register::<BlocksTile>();
    gs.entity_component_system.register::<CombatStats>();
    gs.entity_component_system.register::<WantsToMelee>();
    gs.entity_component_system.register::<SufferDamage>();

    gs.entity_component_system.register::<Item>();
    gs.entity_component_system.register::<ProvidesHealing>();
    gs.entity_component_system.register::<InInventory>();
    gs.entity_component_system.register::<WantsToPickupItem>();
    gs.entity_component_system.register::<WantsToUseItem>();
    gs.entity_component_system.register::<WantsToDropItem>();
    gs.entity_component_system.register::<Consumable>();
    gs.entity_component_system.register::<Ranged>();
    gs.entity_component_system.register::<InflictsDamage>();
    gs.entity_component_system.register::<AreaOfEffect>();
    gs.entity_component_system.register::<Confusion>();

    let map = Map::new_map_rooms_and_corridors();
    let (player_x, player_y) = map.rooms[0].center();
    let player_entity = spawner::player(&mut gs.entity_component_system, player_x, player_y);

    gs.entity_component_system
        .insert(rltk::RandomNumberGenerator::new());
    gs.entity_component_system
        .insert(Point::new(player_x, player_y));
    gs.entity_component_system.insert(player_entity);
    gs.entity_component_system.insert(RunState::MainMenu {
        menu_selection: menu::MainMenuSelection::NewGame,
    });
    gs.entity_component_system.insert(game_log::GameLog {
        entries: vec!["Welcome to Riley's Roguelike".to_string()],
    });

    for room in map.rooms.iter().skip(1) {
        spawner::spawn_room(&mut gs.entity_component_system, room);
    }

    gs.entity_component_system.insert(map);

    rltk::main_loop(context, gs)
}
