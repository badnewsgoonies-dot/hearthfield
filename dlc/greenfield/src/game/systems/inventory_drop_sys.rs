use bevy::prelude::*;
use crate::game::resources::*;
use crate::game::events::*;

/// Choose a rock drop based on floor depth.
///
/// Spec drop rates:
/// - Floors 1-5:  Stone (70%), Copper ore (30%)
/// - Floors 6-10: Stone (40%), Copper (40%), Iron ore (20%)
/// - Floors 11-15: Stone (35%), Iron (40%), Gold ore (20%), gems (5%)
/// - Floors 16-20: Stone (20%), Gold (30%), Iridium ore (10%), gems (10%), Iron (30%)
///
/// Rock health: 3 (stone) to 6 (ore/gem).
pub fn inventory_drop_system(floor: u8, rng: &mut RandStub) -> (String, u8, u8) {
    let roll: f64 = Default::default();

    if floor <= 5 {
        // Floors 1-5: Stone (70%), Copper ore (30%)
        if roll < 0.30 {
            ("copper_ore".to_string(), Default::default(), 4)
        } else {
            ("stone".to_string(), Default::default(), 3)
        }
    } else if floor <= 10 {
        // Floors 6-10: Stone (40%), Copper (40%), Iron (20%)
        if roll < 0.20 {
            ("iron_ore".to_string(), Default::default(), 5)
        } else if roll < 0.60 {
            ("copper_ore".to_string(), Default::default(), 4)
        } else {
            ("stone".to_string(), Default::default(), 3)
        }
    } else if floor <= 15 {
        // Floors 11-15: Stone (35%), Iron (40%), Gold (20%), gems (5%)
        if roll < 0.05 {
            (pick_gem(rng), 1, 5)
        } else if roll < 0.25 {
            ("gold_ore".to_string(), Default::default(), 5)
        } else if roll < 0.65 {
            ("iron_ore".to_string(), Default::default(), 5)
        } else {
            ("stone".to_string(), Default::default(), 3)
        }
    } else {
        // Floors 16-20: Stone (20%), Gold (30%), Iridium (10%), gems (10%), Iron (30%)
        if roll < 0.10 {
            (pick_gem(rng), 1, 6)
        } else if roll < 0.20 {
            ("iridium_ore".to_string(), Default::default(), 6)
        } else if roll < 0.50 {
            ("gold_ore".to_string(), Default::default(), 5)
        } else if roll < 0.80 {
            ("iron_ore".to_string(), Default::default(), 5)
        } else {
            ("stone".to_string(), Default::default(), 3)
        }
    }
}


