//! Cast detection, bite timer, and reaction window logic.

use bevy::prelude::*;
use rand::Rng;

use crate::shared::*;
use super::{Bobber, FishingPhase, FishingState, FishingMinigameState};
use super::fish_select::select_fish;
use super::resolve::end_fishing_escape;

// ─── Constants ───────────────────────────────────────────────────────────────

const BITE_TIMER_MIN: f32 = 2.0;
const BITE_TIMER_MAX: f32 = 8.0;
const REACTION_WINDOW: f32 = 1.0; // seconds to press Space after bite

// ─── Systems ─────────────────────────────────────────────────────────────────

/// Listen for ToolUseEvent with FishingRod.
/// If the target tile is water, enter fishing state.
pub fn handle_tool_use_for_fishing(
    mut tool_events: EventReader<ToolUseEvent>,
    mut fishing_state: ResMut<FishingState>,
    mut commands: Commands,
    player_state: Res<PlayerState>,
    inventory: Res<Inventory>,
    mut sfx_events: EventWriter<PlaySfxEvent>,
    // Query the world map tile kinds to check if target is water
    // Since we don't have a direct tile query API, we approximate by checking
    // if the cast position is reasonable (water detection via map bounds/known areas).
    // In a full implementation, the World plugin would expose a tile query resource.
) {
    for event in tool_events.read() {
        if event.tool != ToolKind::FishingRod {
            continue;
        }

        // Guard: only cast if idle
        if fishing_state.phase != FishingPhase::Idle {
            continue;
        }

        // Determine if target is water based on current map context.
        // The full world tile query is owned by the world domain; here we cast
        // optimistically and trust the player/world domain to only send FishingRod
        // events on valid water tiles.
        let target_x = event.target_x;
        let target_y = event.target_y;

        // Check bait / tackle from inventory
        let bait_equipped = inventory.has("bait", 1);
        let tackle_equipped = inventory.has("tackle", 1);
        let rod_tier = player_state
            .tools
            .get(&ToolKind::FishingRod)
            .copied()
            .unwrap_or(ToolTier::Basic);

        // Compute bite timer (bait halves the wait)
        let mut rng = rand::thread_rng();
        let base_wait = rng.gen_range(BITE_TIMER_MIN..BITE_TIMER_MAX);
        let wait = if bait_equipped {
            base_wait * 0.5
        } else {
            base_wait
        };

        // Update fishing state
        fishing_state.phase = FishingPhase::WaitingForBite;
        fishing_state.bobber_tile = (target_x, target_y);
        fishing_state.bobber_pos = Vec2::new(
            target_x as f32 * TILE_SIZE,
            target_y as f32 * TILE_SIZE,
        );
        fishing_state.bite_timer = Some(Timer::from_seconds(wait, TimerMode::Once));
        fishing_state.bait_equipped = bait_equipped;
        fishing_state.tackle_equipped = tackle_equipped;
        fishing_state.rod_tier = rod_tier;

        // Spawn bobber sprite
        let bobber_world_x = target_x as f32 * TILE_SIZE * PIXEL_SCALE;
        let bobber_world_y = target_y as f32 * TILE_SIZE * PIXEL_SCALE;
        commands.spawn((
            Sprite {
                color: Color::srgb(0.9, 0.2, 0.2), // Red bobber placeholder
                custom_size: Some(Vec2::new(6.0, 8.0)),
                ..default()
            },
            Transform::from_translation(Vec3::new(bobber_world_x, bobber_world_y, 5.0)),
            Bobber {
                bob_timer: Timer::from_seconds(0.5, TimerMode::Repeating),
                bob_direction: 1.0,
                original_y: bobber_world_y,
            },
        ));

        sfx_events.write(PlaySfxEvent {
            sfx_id: "fishing_cast".to_string(),
        });

        // Consume bait if equipped
        // (ItemRemovedEvent is not in shared, so we do it via the fishing domain's
        // consume logic — we don't have cross-domain item removal here, so we
        // note this with a TODO for the player domain to handle)
        // TODO: send ItemRemovedEvent for bait consumption
    }
}

/// Count down bite timer; when it fires, signal that a fish has bitten.
pub fn update_bite_timer(
    mut fishing_state: ResMut<FishingState>,
    mut sfx_events: EventWriter<PlaySfxEvent>,
    time: Res<Time>,
    fish_registry: Res<FishRegistry>,
    calendar: Res<Calendar>,
    player_state: Res<PlayerState>,
) {
    if fishing_state.phase != FishingPhase::WaitingForBite {
        return;
    }

    let timer_finished = if let Some(ref mut timer) = fishing_state.bite_timer {
        timer.tick(time.delta());
        timer.just_finished()
    } else {
        false
    };

    if timer_finished {
        // Select a fish to bite
        let fish_id = select_fish(
            &fish_registry,
            &player_state,
            &calendar,
        );

        fishing_state.selected_fish_id = fish_id;
        fishing_state.phase = FishingPhase::BitePending;
        fishing_state.bite_timer = None;
        fishing_state.reaction_timer = Some(Timer::from_seconds(REACTION_WINDOW, TimerMode::Once));

        // Play bite sound — bobber dip signal
        sfx_events.write(PlaySfxEvent {
            sfx_id: "fish_bite".to_string(),
        });
    }
}

/// Handle the reaction window: player must press Space to start minigame.
pub fn handle_bite_reaction_window(
    mut fishing_state: ResMut<FishingState>,
    mut minigame_state: ResMut<FishingMinigameState>,
    mut next_state: ResMut<NextState<GameState>>,
    mut stamina_events: EventWriter<StaminaDrainEvent>,
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    fish_registry: Res<FishRegistry>,
    bobber_query: Query<Entity, With<Bobber>>,
    mut commands: Commands,
) {
    if fishing_state.phase != FishingPhase::BitePending {
        return;
    }

    let space_pressed = keyboard.just_pressed(KeyCode::Space);

    let reaction_expired = if let Some(ref mut timer) = fishing_state.reaction_timer {
        timer.tick(time.delta());
        timer.just_finished()
    } else {
        false
    };

    if space_pressed {
        // Player reacted in time — start the minigame
        fishing_state.phase = FishingPhase::Minigame;
        fishing_state.reaction_timer = None;

        // Look up difficulty for the selected fish
        let difficulty = if let Some(ref fish_id) = fishing_state.selected_fish_id.clone() {
            fish_registry
                .fish
                .get(fish_id)
                .map(|f| f.difficulty)
                .unwrap_or(0.5)
        } else {
            0.5
        };

        // Setup minigame state
        minigame_state.setup(
            difficulty,
            fishing_state.rod_tier,
            fishing_state.tackle_equipped,
        );

        // Transition to Fishing game state (minigame UI will be spawned by OnEnter)
        next_state.set(GameState::Fishing);
    } else if reaction_expired {
        // Fish got away — the player was too slow
        let bobber_entities: Vec<Entity> = bobber_query.iter().collect();
        end_fishing_escape(
            &mut fishing_state,
            &mut next_state,
            &mut stamina_events,
            &mut commands,
            bobber_entities,
            false, // not transitioning from Fishing state, still in Playing
        );
    }
}

/// Allow the player to cancel fishing by pressing Escape while waiting.
pub fn handle_cancel_fishing(
    mut fishing_state: ResMut<FishingState>,
    mut next_state: ResMut<NextState<GameState>>,
    mut stamina_events: EventWriter<StaminaDrainEvent>,
    keyboard: Res<ButtonInput<KeyCode>>,
    bobber_query: Query<Entity, With<Bobber>>,
    mut commands: Commands,
) {
    if fishing_state.phase == FishingPhase::Idle {
        return;
    }
    if fishing_state.phase == FishingPhase::Minigame {
        // Escape in minigame is handled by the minigame module
        return;
    }

    if keyboard.just_pressed(KeyCode::Escape) {
        let bobber_entities: Vec<Entity> = bobber_query.iter().collect();
        end_fishing_escape(
            &mut fishing_state,
            &mut next_state,
            &mut stamina_events,
            &mut commands,
            bobber_entities,
            false,
        );
    }
}
