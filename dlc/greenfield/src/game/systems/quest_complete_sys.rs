use bevy::prelude::*;
use crate::game::resources::*;
use crate::game::events::*;

pub fn quest_complete_system(
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


