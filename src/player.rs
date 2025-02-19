use raylib::prelude::*;
use crate::Point;
use crate::RunState;
use crate::WantsToMelee;

use super::{Position, Player, TileType,Map,  State, Viewshed, CombatStats, EventLog};
use std::cmp::{min, max};
use specs::prelude::*;

pub fn try_move_player(delta_x: i32, delta_y: i32, ecs: &mut World) {
    let mut positions = ecs.write_storage::<Position>();
    let mut players = ecs.write_storage::<Player>();
    let mut viewsheds = ecs.write_storage::<Viewshed>();
    let combat_stats = ecs.read_storage::<CombatStats>();
    let mut log = ecs.fetch_mut::<EventLog>();
    let map = ecs.fetch::<Map>();
    let entites = ecs.entities();
    let mut wants_to_melee = ecs.write_storage::<WantsToMelee>();

    for (entity,_player, pos, viewshed) in (&entites,&mut players, &mut positions, &mut viewsheds).join() {
        //if the spot is vacant, or occupied by a wall
        if pos.x + delta_x < 1 || pos.x + delta_x > map.width-1 || pos.y + delta_y < 1 || pos.y + delta_y > map.height-1 { return; }
        let destination_idx = map.xy_idx(pos.x + delta_x, pos.y + delta_y);
        
            for potential_target in map.tile_content[destination_idx].iter() {
                let target = combat_stats.get(*potential_target);
                if let Some(_target) = target {
                    wants_to_melee.insert(entity, WantsToMelee{ target: *potential_target }).expect("Add target failed");
                    return;
                }
            }
        if !map.blocked[destination_idx] {
            pos.x = min(79 , max(0, pos.x + delta_x));
            pos.y = min(49, max(0, pos.y + delta_y));
            let mut ppos = ecs.write_resource::<Point>();
            ppos.x = pos.x;
            ppos.y = pos.y;

            viewshed.dirty = true;
        }
    }
}

pub fn player_input(gs: &mut State, handle: &RaylibHandle) -> RunState {
    // Player movement
    if handle.is_key_pressed(KeyboardKey::KEY_LEFT) || handle.is_key_pressed(KeyboardKey::KEY_H) {
        try_move_player(-1, 0, &mut gs.ecs);
        RunState::PlayerTurn
    } else if handle.is_key_pressed(KeyboardKey::KEY_RIGHT)  || handle.is_key_pressed(KeyboardKey::KEY_L) {
        try_move_player(1, 0, &mut gs.ecs);
        RunState::PlayerTurn
    } else if handle.is_key_pressed(KeyboardKey::KEY_UP)  || handle.is_key_pressed(KeyboardKey::KEY_K) {
        try_move_player(0, -1, &mut gs.ecs);
        RunState::PlayerTurn
    } else if handle.is_key_pressed(KeyboardKey::KEY_DOWN)  || handle.is_key_pressed(KeyboardKey::KEY_J) {
        try_move_player(0, 1, &mut gs.ecs);
        RunState::PlayerTurn
    } else if handle.is_key_pressed(KeyboardKey::KEY_Y) {
        try_move_player(-1, -1, &mut gs.ecs);
        RunState::PlayerTurn
    } else if handle.is_key_pressed(KeyboardKey::KEY_U) {
        try_move_player(1, -1, &mut gs.ecs);
        RunState::PlayerTurn
    } else if handle.is_key_pressed(KeyboardKey::KEY_N) {
        try_move_player(1, 1, &mut gs.ecs);
        RunState::PlayerTurn
    } else if handle.is_key_pressed(KeyboardKey::KEY_B) {
        try_move_player(-1, 1, &mut gs.ecs);
        RunState::PlayerTurn
    } else {
        RunState::AwaitingInput
    }
}





