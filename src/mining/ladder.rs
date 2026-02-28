//! Ladder interaction and elevator system.
//!
//! - When the player steps on a revealed ladder, descend to the next floor.
//! - Every 5 floors, an elevator stop is unlocked.
//! - The elevator allows choosing any unlocked floor when entering the mine.

use bevy::prelude::*;

use crate::shared::*;
use super::components::*;

/// System: detect when the player stands on the revealed ladder and descend.
pub fn handle_ladder_interaction(
    mut mine_state: ResMut<MineState>,
    mut active_floor: ResMut<ActiveFloor>,
    mut floor_req: ResMut<FloorSpawnRequest>,
    ladders: Query<(&MineGridPos, &MineLadder)>,
    in_mine: Res<InMine>,
    player_input: Res<PlayerInput>,
    input_blocks: Res<InputBlocks>,
    mut sfx_events: EventWriter<PlaySfxEvent>,
) {
    if !in_mine.0 || !active_floor.spawned {
        return;
    }

    if input_blocks.is_blocked() {
        return;
    }

    // Player must press Space/Enter to interact with ladder
    if !player_input.tool_use && !player_input.ui_confirm {
        return;
    }

    let px = active_floor.player_grid_x;
    let py = active_floor.player_grid_y;

    for (grid_pos, ladder) in ladders.iter() {
        if grid_pos.x == px && grid_pos.y == py && ladder.revealed {
            // Descend!
            let next_floor = mine_state.current_floor + 1;

            // Cap at floor 20
            if next_floor > 20 {
                return;
            }

            sfx_events.send(PlaySfxEvent {
                sfx_id: "mine_descend".to_string(),
            });

            mine_state.current_floor = next_floor;

            // Track deepest floor
            if next_floor > mine_state.deepest_floor_reached {
                mine_state.deepest_floor_reached = next_floor;
            }

            // Unlock elevator every 5 floors
            const ELEVATOR_FLOOR_INTERVAL: u8 = 5;
            if next_floor % ELEVATOR_FLOOR_INTERVAL == 0 && !mine_state.elevator_floors.contains(&next_floor) {
                mine_state.elevator_floors.push(next_floor);
                mine_state.elevator_floors.sort();
            }

            // Request new floor spawn
            floor_req.pending = true;
            floor_req.floor = next_floor;
            active_floor.spawned = false;

            return;
        }
    }
}

/// System: detect when the player steps on the exit tile to leave the mine.
pub fn handle_mine_exit(
    mut mine_state: ResMut<MineState>,
    mut active_floor: ResMut<ActiveFloor>,
    mut in_mine: ResMut<InMine>,
    exits: Query<&MineGridPos, With<MineExit>>,
    player_input: Res<PlayerInput>,
    input_blocks: Res<InputBlocks>,
    mut map_events: EventWriter<MapTransitionEvent>,
    mut sfx_events: EventWriter<PlaySfxEvent>,
) {
    if !in_mine.0 || !active_floor.spawned {
        return;
    }

    if input_blocks.is_blocked() {
        return;
    }

    // Player must press Space/Enter near exit, or be standing on it
    if !player_input.tool_use && !player_input.ui_confirm {
        return;
    }

    let px = active_floor.player_grid_x;
    let py = active_floor.player_grid_y;

    for grid_pos in exits.iter() {
        let dist = (grid_pos.x - px).abs() + (grid_pos.y - py).abs();
        if dist <= 1 {
            sfx_events.send(PlaySfxEvent {
                sfx_id: "mine_exit".to_string(),
            });

            // Reset mine state
            mine_state.current_floor = 0;
            in_mine.0 = false;
            active_floor.spawned = false;

            // Transition back to mine entrance
            map_events.send(MapTransitionEvent {
                to_map: MapId::MineEntrance,
                to_x: 12,
                to_y: 12,
            });

            return;
        }
    }
}

/// System: handle elevator floor selection.
/// When the player first enters the mine (via MapTransitionEvent to Mine),
/// and has elevator floors unlocked, they choose a starting floor.
/// For simplicity, pressing number keys 1-4 selects elevator stops.
/// The elevator UI is managed by the UI domain; here we just handle the
/// selection input when ElevatorUiOpen is true.
pub fn handle_elevator_selection(
    mut mine_state: ResMut<MineState>,
    mut floor_req: ResMut<FloorSpawnRequest>,
    mut elevator_ui: ResMut<ElevatorUiOpen>,
    mut active_floor: ResMut<ActiveFloor>,
    player_input: Res<PlayerInput>,
    input_blocks: Res<InputBlocks>,
    in_mine: Res<InMine>,
    mut sfx_events: EventWriter<PlaySfxEvent>,
) {
    if !elevator_ui.0 || !in_mine.0 {
        return;
    }

    if input_blocks.is_blocked() {
        return;
    }

    // Floor 0 (ground) is always available — mapped to key 1 (tool_slot Some(0))
    // Elevator floors are mapped to keys 2, 3, 4, etc.
    let mut selected_floor: Option<u8> = None;

    match player_input.tool_slot {
        Some(0) => {
            // Key 1 → floor 1 (ground)
            selected_floor = Some(1);
        }
        Some(1) => {
            // Key 2 → first elevator stop
            if let Some(&floor) = mine_state.elevator_floors.get(0) {
                selected_floor = Some(floor);
            }
        }
        Some(2) => {
            // Key 3 → second elevator stop
            if let Some(&floor) = mine_state.elevator_floors.get(1) {
                selected_floor = Some(floor);
            }
        }
        Some(3) => {
            // Key 4 → third elevator stop
            if let Some(&floor) = mine_state.elevator_floors.get(2) {
                selected_floor = Some(floor);
            }
        }
        _ => {}
    }

    if selected_floor.is_none() && player_input.ui_cancel {
        // Cancel elevator, go to floor 1
        selected_floor = Some(1);
    }

    if let Some(floor) = selected_floor {
        sfx_events.send(PlaySfxEvent {
            sfx_id: "mine_elevator".to_string(),
        });

        mine_state.current_floor = floor;
        elevator_ui.0 = false;

        floor_req.pending = true;
        floor_req.floor = floor;
        active_floor.spawned = false;
    }
}
