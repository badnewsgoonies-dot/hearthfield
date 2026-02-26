//! Sprinkler and rain auto-watering systems.

use bevy::prelude::*;
use crate::shared::*;
use super::{FarmEntities, MorningSprinklerEvent};
use super::soil::spawn_or_update_soil_entity;

// ─────────────────────────────────────────────────────────────────────────────
// Sprinkler auto-watering (runs at start of each day, before crop growth)
// ─────────────────────────────────────────────────────────────────────────────

/// Water all tilled tiles adjacent to each Sprinkler object (3×3 area centred
/// on the sprinkler itself). Runs once per MorningSprinklerEvent, which is
/// expected to fire at the transition from night to morning.
pub fn apply_sprinklers(
    mut sprinkler_events: EventReader<MorningSprinklerEvent>,
    mut farm_state: ResMut<FarmState>,
    mut farm_entities: ResMut<FarmEntities>,
    mut commands: Commands,
) {
    // Only run when the event is received.
    if sprinkler_events.read().next().is_none() {
        return;
    }

    // Collect sprinkler positions first (borrow checker).
    let sprinkler_positions: Vec<(i32, i32)> = farm_state
        .objects
        .iter()
        .filter(|(_, obj)| matches!(obj, FarmObject::Sprinkler))
        .map(|(&pos, _)| pos)
        .collect();

    // Water a 3×3 area around each sprinkler.
    for (sx, sy) in sprinkler_positions {
        for dy in -1..=1 {
            for dx in -1..=1 {
                let pos = (sx + dx, sy + dy);
                let current = farm_state.soil.get(&pos).copied();
                if current == Some(SoilState::Tilled) {
                    farm_state.soil.insert(pos, SoilState::Watered);
                    if let Some(crop) = farm_state.crops.get_mut(&pos) {
                        crop.watered_today = true;
                        crop.days_without_water = 0;
                    }
                    spawn_or_update_soil_entity(&mut commands, &mut farm_entities, pos, SoilState::Watered);
                }
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Rain watering
// ─────────────────────────────────────────────────────────────────────────────

/// On rainy days, all tilled soil is watered and all crops get watered_today = true.
/// Called from events_handler::on_day_end before crop growth advancement.
pub fn apply_rain_watering(farm_state: &mut FarmState) {
    // Water all tilled tiles.
    let tilled_positions: Vec<(i32, i32)> = farm_state
        .soil
        .iter()
        .filter(|(_, &s)| s == SoilState::Tilled)
        .map(|(&pos, _)| pos)
        .collect();

    for pos in tilled_positions {
        farm_state.soil.insert(pos, SoilState::Watered);
        if let Some(crop) = farm_state.crops.get_mut(&pos) {
            crop.watered_today = true;
            crop.days_without_water = 0;
        }
    }

    // Also mark all watered crops (in case they were already watered).
    for (_, crop) in farm_state.crops.iter_mut() {
        if !crop.dead {
            crop.watered_today = true;
            crop.days_without_water = 0;
        }
    }
}
