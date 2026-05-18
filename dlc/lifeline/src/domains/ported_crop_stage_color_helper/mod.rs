//! pure host fn substrate port.

#![allow(unused)]
#![allow(dead_code)]

use bevy::prelude::*;



/// Return the colour for a given crop stage index (0 = seedling, max = ripe).
/// Used as a placeholder when no sprite atlas is loaded.
pub fn crop_stage_color_helper(stage: u8, total_stages: u8, dead: bool) -> Color {
    if dead {
        return Color::srgb(0.35, 0.28, 0.20); // dried-out brown
    }
    if total_stages == 0 {
        return Color::srgb(0.3, 0.7, 0.3);
    }
    let progress = stage as f32 / (total_stages.saturating_sub(1).max(1)) as f32;
    // Lerp from pale yellow-green (seedling) to vivid green/orange (mature)
    let r = 0.5 * (1.0 - progress) + 0.2 * progress;
    let g = 0.65 + 0.15 * progress;
    let b = 0.2 * (1.0 - progress);
    Color::srgb(r.clamp(0.0, 1.0), g.clamp(0.0, 1.0), b.clamp(0.0, 1.0))
}


