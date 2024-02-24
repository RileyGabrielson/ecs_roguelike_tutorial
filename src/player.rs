use super::{components, GameLog, Map, RunState, State, MAX_X, MAX_Y, MIN_X, MIN_Y};
use rltk::{Point, Rltk, VirtualKeyCode};
use specs::prelude::*;
use std::cmp::{max, min};

pub fn player_input(gs: &mut State, ctx: &mut Rltk) -> RunState {
    // Player movement
    match ctx.key {
        None => return RunState::AwaitingInput, // Nothing happened
        Some(key) => match key {
            VirtualKeyCode::G => get_item(&mut gs.ecs),
            VirtualKeyCode::I => return RunState::ShowInventory,
            VirtualKeyCode::D => return RunState::ShowDropItem,
            VirtualKeyCode::Escape => return RunState::SaveGame,

            // Cardinal Directions
            VirtualKeyCode::Left | VirtualKeyCode::Numpad4 | VirtualKeyCode::H => {
                try_move_player(-1, 0, &mut gs.ecs)
            }

            VirtualKeyCode::Right | VirtualKeyCode::Numpad6 | VirtualKeyCode::L => {
                try_move_player(1, 0, &mut gs.ecs)
            }

            VirtualKeyCode::Up | VirtualKeyCode::Numpad8 | VirtualKeyCode::K => {
                try_move_player(0, -1, &mut gs.ecs)
            }

            VirtualKeyCode::Down | VirtualKeyCode::Numpad2 | VirtualKeyCode::J => {
                try_move_player(0, 1, &mut gs.ecs)
            }

            // Diagonals
            VirtualKeyCode::Numpad9 | VirtualKeyCode::U => try_move_player(1, -1, &mut gs.ecs),

            VirtualKeyCode::Numpad7 | VirtualKeyCode::Y => try_move_player(-1, -1, &mut gs.ecs),

            VirtualKeyCode::Numpad3 | VirtualKeyCode::N => try_move_player(1, 1, &mut gs.ecs),

            VirtualKeyCode::Numpad1 | VirtualKeyCode::B => try_move_player(-1, 1, &mut gs.ecs),

            _ => return RunState::AwaitingInput,
        },
    }
    RunState::PlayerTurn
}

pub fn try_move_player(delta_x: i32, delta_y: i32, ecs: &mut World) {
    let mut positions = ecs.write_storage::<components::Position>();
    let mut players = ecs.write_storage::<components::Player>();
    let mut viewsheds = ecs.write_storage::<components::Viewshed>();
    let combat_stats = ecs.read_storage::<components::CombatStats>();
    let entities = ecs.entities();
    let mut wants_to_melee = ecs.write_storage::<components::WantsToMelee>();
    let map = ecs.fetch::<Map>();

    for (entity, _player, pos, viewshed) in
        (&entities, &mut players, &mut positions, &mut viewsheds).join()
    {
        let destination_idx = map.xy_idx(pos.x + delta_x, pos.y + delta_y);

        let entities_at_destination = &map.tile_content[destination_idx];

        for potential_target in entities_at_destination {
            let target = combat_stats.get(*potential_target);
            match target {
                None => {}
                Some(_t) => {
                    wants_to_melee
                        .insert(
                            entity,
                            components::WantsToMelee {
                                target: *potential_target,
                            },
                        )
                        .expect("Add Target Failed");
                    return; // So we don't move after attacking
                }
            }
        }

        if !map.blocked[destination_idx] {
            pos.x = min(MAX_X, max(MIN_X, pos.x + delta_x));
            pos.y = min(MAX_Y, max(MIN_Y, pos.y + delta_y));

            viewshed.dirty = true;
            let mut ppos = ecs.write_resource::<Point>();
            ppos.x = pos.x;
            ppos.y = pos.y;
        }
    }
}

fn get_item(ecs: &mut World) {
    let player_pos = ecs.fetch::<Point>();
    let player_entity = ecs.fetch::<Entity>();
    let entities = ecs.entities();
    let items = ecs.read_storage::<components::Item>();
    let positions = ecs.read_storage::<components::Position>();
    let mut gamelog = ecs.fetch_mut::<GameLog>();

    let mut target_item: Option<Entity> = None;
    for (item_entity, _item, position) in (&entities, &items, &positions).join() {
        if position.x == player_pos.x && position.y == player_pos.y {
            target_item = Some(item_entity);
        }
    }

    match target_item {
        None => gamelog
            .entries
            .push("There is nothing here to pick up.".to_string()),
        Some(item) => {
            let mut pickup = ecs.write_storage::<components::WantsToPickupItem>();
            pickup
                .insert(
                    *player_entity,
                    components::WantsToPickupItem {
                        collected_by: *player_entity,
                        item,
                    },
                )
                .expect("Unable to insert want to pickup");
        }
    }
}
