use super::{Map, Player, Position, RunState, State, Viewshed, MAX_X, MAX_Y, MIN_X, MIN_Y};
use rltk::{Point, Rltk, VirtualKeyCode};
use specs::prelude::*;
use std::cmp::{max, min};

pub fn try_move_player(delta_x: i32, delta_y: i32, ecs: &mut World) {
    let mut positions = ecs.write_storage::<Position>();
    let mut players = ecs.write_storage::<Player>();
    let mut viewsheds = ecs.write_storage::<Viewshed>();
    let map = ecs.fetch::<Map>();

    for (_player, pos, viewshed) in (&mut players, &mut positions, &mut viewsheds).join() {
        let destination_idx = map.xy_idx(pos.x + delta_x, pos.y + delta_y);
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

pub fn player_input(gs: &mut State, ctx: &mut Rltk) -> RunState {
    // Player movement
    match ctx.key {
        None => return RunState::Paused, // Nothing happened
        Some(key) => match key {
            VirtualKeyCode::Left | VirtualKeyCode::Numpad4 | VirtualKeyCode::H => {
                try_move_player(-1, 0, &mut gs.entity_component_system)
            }

            VirtualKeyCode::Right | VirtualKeyCode::Numpad6 | VirtualKeyCode::L => {
                try_move_player(1, 0, &mut gs.entity_component_system)
            }

            VirtualKeyCode::Up | VirtualKeyCode::Numpad8 | VirtualKeyCode::K => {
                try_move_player(0, -1, &mut gs.entity_component_system)
            }

            VirtualKeyCode::Down | VirtualKeyCode::Numpad2 | VirtualKeyCode::J => {
                try_move_player(0, 1, &mut gs.entity_component_system)
            }

            // Diagonals
            VirtualKeyCode::Numpad9 | VirtualKeyCode::U => {
                try_move_player(1, -1, &mut gs.entity_component_system)
            }

            VirtualKeyCode::Numpad7 | VirtualKeyCode::Y => {
                try_move_player(-1, -1, &mut gs.entity_component_system)
            }

            VirtualKeyCode::Numpad3 | VirtualKeyCode::N => {
                try_move_player(1, 1, &mut gs.entity_component_system)
            }

            VirtualKeyCode::Numpad1 | VirtualKeyCode::B => {
                try_move_player(-1, 1, &mut gs.entity_component_system)
            }

            _ => return RunState::Paused,
        },
    }
    RunState::Running
}
