//! Logbook screen — flight history viewer with stats and filters.

use crate::shared::*;
use bevy::prelude::*;

// ─── Components ──────────────────────────────────────────────────────────

#[derive(Component)]
pub struct LogbookScreenRoot;

#[derive(Component)]
pub struct LogbookEntry;

#[derive(Component)]
pub struct LogbookStatsPanel;

#[derive(Component)]
pub struct LogbookFilterLabel;

/// Current logbook filter state.
#[derive(Resource, Default, Clone, Debug)]
pub struct LogbookFilter {
    pub airport_filter: Option<AirportId>,
    pub aircraft_class_filter: Option<AircraftClass>,
    pub page: usize,
}

const ENTRIES_PER_PAGE: usize = 10;

// ─── Spawn / Despawn ─────────────────────────────────────────────────────

pub fn spawn_logbook_screen(
    mut commands: Commands,
    font: Res<UiFontHandle>,
    mission_log: Res<MissionLog>,
    play_stats: Res<PlayStats>,
    achievements: Res<Achievements>,
    filter: Res<LogbookFilter>,
) {
    let root = commands
        .spawn((
            LogbookScreenRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(20.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.05, 0.05, 0.12, 0.95)),
            GlobalZIndex(40),
        ))
        .id();

    // Title
    let title = commands
        .spawn((
            Text::new("PILOT LOGBOOK"),
            TextFont {
                font: font.0.clone(),
                font_size: 28.0,
                ..default()
            },
            TextColor(Color::srgb(1.0, 0.9, 0.5)),
            Node {
                margin: UiRect::bottom(Val::Px(16.0)),
                ..default()
            },
        ))
        .id();
    commands.entity(root).add_child(title);

    // ── Stats summary ────────────────────────────────────────────
    let stats_text = format!(
        "Total Flights: {} | Hours: {:.1} | Distance: {:.0} nm | Perfect Landings: {} | Achievements: {}/{}",
        play_stats.total_flights,
        play_stats.total_flight_hours,
        play_stats.total_distance_nm,
        play_stats.perfect_landings,
        achievements.unlocked.len(),
        crate::data::achievements::ALL_ACHIEVEMENTS.len(),
    );
    let stats = commands
        .spawn((
            LogbookStatsPanel,
            Text::new(stats_text),
            TextFont {
                font: font.0.clone(),
                font_size: 14.0,
                ..default()
            },
            TextColor(Color::srgb(0.7, 0.85, 1.0)),
            Node {
                margin: UiRect::bottom(Val::Px(12.0)),
                ..default()
            },
        ))
        .id();
    commands.entity(root).add_child(stats);

    // ── Filter label ─────────────────────────────────────────────
    let filter_desc = build_filter_label(&filter);
    let filter_label = commands
        .spawn((
            LogbookFilterLabel,
            Text::new(filter_desc),
            TextFont {
                font: font.0.clone(),
                font_size: 13.0,
                ..default()
            },
            TextColor(Color::srgb(0.6, 0.6, 0.6)),
            Node {
                margin: UiRect::bottom(Val::Px(10.0)),
                ..default()
            },
        ))
        .id();
    commands.entity(root).add_child(filter_label);

    // ── Flight entries ───────────────────────────────────────────
    let start = filter.page * ENTRIES_PER_PAGE;
    let entries: Vec<&CompletedMission> = mission_log
        .completed
        .iter()
        .rev()
        .skip(start)
        .take(ENTRIES_PER_PAGE)
        .collect();

    if entries.is_empty() {
        let empty = commands
            .spawn((
                Text::new("No flights recorded yet. Get out there and fly!"),
                TextFont {
                    font: font.0.clone(),
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(0.5, 0.5, 0.5)),
            ))
            .id();
        commands.entity(root).add_child(empty);
    } else {
        for entry in &entries {
            let line = format!(
                "Day {} | {} | Grade: {} | +{} XP | +{} gold | {:.0} min",
                entry.day_completed,
                entry.mission_id,
                entry.landing_grade,
                entry.xp_earned,
                entry.gold_earned,
                entry.flight_time_minutes,
            );
            let entry_id = commands
                .spawn((
                    LogbookEntry,
                    Text::new(line),
                    TextFont {
                        font: font.0.clone(),
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.85, 0.85, 0.85)),
                    Node {
                        margin: UiRect::bottom(Val::Px(4.0)),
                        ..default()
                    },
                ))
                .id();
            commands.entity(root).add_child(entry_id);
        }
    }

    // Page indicator
    let total_pages = (mission_log.completed.len().max(1) - 1) / ENTRIES_PER_PAGE + 1;
    let page_text = format!("Page {} / {}", filter.page + 1, total_pages);
    let page_label = commands
        .spawn((
            Text::new(page_text),
            TextFont {
                font: font.0.clone(),
                font_size: 13.0,
                ..default()
            },
            TextColor(Color::srgb(0.5, 0.5, 0.5)),
            Node {
                margin: UiRect::top(Val::Px(8.0)),
                ..default()
            },
        ))
        .id();
    commands.entity(root).add_child(page_label);
}

pub fn despawn_logbook_screen(
    mut commands: Commands,
    query: Query<Entity, With<LogbookScreenRoot>>,
) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}

pub fn handle_logbook_input(input: Res<PlayerInput>, mut next_state: ResMut<NextState<GameState>>) {
    if input.cancel || input.menu_cancel {
        next_state.set(GameState::Playing);
    }
}

fn build_filter_label(filter: &LogbookFilter) -> String {
    let mut parts = vec!["Filters:".to_string()];
    if let Some(airport) = &filter.airport_filter {
        parts.push(format!("Airport={}", airport.display_name()));
    }
    if let Some(class) = &filter.aircraft_class_filter {
        parts.push(format!("Class={}", class.display_name()));
    }
    if parts.len() == 1 {
        parts.push("None".to_string());
    }
    parts.join(" ")
}
