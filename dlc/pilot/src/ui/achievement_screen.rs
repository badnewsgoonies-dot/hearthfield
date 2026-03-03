//! Achievement tracker screen — grid of unlocked/locked/hidden achievements.

use bevy::prelude::*;
use crate::shared::*;
use crate::data::achievements::{ALL_ACHIEVEMENTS, AchievementEntry};

// ─── Components ──────────────────────────────────────────────────────────

#[derive(Component)]
pub struct AchievementScreenRoot;

#[derive(Component)]
pub struct AchievementCard {
    pub achievement_id: &'static str,
}

#[derive(Component)]
pub struct AchievementProgressBar;

// ─── Spawn / Despawn ─────────────────────────────────────────────────────

pub fn spawn_achievement_screen(
    mut commands: Commands,
    font: Res<UiFontHandle>,
    achievements: Res<Achievements>,
    play_stats: Res<PlayStats>,
) {
    let root = commands
        .spawn((
            AchievementScreenRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(20.0)),
                overflow: Overflow::scroll_y(),
                ..default()
            },
            BackgroundColor(Color::srgba(0.05, 0.05, 0.08, 0.95)),
            GlobalZIndex(40),
        ))
        .id();

    // Title
    let unlocked_count = achievements.unlocked.len();
    let total_visible = ALL_ACHIEVEMENTS.iter().filter(|a| !a.hidden).count();
    let title_text = format!(
        "ACHIEVEMENTS — {} / {}",
        unlocked_count, total_visible
    );
    let title = commands
        .spawn((
            Text::new(title_text),
            TextFont {
                font: font.0.clone(),
                font_size: 28.0,
                ..default()
            },
            TextColor(Color::srgb(1.0, 0.85, 0.3)),
            Node {
                margin: UiRect::bottom(Val::Px(16.0)),
                ..default()
            },
        ))
        .id();
    commands.entity(root).add_child(title);

    // Grid container
    let grid = commands
        .spawn((
            Node {
                width: Val::Percent(95.0),
                flex_direction: FlexDirection::Row,
                flex_wrap: FlexWrap::Wrap,
                justify_content: JustifyContent::Center,
                column_gap: Val::Px(10.0),
                row_gap: Val::Px(10.0),
                ..default()
            },
        ))
        .id();
    commands.entity(root).add_child(grid);

    for achievement in ALL_ACHIEVEMENTS {
        let is_unlocked = achievements.is_unlocked(achievement.id);
        spawn_achievement_card(
            &mut commands,
            grid,
            &font,
            achievement,
            is_unlocked,
            &play_stats,
        );
    }
}

pub fn despawn_achievement_screen(
    mut commands: Commands,
    query: Query<Entity, With<AchievementScreenRoot>>,
) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}

// ─── Card builder ────────────────────────────────────────────────────────

fn spawn_achievement_card(
    commands: &mut Commands,
    parent: Entity,
    font: &UiFontHandle,
    entry: &'static AchievementEntry,
    unlocked: bool,
    play_stats: &PlayStats,
) {
    let (bg_color, border_color) = if unlocked {
        (
            Color::srgba(0.25, 0.20, 0.05, 0.9),
            Color::srgb(1.0, 0.85, 0.2),
        )
    } else if entry.hidden {
        (
            Color::srgba(0.08, 0.08, 0.08, 0.9),
            Color::srgb(0.3, 0.3, 0.3),
        )
    } else {
        (
            Color::srgba(0.1, 0.1, 0.12, 0.9),
            Color::srgb(0.4, 0.4, 0.4),
        )
    };

    let card = commands
        .spawn((
            AchievementCard {
                achievement_id: entry.id,
            },
            Node {
                width: Val::Px(220.0),
                min_height: Val::Px(100.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(8.0)),
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(bg_color),
            BorderColor(border_color),
        ))
        .id();
    commands.entity(parent).add_child(card);

    // Icon placeholder + title row
    let icon_color = if unlocked {
        Color::srgb(1.0, 0.85, 0.2)
    } else {
        Color::srgb(0.3, 0.3, 0.3)
    };
    let icon = commands
        .spawn((
            Node {
                width: Val::Px(24.0),
                height: Val::Px(24.0),
                margin: UiRect::bottom(Val::Px(4.0)),
                ..default()
            },
            BackgroundColor(icon_color),
        ))
        .id();
    commands.entity(card).add_child(icon);

    // Title
    let title_text = if entry.hidden && !unlocked {
        "???".to_string()
    } else {
        entry.title.to_string()
    };
    let title = commands
        .spawn((
            Text::new(title_text),
            TextFont {
                font: font.0.clone(),
                font_size: 14.0,
                ..default()
            },
            TextColor(if unlocked {
                Color::srgb(1.0, 0.95, 0.7)
            } else {
                Color::srgb(0.6, 0.6, 0.6)
            }),
            Node {
                margin: UiRect::bottom(Val::Px(2.0)),
                ..default()
            },
        ))
        .id();
    commands.entity(card).add_child(title);

    // Description
    let desc_text = if entry.hidden && !unlocked {
        "Hidden achievement — keep playing to discover!".to_string()
    } else {
        entry.description.to_string()
    };
    let desc = commands
        .spawn((
            Text::new(desc_text),
            TextFont {
                font: font.0.clone(),
                font_size: 11.0,
                ..default()
            },
            TextColor(Color::srgb(0.5, 0.5, 0.5)),
            Node {
                margin: UiRect::bottom(Val::Px(4.0)),
                ..default()
            },
        ))
        .id();
    commands.entity(card).add_child(desc);

    // Progress bar for progressive achievements
    if !unlocked && !entry.hidden {
        if let Some((current, target)) = get_progress(entry.id, play_stats) {
            let pct = (current as f32 / target as f32).clamp(0.0, 1.0);

            let bar_bg = commands
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(6.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
                ))
                .id();
            commands.entity(card).add_child(bar_bg);

            let bar_fill = commands
                .spawn((
                    AchievementProgressBar,
                    Node {
                        width: Val::Percent(pct * 100.0),
                        height: Val::Px(6.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.4, 0.7, 1.0)),
                ))
                .id();
            commands.entity(bar_bg).add_child(bar_fill);

            let progress_text = commands
                .spawn((
                    Text::new(format!("{} / {}", current, target)),
                    TextFont {
                        font: font.0.clone(),
                        font_size: 10.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.5, 0.5, 0.5)),
                    Node {
                        margin: UiRect::top(Val::Px(2.0)),
                        ..default()
                    },
                ))
                .id();
            commands.entity(card).add_child(progress_text);
        }
    }

    // Unlocked badge
    if unlocked {
        let badge = commands
            .spawn((
                Text::new("✓ UNLOCKED"),
                TextFont {
                    font: font.0.clone(),
                    font_size: 11.0,
                    ..default()
                },
                TextColor(Color::srgb(0.3, 1.0, 0.3)),
                Node {
                    margin: UiRect::top(Val::Px(4.0)),
                    ..default()
                },
            ))
            .id();
        commands.entity(card).add_child(badge);
    }
}

/// Returns (current, target) for progressive achievements.
fn get_progress(id: &str, stats: &PlayStats) -> Option<(u32, u32)> {
    match id {
        "flights_10" => Some((stats.total_flights.min(10), 10)),
        "flights_50" => Some((stats.total_flights.min(50), 50)),
        "flights_100" => Some((stats.total_flights.min(100), 100)),
        "flights_500" => Some((stats.total_flights.min(500), 500)),
        "hours_100" => Some(((stats.total_flight_hours as u32).min(100), 100)),
        "hours_1000" => Some(((stats.total_flight_hours as u32).min(1000), 1000)),
        "perfect_10" => Some((stats.perfect_landings.min(10), 10)),
        "perfect_50" => Some((stats.perfect_landings.min(50), 50)),
        "all_airports" => Some((stats.airports_visited.len() as u32, 10)),
        "cargo_king" => Some((stats.missions_completed.min(100), 100)),
        "medical_hero" => Some((stats.missions_completed.min(10), 10)),
        "vip_service" => Some((stats.missions_completed.min(20), 20)),
        "rescue_5" => Some((stats.missions_completed.min(5), 5)),
        _ => None,
    }
}
