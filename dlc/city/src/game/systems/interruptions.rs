use bevy::prelude::*;

use crate::game::events::{
    CoworkerHelpEvent, InterruptionEvent, ManagerCheckInEvent, PanicResponseEvent,
    ResolveCalmlyEvent,
};
use crate::game::resources::{
    format_clock, ActiveInterruptionContext, DayClock, DayStats, InterruptionKind, OfficeRules,
    OfficeRunConfig, PlayerCareerState, PlayerMindState, SocialGraphState, UnlockCatalogState,
};

#[derive(Clone, Copy)]
struct InterruptionScenarioTemplate {
    stress_modifier: i32,
    focus_modifier: i32,
    manager_affinity_delta: i32,
    manager_trust_delta: i32,
    teammate_affinity_delta: i32,
    teammate_trust_delta: i32,
    description: &'static str,
}

const INTERRUPTION_SCENARIOS: [InterruptionScenarioTemplate; 12] = [
    InterruptionScenarioTemplate {
        stress_modifier: 4,
        focus_modifier: -2,
        manager_affinity_delta: -1,
        manager_trust_delta: -1,
        teammate_affinity_delta: 0,
        teammate_trust_delta: 0,
        description: "The printer is jammed again and everyone is looking at you.",
    },
    InterruptionScenarioTemplate {
        stress_modifier: -2,
        focus_modifier: 3,
        manager_affinity_delta: 1,
        manager_trust_delta: 1,
        teammate_affinity_delta: 0,
        teammate_trust_delta: 0,
        description: "Your manager scheduled a last-minute check-in.",
    },
    InterruptionScenarioTemplate {
        stress_modifier: 1,
        focus_modifier: -3,
        manager_affinity_delta: 0,
        manager_trust_delta: 0,
        teammate_affinity_delta: -1,
        teammate_trust_delta: -1,
        description: "The fire alarm went off — false alarm, but you lost your train of thought.",
    },
    InterruptionScenarioTemplate {
        stress_modifier: -1,
        focus_modifier: 2,
        manager_affinity_delta: 0,
        manager_trust_delta: 0,
        teammate_affinity_delta: 1,
        teammate_trust_delta: 1,
        description: "A client is on hold and nobody else is picking up.",
    },
    InterruptionScenarioTemplate {
        stress_modifier: 3,
        focus_modifier: -1,
        manager_affinity_delta: 0,
        manager_trust_delta: -1,
        teammate_affinity_delta: 1,
        teammate_trust_delta: 0,
        description: "Someone microwaved fish in the break room. The smell is unbearable.",
    },
    InterruptionScenarioTemplate {
        stress_modifier: 0,
        focus_modifier: 1,
        manager_affinity_delta: 1,
        manager_trust_delta: 0,
        teammate_affinity_delta: 0,
        teammate_trust_delta: 1,
        description: "The intern accidentally deleted a shared folder. Chaos ensues.",
    },
    InterruptionScenarioTemplate {
        stress_modifier: 2,
        focus_modifier: -2,
        manager_affinity_delta: -1,
        manager_trust_delta: 0,
        teammate_affinity_delta: 0,
        teammate_trust_delta: -1,
        description: "IT pushed an update and now nothing works. Classic.",
    },
    InterruptionScenarioTemplate {
        stress_modifier: -1,
        focus_modifier: 1,
        manager_affinity_delta: 0,
        manager_trust_delta: 1,
        teammate_affinity_delta: 1,
        teammate_trust_delta: 0,
        description: "Free donuts in the kitchen — but the line is absurdly long.",
    },
    InterruptionScenarioTemplate {
        stress_modifier: 5,
        focus_modifier: -3,
        manager_affinity_delta: -1,
        manager_trust_delta: -1,
        teammate_affinity_delta: -1,
        teammate_trust_delta: 0,
        description: "A VIP visitor arrived unannounced. Everyone scrambles to look busy.",
    },
    InterruptionScenarioTemplate {
        stress_modifier: 1,
        focus_modifier: 0,
        manager_affinity_delta: 1,
        manager_trust_delta: 0,
        teammate_affinity_delta: 0,
        teammate_trust_delta: 1,
        description: "The AC broke down and it's getting uncomfortably warm.",
    },
    InterruptionScenarioTemplate {
        stress_modifier: -2,
        focus_modifier: 2,
        manager_affinity_delta: 0,
        manager_trust_delta: 0,
        teammate_affinity_delta: 1,
        teammate_trust_delta: 1,
        description: "Someone brought a therapy dog into the office. Morale spike!",
    },
    InterruptionScenarioTemplate {
        stress_modifier: 3,
        focus_modifier: -2,
        manager_affinity_delta: 0,
        manager_trust_delta: -1,
        teammate_affinity_delta: -1,
        teammate_trust_delta: -1,
        description: "A heated debate about the thermostat has broken out near your desk.",
    },
];

struct InterruptionKindProfile {
    stress_delta: i32,
    duration_minutes: u32,
    calm_choice: &'static str,
    panic_choice: &'static str,
}

fn interruption_profile(kind: InterruptionKind) -> InterruptionKindProfile {
    match kind {
        InterruptionKind::ManagerRequest => InterruptionKindProfile {
            stress_delta: 2,
            duration_minutes: 10,
            calm_choice: "Give a crisp update and note your next milestone.",
            panic_choice: "Rambling status report and over-promising delivery.",
        },
        InterruptionKind::EmergencyMeeting => InterruptionKindProfile {
            stress_delta: 4,
            duration_minutes: 18,
            calm_choice: "Capture action items and clarify your ownership.",
            panic_choice: "Leave without notes and miss key decisions.",
        },
        InterruptionKind::SystemOutage => InterruptionKindProfile {
            stress_delta: 3,
            duration_minutes: 15,
            calm_choice: "Switch to offline tasks and document blockers.",
            panic_choice: "Refresh endlessly and lose focus.",
        },
        InterruptionKind::CoworkerHelp => InterruptionKindProfile {
            stress_delta: 1,
            duration_minutes: 7,
            calm_choice: "Coach them quickly and set a clear next step.",
            panic_choice: "Take over the task and derail your own queue.",
        },
        InterruptionKind::PrinterJam => InterruptionKindProfile {
            stress_delta: 2,
            duration_minutes: 8,
            calm_choice: "Clear the jam and reroute urgent pages.",
            panic_choice: "Mash buttons until the queue doubles.",
        },
        InterruptionKind::FireDrill => InterruptionKindProfile {
            stress_delta: 5,
            duration_minutes: 20,
            calm_choice: "Follow protocol and regroup fast afterward.",
            panic_choice: "Forget your place and lose all momentum.",
        },
        InterruptionKind::BossVisit => InterruptionKindProfile {
            stress_delta: 3,
            duration_minutes: 12,
            calm_choice: "Share a concise update and one smart ask.",
            panic_choice: "Freeze up and dodge direct questions.",
        },
        InterruptionKind::FreeLunch => InterruptionKindProfile {
            stress_delta: -2,
            duration_minutes: 6,
            calm_choice: "Grab a quick slice and recharge.",
            panic_choice: "Overstay the break and miss deadlines.",
        },
    }
}

pub fn pick_interruption_kind(seed: u64, day: u32, hour: u32) -> InterruptionKind {
    match (seed.wrapping_add(day as u64 * 13).wrapping_add(hour as u64 * 7)) % 8 {
        0 => InterruptionKind::ManagerRequest,
        1 => InterruptionKind::EmergencyMeeting,
        2 => InterruptionKind::SystemOutage,
        3 => InterruptionKind::CoworkerHelp,
        4 => InterruptionKind::PrinterJam,
        5 => InterruptionKind::FireDrill,
        6 => InterruptionKind::BossVisit,
        _ => InterruptionKind::FreeLunch,
    }
}

fn scenario_for_interrupt(seed: u64, day_number: u32, cursor: u32) -> InterruptionScenarioTemplate {
    let index = (seed
        .wrapping_add(day_number as u64 * 41)
        .wrapping_add(cursor as u64 * 19)
        % INTERRUPTION_SCENARIOS.len() as u64) as usize;
    INTERRUPTION_SCENARIOS[index]
}

#[allow(clippy::too_many_arguments)]
pub fn handle_interruption_requests(
    mut requests: EventReader<InterruptionEvent>,
    rules: Res<OfficeRules>,
    run_config: Res<OfficeRunConfig>,
    mut clock: ResMut<DayClock>,
    mut stats: ResMut<DayStats>,
    mut mind: ResMut<PlayerMindState>,
    mut social: ResMut<SocialGraphState>,
    mut context: ResMut<ActiveInterruptionContext>,
) {
    for event in requests.read() {
        if clock.ended {
            continue;
        }

        let kind = event.kind;
        let scenario =
            scenario_for_interrupt(run_config.seed, clock.day_number, social.scenario_cursor);
        let kind_profile = interruption_profile(kind);
        social.scenario_cursor = social.scenario_cursor.wrapping_add(1);

        clock.advance(kind_profile.duration_minutes);
        mind.stress = (mind.stress
            + rules.interruption_stress_increase
            + scenario.stress_modifier
            + kind_profile.stress_delta)
            .clamp(0, rules.max_stress);
        mind.focus = (mind.focus - rules.interruption_focus_loss + scenario.focus_modifier)
            .clamp(0, rules.max_focus);
        mind.pending_interruptions = mind.pending_interruptions.saturating_add(1);
        stats.interruptions_triggered = stats.interruptions_triggered.saturating_add(1);

        if let Some(manager) = social.manager_mut() {
            manager.affinity = (manager.affinity + scenario.manager_affinity_delta).clamp(-100, 100);
            manager.trust = (manager.trust + scenario.manager_trust_delta).clamp(-100, 100);
        }
        let teammate_id = social.teammate_for_help(
            run_config.seed,
            clock.day_number,
            social
                .scenario_cursor
                .saturating_add(stats.interruptions_triggered),
        );

        // Build the context description based on kind
        let manager_name = social
            .profiles
            .iter()
            .find(|p| p.role == crate::game::resources::CoworkerRole::Manager)
            .map(|p| p.codename.clone())
            .unwrap_or_else(|| "Your manager".to_string());
        let teammate_name = teammate_id
            .and_then(|id| social.profiles.iter().find(|p| p.id == id))
            .map(|p| p.codename.clone());

        let description = match kind {
            InterruptionKind::ManagerRequest => {
                format!("{manager_name} stops by your desk. 'Got a minute? I need an update on the Henderson file.'")
            }
            InterruptionKind::EmergencyMeeting => {
                "All hands! Emergency meeting in Conference Room B — the client changed the deadline.".to_string()
            }
            InterruptionKind::SystemOutage => {
                "The network just went down. IT says 15 minutes, but who knows...".to_string()
            }
            InterruptionKind::CoworkerHelp => {
                let name = teammate_name.clone().unwrap_or_else(|| "A coworker".to_string());
                format!("{name} leans over: 'Hey, can you help me figure out this spreadsheet formula?'")
            }
            InterruptionKind::PrinterJam => "The printer is jammed again.".to_string(),
            InterruptionKind::FireDrill => "Fire drill! Everyone outside.".to_string(),
            InterruptionKind::BossVisit => "The boss stopped by your desk.".to_string(),
            InterruptionKind::FreeLunch => "Free pizza in the break room!".to_string(),
        };
        let scenario_desc = scenario.description;
        let full_description = format!(
            "{description}\n\n{scenario_desc}\n\nStay Calm: {}\nPanic: {}",
            kind_profile.calm_choice, kind_profile.panic_choice
        );

        context.kind = Some(kind);
        context.coworker_name = teammate_name.clone();
        context.description = full_description;

        if let Some(teammate_id) = teammate_id {
            if let Some(teammate) = social.teammate_mut_by_id(teammate_id) {
                teammate.affinity = (teammate.affinity + scenario.teammate_affinity_delta).clamp(-100, 100);
                teammate.trust = (teammate.trust + scenario.teammate_trust_delta).clamp(-100, 100);
            }
        }
        social.normalize();

        info!(
            "Interruption ({:?})! stress: {}, focus: {}, pending: {}, scenario_cursor: {}, clock: {}",
            kind,
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
    mut context: ResMut<ActiveInterruptionContext>,
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

        if mind.pending_interruptions == 0 {
            *context = ActiveInterruptionContext::default();
        }

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
    mut context: ResMut<ActiveInterruptionContext>,
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

        if mind.pending_interruptions == 0 {
            *context = ActiveInterruptionContext::default();
        }

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
            manager.affinity = (manager.affinity + 1).clamp(-100, 100);
            manager.trust = (manager.trust + if mind.stress > rules.max_stress / 2 {
                -1
            } else {
                2
            }).clamp(-100, 100);
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

pub fn auto_trigger_interruptions(
    clock: Res<DayClock>,
    config: Res<OfficeRunConfig>,
    mut interruption_writer: EventWriter<InterruptionEvent>,
) {
    let current_minute = clock.current_minute;
    let hour_mark = current_minute / 60;
    let seed = config
        .seed
        .wrapping_add(clock.day_number as u64 * 100 + hour_mark as u64);
    let chance = ((seed % 100) as f32) / 100.0;
    let prev_minute = current_minute.saturating_sub(1);
    let prev_hour = prev_minute / 60;
    if hour_mark != prev_hour && chance < config.interruption_chance_per_hour {
        let kind = pick_interruption_kind(config.seed, clock.day_number, hour_mark);
        interruption_writer.send(InterruptionEvent { kind });
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
                teammate.affinity = (teammate.affinity + 2).clamp(-100, 100);
                teammate.trust = (teammate.trust + 1).clamp(-100, 100);
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
