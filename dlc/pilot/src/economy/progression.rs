//! XP and rank progression — requirements, ceremonies, privileges, XP formulas, decay.

use bevy::prelude::*;
use crate::shared::*;

pub struct RankRequirements {
    pub min_flights: u32,
    pub min_hours: f32,
    pub min_reputation: f32,
    pub min_xp: u32,
}

pub fn rank_requirements(rank: PilotRank) -> RankRequirements {
    match rank {
        PilotRank::Student => RankRequirements { min_flights: 0, min_hours: 0.0, min_reputation: 0.0, min_xp: 0 },
        PilotRank::Private => RankRequirements { min_flights: 5, min_hours: 2.0, min_reputation: 20.0, min_xp: 100 },
        PilotRank::Commercial => RankRequirements { min_flights: 20, min_hours: 10.0, min_reputation: 40.0, min_xp: 350 },
        PilotRank::Senior => RankRequirements { min_flights: 50, min_hours: 30.0, min_reputation: 60.0, min_xp: 800 },
        PilotRank::Captain => RankRequirements { min_flights: 100, min_hours: 80.0, min_reputation: 75.0, min_xp: 1500 },
        PilotRank::Ace => RankRequirements { min_flights: 200, min_hours: 150.0, min_reputation: 90.0, min_xp: 3000 },
    }
}

pub fn meets_rank_requirements(pilot: &PilotState) -> bool {
    if let Some(next) = pilot.rank.next() {
        let req = rank_requirements(next);
        pilot.xp >= req.min_xp
            && pilot.total_flights >= req.min_flights
            && pilot.total_flight_hours >= req.min_hours
            && pilot.reputation >= req.min_reputation
    } else {
        false
    }
}

pub fn rank_privileges(rank: PilotRank) -> &'static [&'static str] {
    match rank {
        PilotRank::Student => &["Training flights only", "HomeBase airport only"],
        PilotRank::Private => &["Small single-engine aircraft", "Nearby airports unlocked", "Solo flights"],
        PilotRank::Commercial => &["Multi-engine aircraft", "Passenger flights", "Distant airports", "Cargo missions"],
        PilotRank::Senior => &["Turboprop & light jets", "All weather clearance", "High-altitude airports", "VIP charters"],
        PilotRank::Captain => &["All jets", "Command flights", "Emergency dispatch priority", "Elite airports"],
        PilotRank::Ace => &["Heavy jets", "Legendary status", "All missions", "Skyreach Elite access", "Ace flight suit"],
    }
}

pub fn calculate_flight_xp(
    base_xp: u32,
    landing_grade: &str,
    weather: Weather,
    had_emergency: bool,
    passenger_satisfaction: f32,
    on_time: bool,
) -> u32 {
    let mut total = base_xp as f32;

    total += match landing_grade {
        "Perfect" => 50.0,
        "Good" => 25.0,
        "Acceptable" => 10.0,
        "Rough" => -10.0,
        _ => 0.0,
    };

    total += weather.flight_difficulty() * 30.0;
    if had_emergency { total += 75.0; }
    total += (passenger_satisfaction / 100.0) * 20.0;
    if on_time { total += 15.0; }

    total.max(1.0) as u32
}

#[derive(Resource, Default)]
pub struct ActivityTracker {
    pub last_flight_day: u32,
    pub decay_warned: bool,
}

pub fn apply_xp(
    mut events: EventReader<XpGainEvent>,
    mut pilot_state: ResMut<PilotState>,
    mut play_stats: ResMut<PlayStats>,
    mut activity: ResMut<ActivityTracker>,
    calendar: Res<Calendar>,
) {
    for ev in events.read() {
        pilot_state.xp += ev.amount;
        play_stats.missions_completed += 1;
        activity.last_flight_day = calendar.total_days();
        activity.decay_warned = false;
    }
}

pub fn check_rank_up(
    mut pilot_state: ResMut<PilotState>,
    mut rank_events: EventWriter<RankUpEvent>,
    mut toast_events: EventWriter<ToastEvent>,
    mut sfx_events: EventWriter<PlaySfxEvent>,
    mut cutscene_queue: ResMut<CutsceneQueue>,
) {
    if !meets_rank_requirements(&pilot_state) { return; }

    if let Some(next_rank) = pilot_state.rank.next() {
        pilot_state.rank = next_rank;
        pilot_state.xp_to_next_rank = next_rank.next().map_or(u32::MAX, |r| r.xp_required());

        rank_events.send(RankUpEvent { new_rank: next_rank });
        sfx_events.send(PlaySfxEvent { sfx_id: "rank_up".to_string() });

        let privileges = rank_privileges(next_rank);
        let privilege_text = privileges.join(", ");
        cutscene_queue.pending.push(vec![
            CutsceneStep::FadeOut { duration: 0.5 },
            CutsceneStep::Dialogue {
                speaker: "Tower".to_string(),
                text: format!("Congratulations! You've been promoted to {}!", next_rank.display_name()),
            },
            CutsceneStep::Dialogue {
                speaker: "Tower".to_string(),
                text: format!("New privileges: {}", privilege_text),
            },
            CutsceneStep::PlaySfx { sfx_id: "applause".to_string() },
            CutsceneStep::FadeIn { duration: 0.5 },
        ]);

        toast_events.send(ToastEvent {
            message: format!("🎖 Promoted to {}!", next_rank.display_name()),
            duration_secs: 5.0,
        });
    }
}

pub fn check_rank_decay(
    calendar: Res<Calendar>,
    mut pilot_state: ResMut<PilotState>,
    mut activity: ResMut<ActivityTracker>,
    mut toast_events: EventWriter<ToastEvent>,
) {
    let day = calendar.total_days();
    let days_idle = day.saturating_sub(activity.last_flight_day);

    if days_idle >= 14 && !activity.decay_warned {
        activity.decay_warned = true;
        let decay = ((days_idle - 14) as f32 * 0.5).min(10.0);
        pilot_state.reputation = (pilot_state.reputation - decay).max(0.0);
        toast_events.send(ToastEvent {
            message: format!("📉 Reputation declining! Fly to maintain standing. ({:.0}%)", pilot_state.reputation),
            duration_secs: 4.0,
        });
    }

    if (7..14).contains(&days_idle) && !activity.decay_warned {
        activity.decay_warned = true;
        toast_events.send(ToastEvent {
            message: "You haven't flown in a while. Your skills may get rusty!".to_string(),
            duration_secs: 3.0,
        });
    }
}

pub fn rank_progress_summary(pilot: &PilotState) -> String {
    if let Some(next) = pilot.rank.next() {
        let req = rank_requirements(next);
        format!(
            "Next: {} — XP: {}/{}, Flights: {}/{}, Hours: {:.0}/{:.0}, Rep: {:.0}/{:.0}",
            next.display_name(),
            pilot.xp, req.min_xp,
            pilot.total_flights, req.min_flights,
            pilot.total_flight_hours, req.min_hours,
            pilot.reputation, req.min_reputation,
        )
    } else {
        "Maximum rank achieved!".to_string()
    }
}
