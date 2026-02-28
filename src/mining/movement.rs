//! Player movement within the mine.
//!
//! While in the mine, the player moves on the mine grid using WASD / arrow keys.
//! This updates the ActiveFloor player grid position and drives ToolUseEvent
//! targeting for the tile the player is facing.

use bevy::prelude::*;

use crate::shared::*;
use super::components::*;
use super::floor_gen::{MINE_WIDTH, MINE_HEIGHT};

/// Timer resource to prevent movement from being too fast (grid-based).
#[derive(Resource, Debug)]
pub struct MineMoveCooldown {
    pub timer: Timer,
}

impl Default for MineMoveCooldown {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(0.15, TimerMode::Once),
        }
    }
}

/// System: handle player movement on the mine grid.
pub fn mine_player_movement(
    time: Res<Time>,
    player_input: Res<PlayerInput>,
    input_blocks: Res<InputBlocks>,
    mut active_floor: ResMut<ActiveFloor>,
    mut cooldown: ResMut<MineMoveCooldown>,
    rocks: Query<&MineGridPos, With<MineRock>>,
    in_mine: Res<InMine>,
    elevator_ui: Res<ElevatorUiOpen>,
    // Move the player entity's position to match
    mut player_query: Query<&mut LogicalPosition, With<Player>>,
) {
    if !in_mine.0 || !active_floor.spawned || elevator_ui.0 {
        return;
    }

    if input_blocks.is_blocked() {
        return;
    }

    cooldown.timer.tick(time.delta());
    if !cooldown.timer.finished() {
        return;
    }

    let mut dx = 0i32;
    let mut dy = 0i32;

    if player_input.move_axis.y > 0.0 {
        dy = 1;
    } else if player_input.move_axis.y < 0.0 {
        dy = -1;
    } else if player_input.move_axis.x < 0.0 {
        dx = -1;
    } else if player_input.move_axis.x > 0.0 {
        dx = 1;
    }

    if dx == 0 && dy == 0 {
        return;
    }

    let new_x = active_floor.player_grid_x + dx;
    let new_y = active_floor.player_grid_y + dy;

    // Bounds check (walls)
    if new_x < 1 || new_x >= MINE_WIDTH - 1 || new_y < 0 || new_y >= MINE_HEIGHT - 1 {
        return;
    }

    // Collision with rocks
    for grid_pos in rocks.iter() {
        if grid_pos.x == new_x && grid_pos.y == new_y {
            return; // Blocked by rock
        }
    }

    // Move the player
    active_floor.player_grid_x = new_x;
    active_floor.player_grid_y = new_y;
    cooldown.timer.reset();

    // Sync player entity position (sync system writes Transform in PostUpdate)
    for mut logical_pos in player_query.iter_mut() {
        logical_pos.0.x = new_x as f32 * TILE_SIZE;
        logical_pos.0.y = new_y as f32 * TILE_SIZE;
    }
}

/// System: when the player presses the action key (Space or E), generate a ToolUseEvent
/// targeting the tile the player is facing. Uses the currently equipped tool.
pub fn mine_player_action(
    player_input: Res<PlayerInput>,
    input_blocks: Res<InputBlocks>,
    active_floor: Res<ActiveFloor>,
    player_state: Res<PlayerState>,
    in_mine: Res<InMine>,
    elevator_ui: Res<ElevatorUiOpen>,
    player_movement: Query<&PlayerMovement, With<Player>>,
    mut tool_events: EventWriter<ToolUseEvent>,
) {
    if !in_mine.0 || !active_floor.spawned || elevator_ui.0 {
        return;
    }

    if input_blocks.is_blocked() {
        return;
    }

    // Only fire on press, not hold
    if !player_input.tool_use && !player_input.interact {
        return;
    }

    // Determine facing direction to pick the target tile
    let (dx, dy) = if let Ok(movement) = player_movement.get_single() {
        match movement.facing {
            Facing::Up => (0, 1),
            Facing::Down => (0, -1),
            Facing::Left => (-1, 0),
            Facing::Right => (1, 0),
        }
    } else {
        (0, 1) // default: face up
    };

    let target_x = active_floor.player_grid_x + dx;
    let target_y = active_floor.player_grid_y + dy;

    let tool = player_state.equipped_tool;
    let tier = player_state
        .tools
        .get(&tool)
        .copied()
        .unwrap_or(ToolTier::Basic);

    tool_events.send(ToolUseEvent {
        tool,
        tier,
        target_x,
        target_y,
    });
}
