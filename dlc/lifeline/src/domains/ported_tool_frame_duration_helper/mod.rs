//! pure host fn substrate port.

#![allow(unused)]
#![allow(dead_code)]

use bevy::prelude::*;

pub const TOOL_SWING_DURATION_MULTIPLIER: f32 = 0.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ToolKind {
    #[default]
    Axe,
    FishingRod,
    Hoe,
    Pickaxe,
    Scythe,
    WateringCan,
}


/// Per-tool frame duration in seconds. Heavy tools feel weighty,
/// light tools feel snappy. Total animation = duration x 4 frames.
pub fn tool_frame_duration_helper(tool: ToolKind) -> f32 {
    TOOL_SWING_DURATION_MULTIPLIER
        * match tool {
            ToolKind::Axe => 0.145,        // 0.58s total — heavy, impactful chop
            ToolKind::Pickaxe => 0.135,    // 0.54s total — heavy swing
            ToolKind::Hoe => 0.105,        // 0.42s total — deliberate tilling
            ToolKind::FishingRod => 0.11,  // 0.44s total — quick cast flick
            ToolKind::WateringCan => 0.09, // 0.36s total — smooth pour
            ToolKind::Scythe => 0.075,     // 0.30s total — fast sweep
        }
}


