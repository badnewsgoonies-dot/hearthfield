use bevy::prelude::*;

use crate::game::events::{
    CoworkerHelpEvent, InterruptionEvent, ManagerCheckInEvent, PanicResponseEvent,
    ResolveCalmlyEvent,
};
use crate::game::resources::{
    format_clock, DayClock, DayStats, OfficeRules, OfficeRunConfig, PlayerCareerState,
    PlayerMindState, SocialGraphState, UnlockCatalogState,
};

#[derive(Clone, Copy)]
struct InterruptionScenarioTemplate {
    stress_modifier: i32,
    focus_modifier: i32,
    manager_affinity_delta: i32,
    manager_trust_delta: i32,
    teammate_affinity_delta: i32,
    teammate_trust_delta: i32,
}

const INTERRUPTION_SCENARIOS: [InterruptionScenarioTemplate; 6] = [
    InterruptionScenarioTemplate {
        stress_modifier: 4,
        focus_modifier: -2,
        manager_affinity_delta: -1,
        manager_trust_delta: -1,
        teammate_affinity_delta: 0,
        teammate_trust_delta: 0,
    },
    InterruptionScenarioTemplate {
        stress_modifier: -2,
        focus_modifier: 3,
        manager_affinity_delta: 1,
        manager_trust_delta: 1,
        teammate_affinity_delta: 0,
        teammate_trust_delta: 0,
    },
    InterruptionScenarioTemplate {
        stress_modifier: 1,
        focus_modifier: -3,
        manager_affinity_delta: 0,
        manager_trust_delta: 0,
        teammate_affinity_delta: -1,
        teammate_trust_delta: -1,
    },
    InterruptionScenarioTemplate {
        stress_modifier: -1,
        focus_modifier: 2,
        manager_affinity_delta: 0,
        manager_trust_delta: 0,
        teammate_affinity_delta: 1,
        teammate_trust_delta: 1,
    },
    InterruptionScenarioTemplate {
        stress_modifier: 3,
        focus_modifier: -1,
        manager_affinity_delta: 0,
        manager_trust_delta: -1,
        teammate_affinity_delta: 1,
        teammate_trust_delta: 0,
    },
    InterruptionScenarioTemplate {
        stress_modifier: 0,
        focus_modifier: 1,
        manager_affinity_delta: 1,
        manager_trust_delta: 0,
        teammate_affinity_delta: 0,
        teammate_trust_delta: 1,
    },
];

fn scenario_for_interrupt(seed: u64, day_number: u32, cursor: u32) -> InterruptionScenarioTemplate {
    let index = (seed
        .wrapping_add(day_number as u64 * 41)
        .wrapping_add(cursor as u64 * 19)
        % INTERRUPTION_SCENARIOS.len() as u64) as usize;
    INTERRUPTION_SCENARIOS[index]
}

pub fn handle_interruption_requests(
    mut requests: EventReader<InterruptionEvent>,
    rules: Res<OfficeRules>,
    run_config: Res<OfficeRunConfig>,
    mut clock: ResMut<DayClock>,
    mut stats: ResMut<DayStats>,
    mut mind: ResMut<PlayerMindState>,
    mut social: ResMut<SocialGraphState>,
) {
    for _ in requests.read() {
        if clock.ended {
            continue;
        }

        let scenario =
            scenario_for_interrupt(run_config.seed, clock.day_number, social.scenario_cursor);
        social.scenario_cursor = social.scenario_cursor.saturating_add(1);

        clock.advance(rules.interruption_minutes);
        mind.stress = (mind.stress + rules.interruption_stress_increase + scenario.stress_modifier)
            .clamp(0, rules.max_stress);
        mind.focus = (mind.focus - rules.interruption_focus_loss + scenario.focus_modifier)
            .clamp(0, rules.max_focus);
        mind.pending_interruptions = mind.pending_interruptions.saturating_add(1);
        stats.interruptions_triggered = stats.interruptions_triggered.saturating_add(1);

        if let Some(manager) = social.manager_mut() {
            manager.affinity += scenario.manager_affinity_delta;
            manager.trust += scenario.manager_trust_delta;
        }
        let teammate_id = social.teammate_for_help(
            run_config.seed,
            clock.day_number,
            social
                .scenario_cursor
                .saturating_add(stats.interruptions_triggered),
        );
        if let Some(teammate_id) = teammate_id {
            if let Some(teammate) = social.teammate_mut_by_id(teammate_id) {
                teammate.affinity += scenario.teammate_affinity_delta;
                teammate.trust += scenario.teammate_trust_delta;
            }
        }
        social.normalize();

        info!(
            "Interruption! stress: {}, focus: {}, pending: {}, scenario_cursor: {}, clock: {}",
            mind.stress,
            mind.focus,
            mind.pending_interruptions,
            social.scenario_cursor,
            format_clock(clock.current_minute)
        );
    }
}

pub fn handle_resolve_calmly_requests(
    mut requests: EventReader<ResolveCalmlyEvent>,
    rules: Res<OfficeRules>,
    unlocks: Res<UnlockCatalogState>,
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
        mind.focus = (mind.focus + rules.calm_focus_restore + unlocks.calm_focus_bonus())
            .clamp(0, rules.max_focus);
        mind.stress = (mind.stress - rules.calm_stress_relief - unlocks.calm_stress_relief_bonus())
            .clamp(0, rules.max_stress);
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
    mut social: ResMut<SocialGraphState>,
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
        if let Some(manager) = social.manager_mut() {
            manager.affinity += 1;
            manager.trust += if mind.stress > rules.max_stress / 2 {
                -1
            } else {
                2
            };
        }
        social.normalize();

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

#[allow(clippy::too_many_arguments)]
pub fn handle_coworker_help_requests(
    mut requests: EventReader<CoworkerHelpEvent>,
    rules: Res<OfficeRules>,
    run_config: Res<OfficeRunConfig>,
    mut clock: ResMut<DayClock>,
    mut stats: ResMut<DayStats>,
    mut mind: ResMut<PlayerMindState>,
    mut career: ResMut<PlayerCareerState>,
    mut social: ResMut<SocialGraphState>,
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
        let teammate_id =
            social.teammate_for_help(run_config.seed, clock.day_number, stats.coworker_helps);
        if let Some(teammate_id) = teammate_id {
            if let Some(teammate) = social.teammate_mut_by_id(teammate_id) {
                teammate.affinity += 2;
                teammate.trust += 1;
            }
        }
        social.normalize();

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
