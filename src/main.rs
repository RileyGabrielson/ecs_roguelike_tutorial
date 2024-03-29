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
mod map_builders;
mod menu;
mod random_table;
mod spawner;
mod systems;

pub const MAP_WIDTH: i32 = 80;
pub const MAP_HEIGHT: i32 = 43;
pub const MAP_COUNT: i32 = MAP_WIDTH * MAP_HEIGHT;

pub const MIN_X: i32 = 0;
pub const MAX_X: i32 = MAP_WIDTH - 1;
pub const MIN_Y: i32 = 0;
pub const MAX_Y: i32 = MAP_HEIGHT - 1;

const SHOW_MAPGEN_VISUALIZER: bool = true;

#[derive(PartialEq, Copy, Clone)]
pub enum RunState {
    AwaitingInput,
    PreRun,
    PlayerTurn,
    MonsterTurn,
    ShowInventory,
    ShowRemoveItem,
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
    NextLevel,
    MagicMapReveal {
        row: i32,
    },
    MapGeneration,
}

struct State {
    ecs: World,
    mapgen_next_state: Option<RunState>,
    mapgen_history: Vec<Map>,
    mapgen_index: usize,
    mapgen_timer: f32,
}

impl State {
    fn run_systems(&mut self) {
        systems::run_systems(&mut self.ecs)
    }

    fn new_game(&mut self) {
        systems::saveload_system::delete_save();
        self.ecs.delete_all();
        self.add_new_world_details();
    }

    fn add_new_world_details(&mut self) {
        self.ecs
            .insert(SimpleMarkerAllocator::<components::SerializeMe>::new());
        self.ecs.insert(Map::new(1));
        self.ecs.insert(Point::new(0, 0));
        self.ecs.insert(rltk::RandomNumberGenerator::new());
        let player_entity = spawner::player(&mut self.ecs, 0, 0);
        self.ecs.insert(player_entity);
        self.ecs.insert(RunState::MapGeneration);
        self.ecs.insert(game_log::GameLog {
            entries: vec!["Welcome to Rusty Roguelike".to_string()],
        });
        self.ecs
            .insert(systems::particle_system::ParticleBuilder::new());

        self.generate_world_map(1);
    }

    fn entities_to_remove_on_level_change(&mut self) -> Vec<Entity> {
        let entities = self.ecs.entities();
        let player = self.ecs.read_storage::<components::Player>();
        let inventory = self.ecs.read_storage::<components::InInventory>();
        let equipped = self.ecs.read_storage::<components::Equipped>();
        let player_entity = self.ecs.fetch::<Entity>();

        let mut to_delete: Vec<Entity> = Vec::new();
        for entity in entities.join() {
            let mut should_delete = true;

            // Don't delete the player
            let p = player.get(entity);
            if let Some(_p) = p {
                should_delete = false;
            }

            // Don't delete the player's inventory
            let bp = inventory.get(entity);
            if let Some(bp) = bp {
                if bp.owner == *player_entity {
                    should_delete = false;
                }
            }

            // Don't delete the player's equipment
            let bp = equipped.get(entity);
            if let Some(bp) = bp {
                if bp.owner == *player_entity {
                    should_delete = false;
                }
            }

            if should_delete {
                to_delete.push(entity);
            }
        }

        to_delete
    }

    fn goto_next_level(&mut self) {
        // Delete entities that aren't the player or his/her equipment
        let to_delete = self.entities_to_remove_on_level_change();
        for target in to_delete {
            self.ecs
                .delete_entity(target)
                .expect("Unable to delete entity");
        }

        // Build a new map and place the player
        let current_depth;
        {
            let worldmap_resource = self.ecs.fetch::<Map>();
            current_depth = worldmap_resource.depth;
        }
        self.generate_world_map(current_depth + 1);

        // Notify the player and give them some health
        let player_entity = self.ecs.fetch::<Entity>();
        let mut gamelog = self.ecs.fetch_mut::<game_log::GameLog>();
        gamelog
            .entries
            .push("You descend to the next level, and take a moment to heal.".to_string());
        let mut player_health_store = self.ecs.write_storage::<components::CombatStats>();
        let player_health = player_health_store.get_mut(*player_entity);
        if let Some(player_health) = player_health {
            player_health.hp = i32::max(player_health.hp, player_health.max_hp / 2);
        }
    }

    fn game_over_cleanup(&mut self) {
        // Delete everything
        let mut to_delete = Vec::new();
        for e in self.ecs.entities().join() {
            to_delete.push(e);
        }
        for del in to_delete.iter() {
            self.ecs.delete_entity(*del).expect("Deletion failed");
        }

        // Spawn a new player
        {
            let player_entity = spawner::player(&mut self.ecs, 0, 0);
            let mut player_entity_writer = self.ecs.write_resource::<Entity>();
            *player_entity_writer = player_entity;
        }

        // Build a new map and place the player
        self.generate_world_map(1);
    }

    fn generate_world_map(&mut self, new_depth: i32) {
        self.mapgen_index = 0;
        self.mapgen_timer = 0.0;
        self.mapgen_history.clear();

        let mut builder = map_builders::random_builder(new_depth);
        builder.build_map();
        self.mapgen_history = builder.get_snapshot_history();

        let player_start;
        {
            let mut worldmap_resource = self.ecs.write_resource::<Map>();
            *worldmap_resource = builder.get_map();
            player_start = builder.get_starting_position();
        }

        // Spawn bad guys
        builder.spawn_entities(&mut self.ecs);

        // Place the player and update resources
        let (player_x, player_y) = (player_start.x, player_start.y);
        let mut player_position = self.ecs.write_resource::<Point>();
        *player_position = Point::new(player_x, player_y);
        let mut position_components = self.ecs.write_storage::<components::Position>();
        let player_entity = self.ecs.fetch::<Entity>();
        let player_pos_comp = position_components.get_mut(*player_entity);
        if let Some(player_pos_comp) = player_pos_comp {
            player_pos_comp.x = player_x;
            player_pos_comp.y = player_y;
        }

        // Mark the player's visibility as dirty
        let mut viewshed_components = self.ecs.write_storage::<components::Viewshed>();
        let vs = viewshed_components.get_mut(*player_entity);
        if let Some(vs) = vs {
            vs.dirty = true;
        }
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
                draw_map(&self.ecs.fetch::<Map>(), context);

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
            RunState::MapGeneration => {
                if !SHOW_MAPGEN_VISUALIZER {
                    run_state = self.mapgen_next_state.unwrap();
                }
                context.cls();
                map::draw_map(&self.mapgen_history[self.mapgen_index], context);

                self.mapgen_timer += context.frame_time_ms;
                if self.mapgen_timer > 300.0 {
                    self.mapgen_timer = 0.0;
                    self.mapgen_index += 1;
                    if self.mapgen_index >= self.mapgen_history.len() {
                        run_state = self.mapgen_next_state.unwrap();
                    }
                }
            }
            RunState::Dead => {
                let return_to_menu = gui::show_dead_screen(context);
                match return_to_menu {
                    None => {}
                    Some(_) => {
                        self.game_over_cleanup();
                        run_state = RunState::MainMenu {
                            menu_selection: menu::MainMenuSelection::NewGame,
                        }
                    }
                }
            }
            RunState::MagicMapReveal { row } => {
                let mut map = self.ecs.fetch_mut::<Map>();
                for x in 0..MAP_WIDTH {
                    let idx = map.xy_idx(x as i32, row);
                    map.revealed_tiles[idx] = true;
                }
                if row == MAP_HEIGHT - 1 {
                    run_state = RunState::MonsterTurn;
                } else {
                    run_state = RunState::MagicMapReveal { row: row + 1 };
                }
            }
            RunState::NextLevel => {
                self.goto_next_level();
                run_state = RunState::PreRun;
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
                            self.new_game();
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

                match *self.ecs.fetch::<RunState>() {
                    RunState::MagicMapReveal { .. } => {
                        run_state = RunState::MagicMapReveal { row: 0 }
                    }
                    _ => run_state = RunState::MonsterTurn,
                }
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
            RunState::ShowRemoveItem => {
                let result = gui::remove_item_menu(&mut self.ecs, context);
                match result.0 {
                    gui::ItemMenuResult::Cancel => run_state = RunState::AwaitingInput,
                    gui::ItemMenuResult::NoResponse => {}
                    gui::ItemMenuResult::Selected => {
                        let item_entity = result.1.unwrap();
                        let mut intent = self.ecs.write_storage::<components::WantsToRemoveItem>();
                        intent
                            .insert(
                                *self.ecs.fetch::<Entity>(),
                                components::WantsToRemoveItem { item: item_entity },
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

fn main() -> rltk::BError {
    use rltk::RltkBuilder;

    let mut context = RltkBuilder::simple80x50()
        .with_fitscreen(true)
        .with_title("Roguelike Tutorial")
        .build()?;
    context.with_post_scanlines(true);

    let mut gs = State {
        ecs: World::new(),
        mapgen_next_state: Some(RunState::MainMenu {
            menu_selection: menu::MainMenuSelection::NewGame,
        }),
        mapgen_index: 0,
        mapgen_history: Vec::new(),
        mapgen_timer: 0.0,
    };

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
    gs.ecs.register::<components::Equippable>();
    gs.ecs.register::<components::Equipped>();
    gs.ecs.register::<components::MeleePowerBonus>();
    gs.ecs.register::<components::DefenseBonus>();
    gs.ecs.register::<components::WantsToRemoveItem>();
    gs.ecs.register::<components::MagicMapper>();
    gs.ecs.register::<components::HungerClock>();
    gs.ecs.register::<components::Food>();
    gs.ecs.register::<components::EntryTrigger>();
    gs.ecs.register::<components::EntityMoved>();
    gs.ecs.register::<components::SingleActivation>();

    gs.add_new_world_details();

    rltk::main_loop(context, gs)
}
