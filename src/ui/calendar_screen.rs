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
                        width: Val::Px(520.0),
                        height: Val::Px(460.0),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        padding: UiRect::all(Val::Px(16.0)),
                        row_gap: Val::Px(8.0),
                        border: UiRect::all(Val::Px(3.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.1, 0.12, 0.08, 0.97)),
                    BorderColor(Color::srgb(0.4, 0.55, 0.3)),
                ))
                .with_children(|panel| {
                    // Title
                    panel.spawn((
                        Text::new("CALENDAR"),
                        TextFont {
                            font: font.clone(),
                            font_size: 22.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.9, 1.0, 0.7)),
                    ));

                    // Season + Year
                    panel.spawn((
                        Text::new(header),
                        TextFont {
                            font: font.clone(),
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.8, 0.9, 0.6)),
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
                                    let is_festival = is_festival_day(calendar.season, day);
                                    let birthday_npc = birthday_map
                                        .iter()
                                        .find(|(d, _)| *d == day)
                                        .map(|(_, name)| name.as_str());

                                    // Build label
                                    let label = if is_festival {
                                        format!("*{}*", day)
                                    } else if birthday_npc.is_some() {
                                        format!("{}B", day)
                                    } else {
                                        format!("{}", day)
                                    };

                                    // Pick colors
                                    let (bg_color, text_color, border_color) = if is_today {
                                        (
                                            Color::srgba(0.4, 0.5, 0.2, 0.95),
                                            Color::srgb(1.0, 1.0, 0.8),
                                            Color::srgb(0.9, 1.0, 0.3),
                                        )
                                    } else if is_festival {
                                        (
                                            Color::srgba(0.5, 0.3, 0.1, 0.9),
                                            Color::srgb(1.0, 0.8, 0.4),
                                            Color::srgba(0.6, 0.4, 0.2, 0.7),
                                        )
                                    } else {
                                        (
                                            Color::srgba(0.15, 0.17, 0.12, 0.85),
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
                                            Text::new(label),
                                            TextFont {
                                                font: font.clone(),
                                                font_size: 14.0,
                                                ..default()
                                            },
                                            TextColor(text_color),
                                        ));

                                        // Show festival name or birthday NPC below the day number
                                        if let Some(fname) = festival_name(calendar.season, day) {
                                            cell.spawn((
                                                Text::new(fname),
                                                TextFont {
                                                    font: font.clone(),
                                                    font_size: 8.0,
                                                    ..default()
                                                },
                                                TextColor(Color::srgb(1.0, 0.7, 0.3)),
                                            ));
                                        } else if let Some(npc_name) = birthday_npc {
                                            let short = if npc_name.len() > 6 {
                                                &npc_name.chars().take(6).collect::<String>()
                                            } else {
                                                npc_name
                                            };
                                            cell.spawn((
                                                Text::new(short),
                                                TextFont {
                                                    font: font.clone(),
                                                    font_size: 8.0,
                                                    ..default()
                                                },
                                                TextColor(Color::srgb(0.7, 0.85, 1.0)),
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
    matches!(
        (season, day),
        (Season::Spring, 13) | (Season::Summer, 11) | (Season::Fall, 16) | (Season::Winter, 25)
    )
}

fn festival_name(season: Season, day: u8) -> Option<&'static str> {
    match (season, day) {
        (Season::Spring, 13) => Some("Egg Fest"),
        (Season::Summer, 11) => Some("Luau"),
        (Season::Fall, 16) => Some("Harvest"),
        (Season::Winter, 25) => Some("W.Star"),
        _ => None,
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
