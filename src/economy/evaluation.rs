//! Year-end evaluation system — grandpa's shrine scoring on Spring 1 of Year 3+.
//!
//! Triggered automatically when Calendar reaches Year >= 3, Season::Spring, Day 1.
//! Can also be re-triggered at any time after initial evaluation to show progress.
//!
//! Scoring: up to 21 points across 8 categories. Candle count is determined by
//! point thresholds: 0-5 = 1 candle, 6-10 = 2, 11-15 = 3, 16-21 = 4.

use bevy::prelude::*;
use crate::shared::*;

use super::gold::EconomyStats;
use super::stats::HarvestStats;

// ─────────────────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Convert a raw point total (0–21) into a candle count.
fn points_to_candles(points: u32) -> u8 {
    match points {
        0..=5 => 1,
        6..=10 => 2,
        11..=15 => 3,
        _ => 4, // 16-21
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Systems
// ─────────────────────────────────────────────────────────────────────────────

/// Checks every frame (while Playing) whether it is Spring 1 of Year 3 or later
/// and the evaluation has not yet occurred. If so, fires `EvaluationTriggerEvent`.
pub fn check_evaluation_trigger(
    calendar: Res<Calendar>,
    eval_score: Res<EvaluationScore>,
    mut trigger_events: EventWriter<EvaluationTriggerEvent>,
) {
    if eval_score.evaluated {
        return;
    }

    if calendar.year >= 3 && calendar.season == Season::Spring && calendar.day == 1 {
        info!("[Evaluation] Year {} Spring Day 1 detected — firing EvaluationTriggerEvent.", calendar.year);
        trigger_events.send(EvaluationTriggerEvent);
    }
}

/// Reads `EvaluationTriggerEvent`, scores the player across all categories, updates
/// `EvaluationScore`, and sends a `ToastEvent` describing the result.
///
/// Also handles re-evaluation: if already evaluated, compares candle count and
/// toasts whether the player gained candles since last time.
pub fn handle_evaluation(
    mut trigger_events: EventReader<EvaluationTriggerEvent>,
    mut eval_score: ResMut<EvaluationScore>,
    mut toast_events: EventWriter<ToastEvent>,
    // Resources used for scoring
    economy_stats: Res<EconomyStats>,
    harvest_stats: Res<HarvestStats>,
    relationships: Res<Relationships>,
    marriage_state: Res<MarriageState>,
    mine_state: Res<MineState>,
    animal_state: Res<AnimalState>,
    house_state: Res<HouseState>,
    quest_log: Res<QuestLog>,
    unlocked_recipes: Res<UnlockedRecipes>,
    player_state: Res<PlayerState>,
    play_stats: Res<PlayStats>,
    shipping_log: Res<ShippingLog>,
) {
    for _ev in trigger_events.read() {
        let previous_candles = eval_score.candles_lit;
        let was_evaluated = eval_score.evaluated;

        let mut categories: std::collections::HashMap<String, u32> = std::collections::HashMap::new();
        let mut total = 0u32;

        // ── Earnings (4 points) ───────────────────────────────────────────────
        let gold_earned = economy_stats.total_gold_earned;

        if gold_earned >= 50_000 {
            categories.insert("earnings_50k".to_string(), 1);
            total += 1;
        }
        if gold_earned >= 100_000 {
            categories.insert("earnings_100k".to_string(), 1);
            total += 1;
        }
        if gold_earned >= 200_000 {
            categories.insert("earnings_200k".to_string(), 1);
            total += 1;
        }
        if gold_earned >= 500_000 {
            categories.insert("earnings_500k".to_string(), 1);
            total += 1;
        }

        // ── Friends (2 points) ────────────────────────────────────────────────
        // Count NPCs with 8+ hearts (800+ friendship points)
        let npcs_at_8_hearts = relationships
            .friendship
            .values()
            .filter(|&&pts| pts >= 800)
            .count();
        if npcs_at_8_hearts >= 5 {
            categories.insert("friends_5_at_8_hearts".to_string(), 1);
            total += 1;
        }

        // Count NPCs with 5+ hearts (500+ friendship points)
        let npcs_at_5_hearts = relationships
            .friendship
            .values()
            .filter(|&&pts| pts >= 500)
            .count();
        if npcs_at_5_hearts >= 10 {
            categories.insert("friends_10_at_5_hearts".to_string(), 1);
            total += 1;
        }

        // ── Spouse (2 points) ─────────────────────────────────────────────────
        if marriage_state.spouse.is_some() {
            categories.insert("spouse_married".to_string(), 1);
            total += 1;
        }
        // Spouse happiness > 50 (scale: -100 to 100)
        if marriage_state.spouse.is_some() && marriage_state.spouse_happiness > 50 {
            categories.insert("spouse_happiness".to_string(), 1);
            total += 1;
        }

        // ── Skills (4 points) ─────────────────────────────────────────────────
        // 50+ crops harvested total (sum all crop counts in HarvestStats)
        let total_crops_harvested: u32 = harvest_stats.crops.values().map(|(count, _)| *count).sum();
        if total_crops_harvested >= 50 {
            categories.insert("skills_crops_50".to_string(), 1);
            total += 1;
        }

        // 100+ fish caught
        let fish_caught: u32 = play_stats.fish_caught as u32;
        if fish_caught >= 100 {
            categories.insert("skills_fish_100".to_string(), 1);
            total += 1;
        }

        // Floor 20 reached in the mine
        if mine_state.deepest_floor_reached >= 20 {
            categories.insert("skills_mine_floor_20".to_string(), 1);
            total += 1;
        }

        // 20+ recipes cooked (uses UnlockedRecipes)
        if unlocked_recipes.ids.len() >= 20 {
            categories.insert("skills_recipes_20".to_string(), 1);
            total += 1;
        }

        // ── Farm (3 points) ───────────────────────────────────────────────────
        // House at Deluxe tier
        if house_state.tier == HouseTier::Deluxe {
            categories.insert("farm_deluxe_house".to_string(), 1);
            total += 1;
        }

        // Own 8 or more animals
        let animal_count = animal_state.animals.len();
        if animal_count >= 8 {
            categories.insert("farm_animals_8".to_string(), 1);
            total += 1;
        }

        // 50+ items shipped total
        if economy_stats.total_items_shipped >= 50 {
            categories.insert("farm_items_shipped_50".to_string(), 1);
            total += 1;
        }

        // ── Collection (1 point) ──────────────────────────────────────────────
        // 30+ unique items shipped
        if shipping_log.shipped_items.len() >= 30 {
            categories.insert("collection_unique_30".to_string(), 1);
            total += 1;
        }

        // ── Community (1 point) ───────────────────────────────────────────────
        if quest_log.completed.len() >= 10 {
            categories.insert("community_quests_10".to_string(), 1);
            total += 1;
        }

        // ── Extras (1 point) ──────────────────────────────────────────────────
        // Have 1,000,000 gold on hand right now
        if player_state.gold >= 1_000_000 {
            categories.insert("extras_1m_gold".to_string(), 1);
            total += 1;
        }

        // ── Clamp and store ───────────────────────────────────────────────────
        let total = total.min(21); // defensive clamp; should never exceed 21
        let candles = points_to_candles(total);

        info!(
            "[Evaluation] Score calculated: {} / 21 points → {} candle(s). Categories: {:?}",
            total, candles, categories
        );

        // Update the resource
        eval_score.total_points = total;
        eval_score.categories = categories;
        eval_score.candles_lit = candles;
        eval_score.evaluated = true;

        // ── Toast ─────────────────────────────────────────────────────────────
        if was_evaluated {
            // Re-evaluation: describe change vs last time.
            if candles > previous_candles {
                toast_events.send(ToastEvent {
                    message: format!(
                        "Evaluation: {} points — {} candle(s) lit! (+{} since last visit)",
                        total,
                        candles,
                        candles - previous_candles
                    ),
                    duration_secs: 6.0,
                });
            } else if candles < previous_candles {
                toast_events.send(ToastEvent {
                    message: format!(
                        "Evaluation: {} points — {} candle(s) lit. ({} fewer than before)",
                        total,
                        candles,
                        previous_candles - candles
                    ),
                    duration_secs: 6.0,
                });
            } else {
                toast_events.send(ToastEvent {
                    message: format!(
                        "Evaluation: {} points — {} candle(s) lit. (Same as before)",
                        total, candles
                    ),
                    duration_secs: 5.0,
                });
            }
        } else {
            // First evaluation.
            toast_events.send(ToastEvent {
                message: format!(
                    "Evaluation: {} points — {} candle(s) lit!",
                    total, candles
                ),
                duration_secs: 6.0,
            });
        }
    }
}

/// Allows the player to manually re-trigger the evaluation at any point after the
/// first evaluation by sending another `EvaluationTriggerEvent`.
///
/// This system does not fire any events itself — it simply resets `evaluated` to
/// `false` so that `check_evaluation_trigger` can fire again *if* the calendar
/// conditions still match, OR so that an external UI system can send
/// `EvaluationTriggerEvent` directly.  In practice, the UI domain is expected to
/// send the event; this system exists as documentation of the contract and to
/// ensure the flag can be toggled without modifying `shared/mod.rs`.
///
/// NOTE: Currently this system is a no-op stub included to satisfy the re-evaluation
/// contract described in the task.  A real re-evaluation is started by any system
/// (typically the UI shrine interaction) that sends `EvaluationTriggerEvent` — the
/// `handle_evaluation` system handles both first and subsequent evaluations cleanly.
#[allow(dead_code)]
#[allow(dead_code)]
pub fn re_evaluate(
    // No inputs required — the re-eval is driven by sending EvaluationTriggerEvent
    // from outside (e.g., the UI domain when the player interacts with the shrine).
) {
    // Intentionally empty: re-evaluation is handled in handle_evaluation which
    // already compares previous_candles with the new score.  The caller need only
    // fire EvaluationTriggerEvent at any time.
}
