use crate::shared::*;
use bevy::prelude::*;

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
    // Fix 4: Shipping bin hint
    HintDef {
        id: "shipping_bin",
        message: "Put items in the Shipping Bin near your house to sell them overnight for gold!",
    },
    // Fix 5: Inventory hint
    HintDef {
        id: "open_inventory",
        message: "Press E to open your inventory and see your items and tools!",
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

#[allow(clippy::too_many_arguments)]
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
                matches!(player_state.current_map, MapId::Mine | MapId::MineEntrance)
            }
            "npc_nearby" => player_state.current_map == MapId::Town,
            "rainy_day" => {
                calendar.weather == Weather::Rainy && player_state.current_map == MapId::Farm
            }
            "season_change" => {
                calendar.day == 1
                    && calendar.season != Season::Spring
                    && calendar.total_days_elapsed() > 0
            }
            "low_stamina" => player_state.stamina < 20.0,
            // Fix 4: Shipping bin hint — triggers when any crop is nearing maturity
            "shipping_bin" => farm_state.crops.values().any(|c| c.current_stage >= 3),
            // Fix 5: Inventory hint — triggers on Day 1 morning
            "open_inventory" => calendar.day == 1 && calendar.year == 1 && calendar.hour >= 7,
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
// Sequential objectives that guide the player through Days 1-3.
// ═══════════════════════════════════════════════════════════════════════

// Fix 1: Added "exit_house" as the first objective (index 0).
// Fix 2: Updated all objective descriptions for clarity.
pub const OBJECTIVES: &[(&str, &str)] = &[
    ("exit_house",   "Leave your house \u{2014} walk south to the door and exit"),
    ("till_soil",    "Till some soil \u{2014} select your Hoe with [ or ], then press Space on grass"),
    ("plant_seeds",  "Plant seeds \u{2014} press E to open inventory, click turnip seeds to select, close inventory, then press F on tilled soil"),
    ("water_crops",  "Water your crops \u{2014} select Watering Can with [ or ], then press Space on planted soil"),
    ("visit_town",   "Visit the town \u{2014} walk south from your farm to explore"),
    ("go_to_bed",    "End the day \u{2014} go home, walk to your bed, and press F to sleep"),
];

// Fix 3: Day 2 objectives
const DAY2_OBJECTIVES: &[(&str, &str)] = &[(
    "check_crops",
    "Check your crops \u{2014} walk to your farm and see how they're growing",
)];

// Fix 3: Day 3+ objectives
const DAY3_OBJECTIVES: &[(&str, &str)] = &[
    ("use_shipping_bin", "Ship your items for gold \u{2014} walk to the shipping bin near your house and press F with an item selected"),
];

fn is_objective_complete(
    id: &str,
    farm: &FarmState,
    calendar: &Calendar,
    player: &PlayerState,
    shipping_bin: &ShippingBin,
) -> bool {
    match id {
        // Fix 1: exit_house completion check
        "exit_house" => player.current_map != MapId::PlayerHouse,
        "till_soil" => farm
            .soil
            .values()
            .any(|s| *s == SoilState::Tilled || *s == SoilState::Watered),
        "plant_seeds" => !farm.crops.is_empty(),
        "water_crops" => farm.soil.values().any(|s| *s == SoilState::Watered),
        "visit_town" => player.current_map == MapId::Town,
        "go_to_bed" => calendar.day >= 2,
        // Fix 3: Day 2 objective
        "check_crops" => player.current_map == MapId::Farm && calendar.hour >= 7,
        // Fix 3: Day 3+ objective
        "use_shipping_bin" => !shipping_bin.items.is_empty(),
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
    shipping_bin: Res<ShippingBin>,
) {
    if tutorial.tutorial_complete {
        return;
    }

    // Initialize objectives based on current day.
    if tutorial.current_objective.is_none()
        && !tutorial.hints_shown.iter().any(|h| h == "objectives_done")
    {
        if calendar.year == 1 {
            if calendar.day == 1 {
                // Day 1: start the main tutorial sequence.
                tutorial.current_objective = Some(OBJECTIVES[0].0.to_string());
            } else if calendar.day == 2 {
                // Fix 3: Day 2 objectives.
                tutorial.current_objective = Some(DAY2_OBJECTIVES[0].0.to_string());
            } else if calendar.day >= 3 && !tutorial.hints_shown.iter().any(|h| h == "shipped_once")
            {
                // Fix 3: Day 3+ shipping objective (only if player has never shipped).
                tutorial.current_objective = Some(DAY3_OBJECTIVES[0].0.to_string());
            }
        }
        return;
    }

    // Check if current objective is complete.
    let Some(ref current_id) = tutorial.current_objective else {
        return;
    };

    if !is_objective_complete(
        current_id,
        &farm_state,
        &calendar,
        &player_state,
        &shipping_bin,
    ) {
        return;
    }

    // Determine which objective list the current objective belongs to and find its index.
    let (obj_list, current_idx) = if let Some(idx) = OBJECTIVES
        .iter()
        .position(|(id, _)| *id == current_id.as_str())
    {
        (OBJECTIVES, Some(idx))
    } else if let Some(idx) = DAY2_OBJECTIVES
        .iter()
        .position(|(id, _)| *id == current_id.as_str())
    {
        (DAY2_OBJECTIVES, Some(idx))
    } else if let Some(idx) = DAY3_OBJECTIVES
        .iter()
        .position(|(id, _)| *id == current_id.as_str())
    {
        (DAY3_OBJECTIVES, Some(idx))
    } else {
        // Unknown objective — clear it.
        tutorial.current_objective = None;
        return;
    };

    let Some(idx) = current_idx else {
        tutorial.current_objective = None;
        return;
    };

    // Send completion toast.
    let (_, display) = obj_list[idx];
    toast_writer.send(ToastEvent {
        message: format!("Done: {}", display),
        duration_secs: 3.0,
    });

    // Mark shipping as done so we don't re-trigger the objective.
    if current_id == "use_shipping_bin" {
        tutorial.hints_shown.push("shipped_once".to_string());
    }

    // Advance to next objective in the same list, or finish.
    if idx + 1 < obj_list.len() {
        tutorial.current_objective = Some(obj_list[idx + 1].0.to_string());
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
    fn test_hints_table_has_7_entries() {
        assert_eq!(HINTS.len(), 7);
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
            assert!(
                !hint.message.is_empty(),
                "Hint {} has empty message",
                hint.id
            );
        }
    }

    #[test]
    fn test_objectives_table_has_6_entries() {
        assert_eq!(OBJECTIVES.len(), 6);
    }

    #[test]
    fn test_exit_house_is_first_objective() {
        assert_eq!(OBJECTIVES[0].0, "exit_house");
    }

    #[test]
    fn test_day2_objectives_table_has_1_entry() {
        assert_eq!(DAY2_OBJECTIVES.len(), 1);
    }

    #[test]
    fn test_day3_objectives_table_has_1_entry() {
        assert_eq!(DAY3_OBJECTIVES.len(), 1);
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
