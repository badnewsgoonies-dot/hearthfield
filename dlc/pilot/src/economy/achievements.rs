//! Achievement tracking system.

use crate::shared::*;
use bevy::prelude::*;

#[allow(clippy::too_many_arguments)]
pub fn check_achievements(
    pilot_state: Res<PilotState>,
    play_stats: Res<PlayStats>,
    _fleet: Res<Fleet>,
    _relationships: Res<Relationships>,
    gold: Res<Gold>,
    mut achievements: ResMut<Achievements>,
    mut achievement_events: EventWriter<AchievementUnlockedEvent>,
    mut toast_events: EventWriter<ToastEvent>,
) {
    let checks: Vec<(&str, bool)> = vec![
        ("first_flight", play_stats.total_flights >= 1),
        ("perfect_10", play_stats.perfect_landings >= 10),
        ("all_airports", play_stats.airports_visited.len() >= 10),
        ("captain_rank", pilot_state.rank >= PilotRank::Captain),
        ("ace_rank", pilot_state.rank >= PilotRank::Ace),
        (
            "millionaire",
            play_stats.total_flights > 0 && gold.amount >= 1_000_000,
        ),
        ("100_flights", play_stats.total_flights >= 100),
        ("500_flights", play_stats.total_flights >= 500),
        ("all_licenses", pilot_state.licenses.len() >= 7),
        ("cargo_king", play_stats.missions_completed >= 100),
        (
            "no_damage",
            play_stats.total_flights >= 50 && play_stats.rough_landings == 0,
        ),
    ];

    for (id, condition) in checks {
        if condition && !achievements.is_unlocked(id) {
            achievements.unlock(id);
            achievement_events.send(AchievementUnlockedEvent {
                achievement_id: id.to_string(),
            });
            if let Some(def) = ACHIEVEMENTS.iter().find(|a| a.id == id) {
                toast_events.send(ToastEvent {
                    message: format!("🏆 Achievement: {}", def.name),
                    duration_secs: 5.0,
                });
            }
        }
    }
}
