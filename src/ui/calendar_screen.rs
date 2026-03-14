use super::UiFontHandle;
use crate::shared::*;
use bevy::prelude::*;

// ═══════════════════════════════════════════════════════════════════════
// MARKER COMPONENTS
// ═══════════════════════════════════════════════════════════════════════

#[derive(Component)]
pub struct CalendarScreenRoot;

/// Tracks overlay visibility (toggled by F1 during Playing).
#[derive(Resource, Default)]
pub struct CalendarOverlayState {
    pub visible: bool,
}

// ═══════════════════════════════════════════════════════════════════════
// TOGGLE
// ═══════════════════════════════════════════════════════════════════════

pub fn toggle_calendar_overlay(
    keys: Res<ButtonInput<KeyCode>>,
    mut overlay: ResMut<CalendarOverlayState>,
) {
    if keys.just_pressed(KeyCode::F1) {
        overlay.visible = !overlay.visible;
    }
}

// ═══════════════════════════════════════════════════════════════════════
// LIFECYCLE — reactive spawn/despawn based on CalendarOverlayState
// ═══════════════════════════════════════════════════════════════════════

pub fn update_calendar_lifecycle(
    mut commands: Commands,
    overlay: Res<CalendarOverlayState>,
    font_handle: Res<UiFontHandle>,
    calendar: Res<Calendar>,
    npc_registry: Res<NpcRegistry>,
    existing: Query<Entity, With<CalendarScreenRoot>>,
) {
    let ui_exists = !existing.is_empty();

    if overlay.visible && !ui_exists {
        spawn_calendar_screen(&mut commands, &font_handle, &calendar, &npc_registry);
    } else if !overlay.visible && ui_exists {
        for entity in &existing {
            commands.entity(entity).despawn_recursive();
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// SPAWN
// ═══════════════════════════════════════════════════════════════════════

fn spawn_calendar_screen(
    commands: &mut Commands,
    font_handle: &UiFontHandle,
    calendar: &Calendar,
    npc_registry: &NpcRegistry,
) {
    let font = font_handle.0.clone();

    // Build a map of day → NPC birthday names for current season
    let mut birthday_map: Vec<(u8, String)> = Vec::new();
    for npc_def in npc_registry.npcs.values() {
        if npc_def.birthday_season == calendar.season {
            birthday_map.push((npc_def.birthday_day, npc_def.name.clone()));
        }
    }

    let season_name = format!("{:?}", calendar.season);
    let header = format!("{} — Year {}", season_name, calendar.year);
    let upcoming_festival = upcoming_festival_in_season(calendar.season, calendar.day);

    commands
        .spawn((
            CalendarScreenRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.5)),
            GlobalZIndex(60),
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        width: Val::Px(560.0),
                        height: Val::Px(500.0),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        padding: UiRect::all(Val::Px(20.0)),
                        row_gap: Val::Px(10.0),
                        border: UiRect::all(Val::Px(3.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.23, 0.18, 0.13, 0.97)),
                    BorderColor(Color::srgb(0.73, 0.60, 0.42)),
                ))
                .with_children(|panel| {
                    // Title
                    panel.spawn((
                        Text::new("Town Planner"),
                        TextFont {
                            font: font.clone(),
                            font_size: 20.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.98, 0.90, 0.74)),
                    ));

                    // Season + Year
                    panel.spawn((
                        Text::new(header),
                        TextFont {
                            font: font.clone(),
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.86, 0.78, 0.64)),
                    ));

                    // Hint
                    panel.spawn((
                        Text::new("F1 / Esc: Close"),
                        TextFont {
                            font: font.clone(),
                            font_size: 11.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.55, 0.55, 0.55)),
                    ));

                    // Planner summary strip
                    panel
                        .spawn(Node {
                            width: Val::Percent(100.0),
                            flex_direction: FlexDirection::Row,
                            justify_content: JustifyContent::SpaceBetween,
                            column_gap: Val::Px(6.0),
                            ..default()
                        })
                        .with_children(|row| {
                            for (title, value) in [
                                (
                                    "Yesterday",
                                    summarize_relative_day(
                                        calendar.season,
                                        calendar.day.checked_sub(1),
                                        &birthday_map,
                                    ),
                                ),
                                (
                                    "Today",
                                    summarize_relative_day(
                                        calendar.season,
                                        Some(calendar.day),
                                        &birthday_map,
                                    ),
                                ),
                                (
                                    "Tomorrow",
                                    summarize_relative_day(
                                        calendar.season,
                                        next_day_in_season(calendar.day),
                                        &birthday_map,
                                    ),
                                ),
                                (
                                    "Next festival",
                                    summarize_next_festival(calendar.day, upcoming_festival),
                                ),
                            ] {
                                row.spawn((
                                    Node {
                                        width: Val::Px(118.0),
                                        flex_direction: FlexDirection::Column,
                                        align_items: AlignItems::Center,
                                        justify_content: JustifyContent::Center,
                                        padding: UiRect::axes(Val::Px(6.0), Val::Px(4.0)),
                                        border: UiRect::all(Val::Px(1.0)),
                                        ..default()
                                    },
                                    BackgroundColor(Color::srgba(0.14, 0.16, 0.11, 0.9)),
                                    BorderColor(Color::srgba(0.3, 0.35, 0.25, 0.5)),
                                ))
                                .with_children(|item| {
                                    item.spawn((
                                        Text::new(title),
                                        TextFont {
                                            font: font.clone(),
                                            font_size: 10.0,
                                            ..default()
                                        },
                                        TextColor(Color::srgb(0.75, 0.78, 0.68)),
                                    ));
                                    item.spawn((
                                        Text::new(value),
                                        TextFont {
                                            font: font.clone(),
                                            font_size: 9.0,
                                            ..default()
                                        },
                                        TextColor(Color::srgb(0.9, 0.94, 0.82)),
                                    ));
                                });
                            }
                        });

                    // Day-of-week header row
                    panel
                        .spawn(Node {
                            width: Val::Percent(100.0),
                            flex_direction: FlexDirection::Row,
                            justify_content: JustifyContent::SpaceEvenly,
                            ..default()
                        })
                        .with_children(|row| {
                            for day_name in &["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"] {
                                row.spawn((
                                    Text::new(*day_name),
                                    TextFont {
                                        font: font.clone(),
                                        font_size: 12.0,
                                        ..default()
                                    },
                                    TextColor(Color::srgb(0.7, 0.7, 0.7)),
                                    Node {
                                        width: Val::Px(60.0),
                                        justify_content: JustifyContent::Center,
                                        ..default()
                                    },
                                ));
                            }
                        });

                    // 4 weeks x 7 days grid (28 days per season)
                    for week in 0..4u8 {
                        panel
                            .spawn(Node {
                                width: Val::Percent(100.0),
                                flex_direction: FlexDirection::Row,
                                justify_content: JustifyContent::SpaceEvenly,
                                ..default()
                            })
                            .with_children(|row| {
                                for dow in 0..7u8 {
                                    let day = week * 7 + dow + 1;
                                    let is_today = day == calendar.day;
                                    let birthday_npc = birthday_map
                                        .iter()
                                        .find(|(d, _)| *d == day)
                                        .map(|(_, name)| name.as_str());
                                    let day_tag = day_tag(
                                        day,
                                        calendar.day,
                                        birthday_npc.is_some(),
                                        upcoming_festival.map(|(festival_day, _)| festival_day),
                                    );

                                    // Pick colors
                                    let (bg_color, text_color, border_color) = if is_today {
                                        (
                                            Color::srgba(0.56, 0.43, 0.30, 0.95),
                                            Color::srgb(1.0, 1.0, 0.8),
                                            Color::srgb(0.90, 0.82, 0.62),
                                        )
                                    } else if matches!(day_tag, Some("Festival Soon")) {
                                        (
                                            Color::srgba(0.74, 0.42, 0.24, 0.9),
                                            Color::srgb(1.0, 0.93, 0.82),
                                            Color::srgba(0.6, 0.4, 0.2, 0.7),
                                        )
                                    } else {
                                        (
                                            Color::srgba(0.24, 0.20, 0.16, 0.85),
                                            Color::srgb(0.8, 0.8, 0.8),
                                            Color::srgba(0.3, 0.35, 0.25, 0.5),
                                        )
                                    };

                                    row.spawn((
                                        Node {
                                            width: Val::Px(60.0),
                                            height: Val::Px(50.0),
                                            flex_direction: FlexDirection::Column,
                                            align_items: AlignItems::Center,
                                            justify_content: JustifyContent::Center,
                                            border: UiRect::all(Val::Px(2.0)),
                                            margin: UiRect::all(Val::Px(1.0)),
                                            ..default()
                                        },
                                        BackgroundColor(bg_color),
                                        BorderColor(border_color),
                                    ))
                                    .with_children(|cell| {
                                        cell.spawn((
                                            Text::new(day.to_string()),
                                            TextFont {
                                                font: font.clone(),
                                                font_size: 14.0,
                                                ..default()
                                            },
                                            TextColor(text_color),
                                        ));

                                        if let Some(tag) = day_tag {
                                            cell.spawn((
                                                Text::new(tag),
                                                TextFont {
                                                    font: font.clone(),
                                                    font_size: 8.0,
                                                    ..default()
                                                },
                                                TextColor(tag_color(tag)),
                                            ));
                                        }
                                    });
                                }
                            });
                    }
                });
        });
}

// ═══════════════════════════════════════════════════════════════════════
// HELPERS
// ═══════════════════════════════════════════════════════════════════════

fn is_festival_day(season: Season, day: u8) -> bool {
    festival_name(season, day).is_some()
}

fn festival_name(season: Season, day: u8) -> Option<&'static str> {
    let (festival_day, festival_name) = festival_for_season(season);
    (festival_day == day).then_some(festival_name)
}

fn festival_for_season(season: Season) -> (u8, &'static str) {
    match season {
        Season::Spring => (13, "Egg Fest"),
        Season::Summer => (11, "Luau"),
        Season::Fall => (16, "Harvest"),
        Season::Winter => (25, "W.Star"),
    }
}

fn upcoming_festival_in_season(season: Season, current_day: u8) -> Option<(u8, &'static str)> {
    let (festival_day, festival_name) = festival_for_season(season);
    (festival_day >= current_day).then_some((festival_day, festival_name))
}

fn next_day_in_season(day: u8) -> Option<u8> {
    (day < 28).then_some(day + 1)
}

fn summarize_relative_day(season: Season, day: Option<u8>, birthdays: &[(u8, String)]) -> String {
    let Some(day) = day else {
        return "Season edge".to_string();
    };

    let weekday = weekday_name(day);
    if let Some(festival) = festival_name(season, day) {
        format!("{weekday} {day} • {festival}")
    } else if birthdays.iter().any(|(birthday_day, _)| *birthday_day == day) {
        format!("{weekday} {day} • Birthday")
    } else {
        format!("{weekday} {day}")
    }
}

fn summarize_next_festival(current_day: u8, festival: Option<(u8, &'static str)>) -> String {
    match festival {
        Some((festival_day, festival_name)) if festival_day == current_day => {
            format!("Today • {festival_name}")
        }
        Some((festival_day, festival_name)) if festival_day == current_day + 1 => {
            format!("Tomorrow • {festival_name}")
        }
        Some((festival_day, festival_name)) => format!("Day {festival_day} • {festival_name}"),
        None => "Passed this season".to_string(),
    }
}

fn weekday_name(day: u8) -> &'static str {
    match (day - 1) % 7 {
        0 => "Mon",
        1 => "Tue",
        2 => "Wed",
        3 => "Thu",
        4 => "Fri",
        5 => "Sat",
        _ => "Sun",
    }
}

fn day_tag(
    day: u8,
    current_day: u8,
    has_birthday: bool,
    upcoming_festival_day: Option<u8>,
) -> Option<&'static str> {
    if day == current_day {
        Some("Today")
    } else if day + 1 == current_day {
        Some("Yesterday")
    } else if day == current_day + 1 {
        Some("Tomorrow")
    } else if has_birthday {
        Some("Birthday")
    } else if upcoming_festival_day == Some(day) {
        Some("Festival Soon")
    } else {
        None
    }
}

fn tag_color(tag: &str) -> Color {
    match tag {
        "Today" => Color::srgb(1.0, 1.0, 0.8),
        "Tomorrow" => Color::srgb(0.8, 0.92, 1.0),
        "Yesterday" => Color::srgb(0.72, 0.76, 0.76),
        "Birthday" => Color::srgb(0.7, 0.85, 1.0),
        "Festival Soon" => Color::srgb(1.0, 0.7, 0.3),
        _ => Color::srgb(0.8, 0.8, 0.8),
    }
}

/// Close overlay on Escape as well.
pub fn calendar_close_on_escape(
    keys: Res<ButtonInput<KeyCode>>,
    mut overlay: ResMut<CalendarOverlayState>,
) {
    if overlay.visible && keys.just_pressed(KeyCode::Escape) {
        overlay.visible = false;
    }
}
