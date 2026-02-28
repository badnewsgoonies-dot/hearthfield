use bevy::prelude::*;
use crate::shared::*;

// ═══════════════════════════════════════════════════════════════════════
// CONTEXTUAL HINT DEFINITIONS
// (Fire-and-forget hints for situations outside the objective sequence.)
// ═══════════════════════════════════════════════════════════════════════

struct HintDef {
    id: &'static str,
    message: &'static str,
}

const HINTS: &[HintDef] = &[
    HintDef {
        id: "mine_entrance",
        message: "Break rocks with your pickaxe. Watch your health!",
    },
    HintDef {
        id: "npc_nearby",
        message: "Press F to talk to villagers. Give gifts to build friendship!",
    },
    HintDef {
        id: "rainy_day",
        message: "Rain waters your crops automatically. Lucky!",
    },
    HintDef {
        id: "season_change",
        message: "New season! Check which crops grow in this season.",
    },
    HintDef {
        id: "low_stamina",
        message: "Low energy! Eat food or go to bed early to recover.",
    },
];

// ═══════════════════════════════════════════════════════════════════════
// HELPER — check if a crop is fully grown
// ═══════════════════════════════════════════════════════════════════════

#[allow(dead_code)]
fn is_crop_ready(crop_tile: &CropTile, crop_registry: &CropRegistry) -> bool {
    if crop_tile.dead {
        return false;
    }
    if let Some(def) = crop_registry.crops.get(&crop_tile.crop_id) {
        let max_stage = def.growth_days.len() as u8;
        crop_tile.current_stage >= max_stage
    } else {
        false
    }
}

// ═══════════════════════════════════════════════════════════════════════
// SYSTEM — check contextual hints each frame
// ═══════════════════════════════════════════════════════════════════════

pub fn check_tutorial_hints(
    mut tutorial: ResMut<TutorialState>,
    mut hint_writer: EventWriter<HintEvent>,
    player_state: Res<PlayerState>,
    calendar: Res<Calendar>,
    #[allow(unused)] inventory: Res<Inventory>,
    #[allow(unused)] farm_state: Res<FarmState>,
    #[allow(unused)] crop_registry: Res<CropRegistry>,
    #[allow(unused)] item_registry: Res<ItemRegistry>,
    #[allow(unused)] play_stats: Res<PlayStats>,
) {
    if tutorial.tutorial_complete {
        return;
    }

    let mut newly_shown = Vec::new();

    for hint in HINTS {
        if tutorial.hints_shown.contains(&hint.id.to_string()) {
            continue;
        }

        let triggered = match hint.id {
            "mine_entrance" => {
                matches!(
                    player_state.current_map,
                    MapId::Mine | MapId::MineEntrance
                )
            }
            "npc_nearby" => player_state.current_map == MapId::Town,
            "rainy_day" => {
                calendar.weather == Weather::Rainy
                    && player_state.current_map == MapId::Farm
            }
            "season_change" => {
                calendar.day == 1
                    && calendar.season != Season::Spring
                    && calendar.total_days_elapsed() > 0
            }
            "low_stamina" => player_state.stamina < 20.0,
            _ => false,
        };

        if triggered {
            newly_shown.push(hint.id.to_string());
            hint_writer.send(HintEvent {
                hint_id: hint.id.to_string(),
                message: hint.message.to_string(),
            });
        }
    }

    for id in newly_shown {
        tutorial.hints_shown.push(id);
    }

    // Mark tutorial complete once all hints have been shown AND objectives are done.
    let all_hints_shown = tutorial.hints_shown.len() >= HINTS.len();
    let objectives_done = tutorial.current_objective.is_none()
        && tutorial.hints_shown.iter().any(|h| h == "objectives_done");
    if all_hints_shown && objectives_done {
        tutorial.tutorial_complete = true;
    }
}

// ═══════════════════════════════════════════════════════════════════════
// SYSTEM — forward HintEvent to ToastEvent
// ═══════════════════════════════════════════════════════════════════════

pub fn forward_hint_to_toast(
    mut hints: EventReader<HintEvent>,
    mut toasts: EventWriter<ToastEvent>,
) {
    for hint in hints.read() {
        toasts.send(ToastEvent {
            message: hint.message.clone(),
            duration_secs: 5.0,
        });
    }
}

// ═══════════════════════════════════════════════════════════════════════
// OBJECTIVE-DRIVEN TUTORIAL
// Sequential objectives that guide the player through Day 1.
// ═══════════════════════════════════════════════════════════════════════

pub const OBJECTIVES: &[(&str, &str)] = &[
    ("till_soil",    "Till some soil with your hoe (Space)"),
    ("plant_seeds",  "Plant your turnip seeds (select seeds, press F)"),
    ("water_crops",  "Water your crops (select watering can, press Space)"),
    ("visit_town",   "Visit the town (walk south from the farm)"),
    ("go_to_bed",    "Go home and sleep (press F on your farm)"),
];

fn is_objective_complete(
    id: &str,
    farm: &FarmState,
    calendar: &Calendar,
    player: &PlayerState,
) -> bool {
    match id {
        "till_soil"    => farm.soil.values().any(|s| *s == SoilState::Tilled || *s == SoilState::Watered),
        "plant_seeds"  => !farm.crops.is_empty(),
        "water_crops"  => farm.soil.values().any(|s| *s == SoilState::Watered),
        "visit_town"   => player.current_map == MapId::Town,
        "go_to_bed"    => calendar.day >= 2,
        _ => false,
    }
}

/// Sequenced objective system. Sets `current_objective`, checks completion,
/// advances to the next objective, and sends a toast on completion.
pub fn check_objectives(
    mut tutorial: ResMut<TutorialState>,
    mut toast_writer: EventWriter<ToastEvent>,
    farm_state: Res<FarmState>,
    calendar: Res<Calendar>,
    player_state: Res<PlayerState>,
) {
    if tutorial.tutorial_complete {
        return;
    }

    // Initialize first objective on Day 1 morning.
    if tutorial.current_objective.is_none()
        && !tutorial.hints_shown.iter().any(|h| h == "objectives_done")
    {
        // Only start objectives on Day 1.
        if calendar.day == 1 && calendar.year == 1 {
            tutorial.current_objective = Some(OBJECTIVES[0].0.to_string());
        }
        return;
    }

    // Check if current objective is complete.
    let Some(ref current_id) = tutorial.current_objective else {
        return;
    };

    if !is_objective_complete(current_id, &farm_state, &calendar, &player_state) {
        return;
    }

    // Find current objective index.
    let current_idx = OBJECTIVES
        .iter()
        .position(|(id, _)| *id == current_id.as_str());

    let Some(idx) = current_idx else {
        // Unknown objective — clear it.
        tutorial.current_objective = None;
        return;
    };

    // Send completion toast.
    let (_, display) = OBJECTIVES[idx];
    toast_writer.send(ToastEvent {
        message: format!("Done: {}", display),
        duration_secs: 3.0,
    });

    // Advance to next objective, or finish.
    if idx + 1 < OBJECTIVES.len() {
        tutorial.current_objective = Some(OBJECTIVES[idx + 1].0.to_string());
    } else {
        tutorial.current_objective = None;
        // Mark objectives as complete so hints system knows.
        tutorial.hints_shown.push("objectives_done".to_string());
    }
}

// ═══════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hints_table_has_5_entries() {
        assert_eq!(HINTS.len(), 5);
    }

    #[test]
    fn test_all_hint_ids_are_unique() {
        let mut ids = std::collections::HashSet::new();
        for hint in HINTS {
            assert!(ids.insert(hint.id), "Duplicate hint id: {}", hint.id);
        }
    }

    #[test]
    fn test_all_hint_messages_are_non_empty() {
        for hint in HINTS {
            assert!(!hint.message.is_empty(), "Hint {} has empty message", hint.id);
        }
    }

    #[test]
    fn test_objectives_table_has_5_entries() {
        assert_eq!(OBJECTIVES.len(), 5);
    }

    #[test]
    fn test_is_crop_ready_dead_crop_is_not_ready() {
        let crop_tile = CropTile {
            crop_id: "turnip".to_string(),
            current_stage: 5,
            days_in_stage: 0,
            watered_today: false,
            days_without_water: 0,
            dead: true,
        };
        let registry = CropRegistry::default();
        assert!(!is_crop_ready(&crop_tile, &registry));
    }

    #[test]
    fn test_is_crop_ready_unknown_crop() {
        let crop_tile = CropTile {
            crop_id: "nonexistent_crop".to_string(),
            current_stage: 99,
            days_in_stage: 0,
            watered_today: false,
            days_without_water: 0,
            dead: false,
        };
        let registry = CropRegistry::default();
        assert!(!is_crop_ready(&crop_tile, &registry));
    }

    #[test]
    fn test_tutorial_state_default_not_complete() {
        let state = TutorialState::default();
        assert!(!state.tutorial_complete);
        assert!(state.hints_shown.is_empty());
        assert!(state.current_objective.is_none());
    }
}
