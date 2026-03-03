//! Player journal/diary — auto-entries, custom notes, milestones.

use bevy::prelude::*;
use crate::shared::*;

// ═══════════════════════════════════════════════════════════════════════════
// TYPES
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum JournalCategory {
    Flight,
    Airport,
    Crew,
    Milestone,
    Emergency,
    Personal,
    Achievement,
}

impl JournalCategory {
    pub fn display_name(&self) -> &'static str {
        match self {
            JournalCategory::Flight => "✈ Flight",
            JournalCategory::Airport => "🏢 Airport",
            JournalCategory::Crew => "👥 Crew",
            JournalCategory::Milestone => "⭐ Milestone",
            JournalCategory::Emergency => "🚨 Emergency",
            JournalCategory::Personal => "📝 Personal",
            JournalCategory::Achievement => "🏆 Achievement",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            JournalCategory::Flight => "✈",
            JournalCategory::Airport => "🏢",
            JournalCategory::Crew => "👥",
            JournalCategory::Milestone => "⭐",
            JournalCategory::Emergency => "🚨",
            JournalCategory::Personal => "📝",
            JournalCategory::Achievement => "🏆",
        }
    }
}

#[derive(Clone, Debug)]
pub struct JournalEntry {
    pub day: u32,
    pub season: Season,
    pub year: u32,
    pub title: String,
    pub body: String,
    pub category: JournalCategory,
}

impl JournalEntry {
    pub fn new(
        calendar: &Calendar,
        title: &str,
        body: &str,
        category: JournalCategory,
    ) -> Self {
        Self {
            day: calendar.day,
            season: calendar.season,
            year: calendar.year,
            title: title.to_string(),
            body: body.to_string(),
            category,
        }
    }

    pub fn formatted_date(&self) -> String {
        format!("{} Day {}, Year {}", self.season, self.day, self.year)
    }
}

/// Player journal resource.
#[derive(Resource, Clone, Debug, Default)]
pub struct Journal {
    pub entries: Vec<JournalEntry>,
    pub airports_first_visited: Vec<AirportId>,
    pub milestones_recorded: Vec<String>,
    pub custom_entry_count: u32,
}

impl Journal {
    pub fn add_entry(&mut self, entry: JournalEntry) {
        self.entries.push(entry);
    }

    pub fn entries_by_category(&self, category: JournalCategory) -> Vec<&JournalEntry> {
        self.entries
            .iter()
            .filter(|e| e.category == category)
            .collect()
    }

    pub fn recent_entries(&self, count: usize) -> Vec<&JournalEntry> {
        self.entries.iter().rev().take(count).collect()
    }

    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }

    pub fn has_milestone(&self, id: &str) -> bool {
        self.milestones_recorded.iter().any(|m| m == id)
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// SYSTEMS
// ═══════════════════════════════════════════════════════════════════════════

/// Record journal entry for first visit to each airport.
pub fn record_first_airport_visit(
    mut arrival_events: EventReader<AirportArrivalEvent>,
    calendar: Res<Calendar>,
    mut journal: ResMut<Journal>,
    mut toast_events: EventWriter<ToastEvent>,
) {
    for ev in arrival_events.read() {
        if journal.airports_first_visited.contains(&ev.airport) {
            continue;
        }

        journal.airports_first_visited.push(ev.airport);

        let entry = JournalEntry::new(
            &calendar,
            &format!("First Visit: {}", ev.airport.display_name()),
            &format!(
                "Touched down at {} for the first time. ICAO: {}.",
                ev.airport.display_name(),
                ev.airport.icao_code()
            ),
            JournalCategory::Airport,
        );
        journal.add_entry(entry);

        toast_events.send(ToastEvent {
            message: format!("📓 Journal: First visit to {}!", ev.airport.display_name()),
            duration_secs: 3.0,
        });
    }
}

/// Record journal entry for first emergency.
pub fn record_first_emergency(
    mut emergency_events: EventReader<EmergencyEvent>,
    calendar: Res<Calendar>,
    mut journal: ResMut<Journal>,
    mut toast_events: EventWriter<ToastEvent>,
) {
    for ev in emergency_events.read() {
        let milestone_id = format!("emergency_{:?}", ev.kind);
        if journal.has_milestone(&milestone_id) {
            continue;
        }

        journal.milestones_recorded.push(milestone_id);

        let entry = JournalEntry::new(
            &calendar,
            &format!("Emergency: {:?}", ev.kind),
            &format!(
                "Experienced a {:?} during flight. Handled it the best I could.",
                ev.kind
            ),
            JournalCategory::Emergency,
        );
        journal.add_entry(entry);

        toast_events.send(ToastEvent {
            message: "📓 Journal: Emergency recorded.".to_string(),
            duration_secs: 2.0,
        });
    }
}

/// Record journal entry on rank promotion.
pub fn record_rank_promotion(
    mut rank_events: EventReader<RankUpEvent>,
    calendar: Res<Calendar>,
    mut journal: ResMut<Journal>,
    mut toast_events: EventWriter<ToastEvent>,
) {
    for ev in rank_events.read() {
        let entry = JournalEntry::new(
            &calendar,
            &format!("Promoted to {}", ev.new_rank.display_name()),
            &format!(
                "Hard work paid off. I've been promoted to {}. New opportunities await.",
                ev.new_rank.display_name()
            ),
            JournalCategory::Milestone,
        );
        journal.add_entry(entry);

        toast_events.send(ToastEvent {
            message: format!("📓 Journal: Promotion to {} recorded!", ev.new_rank.display_name()),
            duration_secs: 3.0,
        });
    }
}

/// Record journal entry on achievement unlock.
pub fn record_achievement(
    mut achievement_events: EventReader<AchievementUnlockedEvent>,
    calendar: Res<Calendar>,
    mut journal: ResMut<Journal>,
) {
    for ev in achievement_events.read() {
        let entry = JournalEntry::new(
            &calendar,
            &format!("Achievement: {}", ev.achievement_id),
            &format!("Unlocked the '{}' achievement today!", ev.achievement_id),
            JournalCategory::Achievement,
        );
        journal.add_entry(entry);
    }
}

/// Check for time-based milestones.
pub fn check_journal_milestones(
    mut day_end_events: EventReader<DayEndEvent>,
    calendar: Res<Calendar>,
    play_stats: Res<PlayStats>,
    mut journal: ResMut<Journal>,
    mut toast_events: EventWriter<ToastEvent>,
) {
    for _ev in day_end_events.read() {
        let day = calendar.total_days();

        // Day 100 milestone
        if day == 100 && !journal.has_milestone("day_100") {
            journal.milestones_recorded.push("day_100".to_string());
            let entry = JournalEntry::new(
                &calendar,
                "Day 100 in the Sky",
                "100 days since I started this journey. The sky feels like home now.",
                JournalCategory::Milestone,
            );
            journal.add_entry(entry);
            toast_events.send(ToastEvent {
                message: "📓 Milestone: 100 days as a pilot!".to_string(),
                duration_secs: 4.0,
            });
        }

        // Halfway through airports
        let total_airports = 10;
        let visited = journal.airports_first_visited.len();
        if visited >= total_airports / 2
            && !journal.has_milestone("half_airports")
        {
            journal
                .milestones_recorded
                .push("half_airports".to_string());
            let entry = JournalEntry::new(
                &calendar,
                "Halfway Through All Airports",
                &format!(
                    "Visited {}/{} airports. The world is getting smaller.",
                    visited, total_airports
                ),
                JournalCategory::Milestone,
            );
            journal.add_entry(entry);
            toast_events.send(ToastEvent {
                message: "📓 Milestone: Halfway through all airports!".to_string(),
                duration_secs: 4.0,
            });
        }

        // All airports visited
        if visited >= total_airports && !journal.has_milestone("all_airports") {
            journal
                .milestones_recorded
                .push("all_airports".to_string());
            let entry = JournalEntry::new(
                &calendar,
                "Globe Trotter",
                "I've been to every airport on the map. What a journey.",
                JournalCategory::Milestone,
            );
            journal.add_entry(entry);
        }

        // 50 flights milestone
        if play_stats.total_flights >= 50 && !journal.has_milestone("50_flights") {
            journal
                .milestones_recorded
                .push("50_flights".to_string());
            let entry = JournalEntry::new(
                &calendar,
                "50 Flights Completed",
                "Half a century of flights under my belt. Every one taught me something.",
                JournalCategory::Milestone,
            );
            journal.add_entry(entry);
        }
    }
}

/// Write a custom journal entry (triggered via apartment bookshelf).
pub fn write_custom_entry(
    input: Res<PlayerInput>,
    calendar: Res<Calendar>,
    player_location: Res<PlayerLocation>,
    mut journal: ResMut<Journal>,
    mut toast_events: EventWriter<ToastEvent>,
) {
    // Custom entries written when interacting with bookshelf in apartment
    if player_location.zone != MapZone::CrewQuarters {
        return;
    }
    if !input.interact {
        return;
    }

    // In a full implementation, this would open a text input UI.
    // For now, auto-generate a reflection entry.
    journal.custom_entry_count += 1;
    let entry = JournalEntry::new(
        &calendar,
        &format!("Personal Reflection #{}", journal.custom_entry_count),
        "Another day in the life of a pilot. The view from up here never gets old.",
        JournalCategory::Personal,
    );
    journal.add_entry(entry);

    toast_events.send(ToastEvent {
        message: "📓 Journal entry written.".to_string(),
        duration_secs: 2.0,
    });
}
