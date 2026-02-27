use bevy::prelude::*;
use crate::shared::*;
use super::{ToolCooldown, stamina_cost, facing_offset, TOOL_ORDER};

/// Cycle the equipped tool forward (E) or backward (Q).
pub fn tool_cycle(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut player_state: ResMut<PlayerState>,
    input_blocks: Res<InputBlocks>,
) {
    if input_blocks.is_blocked() {
        return;
    }
    let current_idx = TOOL_ORDER
        .iter()
        .position(|t| *t == player_state.equipped_tool)
        .unwrap_or(0);

    if keyboard.just_pressed(KeyCode::KeyE) {
        let next = (current_idx + 1) % TOOL_ORDER.len();
        player_state.equipped_tool = TOOL_ORDER[next];
    }

    if keyboard.just_pressed(KeyCode::KeyQ) {
        let prev = if current_idx == 0 {
            TOOL_ORDER.len() - 1
        } else {
            current_idx - 1
        };
        player_state.equipped_tool = TOOL_ORDER[prev];
    }
}

/// Use the currently equipped tool on the tile the player is facing.
/// Sends a `ToolUseEvent` and a `StaminaDrainEvent` when successful.
pub fn tool_use(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    player_state: Res<PlayerState>,
    input_blocks: Res<InputBlocks>,
    mut cooldown: ResMut<ToolCooldown>,
    query: Query<(&Transform, &PlayerMovement), With<Player>>,
    mut tool_events: EventWriter<ToolUseEvent>,
    mut stamina_events: EventWriter<StaminaDrainEvent>,
    mut sfx_events: EventWriter<PlaySfxEvent>,
) {
    if input_blocks.is_blocked() {
        return;
    }

    // Tick the cooldown.
    cooldown.timer.tick(time.delta());

    let use_pressed = keyboard.just_pressed(KeyCode::Space)
        || keyboard.just_pressed(KeyCode::Enter);

    if !use_pressed {
        return;
    }

    // Don't allow use while cooldown is active.
    if !cooldown.timer.finished() {
        return;
    }

    let Ok((transform, movement)) = query.get_single() else {
        return;
    };

    let tool = player_state.equipped_tool;
    let cost = stamina_cost(&tool);

    // Check stamina — disallow if insufficient.
    if player_state.stamina < cost {
        // Could send a UI notification event here.
        return;
    }

    // Calculate target tile: player's current grid + facing offset.
    let (px, py) = super::world_to_grid(transform.translation.x, transform.translation.y);
    let (dx, dy) = facing_offset(&movement.facing);
    let target_x = px + dx;
    let target_y = py + dy;

    // Determine tool tier.
    let tier = player_state
        .tools
        .get(&tool)
        .copied()
        .unwrap_or(ToolTier::Basic);

    // Send the tool use event for other domains to react.
    tool_events.send(ToolUseEvent {
        tool,
        tier,
        target_x,
        target_y,
    });

    // Drain stamina.
    stamina_events.send(StaminaDrainEvent { amount: cost });

    // Play a sound effect for the tool.
    let sfx_id = match tool {
        ToolKind::Hoe => "sfx_hoe",
        ToolKind::WateringCan => "sfx_water",
        ToolKind::Axe => "sfx_axe",
        ToolKind::Pickaxe => "sfx_pickaxe",
        ToolKind::FishingRod => "sfx_cast",
        ToolKind::Scythe => "sfx_scythe",
    };
    sfx_events.send(PlaySfxEvent {
        sfx_id: sfx_id.to_string(),
    });

    // Reset cooldown.
    cooldown.timer.reset();
}

/// Read `StaminaDrainEvent`s and apply them to `PlayerState.stamina`.
pub fn stamina_drain_handler(
    mut events: EventReader<StaminaDrainEvent>,
    mut player_state: ResMut<PlayerState>,
) {
    for ev in events.read() {
        player_state.stamina = (player_state.stamina - ev.amount).max(0.0);
    }
}
