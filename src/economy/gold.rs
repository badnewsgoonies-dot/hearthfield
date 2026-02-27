use bevy::prelude::*;
use crate::shared::*;

/// Tracks economy statistics for save data and achievements.
#[derive(Resource, Debug, Clone, Default)]
pub struct EconomyStats {
    pub total_gold_earned: u64,
    pub total_gold_spent: u64,
    pub total_items_shipped: u64,
    pub total_transactions: u64,
}

/// Applies GoldChangeEvents to PlayerState.gold.
/// Validates that spending does not put gold below 0 (clamped to 0).
/// Tracks total_gold_earned for EconomyStats.
pub fn apply_gold_changes(
    mut gold_events: EventReader<GoldChangeEvent>,
    mut player_state: ResMut<PlayerState>,
    mut stats: ResMut<EconomyStats>,
) {
    for ev in gold_events.read() {
        if ev.amount >= 0 {
            let gain = ev.amount as u32;
            player_state.gold = player_state.gold.saturating_add(gain);
            stats.total_gold_earned = stats.total_gold_earned.saturating_add(gain as u64);
            info!(
                "[Economy] Gold +{}: {}. New balance: {}g",
                gain, ev.reason, player_state.gold
            );
        } else {
            let cost = (-ev.amount) as u32;
            if player_state.gold >= cost {
                player_state.gold -= cost;
                stats.total_gold_spent = stats.total_gold_spent.saturating_add(cost as u64);
                info!(
                    "[Economy] Gold -{}: {}. New balance: {}g",
                    cost, ev.reason, player_state.gold
                );
            } else {
                // Not enough gold — this should have been validated before sending the event.
                // Log a warning but still clamp to 0 rather than panic.
                warn!(
                    "[Economy] Tried to spend {}g but only have {}g (reason: {}). Clamping to 0.",
                    cost, player_state.gold, ev.reason
                );
                stats.total_gold_spent =
                    stats.total_gold_spent.saturating_add(player_state.gold as u64);
                player_state.gold = 0;
            }
        }
        stats.total_transactions += 1;
    }
}

/// Format a gold amount as a display string (e.g. "1,234g").
#[allow(dead_code)]
pub fn format_gold(amount: u32) -> String {
    let s = amount.to_string();
    let mut result = String::new();
    let digits: Vec<char> = s.chars().collect();
    for (i, ch) in digits.iter().enumerate() {
        if i > 0 && (digits.len() - i) % 3 == 0 {
            result.push(',');
        }
        result.push(*ch);
    }
    result.push('g');
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_gold() {
        assert_eq!(format_gold(0), "0g");
        assert_eq!(format_gold(500), "500g");
        assert_eq!(format_gold(1234), "1,234g");
        assert_eq!(format_gold(25000), "25,000g");
        assert_eq!(format_gold(1000000), "1,000,000g");
    }

    #[test]
    fn test_format_gold_single_digit() {
        assert_eq!(format_gold(1), "1g");
        assert_eq!(format_gold(9), "9g");
    }

    #[test]
    fn test_format_gold_exact_thousands() {
        assert_eq!(format_gold(1000), "1,000g");
        assert_eq!(format_gold(100000), "100,000g");
    }

    #[test]
    fn test_economy_stats_default() {
        let stats = EconomyStats::default();
        assert_eq!(stats.total_gold_earned, 0);
        assert_eq!(stats.total_gold_spent, 0);
        assert_eq!(stats.total_items_shipped, 0);
        assert_eq!(stats.total_transactions, 0);
    }
}
