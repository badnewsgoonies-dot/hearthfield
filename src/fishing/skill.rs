//! Fishing skill progression system.
//!
//! The player gains fishing skill by catching fish. Every 10 catches grants a
//! level-up that makes fishing easier via:
//!   - `bite_speed_bonus`: reduces the bite wait time (faster bites).
//!   - `catch_zone_bonus`: enlarges the catch bar in the minigame.
//!
//! The skill resource is saved/restored independently of the fishing state so
//! progress persists across sessions.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::shared::*;

// ─── Resource ────────────────────────────────────────────────────────────────

/// Persistent fishing skill that improves as the player catches more fish.
#[derive(Resource, Debug, Clone, Default, Serialize, Deserialize)]
pub struct FishingSkill {
    /// Cumulative count of all fish successfully caught.
    pub total_catches: u32,
    /// Current skill level. Increases by 1 every 10 catches (uncapped in data,
    /// but bonuses cap at level 10 for balance).
    pub level: u32,
    /// Fraction by which the bite timer is reduced (0.0 → 0.5).
    /// Applied as: effective_wait = base_wait * (1.0 - bite_speed_bonus).
    pub bite_speed_bonus: f32,
    /// Fraction added to the catch bar half-height (0.0 → 0.3).
    /// Applied as: catch_bar_half *= (1.0 + catch_zone_bonus).
    pub catch_zone_bonus: f32,
}

impl FishingSkill {
    /// Maximum bite-speed bonus.
    pub const MAX_BITE_SPEED: f32 = 0.5;
    /// Maximum catch-zone bonus.
    pub const MAX_CATCH_ZONE: f32 = 0.3;
    /// Catches required to advance one level.
    pub const CATCHES_PER_LEVEL: u32 = 10;
    /// Bite-speed improvement per level.
    pub const BITE_SPEED_PER_LEVEL: f32 = 0.05;
    /// Catch-zone improvement per level.
    pub const CATCH_ZONE_PER_LEVEL: f32 = 0.03;

    /// Recalculate derived fields from `total_catches`.
    ///
    /// Called after incrementing `total_catches` so that `level`, `bite_speed_bonus`,
    /// and `catch_zone_bonus` stay consistent.
    pub fn recalculate(&mut self) {
        self.level = self.total_catches / Self::CATCHES_PER_LEVEL;

        self.bite_speed_bonus =
            (self.level as f32 * Self::BITE_SPEED_PER_LEVEL).min(Self::MAX_BITE_SPEED);

        self.catch_zone_bonus =
            (self.level as f32 * Self::CATCH_ZONE_PER_LEVEL).min(Self::MAX_CATCH_ZONE);
    }

    /// Apply the bite-speed bonus to a raw wait duration (in seconds).
    pub fn apply_bite_speed(&self, base_wait: f32) -> f32 {
        base_wait * (1.0 - self.bite_speed_bonus)
    }

    /// Apply the catch-zone bonus to a catch-bar half-height.
    pub fn apply_catch_zone(&self, base_half: f32) -> f32 {
        base_half * (1.0 + self.catch_zone_bonus)
    }
}

// ─── Events ──────────────────────────────────────────────────────────────────

/// Sent internally by the fishing skill system when the player levels up.
/// Other systems (UI, achievements) can listen for this.
#[derive(Event, Debug, Clone)]
pub struct FishingLevelUpEvent {
    /// New skill level reached.
    pub new_level: u32,
}

// ─── Systems ─────────────────────────────────────────────────────────────────

/// Listens for `ItemPickupEvent` events whose `item_id` matches a fish in the
/// `FishRegistry`. For each fish caught the skill counter is incremented and,
/// if a level boundary is crossed, a `FishingLevelUpEvent` + `ToastEvent` are
/// sent to inform the player.
pub fn update_fishing_skill(
    mut item_pickup: EventReader<ItemPickupEvent>,
    fish_registry: Res<FishRegistry>,
    mut skill: ResMut<FishingSkill>,
    mut level_up_events: EventWriter<FishingLevelUpEvent>,
    mut toast_events: EventWriter<ToastEvent>,
) {
    for event in item_pickup.read() {
        // Only count items that exist in the fish registry.
        if !fish_registry.fish.contains_key(&event.item_id) {
            continue;
        }

        let prev_level = skill.level;

        // Count each individual fish in a multi-quantity pickup (normally 1).
        skill.total_catches += event.quantity as u32;
        skill.recalculate();

        // Did we cross a level boundary?
        if skill.level > prev_level {
            level_up_events.send(FishingLevelUpEvent {
                new_level: skill.level,
            });

            toast_events.send(ToastEvent {
                message: format!(
                    "Fishing skill up! Level {} — bite speed +{:.0}%, catch zone +{:.0}%",
                    skill.level,
                    skill.bite_speed_bonus * 100.0,
                    skill.catch_zone_bonus * 100.0,
                ),
                duration_secs: 4.0,
            });
        }
    }
}
