use super::{xy_index, Player, Position, State, TileType, MAX_X, MAX_Y, MIN_X, MIN_Y};
use rltk::{Rltk, VirtualKeyCode};
use specs::prelude::*;
use std::cmp::{max, min};

fn try_move_player(delta_x: i32, delta_y: i32, ecs: &mut World) {
    let mut positions = ecs.write_storage::<Position>();
    let mut players = ecs.write_storage::<Player>();

    let map = ecs.fetch::<Vec<TileType>>();

    for (_player, pos) in (&mut players, &mut positions).join() {
        let destination_idx = xy_index(pos.x + delta_x, pos.y + delta_y);
        if map[destination_idx] != TileType::Wall {
            pos.x = min(MAX_X, max(MIN_X, pos.x + delta_x));
            pos.y = min(MAX_Y, max(MIN_Y, pos.y + delta_y));
        }
    }
}

pub fn player_input(state: &mut State, ctx: &mut Rltk) {
    // Player movement
    match ctx.key {
        None => {} // Nothing happened
        Some(key) => match key {
            VirtualKeyCode::Left | VirtualKeyCode::H => {
                try_move_player(-1, 0, &mut state.entity_component_system)
            }
            VirtualKeyCode::Right | VirtualKeyCode::L => {
                try_move_player(1, 0, &mut state.entity_component_system)
            }
            VirtualKeyCode::Up | VirtualKeyCode::K => {
                try_move_player(0, -1, &mut state.entity_component_system)
            }
            VirtualKeyCode::Down | VirtualKeyCode::J => {
                try_move_player(0, 1, &mut state.entity_component_system)
            }
            _ => {}
        },
    }
}
