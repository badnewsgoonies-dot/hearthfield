//! Pilot profile / stats screen — rank, skills, hours, achievements, finances.

use crate::shared::*;
use bevy::prelude::*;

// ═══════════════════════════════════════════════════════════════════════════
// COMPONENTS
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Component)]
pub struct ProfileScreenRoot;

#[derive(Component)]
pub struct SkillBar {
    pub skill_name: String,
}

// ═══════════════════════════════════════════════════════════════════════════
// SKILL DEFINITIONS
// ═══════════════════════════════════════════════════════════════════════════

/// The 8 pilot skills tracked in the profile.
pub const SKILL_NAMES: &[&str] = &[
    "Navigation",
    "Instrument Flying",
    "Weather Reading",
    "Landing Precision",
    "Fuel Management",
    "Passenger Comfort",
    "Emergency Handling",
    "Aircraft Knowledge",
];

/// Derive a skill level (0.0–1.0) from pilot stats.
fn compute_skill(skill_index: usize, pilot: &PilotState, stats: &PlayStats) -> f32 {
    let raw = match skill_index {
        0 => (stats.total_distance_nm / 5000.0).min(1.0), // Navigation
        1 => (stats.total_flight_hours / 200.0).min(1.0), // Instrument Flying
        2 => (stats.total_flights as f32 / 100.0).min(1.0), // Weather Reading
        3 => (stats.perfect_landings as f32 / 50.0).min(1.0), // Landing Precision
        4 => {
            if stats.total_flights > 0 {
                (1.0 - stats.rough_landings as f32 / stats.total_flights as f32).max(0.0)
            } else {
                0.0
            }
        } // Fuel Management (proxy: fewer bad landings)
        5 => (pilot.reputation / 100.0).min(1.0),         // Passenger Comfort
        6 => (stats.missions_completed as f32 / 80.0).min(1.0), // Emergency Handling
        7 => (stats.aircraft_owned.len() as f32 / 8.0).min(1.0), // Aircraft Knowledge
        _ => 0.0,
    };
    raw.clamp(0.0, 1.0)
}

// ═══════════════════════════════════════════════════════════════════════════
// SPAWN / DESPAWN
// ═══════════════════════════════════════════════════════════════════════════

#[allow(clippy::too_many_arguments)]
pub fn spawn_profile_screen(
    mut commands: Commands,
    font: Res<UiFontHandle>,
    pilot: Res<PilotState>,
    gold: Res<Gold>,
    stats: Res<PlayStats>,
    achievements: Res<Achievements>,
    economy: Res<EconomyStats>,
    fleet: Res<Fleet>,
    mission_board: Res<MissionBoard>,
) {
    let root = commands
        .spawn((
            ProfileScreenRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(20.0)),
                overflow: Overflow::scroll_y(),
                ..default()
            },
            BackgroundColor(Color::srgba(0.04, 0.04, 0.08, 0.95)),
            GlobalZIndex(40),
        ))
        .id();

    let text_style = TextFont {
        font: font.0.clone(),
        font_size: 14.0,
        ..default()
    };
    let _header_style = TextFont {
        font: font.0.clone(),
        font_size: 18.0,
        ..default()
    };
    let section_style = TextFont {
        font: font.0.clone(),
        font_size: 16.0,
        ..default()
    };

    // ── Title ────────────────────────────────────────────────────────────
    let title = commands
        .spawn((
            Text::new("PILOT PROFILE"),
            TextFont {
                font: font.0.clone(),
                font_size: 22.0,
                ..default()
            },
            TextColor(Color::WHITE),
            Node {
                margin: UiRect::bottom(Val::Px(12.0)),
                ..default()
            },
        ))
        .id();
    commands.entity(root).add_child(title);

    // ── Basic Info ───────────────────────────────────────────────────────
    let info_lines = vec![
        format!("Name: {}", pilot.name),
        format!("Rank: {}", pilot.rank.display_name()),
        format!("XP: {} / {}", pilot.xp, pilot.xp_to_next_rank),
        format!("Total Flights: {}", pilot.total_flights),
        format!("Flight Hours: {:.1}", pilot.total_flight_hours),
        format!("Perfect Landings: {}", pilot.perfect_landings),
        format!("Reputation: {:.0}%", pilot.reputation),
    ];

    let info_section = commands
        .spawn((
            Text::new("─── Pilot Info ───"),
            section_style.clone(),
            TextColor(Color::srgb(0.6, 0.8, 1.0)),
            Node {
                margin: UiRect::new(Val::Px(0.0), Val::Px(0.0), Val::Px(8.0), Val::Px(4.0)),
                ..default()
            },
        ))
        .id();
    commands.entity(root).add_child(info_section);

    for line in &info_lines {
        let child = commands
            .spawn((
                Text::new(line.clone()),
                text_style.clone(),
                TextColor(Color::WHITE),
                Node {
                    margin: UiRect::bottom(Val::Px(2.0)),
                    ..default()
                },
            ))
            .id();
        commands.entity(root).add_child(child);
    }

    // ── Favorite Aircraft ────────────────────────────────────────────────
    let fav_aircraft = if let Some(active) = fleet.active() {
        active.nickname.clone()
    } else {
        "None".to_string()
    };

    let fav_airport = if stats.airports_visited.is_empty() {
        "None".to_string()
    } else {
        // Most visited = first in list (simple proxy)
        stats.airports_visited[0].display_name().to_string()
    };

    let extra_lines = vec![
        format!("Favorite Aircraft: {fav_aircraft}"),
        format!("Most Visited Airport: {fav_airport}"),
    ];

    for line in &extra_lines {
        let child = commands
            .spawn((
                Text::new(line.clone()),
                text_style.clone(),
                TextColor(Color::srgb(0.9, 0.9, 0.8)),
                Node {
                    margin: UiRect::bottom(Val::Px(2.0)),
                    ..default()
                },
            ))
            .id();
        commands.entity(root).add_child(child);
    }

    // ── Skills ───────────────────────────────────────────────────────────
    let skills_header = commands
        .spawn((
            Text::new("─── Skills ───"),
            section_style.clone(),
            TextColor(Color::srgb(0.6, 0.8, 1.0)),
            Node {
                margin: UiRect::new(Val::Px(0.0), Val::Px(0.0), Val::Px(12.0), Val::Px(4.0)),
                ..default()
            },
        ))
        .id();
    commands.entity(root).add_child(skills_header);

    for (i, &skill_name) in SKILL_NAMES.iter().enumerate() {
        let level = compute_skill(i, &pilot, &stats);
        let bar_filled = (level * 20.0) as usize;
        let bar_empty = 20 - bar_filled;
        let bar_str = format!(
            "{}: [{}{}] {:.0}%",
            skill_name,
            "█".repeat(bar_filled),
            "░".repeat(bar_empty),
            level * 100.0
        );

        let child = commands
            .spawn((
                SkillBar {
                    skill_name: skill_name.to_string(),
                },
                Text::new(bar_str),
                text_style.clone(),
                TextColor(Color::srgb(0.8, 0.9, 0.7)),
                Node {
                    margin: UiRect::bottom(Val::Px(2.0)),
                    ..default()
                },
            ))
            .id();
        commands.entity(root).add_child(child);
    }

    // ── Achievements ─────────────────────────────────────────────────────
    let ach_header = commands
        .spawn((
            Text::new(format!(
                "─── Achievements: {} / {} ───",
                achievements.unlocked.len(),
                ACHIEVEMENTS.len()
            )),
            section_style.clone(),
            TextColor(Color::srgb(0.6, 0.8, 1.0)),
            Node {
                margin: UiRect::new(Val::Px(0.0), Val::Px(0.0), Val::Px(12.0), Val::Px(4.0)),
                ..default()
            },
        ))
        .id();
    commands.entity(root).add_child(ach_header);

    // ── Finances ─────────────────────────────────────────────────────────
    let fin_header = commands
        .spawn((
            Text::new("─── Finances ───"),
            section_style.clone(),
            TextColor(Color::srgb(0.6, 0.8, 1.0)),
            Node {
                margin: UiRect::new(Val::Px(0.0), Val::Px(0.0), Val::Px(12.0), Val::Px(4.0)),
                ..default()
            },
        ))
        .id();
    commands.entity(root).add_child(fin_header);

    let fin_lines = vec![
        format!("Net Worth: {} gold", gold.amount),
        format!("Total Earned: {} gold", economy.total_earned),
        format!("Total Spent: {} gold", economy.total_spent),
    ];

    for line in &fin_lines {
        let child = commands
            .spawn((
                Text::new(line.clone()),
                text_style.clone(),
                TextColor(Color::WHITE),
                Node {
                    margin: UiRect::bottom(Val::Px(2.0)),
                    ..default()
                },
            ))
            .id();
        commands.entity(root).add_child(child);
    }

    // ── Active Contracts ─────────────────────────────────────────────────
    let contracts_header = commands
        .spawn((
            Text::new("─── Active Mission ───"),
            section_style.clone(),
            TextColor(Color::srgb(0.6, 0.8, 1.0)),
            Node {
                margin: UiRect::new(Val::Px(0.0), Val::Px(0.0), Val::Px(12.0), Val::Px(4.0)),
                ..default()
            },
        ))
        .id();
    commands.entity(root).add_child(contracts_header);

    let mission_text = if let Some(active) = &mission_board.active {
        format!(
            "{} — {} → {}",
            active.mission.title,
            active.mission.origin.display_name(),
            active.mission.destination.display_name()
        )
    } else {
        "No active mission".to_string()
    };

    let mission_label = commands
        .spawn((
            Text::new(mission_text),
            text_style.clone(),
            TextColor(Color::srgb(0.9, 0.85, 0.7)),
            Node {
                margin: UiRect::bottom(Val::Px(2.0)),
                ..default()
            },
        ))
        .id();
    commands.entity(root).add_child(mission_label);
}

pub fn despawn_profile_screen(
    mut commands: Commands,
    query: Query<Entity, With<ProfileScreenRoot>>,
) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}

pub fn handle_profile_input(input: Res<PlayerInput>, mut next_state: ResMut<NextState<GameState>>) {
    if input.cancel || input.menu_cancel {
        next_state.set(GameState::Playing);
    }
}
