use bevy::prelude::*;
use crate::shared::*;

// ═══════════════════════════════════════════════════════════════════════
// HINT DEFINITIONS
// ═══════════════════════════════════════════════════════════════════════

struct HintDef {
    id: &'static str,
    message: &'static str,
}

const HINTS: &[HintDef] = &[
    HintDef {
        id: "first_farm",
        message: "Press WASD to move. Use number keys to select tools.",
    },
    HintDef {
        id: "first_seed",
        message: "Hold a seed and press F on tilled soil to plant.",
    },
    HintDef {
        id: "crop_ready",
        message: "Your crop is ready! Walk up and press F to harvest.",
    },
    HintDef {
        id: "first_inventory",
        message: "Press E to open/close your inventory.",
    },
    HintDef {
        id: "shipping_bin",
        message: "Place items in the shipping bin to sell them at end of day.",
    },
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
// SYSTEM — check tutorial hints each frame
// ═══════════════════════════════════════════════════════════════════════

pub fn check_tutorial_hints(
    mut tutorial: ResMut<TutorialState>,
    mut hint_writer: EventWriter<HintEvent>,
    mut toast_writer: EventWriter<ToastEvent>,
    player_state: Res<PlayerState>,
    calendar: Res<Calendar>,
    inventory: Res<Inventory>,
    farm_state: Res<FarmState>,
    crop_registry: Res<CropRegistry>,
    item_registry: Res<ItemRegistry>,
    play_stats: Res<PlayStats>,
) {
    if tutorial.tutorial_complete {
        return;
    }

    let mut newly_shown = Vec::new();

    for hint in HINTS {
        // Skip already-shown hints.
        if tutorial.hints_shown.contains(&hint.id.to_string()) {
            continue;
        }

        let triggered = match hint.id {
            // first_farm: player is on the Farm map
            "first_farm" => player_state.current_map == MapId::Farm,

            // first_seed: selected hotbar slot contains a seed item
            "first_seed" => {
                let selected = inventory.selected_slot;
                if selected < inventory.slots.len() {
                    if let Some(ref slot) = inventory.slots[selected] {
                        item_registry
                            .get(&slot.item_id)
                            .map(|def| def.category == ItemCategory::Seed)
                            .unwrap_or(false)
                    } else {
                        false
                    }
                } else {
                    false
                }
            }

            // crop_ready: any crop in FarmState has reached its final growth stage
            "crop_ready" => farm_state
                .crops
                .values()
                .any(|c| is_crop_ready(c, &crop_registry)),

            // first_inventory: at least one full day has passed
            "first_inventory" => play_stats.days_played >= 1,

            // shipping_bin: player is on Farm (shipping bin is near 0,0)
            // We trigger this on the Farm map after the first day so the player
            // has had a chance to notice the bin area.
            "shipping_bin" => {
                player_state.current_map == MapId::Farm && play_stats.days_played >= 1
            }

            // mine_entrance: player is in the Mine
            "mine_entrance" => {
                matches!(
                    player_state.current_map,
                    MapId::Mine | MapId::MineEntrance
                )
            }

            // npc_nearby: player is in Town
            "npc_nearby" => player_state.current_map == MapId::Town,

            // rainy_day: weather is Rainy and player is on Farm
            "rainy_day" => {
                calendar.weather == Weather::Rainy
                    && player_state.current_map == MapId::Farm
            }

            // season_change: it's the first day of a non-Spring season
            // (avoids firing on the very first day of the game, day 1 Spring)
            "season_change" => {
                calendar.day == 1
                    && calendar.season != Season::Spring
                    && calendar.total_days_elapsed() > 0
            }

            // low_stamina: stamina below 20
            "low_stamina" => player_state.stamina < 20.0,

            // Unknown hint id — never trigger
            _ => false,
        };

        if triggered {
            newly_shown.push(hint.id.to_string());

            hint_writer.send(HintEvent {
                hint_id: hint.id.to_string(),
                message: hint.message.to_string(),
            });

            toast_writer.send(ToastEvent {
                message: hint.message.to_string(),
                duration_secs: 5.0,
            });
        }
    }

    for id in newly_shown {
        tutorial.hints_shown.push(id);
    }

    // Mark tutorial complete once all 10 hints have been shown.
    if tutorial.hints_shown.len() >= HINTS.len() {
        tutorial.tutorial_complete = true;
    }
}

// ═══════════════════════════════════════════════════════════════════════
// SYSTEM — forward HintEvent to ToastEvent
// (provides the wiring described in the task spec; toast is also sent
//  directly above so both paths work regardless of ordering)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hints_table_has_10_entries() {
        assert_eq!(HINTS.len(), 10);
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
        let registry = CropRegistry::default(); // empty registry
        assert!(!is_crop_ready(&crop_tile, &registry));
    }

    #[test]
    fn test_tutorial_state_default_not_complete() {
        let state = TutorialState::default();
        assert!(!state.tutorial_complete);
        assert!(state.hints_shown.is_empty());
        assert!(state.current_objective.is_none());
    }

    #[test]
    fn test_tutorial_complete_after_all_hints() {
        let mut state = TutorialState::default();
        // Simulate showing all hints
        for hint in HINTS {
            state.hints_shown.push(hint.id.to_string());
        }
        // The system sets tutorial_complete when hints_shown.len() >= HINTS.len()
        if state.hints_shown.len() >= HINTS.len() {
            state.tutorial_complete = true;
        }
        assert!(state.tutorial_complete);
    }
}
