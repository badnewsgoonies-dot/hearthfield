//! Crew random events — birthdays, conflicts, requests, achievements, departures.

use crate::shared::*;
use bevy::prelude::*;

// ═══════════════════════════════════════════════════════════════════════════
// TYPES
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CrewEventKind {
    Birthday,
    Conflict,
    PersonalRequest,
    Achievement,
    HolidayParty,
    Departure,
    NewArrival,
    Mentorship,
}

impl CrewEventKind {
    pub fn display_name(&self) -> &'static str {
        match self {
            CrewEventKind::Birthday => "Birthday",
            CrewEventKind::Conflict => "Crew Conflict",
            CrewEventKind::PersonalRequest => "Personal Request",
            CrewEventKind::Achievement => "Crew Achievement",
            CrewEventKind::HolidayParty => "Holiday Celebration",
            CrewEventKind::Departure => "Crew Departure",
            CrewEventKind::NewArrival => "New Crew Member",
            CrewEventKind::Mentorship => "Mentorship Opportunity",
        }
    }
}

#[derive(Clone, Debug)]
pub struct CrewEvent {
    pub kind: CrewEventKind,
    pub npc_id: String,
    pub title: String,
    pub description: String,
    pub friendship_change: i32,
    pub gold_cost: Option<u32>,
    pub resolved: bool,
}

// Birthday data: (npc_id, birthday_day_of_season)
const CREW_BIRTHDAYS: &[(&str, u32)] = &[
    ("captain_elena", 5),
    ("copilot_marco", 12),
    ("navigator_yuki", 19),
    ("mechanic_hank", 3),
    ("attendant_sofia", 22),
    ("controller_raj", 8),
    ("instructor_chen", 15),
    ("veteran_pete", 25),
    ("rookie_alex", 10),
    ("charter_diana", 17),
];

// Achievement milestones
const CREW_ACHIEVEMENTS: &[(&str, &str, &str)] = &[
    (
        "copilot_marco",
        "Marco earned his captain's license!",
        "After years of dedication, Marco passed his captain's exam. He credits your mentorship.",
    ),
    (
        "rookie_alex",
        "Alex overcame their flight anxiety!",
        "Alex flew a solo cross-country without a single moment of panic. They're beaming with pride.",
    ),
    (
        "navigator_yuki",
        "Yuki published a navigation paper!",
        "Yuki's research on optimal mountain approach paths was published in Aviation Weekly.",
    ),
    (
        "mechanic_hank",
        "Hank restored a vintage DC-3!",
        "Hank spent months rebuilding a classic DC-3 in the hangar. It flies like new.",
    ),
    (
        "attendant_sofia",
        "Sofia won Best Attendant award!",
        "Sofia received the annual Best Flight Attendant award from the Aviation Association.",
    ),
    (
        "charter_diana",
        "Diana completed the Desert Rally!",
        "Diana flew the grueling 72-hour Desert Rally and placed in the top 3.",
    ),
];

// Holiday/celebration events
const HOLIDAYS: &[(u32, &str, &str)] = &[
    (
        1,
        "New Year's Party",
        "The crew gathers in the lounge for a New Year's celebration.",
    ),
    (
        7,
        "Spring Festival",
        "Spring Festival! The crew decorates the lounge with flowers.",
    ),
    (
        14,
        "Midsummer BBQ",
        "Hank fires up the grill on the tarmac. The whole crew is invited.",
    ),
    (
        21,
        "Harvest Feast",
        "The crew shares a harvest feast in the crew quarters.",
    ),
    (
        25,
        "Winter Holiday",
        "Holiday decorations fill the lounge. Everyone exchanges small gifts.",
    ),
];

// ── State ────────────────────────────────────────────────────────────────

#[derive(Resource, Clone, Debug, Default)]
pub struct CrewEventState {
    pub pending_events: Vec<CrewEvent>,
    pub resolved_events: Vec<CrewEvent>,
    pub birthdays_celebrated: Vec<String>,
    pub achievements_triggered: Vec<String>,
    pub departed_crew: Vec<String>,
    pub new_crew: Vec<String>,
    pub event_check_timer: f32,
}

// ═══════════════════════════════════════════════════════════════════════════
// SYSTEMS
// ═══════════════════════════════════════════════════════════════════════════

/// Check for birthday events on day end.
pub fn check_birthdays(
    mut day_end_events: EventReader<DayEndEvent>,
    calendar: Res<Calendar>,
    mut event_state: ResMut<CrewEventState>,
    crew_registry: Res<CrewRegistry>,
    mut toast_events: EventWriter<ToastEvent>,
) {
    for _ev in day_end_events.read() {
        for &(npc_id, birthday_day) in CREW_BIRTHDAYS {
            if calendar.day != birthday_day {
                continue;
            }
            if event_state.birthdays_celebrated.contains(&format!(
                "{}_{}_{}",
                npc_id, calendar.season as u32, calendar.year
            )) {
                continue;
            }

            let name = crew_registry
                .members
                .get(npc_id)
                .map(|m| m.name.clone())
                .unwrap_or_else(|| npc_id.to_string());

            let event = CrewEvent {
                kind: CrewEventKind::Birthday,
                npc_id: npc_id.to_string(),
                title: format!("{}'s Birthday!", name),
                description: format!(
                    "It's {}'s birthday! Give them a gift for a big friendship boost.",
                    name
                ),
                friendship_change: 15,
                gold_cost: None,
                resolved: false,
            };

            event_state.pending_events.push(event);

            toast_events.send(ToastEvent {
                message: format!("🎂 It's {}'s birthday today!", name),
                duration_secs: 4.0,
            });
        }
    }
}

/// Handle gift giving on birthdays for bonus friendship.
pub fn birthday_gift_bonus(
    mut gift_events: EventReader<GiftGivenEvent>,
    mut event_state: ResMut<CrewEventState>,
    mut relationships: ResMut<Relationships>,
    calendar: Res<Calendar>,
    mut toast_events: EventWriter<ToastEvent>,
) {
    for ev in gift_events.read() {
        let _birthday_key = format!("{}_{}", ev.npc_id, calendar.day);
        let is_birthday = CREW_BIRTHDAYS
            .iter()
            .any(|&(id, day)| id == ev.npc_id && day == calendar.day);

        if is_birthday {
            relationships.add_friendship(&ev.npc_id, 15);
            let season_key = format!("{}_{}_{}", ev.npc_id, calendar.season as u32, calendar.year);
            event_state.birthdays_celebrated.push(season_key);

            // Resolve pending birthday event
            for pending in &mut event_state.pending_events {
                if pending.npc_id == ev.npc_id && pending.kind == CrewEventKind::Birthday {
                    pending.resolved = true;
                }
            }

            toast_events.send(ToastEvent {
                message: format!(
                    "🎁 Birthday gift! {} is thrilled! (+15 friendship)",
                    ev.npc_id
                ),
                duration_secs: 3.0,
            });
        }
    }
}

/// Trigger crew conflicts between rival pairs occasionally.
pub fn check_crew_conflicts(
    time: Res<Time>,
    mut event_state: ResMut<CrewEventState>,
    _relationships: Res<Relationships>,
    mut toast_events: EventWriter<ToastEvent>,
) {
    event_state.event_check_timer += time.delta_secs();
    if event_state.event_check_timer < 600.0 {
        return;
    }
    event_state.event_check_timer = 0.0;

    let t = time.elapsed_secs();
    let roll = ((t * 3.7).sin().abs() * 100.0) as u32;

    if roll > 15 {
        return;
    }

    // Pick two crew members at semi-random
    let pairs = [
        ("copilot_marco", "rookie_alex"),
        ("captain_elena", "veteran_pete"),
        ("mechanic_hank", "instructor_chen"),
    ];
    let pair_idx = (t as usize) % pairs.len();
    let (crew_a, crew_b) = pairs[pair_idx];

    let event = CrewEvent {
        kind: CrewEventKind::Conflict,
        npc_id: crew_a.to_string(),
        title: format!("Argument between {} and {}", crew_a, crew_b),
        description: format!(
            "{} and {} are arguing in the lounge. Step in to mediate?",
            crew_a, crew_b
        ),
        friendship_change: 0,
        gold_cost: None,
        resolved: false,
    };

    toast_events.send(ToastEvent {
        message: format!("⚡ {} and {} are having an argument!", crew_a, crew_b),
        duration_secs: 4.0,
    });

    event_state.pending_events.push(event);
}

/// Trigger crew achievements based on time/friendship.
pub fn check_crew_achievements(
    mut day_end_events: EventReader<DayEndEvent>,
    calendar: Res<Calendar>,
    relationships: Res<Relationships>,
    mut event_state: ResMut<CrewEventState>,
    mut toast_events: EventWriter<ToastEvent>,
) {
    for _ev in day_end_events.read() {
        for &(npc_id, title, desc) in CREW_ACHIEVEMENTS {
            if event_state
                .achievements_triggered
                .contains(&npc_id.to_string())
            {
                continue;
            }

            // Require high friendship and enough time passed
            let friendship = relationships.friendship_level(npc_id);
            let days_played = calendar.total_days();

            if friendship >= 60 && days_played >= 50 {
                event_state.achievements_triggered.push(npc_id.to_string());

                toast_events.send(ToastEvent {
                    message: format!("🌟 {}", title),
                    duration_secs: 5.0,
                });

                event_state.pending_events.push(CrewEvent {
                    kind: CrewEventKind::Achievement,
                    npc_id: npc_id.to_string(),
                    title: title.to_string(),
                    description: desc.to_string(),
                    friendship_change: 10,
                    gold_cost: None,
                    resolved: false,
                });
            }
        }
    }
}

/// Crew departure if relationship is too low.
pub fn check_crew_departure(
    mut day_end_events: EventReader<DayEndEvent>,
    relationships: Res<Relationships>,
    mut event_state: ResMut<CrewEventState>,
    mut toast_events: EventWriter<ToastEvent>,
) {
    for _ev in day_end_events.read() {
        for npc_id in CREW_IDS {
            if event_state.departed_crew.contains(&npc_id.to_string()) {
                continue;
            }

            let friendship = relationships.friendship_level(npc_id);
            if friendship <= -50 {
                event_state.departed_crew.push(npc_id.to_string());

                toast_events.send(ToastEvent {
                    message: format!(
                        "📤 {} has transferred away. Relationship was too strained.",
                        npc_id
                    ),
                    duration_secs: 5.0,
                });

                event_state.pending_events.push(CrewEvent {
                    kind: CrewEventKind::Departure,
                    npc_id: npc_id.to_string(),
                    title: format!("{} Leaves", npc_id),
                    description: format!(
                        "{} has had enough and transferred to another airport.",
                        npc_id
                    ),
                    friendship_change: 0,
                    gold_cost: None,
                    resolved: true,
                });
            }
        }
    }
}

/// Holiday celebrations in the lounge.
pub fn check_holiday_events(
    mut day_end_events: EventReader<DayEndEvent>,
    calendar: Res<Calendar>,
    mut event_state: ResMut<CrewEventState>,
    mut toast_events: EventWriter<ToastEvent>,
) {
    for _ev in day_end_events.read() {
        for &(day, title, desc) in HOLIDAYS {
            if calendar.day != day {
                continue;
            }

            toast_events.send(ToastEvent {
                message: format!("🎉 {} — {}", title, desc),
                duration_secs: 5.0,
            });

            event_state.pending_events.push(CrewEvent {
                kind: CrewEventKind::HolidayParty,
                npc_id: String::new(),
                title: title.to_string(),
                description: desc.to_string(),
                friendship_change: 3, // Small boost to all crew
                gold_cost: None,
                resolved: false,
            });
        }
    }
}

/// Mentorship system: high-friendship crew teach skills faster.
pub fn check_mentorship(
    mut day_end_events: EventReader<DayEndEvent>,
    relationships: Res<Relationships>,
    _event_state: ResMut<CrewEventState>,
    mut xp_events: EventWriter<XpGainEvent>,
    _toast_events: EventWriter<ToastEvent>,
) {
    for _ev in day_end_events.read() {
        // Instructor Chen mentorship at high friendship
        let chen_friendship = relationships.friendship_level("instructor_chen");
        if chen_friendship >= 55 {
            xp_events.send(XpGainEvent {
                amount: 5,
                source: "Instructor Chen mentorship".to_string(),
            });
        }

        // Elena mentorship
        let elena_friendship = relationships.friendship_level("captain_elena");
        if elena_friendship >= 55 {
            xp_events.send(XpGainEvent {
                amount: 3,
                source: "Captain Elena guidance".to_string(),
            });
        }
    }
}

/// Resolve pending events that auto-resolve at day end.
pub fn cleanup_resolved_events(mut event_state: ResMut<CrewEventState>) {
    let resolved: Vec<CrewEvent> = event_state
        .pending_events
        .iter()
        .filter(|e| e.resolved)
        .cloned()
        .collect();

    event_state.pending_events.retain(|e| !e.resolved);
    event_state.resolved_events.extend(resolved);
}
