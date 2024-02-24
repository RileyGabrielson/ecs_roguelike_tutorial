use rltk::{GameState, Point, Rltk};
use specs::prelude::*;
use specs::saveload::{SimpleMarker, SimpleMarkerAllocator};
extern crate serde;

mod components;
mod map;
pub use map::*;
mod player;
use player::*;
mod rect;
pub use rect::Rect;
mod game_log;
pub use game_log::GameLog;
mod character_creation;
mod gui;
mod menu;
mod spawner;
mod systems;

pub const MAP_WIDTH: i32 = 80;
pub const MAP_HEIGHT: i32 = 43;
pub const MAP_COUNT: i32 = MAP_WIDTH * MAP_HEIGHT;

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
    SaveGame,
    Dead,
    CharacterCreation,
}

struct State {
    ecs: World,
}

impl State {
    fn run_systems(&mut self) {
        systems::run_systems(&mut self.ecs)
    }
}

impl GameState for State {
    fn tick(&mut self, context: &mut Rltk) {
        let mut run_state = *self.ecs.fetch::<RunState>();
        context.cls();
        systems::particle_system::cull_dead_particles(&mut self.ecs, context);

        match run_state {
            RunState::MainMenu { .. } | RunState::CharacterCreation => {}
            _ => {
                draw_map(&self.ecs, context);

                {
                    let positions = self.ecs.read_storage::<components::Position>();
                    let renderables = self.ecs.read_storage::<components::Renderable>();
                    let invisibles = self.ecs.read_storage::<components::Invisible>();
                    let players = self.ecs.read_storage::<components::Player>();
                    let entities = self.ecs.entities();
                    let map = self.ecs.fetch::<Map>();

                    let mut data = (&positions, &renderables, &entities)
                        .join()
                        .collect::<Vec<_>>();
                    data.sort_by(|&a, &b| b.1.layer.cmp(&a.1.layer));
                    for (pos, render, entity) in data.iter() {
                        let is_invisible = invisibles.get(*entity);
                        match is_invisible {
                            Some(_) => {
                                let is_player = players.get(*entity);
                                match is_player {
                                    Some(_) => {
                                        let idx = map.xy_idx(pos.x, pos.y);
                                        if map.visible_tiles[idx] {
                                            context.set(
                                                pos.x,
                                                pos.y,
                                                rltk::GRAY65,
                                                render.bg,
                                                render.glyph,
                                            )
                                        }
                                    }
                                    None => {}
                                }
                            }
                            None => {
                                let idx = map.xy_idx(pos.x, pos.y);
                                if map.visible_tiles[idx] {
                                    context.set(pos.x, pos.y, render.fg, render.bg, render.glyph)
                                }
                            }
                        }
                    }

                    gui::draw_ui(&self.ecs, context);
                }
                {
                    let is_player_dead = systems::damage_system::delete_the_dead(&mut self.ecs);
                    match is_player_dead {
                        None => {}
                        Some(_) => run_state = RunState::Dead,
                    }
                    gui::draw_ui(&self.ecs, context);
                }
            }
        }

        match run_state {
            RunState::Dead => {
                let return_to_menu = gui::show_dead_screen(context);
                match return_to_menu {
                    None => {}
                    Some(_) => {
                        new_game(&mut self.ecs);
                        run_state = RunState::MainMenu {
                            menu_selection: menu::MainMenuSelection::NewGame,
                        }
                    }
                }
            }
            RunState::MainMenu { .. } => {
                let result = menu::main_menu(&mut self.ecs, context);
                match result {
                    menu::MainMenuResult::NoSelection { selected } => {
                        run_state = RunState::MainMenu {
                            menu_selection: selected,
                        }
                    }
                    menu::MainMenuResult::Selected { selected } => match selected {
                        menu::MainMenuSelection::NewGame => {
                            new_game(&mut self.ecs);
                            run_state = RunState::CharacterCreation;
                        }
                        menu::MainMenuSelection::LoadGame => {
                            systems::saveload_system::load_game(&mut self.ecs);
                            run_state = RunState::AwaitingInput;
                            systems::saveload_system::delete_save();
                        }
                        menu::MainMenuSelection::Quit => {
                            ::std::process::exit(0);
                        }
                    },
                }
            }
            RunState::CharacterCreation => {
                let selected_item = character_creation::create_character(
                    context,
                    vec![
                        "Invisibility Timer".to_string(),
                        "Confusion Wand".to_string(),
                    ],
                );

                let mut item_to_add: Option<Entity> = None;
                match selected_item {
                    None => {}
                    Some(item_name) => {
                        match item_name.as_ref() {
                            "Invisibility Timer" => {
                                item_to_add = Some(spawner::invisibility_timer(&mut self.ecs));
                            }
                            "Confusion Wand" => {
                                item_to_add = Some(spawner::confusion_wand(&mut self.ecs));
                            }
                            _ => {}
                        };
                    }
                }

                let player_entity = self.ecs.fetch::<Entity>();
                match item_to_add {
                    None => {}
                    Some(item) => {
                        self.ecs
                            .write_storage::<components::InInventory>()
                            .insert(
                                item,
                                components::InInventory {
                                    owner: *player_entity,
                                },
                            )
                            .expect("Could not insert in inventory");

                        run_state = RunState::PreRun;
                    }
                }
            }
            RunState::SaveGame => {
                systems::saveload_system::save_game(&mut self.ecs);
                run_state = RunState::MainMenu {
                    menu_selection: menu::MainMenuSelection::LoadGame,
                };
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
                    gui::ranged_target(&mut self.ecs, context, range);
                match item_menu_result {
                    gui::ItemMenuResult::Cancel => run_state = RunState::AwaitingInput,
                    gui::ItemMenuResult::NoResponse => {}
                    gui::ItemMenuResult::Selected => {
                        let mut intent = self.ecs.write_storage::<components::WantsToUseItem>();
                        intent
                            .insert(
                                *self.ecs.fetch::<Entity>(),
                                components::WantsToUseItem {
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
                let (menu_state, entity_result) = gui::drop_item_menu(&mut self.ecs, context);
                match menu_state {
                    gui::ItemMenuResult::Cancel => run_state = RunState::AwaitingInput,
                    gui::ItemMenuResult::NoResponse => {}
                    gui::ItemMenuResult::Selected => {
                        let item_entity = entity_result.unwrap();
                        let mut intent = self.ecs.write_storage::<components::WantsToDropItem>();
                        intent
                            .insert(
                                *self.ecs.fetch::<Entity>(),
                                components::WantsToDropItem { item: item_entity },
                            )
                            .expect("Unable to insert intent");

                        run_state = RunState::PlayerTurn;
                    }
                }
            }
            RunState::ShowInventory => {
                let (menu_state, entity_result) = gui::show_inventory(&mut self.ecs, context);
                match menu_state {
                    gui::ItemMenuResult::Cancel => run_state = RunState::AwaitingInput,
                    gui::ItemMenuResult::NoResponse => {}
                    gui::ItemMenuResult::Selected => {
                        let item_entity = entity_result.unwrap();

                        let is_ranged = self.ecs.read_storage::<components::Ranged>();
                        let is_item_ranged = is_ranged.get(item_entity);
                        if let Some(is_item_ranged) = is_item_ranged {
                            run_state = RunState::ShowTargeting {
                                range: is_item_ranged.range,
                                item: item_entity,
                            };
                        } else {
                            let mut intent = self.ecs.write_storage::<components::WantsToUseItem>();
                            intent
                                .insert(
                                    *self.ecs.fetch::<Entity>(),
                                    components::WantsToUseItem {
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
            let mut runwriter = self.ecs.write_resource::<RunState>();
            *runwriter = run_state;
        }
    }
}

fn add_new_world_details(ecs: &mut World) {
    ecs.insert(SimpleMarkerAllocator::<components::SerializeMe>::new());

    let map = Map::new_map_rooms_and_corridors();
    let (player_x, player_y) = map.rooms[0].center();
    let player_entity = spawner::player(ecs, player_x, player_y);

    ecs.insert(rltk::RandomNumberGenerator::new());
    ecs.insert(Point::new(player_x, player_y));
    ecs.insert(player_entity);
    ecs.insert(RunState::MainMenu {
        menu_selection: menu::MainMenuSelection::NewGame,
    });
    ecs.insert(game_log::GameLog {
        entries: vec!["Welcome to Riley's Roguelike".to_string()],
    });
    ecs.insert(systems::particle_system::ParticleBuilder::new());

    for room in map.rooms.iter().skip(1) {
        spawner::spawn_room(ecs, room);
    }

    ecs.insert(map);
}

fn new_game(ecs: &mut World) {
    systems::saveload_system::delete_save();
    ecs.delete_all();
    add_new_world_details(ecs);
}

fn main() -> rltk::BError {
    use rltk::RltkBuilder;

    let mut context = RltkBuilder::simple80x50()
        .with_fitscreen(true)
        .with_title("Roguelike Tutorial")
        .build()?;
    context.with_post_scanlines(true);

    let mut gs = State { ecs: World::new() };

    gs.ecs.register::<components::Position>();
    gs.ecs.register::<components::Renderable>();
    gs.ecs.register::<components::Player>();
    gs.ecs.register::<components::Viewshed>();
    gs.ecs.register::<components::Monster>();
    gs.ecs.register::<components::Name>();
    gs.ecs.register::<components::BlocksTile>();
    gs.ecs.register::<components::CombatStats>();
    gs.ecs.register::<components::WantsToMelee>();
    gs.ecs.register::<components::SufferDamage>();
    gs.ecs.register::<components::Item>();
    gs.ecs.register::<components::ProvidesHealing>();
    gs.ecs.register::<components::InInventory>();
    gs.ecs.register::<components::WantsToPickupItem>();
    gs.ecs.register::<components::WantsToUseItem>();
    gs.ecs.register::<components::WantsToDropItem>();
    gs.ecs.register::<components::Consumable>();
    gs.ecs.register::<components::Ranged>();
    gs.ecs.register::<components::InflictsDamage>();
    gs.ecs.register::<components::AreaOfEffect>();
    gs.ecs.register::<components::Confusion>();
    gs.ecs.register::<SimpleMarker<components::SerializeMe>>();
    gs.ecs.register::<components::SerializationHelper>();
    gs.ecs.register::<components::Invisible>();
    gs.ecs.register::<components::WantsBeInvisible>();
    gs.ecs.register::<components::AppliesInvisiblity>();
    gs.ecs.register::<components::Cooldown>();
    gs.ecs.register::<components::ActiveCooldown>();
    gs.ecs.register::<components::CausesConfusion>();
    gs.ecs.register::<components::ParticleLifetime>();

    add_new_world_details(&mut gs.ecs);

    rltk::main_loop(context, gs)
}
