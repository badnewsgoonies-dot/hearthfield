//! pure host fn substrate port.

#![allow(unused)]
#![allow(dead_code)]

use bevy::prelude::*;



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
pub fn bait_bite_multiplier_helper(bait_id: &str) -> f32 {
    match bait_id {
        "worm_bait" => 0.75,
        "magnet_bait" => 1.00, // magnet bait benefits treasure, not speed
        "wild_bait" => 0.70,
        "bait" => 0.85, // generic bait = moderate 15% faster
        _ => 1.00,      // unknown bait IDs get no speed bonus
    }
}


