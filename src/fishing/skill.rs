//! Fishing skill progression system.
//!
//! The player gains fishing XP by catching fish. XP awarded depends on rarity:
//!   - Common:    3 XP
//!   - Uncommon:  8 XP
//!   - Rare:     15 XP
//!   - Legendary: 25 XP
//!
//! Skill levels 1–10 with cumulative XP thresholds:
//!   10, 25, 50, 100, 200, 350, 550, 800, 1100, 1500
//!
//! Skill benefits:
//!   - Bite wait reduced by 0.5 seconds per level
//!   - Catch bar size: 40px base + 3px per level

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::shared::*;

// ─── Constants ──────────────────────────────────────────────────────────────

/// XP thresholds for each level (1–10). Index 0 = level 1 threshold.
pub const LEVEL_THRESHOLDS: [u32; 10] = [10, 25, 50, 100, 200, 350, 550, 800, 1100, 1500];

/// Max skill level.
pub const MAX_LEVEL: u32 = 10;

/// Base catch bar size in pixels.
pub const BASE_BAR_SIZE_PX: f32 = 40.0;
/// Additional bar size per skill level in pixels.
pub const BAR_SIZE_PER_LEVEL_PX: f32 = 3.0;

/// Bite wait reduction per level in seconds.
pub const BITE_WAIT_REDUCTION_PER_LEVEL: f32 = 0.5;

/// XP awarded per catch by rarity.
pub fn xp_for_rarity(rarity: Rarity) -> u32 {
    match rarity {
        Rarity::Common => 3,
        Rarity::Uncommon => 8,
        Rarity::Rare => 15,
        Rarity::Legendary => 25,
    }
}

// ─── Resource ────────────────────────────────────────────────────────────────

/// Persistent fishing skill that improves as the player catches more fish.
#[derive(Resource, Debug, Clone, Default, Serialize, Deserialize)]
pub struct FishingSkill {
    /// Cumulative XP earned from catching fish.
    pub xp: u32,
    /// Cumulative count of all fish successfully caught.
    pub total_catches: u32,
    /// Current skill level (0–10). Level 0 = beginner.
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
    /// Catches required to advance one level (legacy, kept for compatibility).
    #[allow(dead_code)]
    pub const CATCHES_PER_LEVEL: u32 = 10;
    /// Bite-speed improvement per level.
    pub const BITE_SPEED_PER_LEVEL: f32 = 0.05;
    /// Catch-zone improvement per level.
    pub const CATCH_ZONE_PER_LEVEL: f32 = 0.03;

    /// Recalculate level and derived bonuses from current XP.
    pub fn recalculate(&mut self) {
        // Determine level from XP thresholds
        self.level = 0;
        for (i, &threshold) in LEVEL_THRESHOLDS.iter().enumerate() {
            if self.xp >= threshold {
                self.level = (i as u32) + 1;
            } else {
                break;
            }
        }
        self.level = self.level.min(MAX_LEVEL);

        self.bite_speed_bonus =
            (self.level as f32 * Self::BITE_SPEED_PER_LEVEL).min(Self::MAX_BITE_SPEED);

        self.catch_zone_bonus =
            (self.level as f32 * Self::CATCH_ZONE_PER_LEVEL).min(Self::MAX_CATCH_ZONE);
    }

    /// Add XP for catching a fish of the given rarity.
    pub fn add_catch_xp(&mut self, rarity: Rarity) {
        self.total_catches += 1;
        self.xp += xp_for_rarity(rarity);
        self.recalculate();
    }

    /// Apply the bite-speed bonus to a raw wait duration (in seconds).
    #[allow(dead_code)]
    pub fn apply_bite_speed(&self, base_wait: f32) -> f32 {
        base_wait * (1.0 - self.bite_speed_bonus)
    }

    /// Apply the catch-zone bonus to a catch-bar half-height.
    #[allow(dead_code)]
    pub fn apply_catch_zone(&self, base_half: f32) -> f32 {
        base_half * (1.0 + self.catch_zone_bonus)
    }

    /// Compute the catch bar size in pixels at the current skill level.
    /// Formula: 40px base + 3px per skill level.
    pub fn bar_size_px(&self) -> f32 {
        BASE_BAR_SIZE_PX + BAR_SIZE_PER_LEVEL_PX * self.level as f32
    }

    /// Compute the bite wait reduction in seconds at the current skill level.
    /// Formula: 0.5 seconds per level.
    pub fn bite_wait_reduction(&self) -> f32 {
        BITE_WAIT_REDUCTION_PER_LEVEL * self.level as f32
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

/// Drains `FishingLevelUpEvent` and logs level-up milestones.
/// The sender (`update_fishing_skill`) already fires a `ToastEvent` for player
/// feedback; this handler exists solely to consume the event and avoid Bevy
/// "event not read" warnings.
pub fn track_fishing_level_up(mut events: EventReader<FishingLevelUpEvent>) {
    for event in events.read() {
        info!("[Fishing] Skill level-up: now level {}", event.new_level);
    }
}

/// Listens for `ItemPickupEvent` events whose `item_id` matches a fish in the
/// `FishRegistry`. For each fish caught the skill counter is incremented with
/// XP based on rarity. If a level boundary is crossed, a `FishingLevelUpEvent`
/// + `ToastEvent` are sent to inform the player.
pub fn update_fishing_skill(
    mut item_pickup: EventReader<ItemPickupEvent>,
    fish_registry: Res<FishRegistry>,
    mut skill: ResMut<FishingSkill>,
    mut level_up_events: EventWriter<FishingLevelUpEvent>,
    mut toast_events: EventWriter<ToastEvent>,
) {
    for event in item_pickup.read() {
        // Only count items that exist in the fish registry.
        let fish_def = match fish_registry.fish.get(&event.item_id) {
            Some(f) => f,
            None => continue,
        };

        let prev_level = skill.level;
        let rarity = fish_def.rarity;

        // Count each individual fish in a multi-quantity pickup (normally 1).
        for _ in 0..event.quantity {
            skill.add_catch_xp(rarity);
        }

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fishing_skill_default_level_zero() {
        let skill = FishingSkill::default();
        assert_eq!(skill.level, 0);
        assert_eq!(skill.total_catches, 0);
        assert_eq!(skill.xp, 0);
        assert!((skill.bite_speed_bonus).abs() < f32::EPSILON);
        assert!((skill.catch_zone_bonus).abs() < f32::EPSILON);
    }

    #[test]
    fn test_fishing_skill_level_zero_bonuses() {
        let mut skill = FishingSkill::default();
        skill.recalculate();
        assert_eq!(skill.level, 0);
        assert!((skill.bite_speed_bonus).abs() < f32::EPSILON);
        assert!((skill.catch_zone_bonus).abs() < f32::EPSILON);
    }

    #[test]
    fn test_fishing_skill_xp_thresholds() {
        let mut skill = FishingSkill::default();

        // 10 XP = level 1
        skill.xp = 10;
        skill.recalculate();
        assert_eq!(skill.level, 1);

        // 25 XP = level 2
        skill.xp = 25;
        skill.recalculate();
        assert_eq!(skill.level, 2);

        // 50 XP = level 3
        skill.xp = 50;
        skill.recalculate();
        assert_eq!(skill.level, 3);

        // 100 XP = level 4
        skill.xp = 100;
        skill.recalculate();
        assert_eq!(skill.level, 4);

        // 1500 XP = level 10
        skill.xp = 1500;
        skill.recalculate();
        assert_eq!(skill.level, 10);

        // Beyond max stays at 10
        skill.xp = 5000;
        skill.recalculate();
        assert_eq!(skill.level, 10);
    }

    #[test]
    fn test_fishing_skill_between_thresholds() {
        let mut skill = FishingSkill::default();

        // 9 XP = still level 0
        skill.xp = 9;
        skill.recalculate();
        assert_eq!(skill.level, 0);

        // 24 XP = level 1
        skill.xp = 24;
        skill.recalculate();
        assert_eq!(skill.level, 1);

        // 49 XP = level 2
        skill.xp = 49;
        skill.recalculate();
        assert_eq!(skill.level, 2);
    }

    #[test]
    fn test_xp_per_rarity() {
        assert_eq!(xp_for_rarity(Rarity::Common), 3);
        assert_eq!(xp_for_rarity(Rarity::Uncommon), 8);
        assert_eq!(xp_for_rarity(Rarity::Rare), 15);
        assert_eq!(xp_for_rarity(Rarity::Legendary), 25);
    }

    #[test]
    fn test_add_catch_xp() {
        let mut skill = FishingSkill::default();
        skill.add_catch_xp(Rarity::Common);
        assert_eq!(skill.xp, 3);
        assert_eq!(skill.total_catches, 1);

        skill.add_catch_xp(Rarity::Uncommon);
        assert_eq!(skill.xp, 11); // 3 + 8 = 11, crosses level 1 threshold
        assert_eq!(skill.total_catches, 2);
        assert_eq!(skill.level, 1);
    }

    #[test]
    fn test_bar_size_px() {
        let mut skill = FishingSkill::default();
        // Level 0: 40px
        assert!((skill.bar_size_px() - 40.0).abs() < f32::EPSILON);

        // Level 5: 40 + 15 = 55px
        skill.xp = 200;
        skill.recalculate();
        assert_eq!(skill.level, 5);
        assert!((skill.bar_size_px() - 55.0).abs() < f32::EPSILON);

        // Level 10: 40 + 30 = 70px
        skill.xp = 1500;
        skill.recalculate();
        assert_eq!(skill.level, 10);
        assert!((skill.bar_size_px() - 70.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_bite_wait_reduction() {
        let mut skill = FishingSkill::default();
        // Level 0: 0s reduction
        assert!((skill.bite_wait_reduction()).abs() < f32::EPSILON);

        // Level 3: 1.5s reduction
        skill.xp = 50;
        skill.recalculate();
        assert_eq!(skill.level, 3);
        assert!((skill.bite_wait_reduction() - 1.5).abs() < f32::EPSILON);

        // Level 10: 5.0s reduction
        skill.xp = 1500;
        skill.recalculate();
        assert!((skill.bite_wait_reduction() - 5.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_fishing_skill_max_level_cap() {
        let mut skill = FishingSkill { xp: 5000, ..FishingSkill::default() };
        skill.recalculate();
        assert_eq!(skill.level, 10);
        // bite_speed_bonus caps at 0.5
        assert!((skill.bite_speed_bonus - FishingSkill::MAX_BITE_SPEED).abs() < f32::EPSILON);
        // catch_zone_bonus caps at 0.3
        assert!((skill.catch_zone_bonus - FishingSkill::MAX_CATCH_ZONE).abs() < f32::EPSILON);
    }

    #[test]
    fn test_apply_bite_speed_zero_bonus() {
        let skill = FishingSkill::default();
        let result = skill.apply_bite_speed(5.0);
        assert!((result - 5.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_apply_bite_speed_with_bonus() {
        let mut skill = FishingSkill { xp: 10, ..FishingSkill::default() };
        skill.recalculate();
        let result = skill.apply_bite_speed(10.0);
        // level 1 -> 0.05 bonus: 10.0 * (1.0 - 0.05) = 9.5
        assert!((result - 9.5).abs() < 0.001);
    }

    #[test]
    fn test_apply_bite_speed_zero_input() {
        let mut skill = FishingSkill { xp: 100, ..FishingSkill::default() };
        skill.recalculate();
        let result = skill.apply_bite_speed(0.0);
        assert!(
            (result).abs() < f32::EPSILON,
            "0 input should produce 0 output"
        );
    }

    #[test]
    fn test_apply_catch_zone_with_bonus() {
        let mut skill = FishingSkill { xp: 25, ..FishingSkill::default() };
        skill.recalculate();
        assert_eq!(skill.level, 2);
        let result = skill.apply_catch_zone(50.0);
        // 50.0 * (1.0 + 0.06) = 53.0
        assert!((result - 53.0).abs() < 0.001);
    }
}
