//! pure host fn substrate port.

#![allow(unused)]
#![allow(dead_code)]

use bevy::prelude::*;



/// Map a crop growth stage to a plants.png atlas index (row 0: indices 0-5).
///
/// Uses the formula:
///   `let atlas_idx = (stage * 5 / total_stages.max(1)).min(5)`
/// so every crop maps smoothly onto the 6 available growth frames regardless
/// of how many growth days are defined.
pub fn patient_atlas_index(stage: u8, total_stages: u8) -> usize {
    let total = total_stages.max(1) as usize;
    ((stage as usize * 5) / total).min(5)
}


