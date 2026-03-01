//! Cast detection, bite timer, and reaction window logic.

use bevy::prelude::*;
use rand::Rng;

use crate::shared::*;
use super::{Bobber, FishingPhase, FishingState, FishingMinigameState, TackleKind};
use super::fish_select::select_fish;
use super::resolve::end_fishing_escape;
use super::skill::FishingSkill;
use super::legendaries::is_legendary;

// ─── Constants ───────────────────────────────────────────────────────────────

const BITE_TIMER_MIN: f32 = 2.0;
const BITE_TIMER_MAX: f32 = 8.0;
const REACTION_WINDOW: f32 = 1.0; // seconds to press Space after bite

// ─── Bait helpers ─────────────────────────────────────────────────────────────

/// Returns a multiplier for the bite-wait timer based on the equipped bait type.
///
/// A multiplier < 1.0 means bites arrive faster.
///
/// | Bait ID       | Multiplier | Effect                                    |
/// |---------------|------------|-------------------------------------------|
/// | worm_bait     | 0.75       | 25% faster bite                           |
/// | magnet_bait   | 1.00       | Normal speed — bonus is treasure chance   |
/// | wild_bait     | 0.70       | 30% faster bite + 15% double-catch chance |
/// | (generic bait)| 0.85       | 15% faster bite                           |
/// | (unknown)     | 1.00       | No speed bonus                            |
pub fn bait_bite_multiplier(bait_id: &str) -> f32 {
    match bait_id {
        "worm_bait"   => 0.75,
        "magnet_bait" => 1.00, // magnet bait benefits treasure, not speed
        "wild_bait"   => 0.70,
        "bait"        => 0.85, // generic bait = moderate 15% faster
        _             => 1.00, // unknown bait IDs get no speed bonus
    }
}

/// Check the player's inventory for a known bait item. Returns the item ID
/// of the first matching bait found, or `None` if no bait is equipped.
///
/// Priority: wild_bait > magnet_bait > worm_bait > generic bait
fn detect_bait(inventory: &Inventory) -> Option<String> {
    let priority = ["wild_bait", "magnet_bait", "worm_bait", "bait"];
    for &id in &priority {
        if inventory.has(id, 1) {
            return Some(id.to_string());
        }
    }
    None
}

// ─── Systems ─────────────────────────────────────────────────────────────────

/// Listen for ToolUseEvent with FishingRod.
/// When the player uses the fishing rod, start the fishing sequence.
pub fn handle_tool_use_for_fishing(
    mut tool_events: EventReader<ToolUseEvent>,
    mut fishing_state: ResMut<FishingState>,
    mut commands: Commands,
    player_state: Res<PlayerState>,
    mut inventory: ResMut<Inventory>,
    mut sfx_events: EventWriter<PlaySfxEvent>,
    mut item_removed_events: EventWriter<ItemRemovedEvent>,
    skill: Res<FishingSkill>,
    _toast_events: EventWriter<ToastEvent>,
) {
    for event in tool_events.read() {
        if event.tool != ToolKind::FishingRod {
            continue;
        }

        // Guard: only cast if idle
        if fishing_state.phase != FishingPhase::Idle {
            continue;
        }

        let target_x = event.target_x;
        let target_y = event.target_y;

        // Detect bait type — priority ordering is handled inside detect_bait
        let bait_id = detect_bait(&inventory);
        let bait_equipped = bait_id.is_some();

        // Detect specific tackle type. Priority order: spinner > trap_bobber > lead_bobber > generic.
        let tackle_kind = if inventory.has("spinner", 1) {
            TackleKind::Spinner
        } else if inventory.has("trap_bobber", 1) {
            TackleKind::TrapBobber
        } else if inventory.has("lead_bobber", 1) {
            TackleKind::LeadBobber
        } else if inventory.has("tackle", 1) {
            // Generic tackle item (treated as no specific modifier).
            TackleKind::None
        } else {
            TackleKind::None
        };
        let tackle_equipped = tackle_kind != TackleKind::None || inventory.has("tackle", 1);

        let rod_tier = player_state
            .tools
            .get(&ToolKind::FishingRod)
            .copied()
            .unwrap_or(ToolTier::Basic);

        // Compute bite timer:
        //  1. Start with a random base in [BITE_TIMER_MIN, BITE_TIMER_MAX].
        //  2. Apply bait multiplier (type-specific).
        //  3. Apply fishing skill bite-speed bonus.
        let mut rng = rand::thread_rng();
        let base_wait = rng.gen_range(BITE_TIMER_MIN..BITE_TIMER_MAX);

        let bait_mult = match &bait_id {
            Some(id) => bait_bite_multiplier(id),
            None => 1.0,
        };
        let wait_after_bait = base_wait * bait_mult;
        // Clamp to a minimum of 0.5s so max bait+skill never yields an instant bite.
        let wait = skill.apply_bite_speed(wait_after_bait).max(0.5);

        // Update fishing state
        fishing_state.phase = FishingPhase::WaitingForBite;
        fishing_state.bobber_tile = (target_x, target_y);
        fishing_state.bobber_pos = Vec2::new(
            target_x as f32 * TILE_SIZE,
            target_y as f32 * TILE_SIZE,
        );
        fishing_state.bite_timer = Some(Timer::from_seconds(wait, TimerMode::Once));
        fishing_state.bait_id = bait_id.clone();
        fishing_state.bait_equipped = bait_equipped;
        fishing_state.tackle_equipped = tackle_equipped;
        fishing_state.tackle_kind = tackle_kind;
        fishing_state.rod_tier = rod_tier;

        // Spawn bobber sprite in world space (grid * TILE_SIZE)
        let bobber_world_x = target_x as f32 * TILE_SIZE;
        let bobber_world_y = target_y as f32 * TILE_SIZE;
        commands.spawn((
            Sprite {
                color: Color::srgb(1.0, 0.3, 0.2), // Bright red bobber
                custom_size: Some(Vec2::new(6.0, 8.0)),
                ..default()
            },
            Transform::from_translation(Vec3::new(bobber_world_x, bobber_world_y, Z_EFFECTS)),
            Bobber {
                bob_timer: Timer::from_seconds(0.5, TimerMode::Repeating),
                _bob_direction: 1.0,
                original_y: bobber_world_y,
            },
        ));

        sfx_events.send(PlaySfxEvent {
            sfx_id: "fishing_cast".to_string(),
        });

        // Consume one unit of the equipped bait
        if let Some(ref used_bait_id) = bait_id {
            let removed = inventory.try_remove(used_bait_id, 1);
            if removed > 0 {
                item_removed_events.send(ItemRemovedEvent {
                    item_id: used_bait_id.clone(),
                    quantity: removed,
                });
            }
        }
    }
}

/// Count down bite timer; when it fires, signal that a fish has bitten.
pub fn update_bite_timer(
    mut fishing_state: ResMut<FishingState>,
    mut sfx_events: EventWriter<PlaySfxEvent>,
    mut toast_events: EventWriter<ToastEvent>,
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
        // Select a fish to bite based on location, season, time, weather.
        // Legendary check is embedded inside select_fish via try_roll_legendary.
        let fish_id = select_fish(&fish_registry, &player_state, &calendar);

        // If the selected fish is legendary, show a subtle hint so the player
        // knows something special might happen.
        if let Some(ref id) = fish_id {
            if is_legendary(id) {
                toast_events.send(ToastEvent {
                    message: "Something legendary is biting...".to_string(),
                    duration_secs: 2.0,
                });
            }
        }

        fishing_state.selected_fish_id = fish_id;
        fishing_state.phase = FishingPhase::BitePending;
        fishing_state.bite_timer = None;
        fishing_state.reaction_timer = Some(Timer::from_seconds(REACTION_WINDOW, TimerMode::Once));

        // Play bite sound — bobber dips visually
        sfx_events.send(PlaySfxEvent {
            sfx_id: "fish_bite".to_string(),
        });
    }
}

/// Handle the reaction window: player must press Space to start the minigame.
/// If the reaction window expires, the fish escapes.
pub fn handle_bite_reaction_window(
    mut fishing_state: ResMut<FishingState>,
    mut minigame_state: ResMut<FishingMinigameState>,
    mut next_state: ResMut<NextState<GameState>>,
    mut stamina_events: EventWriter<StaminaDrainEvent>,
    player_input: Res<PlayerInput>,
    input_blocks: Res<InputBlocks>,
    time: Res<Time>,
    fish_registry: Res<FishRegistry>,
    bobber_query: Query<Entity, With<Bobber>>,
    mut commands: Commands,
    skill: Res<FishingSkill>,
) {
    if fishing_state.phase != FishingPhase::BitePending {
        return;
    }

    if input_blocks.is_blocked() {
        return;
    }

    let space_pressed = player_input.tool_use;

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

        // Look up fish difficulty. For legendary fish, use the table difficulty
        // directly (may differ from what's in the registry if the data layer
        // hasn't been loaded yet).
        let fish_id_opt = fishing_state.selected_fish_id.clone();
        let difficulty = if let Some(ref fish_id) = fish_id_opt {
            // Try legendary table first for accuracy
            let legendary_difficulty = super::legendaries::LEGENDARY_FISH
                .iter()
                .find(|&&(id, _, _, _, _)| id == fish_id)
                .map(|&(_, _, _, diff, _)| diff);

            legendary_difficulty.unwrap_or_else(|| {
                fish_registry
                    .fish
                    .get(fish_id)
                    .map(|f| f.difficulty)
                    .unwrap_or(0.5)
            })
        } else {
            0.5
        };

        // Initialize minigame state; skill catch_zone_bonus is applied inside setup
        // via the skill resource accessed in minigame startup.
        minigame_state.setup_with_skill(
            difficulty,
            fishing_state.rod_tier,
            fishing_state.tackle_kind,
            &skill,
        );

        // Transition to Fishing game state; OnEnter will spawn the minigame UI
        next_state.set(GameState::Fishing);
    } else if reaction_expired {
        // Fish got away — too slow
        let bobber_entities: Vec<Entity> = bobber_query.iter().collect();
        let fs: &mut FishingState = &mut fishing_state;
        let ns: &mut NextState<GameState> = &mut next_state;
        let se: &mut EventWriter<StaminaDrainEvent> = &mut stamina_events;
        let cmd: &mut Commands = &mut commands;
        end_fishing_escape(fs, ns, se, cmd, bobber_entities, false);
    }
}

/// Allow the player to cancel fishing by pressing Escape while waiting for a bite.
pub fn handle_cancel_fishing(
    mut fishing_state: ResMut<FishingState>,
    mut next_state: ResMut<NextState<GameState>>,
    mut stamina_events: EventWriter<StaminaDrainEvent>,
    player_input: Res<PlayerInput>,
    input_blocks: Res<InputBlocks>,
    bobber_query: Query<Entity, With<Bobber>>,
    mut commands: Commands,
) {
    if fishing_state.phase == FishingPhase::Idle {
        return;
    }
    if fishing_state.phase == FishingPhase::Minigame {
        // Escape during minigame is handled in check_minigame_result
        return;
    }

    if input_blocks.is_blocked() {
        return;
    }

    if player_input.ui_cancel {
        let bobber_entities: Vec<Entity> = bobber_query.iter().collect();
        let fs: &mut FishingState = &mut fishing_state;
        let ns: &mut NextState<GameState> = &mut next_state;
        let se: &mut EventWriter<StaminaDrainEvent> = &mut stamina_events;
        let cmd: &mut Commands = &mut commands;
        end_fishing_escape(fs, ns, se, cmd, bobber_entities, false);
    }
}

// ─── Wild bait double-catch helper ───────────────────────────────────────────

/// Check whether wild bait should trigger a double catch this cast.
///
/// Wild bait has a 15% chance to produce a second fish immediately after a
/// successful catch. The second fish is a fresh random selection from the pool.
pub fn wild_bait_double_catch_roll() -> bool {
    let mut rng = rand::thread_rng();
    rng.gen_bool(0.15)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bait_bite_multiplier_worm() {
        assert!((bait_bite_multiplier("worm_bait") - 0.75).abs() < f32::EPSILON);
    }

    #[test]
    fn test_bait_bite_multiplier_magnet() {
        assert!((bait_bite_multiplier("magnet_bait") - 1.00).abs() < f32::EPSILON);
    }

    #[test]
    fn test_bait_bite_multiplier_wild() {
        assert!((bait_bite_multiplier("wild_bait") - 0.70).abs() < f32::EPSILON);
    }

    #[test]
    fn test_bait_bite_multiplier_generic() {
        // Generic "bait" gets moderate 15% speed bonus
        assert!((bait_bite_multiplier("bait") - 0.85).abs() < f32::EPSILON);
        // Unknown bait IDs get no speed bonus
        assert!((bait_bite_multiplier("some_other_bait") - 1.00).abs() < f32::EPSILON);
    }

    #[test]
    fn test_wild_bait_double_catch_roll_returns_bool() {
        // Just verify it returns a bool and doesn't panic
        let result = wild_bait_double_catch_roll();
        assert!(result || !result);
    }

    #[test]
    fn test_wild_bait_double_catch_statistical() {
        // Over 10000 trials, ~15% should be true (very loose bounds)
        let mut trues = 0u32;
        for _ in 0..10_000 {
            if wild_bait_double_catch_roll() {
                trues += 1;
            }
        }
        // With 15% chance, expect ~1500. Allow wide range [500, 3000].
        assert!(trues > 500, "Expected some double catches, got {}", trues);
        assert!(trues < 3000, "Too many double catches: {}", trues);
    }
}

