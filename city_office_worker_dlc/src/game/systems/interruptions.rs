use bevy::prelude::*;

use crate::game::events::{
    CoworkerHelpEvent, InterruptionEvent, ManagerCheckInEvent, PanicResponseEvent,
    ResolveCalmlyEvent,
};
use crate::game::resources::{
    format_clock, DayClock, DayStats, OfficeRules, PlayerCareerState, PlayerMindState,
};

pub fn handle_interruption_requests(
    mut requests: EventReader<InterruptionEvent>,
    rules: Res<OfficeRules>,
    mut clock: ResMut<DayClock>,
    mut stats: ResMut<DayStats>,
    mut mind: ResMut<PlayerMindState>,
) {
    for _ in requests.read() {
        if clock.ended {
            continue;
        }

        clock.advance(rules.interruption_minutes);
        mind.stress = (mind.stress + rules.interruption_stress_increase).clamp(0, rules.max_stress);
        mind.focus = (mind.focus - rules.interruption_focus_loss).clamp(0, rules.max_focus);
        mind.pending_interruptions = mind.pending_interruptions.saturating_add(1);
        stats.interruptions_triggered = stats.interruptions_triggered.saturating_add(1);

        info!(
            "Interruption! stress: {}, focus: {}, pending: {}, clock: {}",
            mind.stress,
            mind.focus,
            mind.pending_interruptions,
            format_clock(clock.current_minute)
        );
    }
}

pub fn handle_resolve_calmly_requests(
    mut requests: EventReader<ResolveCalmlyEvent>,
    rules: Res<OfficeRules>,
    clock: Res<DayClock>,
    mut stats: ResMut<DayStats>,
    mut mind: ResMut<PlayerMindState>,
) {
    for _ in requests.read() {
        if clock.ended {
            continue;
        }

        if mind.pending_interruptions == 0 {
            info!("No interruption is pending; calm response ignored.");
            continue;
        }

        let before_focus = mind.focus;
        let before_stress = mind.stress;

        mind.pending_interruptions -= 1;
        mind.focus = (mind.focus + rules.calm_focus_restore).clamp(0, rules.max_focus);
        mind.stress = (mind.stress - rules.calm_stress_relief).clamp(0, rules.max_stress);
        stats.calm_responses = stats.calm_responses.saturating_add(1);

        info!(
            "Resolved calmly -> focus: {} -> {}, stress: {} -> {}, pending: {}",
            before_focus, mind.focus, before_stress, mind.stress, mind.pending_interruptions
        );
    }
}

pub fn handle_panic_response_requests(
    mut requests: EventReader<PanicResponseEvent>,
    rules: Res<OfficeRules>,
    clock: Res<DayClock>,
    mut stats: ResMut<DayStats>,
    mut mind: ResMut<PlayerMindState>,
) {
    for _ in requests.read() {
        if clock.ended {
            continue;
        }

        if mind.pending_interruptions == 0 {
            info!("No interruption is pending; panic response ignored.");
            continue;
        }

        let before_focus = mind.focus;
        let before_stress = mind.stress;

        mind.pending_interruptions -= 1;
        mind.focus = (mind.focus - rules.panic_focus_loss).clamp(0, rules.max_focus);
        mind.stress = (mind.stress + rules.panic_stress_increase).clamp(0, rules.max_stress);
        stats.panic_responses = stats.panic_responses.saturating_add(1);

        info!(
            "Panicked response -> focus: {} -> {}, stress: {} -> {}, pending: {}",
            before_focus, mind.focus, before_stress, mind.stress, mind.pending_interruptions
        );
    }
}

pub fn handle_manager_checkin_requests(
    mut requests: EventReader<ManagerCheckInEvent>,
    rules: Res<OfficeRules>,
    mut clock: ResMut<DayClock>,
    mut stats: ResMut<DayStats>,
    mut mind: ResMut<PlayerMindState>,
    mut career: ResMut<PlayerCareerState>,
) {
    for _ in requests.read() {
        if clock.ended {
            continue;
        }

        let before_stress = mind.stress;
        let before_rep = career.reputation;

        clock.advance(rules.manager_checkin_minutes);
        mind.stress =
            (mind.stress + rules.manager_checkin_stress_increase).clamp(0, rules.max_stress);
        career.reputation = (career.reputation + rules.manager_checkin_reputation_gain)
            .clamp(-rules.max_reputation, rules.max_reputation);
        stats.manager_checkins = stats.manager_checkins.saturating_add(1);

        info!(
            "Manager check-in -> stress: {} -> {}, reputation: {} -> {}, clock: {}",
            before_stress,
            mind.stress,
            before_rep,
            career.reputation,
            format_clock(clock.current_minute)
        );
    }
}

pub fn handle_coworker_help_requests(
    mut requests: EventReader<CoworkerHelpEvent>,
    rules: Res<OfficeRules>,
    mut clock: ResMut<DayClock>,
    mut stats: ResMut<DayStats>,
    mut mind: ResMut<PlayerMindState>,
    mut career: ResMut<PlayerCareerState>,
) {
    for _ in requests.read() {
        if clock.ended {
            continue;
        }

        let before_focus = mind.focus;
        let before_stress = mind.stress;
        let before_rep = career.reputation;

        clock.advance(rules.coworker_help_minutes);
        mind.focus = (mind.focus - rules.coworker_help_focus_cost).clamp(0, rules.max_focus);
        mind.stress = (mind.stress - rules.coworker_help_stress_relief).clamp(0, rules.max_stress);
        career.reputation = (career.reputation + rules.coworker_help_reputation_gain)
            .clamp(-rules.max_reputation, rules.max_reputation);
        stats.coworker_helps = stats.coworker_helps.saturating_add(1);

        info!(
            "Helped coworker -> focus: {} -> {}, stress: {} -> {}, reputation: {} -> {}, clock: {}",
            before_focus,
            mind.focus,
            before_stress,
            mind.stress,
            before_rep,
            career.reputation,
            format_clock(clock.current_minute)
        );
    }
}
