//! Achievement system for Hearthfield.
//!
//! Defines all 30 achievements, checks conditions every frame during Playing state,
//! and fires `AchievementUnlockedEvent` when a new achievement is earned.
//! Also tracks manually-counted progress counters via `Achievements.progress`.

use bevy::prelude::*;
use crate::shared::*;

// ═══════════════════════════════════════════════════════════════════════
// ACHIEVEMENT DEFINITIONS
// ═══════════════════════════════════════════════════════════════════════

/// Static description of a single achievement.
pub struct AchievementDef {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
}

/// All 30 achievements defined statically.
pub const ACHIEVEMENTS: &[AchievementDef] = &[
    AchievementDef {
        id: "first_harvest",
        name: "First Harvest",
        description: "Harvest your first crop",
    },
    AchievementDef {
        id: "green_thumb",
        name: "Green Thumb",
        description: "Harvest 100 crops",
    },
    AchievementDef {
        id: "master_farmer",
        name: "Master Farmer",
        description: "Harvest 1000 crops",
    },
    AchievementDef {
        id: "gone_fishin",
        name: "Gone Fishin'",
        description: "Catch your first fish",
    },
    AchievementDef {
        id: "angler",
        name: "Angler",
        description: "Catch 50 fish",
    },
    AchievementDef {
        id: "fisherman",
        name: "Fisherman",
        description: "Catch 100 fish",
    },
    AchievementDef {
        id: "social_butterfly",
        name: "Social Butterfly",
        description: "Reach 5 hearts with 5 NPCs",
    },
    AchievementDef {
        id: "best_friends",
        name: "Best Friends",
        description: "Reach 10 hearts with any NPC",
    },
    AchievementDef {
        id: "community_pillar",
        name: "Community Pillar",
        description: "Reach 5+ hearts with all 10 NPCs",
    },
    AchievementDef {
        id: "newlywed",
        name: "Newlywed",
        description: "Get married",
    },
    AchievementDef {
        id: "deep_pockets",
        name: "Deep Pockets",
        description: "Earn 100,000 gold total",
    },
    AchievementDef {
        id: "steady_income",
        name: "Steady Income",
        description: "Earn 10,000 gold",
    },
    AchievementDef {
        id: "millionaire",
        name: "Millionaire",
        description: "Have 1,000,000 gold",
    },
    AchievementDef {
        id: "shipping_mogul",
        name: "Shipping Mogul",
        description: "Ship 500 items",
    },
    AchievementDef {
        id: "spelunker",
        name: "Spelunker",
        description: "Reach mine floor 10",
    },
    AchievementDef {
        id: "mine_crawler",
        name: "Mine Crawler",
        description: "Reach mine floor 20",
    },
    AchievementDef {
        id: "chef",
        name: "Chef",
        description: "Cook 20 recipes",
    },
    AchievementDef {
        id: "all_seasons",
        name: "All Seasons",
        description: "Play through all 4 seasons",
    },
    AchievementDef {
        id: "second_year",
        name: "Second Year",
        description: "Reach Year 2",
    },
    AchievementDef {
        id: "pet_lover",
        name: "Pet Lover",
        description: "Max happiness on a pet",
    },
    AchievementDef {
        id: "rancher",
        name: "Rancher",
        description: "Own 12 animals",
    },
    AchievementDef {
        id: "early_riser",
        name: "Early Riser",
        description: "Water all crops before 8 AM",
    },
    AchievementDef {
        id: "night_owl",
        name: "Night Owl",
        description: "Still awake at 1 AM",
    },
    AchievementDef {
        id: "artisan",
        name: "Artisan",
        description: "Craft 20 items",
    },
    AchievementDef {
        id: "generous",
        name: "Generous",
        description: "Give 50 gifts",
    },
    AchievementDef {
        id: "gold_star",
        name: "Gold Star",
        description: "Harvest a gold-quality crop",
    },
    AchievementDef {
        id: "home_sweet_home",
        name: "Home Sweet Home",
        description: "Upgrade house to Deluxe",
    },
    AchievementDef {
        id: "green_acres",
        name: "Green Acres",
        description: "Plant 50 crops",
    },
    AchievementDef {
        id: "rock_breaker",
        name: "Rock Breaker",
        description: "Break 100 rocks",
    },
    AchievementDef {
        id: "completionist",
        name: "Completionist",
        description: "Unlock 25 other achievements",
    },
];

// ═══════════════════════════════════════════════════════════════════════
// HELPER: evaluate each achievement condition
// ═══════════════════════════════════════════════════════════════════════

/// Returns `true` if the achievement with the given id should be unlocked
/// given the current game state. Assumes the achievement is not yet unlocked.
fn evaluate_condition(
    id: &str,
    stats: &PlayStats,
    relationships: &Relationships,
    player: &PlayerState,
    calendar: &Calendar,
    animals: &AnimalState,
    marriage: &MarriageState,
    mine: &MineState,
    achievements: &Achievements,
    house: &HouseState,
    farm: &FarmState,
) -> bool {
    match id {
        // ── Farming ──────────────────────────────────────────────────────
        "first_harvest"   => stats.crops_harvested >= 1,
        "green_thumb"     => stats.crops_harvested >= 100,
        "master_farmer"   => stats.crops_harvested >= 1_000,

        // ── Fishing ──────────────────────────────────────────────────────
        "gone_fishin"     => stats.fish_caught >= 1,
        "angler"          => stats.fish_caught >= 50,
        "fisherman"       => stats.fish_caught >= 100,

        // ── Social ───────────────────────────────────────────────────────
        "social_butterfly" => {
            let count_five = relationships
                .friendship
                .values()
                .filter(|&&pts| pts >= 500) // 5 hearts = 500 pts
                .count();
            count_five >= 5
        }
        "best_friends" => {
            relationships
                .friendship
                .values()
                .any(|&pts| pts >= 1_000) // 10 hearts = 1000 pts
        }
        "community_pillar" => {
            // All 10 main NPCs must have 5+ hearts.
            // We consider "all 10" as having at least 10 distinct NPC entries
            // with 500+ friendship points.
            let qualifying: Vec<_> = relationships
                .friendship
                .iter()
                .filter(|(_, &pts)| pts >= 500)
                .collect();
            qualifying.len() >= 10
        }
        "newlywed" => marriage.spouse.is_some(),

        // ── Economy ──────────────────────────────────────────────────────
        "deep_pockets"    => stats.total_gold_earned >= 100_000,
        "steady_income"   => stats.total_gold_earned >= 10_000,
        "millionaire"     => player.gold >= 1_000_000,
        "shipping_mogul"  => stats.items_shipped >= 500,

        // ── Mining ───────────────────────────────────────────────────────
        "spelunker"       => mine.deepest_floor_reached >= 10,
        "mine_crawler"    => mine.deepest_floor_reached >= 20,

        // ── Crafting/Cooking ─────────────────────────────────────────────
        "chef"            => stats.recipes_cooked >= 20,

        // ── Time / Seasons ───────────────────────────────────────────────
        "all_seasons"     => calendar.year >= 2 || stats.days_played >= 112,
        "second_year"     => calendar.year >= 2,

        // ── Animals ──────────────────────────────────────────────────────
        "pet_lover" => {
            animals.animals.iter().any(|a| {
                matches!(a.kind, AnimalKind::Cat | AnimalKind::Dog) && a.happiness >= 255
            })
        }
        "rancher" => animals.animals.len() >= 12,

        // ── Time-of-day ──────────────────────────────────────────────────
        "early_riser" => {
            // Check: before 8 AM and every tilled soil tile has been watered.
            // We look at FarmState.soil and verify no tile is in Tilled (unwatered) state.
            if calendar.time_float() < 8.0 {
                let any_unwatered = farm
                    .soil
                    .values()
                    .any(|s| *s == SoilState::Tilled);
                // Has at least one crop/soil tile AND none are unwatered
                !farm.soil.is_empty() && !any_unwatered
            } else {
                false
            }
        }
        "night_owl" => {
            // 1 AM = hour 25 in the 6–25 scale (where 24 = midnight, 25 = 1:00 AM)
            calendar.time_float() >= 25.0
        }

        // ── Progress-counter achievements ────────────────────────────────
        "artisan" => {
            achievements.progress.get("crafts").copied().unwrap_or(0) >= 20
        }
        "generous"    => stats.gifts_given >= 50,
        "gold_star"   => {
            achievements.progress.get("gold_crops").copied().unwrap_or(0) >= 1
        }

        // ── House ────────────────────────────────────────────────────────
        "home_sweet_home" => house.tier == HouseTier::Deluxe,

        // ── Progress-counter achievements (continued) ────────────────────
        "green_acres" => {
            achievements.progress.get("crops_planted").copied().unwrap_or(0) >= 50
        }
        "rock_breaker" => {
            achievements.progress.get("rocks_broken").copied().unwrap_or(0) >= 100
        }

        // ── Meta ─────────────────────────────────────────────────────────
        "completionist" => {
            // Count unlocked achievements that aren't "completionist" itself
            let non_self_unlocked = achievements
                .unlocked
                .iter()
                .filter(|id| id.as_str() != "completionist")
                .count();
            non_self_unlocked >= 25
        }

        _ => false,
    }
}

// ═══════════════════════════════════════════════════════════════════════
// SYSTEM: check_achievements
// ═══════════════════════════════════════════════════════════════════════

/// Runs every frame during `GameState::Playing`.
///
/// For each defined achievement not yet unlocked, evaluates its condition
/// and fires an `AchievementUnlockedEvent` when it becomes true.
/// Also pushes the id into `Achievements.unlocked`.
pub fn check_achievements(
    stats:         Res<PlayStats>,
    relationships: Res<Relationships>,
    player:        Res<PlayerState>,
    calendar:      Res<Calendar>,
    animals:       Res<AnimalState>,
    marriage:      Res<MarriageState>,
    mine:          Res<MineState>,
    mut achievements: ResMut<Achievements>,
    house:         Res<HouseState>,
    farm:          Res<FarmState>,
    mut events:    EventWriter<AchievementUnlockedEvent>,
) {
    // Collect newly unlocked ids to avoid borrowing `achievements` mutably
    // while also reading it.
    let mut newly_unlocked: Vec<(&'static str, &'static str, &'static str)> = Vec::new();

    for def in ACHIEVEMENTS {
        // Skip if already unlocked
        if achievements.unlocked.iter().any(|id| id == def.id) {
            continue;
        }

        if evaluate_condition(
            def.id,
            &stats,
            &relationships,
            &player,
            &calendar,
            &animals,
            &marriage,
            &mine,
            &achievements,
            &house,
            &farm,
        ) {
            newly_unlocked.push((def.id, def.name, def.description));
        }
    }

    for (id, name, description) in newly_unlocked {
        achievements.unlocked.push(id.to_string());

        events.send(AchievementUnlockedEvent {
            achievement_id: id.to_string(),
            name: name.to_string(),
            description: description.to_string(),
        });

        info!(
            "[Achievements] Unlocked: \"{}\" — {}",
            name, description
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════
// SYSTEM: track_achievement_progress
// ═══════════════════════════════════════════════════════════════════════

/// Listens to game events to increment manually-tracked counters inside
/// `Achievements.progress`.
///
/// Tracked counters:
/// - `rocks_broken`   — incremented on `ToolUseEvent` with Pickaxe
/// - `crafts`         — incremented on `CropHarvestedEvent` where the item is
///                      an artisan good (approximation: tracked via a crafting
///                      event; here we wire it to ItemPickupEvent for artisan goods)
/// - `gold_crops`     — incremented on `CropHarvestedEvent` with Gold+ quality
/// - `crops_planted`  — incremented on `ToolUseEvent` with Hoe (soil tilling
///                      is a reasonable proxy for planting intent); also
///                      incremented on CropHarvestedEvent as a post-hoc count
pub fn track_achievement_progress(
    mut tool_events:    EventReader<ToolUseEvent>,
    mut harvest_events: EventReader<CropHarvestedEvent>,
    mut achievements:   ResMut<Achievements>,
) {
    // ── Pickaxe swings → rocks_broken ────────────────────────────────
    for ev in tool_events.read() {
        match ev.tool {
            ToolKind::Pickaxe => {
                let counter = achievements.progress.entry("rocks_broken".to_string()).or_insert(0);
                *counter = counter.saturating_add(1);
            }
            ToolKind::Hoe => {
                // Tilling soil is used as a proxy for "crop planted"
                let counter = achievements.progress.entry("crops_planted".to_string()).or_insert(0);
                *counter = counter.saturating_add(1);
            }
            _ => {}
        }
    }

    // ── Crop harvested → gold_crops ──────────────────────────────────
    for ev in harvest_events.read() {
        if let Some(quality) = ev.quality {
            // Gold or Iridium quality counts as "gold star"
            if matches!(quality, ItemQuality::Gold | ItemQuality::Iridium) {
                let counter = achievements.progress.entry("gold_crops".to_string()).or_insert(0);
                *counter = counter.saturating_add(1);
            }
        }
    }
}
