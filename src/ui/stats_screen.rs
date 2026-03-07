use super::UiFontHandle;
use crate::shared::*;
use bevy::prelude::*;

// ═══════════════════════════════════════════════════════════════════════
// MARKER COMPONENTS
// ═══════════════════════════════════════════════════════════════════════

#[derive(Component)]
pub struct StatsScreenRoot;

/// Tracks overlay visibility (toggled by F2 during Playing).
#[derive(Resource, Default)]
pub struct StatsOverlayState {
    pub visible: bool,
}

// ═══════════════════════════════════════════════════════════════════════
// TOGGLE
// ═══════════════════════════════════════════════════════════════════════

pub fn toggle_stats_overlay(
    keys: Res<ButtonInput<KeyCode>>,
    mut overlay: ResMut<StatsOverlayState>,
) {
    if keys.just_pressed(KeyCode::F2) {
        overlay.visible = !overlay.visible;
    }
}

// ═══════════════════════════════════════════════════════════════════════
// LIFECYCLE — reactive spawn/despawn based on StatsOverlayState
// ═══════════════════════════════════════════════════════════════════════

pub fn update_stats_lifecycle(
    mut commands: Commands,
    overlay: Res<StatsOverlayState>,
    font_handle: Res<UiFontHandle>,
    stats: Res<PlayStats>,
    existing: Query<Entity, With<StatsScreenRoot>>,
) {
    let ui_exists = !existing.is_empty();

    if overlay.visible && !ui_exists {
        spawn_stats_screen(&mut commands, &font_handle, &stats);
    } else if !overlay.visible && ui_exists {
        for entity in &existing {
            commands.entity(entity).despawn_recursive();
        }
    }
}

/// Rebuild display when PlayStats changes while overlay is open.
pub fn refresh_stats_display(
    mut commands: Commands,
    overlay: Res<StatsOverlayState>,
    stats: Res<PlayStats>,
    font_handle: Res<UiFontHandle>,
    existing: Query<Entity, With<StatsScreenRoot>>,
) {
    if !overlay.visible || !stats.is_changed() {
        return;
    }
    for entity in &existing {
        commands.entity(entity).despawn_recursive();
    }
    spawn_stats_screen(&mut commands, &font_handle, &stats);
}

// ═══════════════════════════════════════════════════════════════════════
// SPAWN
// ═══════════════════════════════════════════════════════════════════════

fn spawn_stats_screen(commands: &mut Commands, font_handle: &UiFontHandle, stats: &PlayStats) {
    let font = font_handle.0.clone();

    let stat_rows: Vec<(&str, u64)> = vec![
        ("Crops Harvested", stats.crops_harvested),
        ("Fish Caught", stats.fish_caught),
        ("Items Shipped", stats.items_shipped),
        ("Gifts Given", stats.gifts_given),
        ("Mine Floors Cleared", stats.mine_floors_cleared),
        ("Animal Products Collected", stats.animal_products_collected),
        ("Food Eaten", stats.food_eaten),
        ("Total Gold Earned", stats.total_gold_earned),
        ("Days Played", stats.days_played),
        ("Festivals Attended", stats.festivals_attended),
    ];

    commands
        .spawn((
            StatsScreenRoot,
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
                        width: Val::Px(460.0),
                        height: Val::Px(420.0),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        padding: UiRect::all(Val::Px(16.0)),
                        row_gap: Val::Px(8.0),
                        border: UiRect::all(Val::Px(3.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.08, 0.08, 0.14, 0.97)),
                    BorderColor(Color::srgb(0.3, 0.35, 0.6)),
                ))
                .with_children(|panel| {
                    // Title
                    panel.spawn((
                        Text::new("STATISTICS"),
                        TextFont {
                            font: font.clone(),
                            font_size: 22.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.8, 0.85, 1.0)),
                    ));

                    // Hint
                    panel.spawn((
                        Text::new("F2 / Esc: Close"),
                        TextFont {
                            font: font.clone(),
                            font_size: 11.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.55, 0.55, 0.55)),
                    ));

                    // Divider
                    panel.spawn((
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Px(2.0),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.3, 0.35, 0.6)),
                    ));

                    // Stat rows — two-column layout
                    for (label, value) in &stat_rows {
                        panel
                            .spawn(Node {
                                width: Val::Percent(100.0),
                                flex_direction: FlexDirection::Row,
                                justify_content: JustifyContent::SpaceBetween,
                                padding: UiRect::axes(Val::Px(12.0), Val::Px(4.0)),
                                ..default()
                            })
                            .with_children(|row| {
                                row.spawn((
                                    Text::new(*label),
                                    TextFont {
                                        font: font.clone(),
                                        font_size: 14.0,
                                        ..default()
                                    },
                                    TextColor(Color::srgb(0.85, 0.85, 0.9)),
                                ));
                                row.spawn((
                                    Text::new(format!("{}", value)),
                                    TextFont {
                                        font: font.clone(),
                                        font_size: 14.0,
                                        ..default()
                                    },
                                    TextColor(Color::srgb(1.0, 0.9, 0.5)),
                                ));
                            });
                    }
                });
        });
}

/// Close overlay on Escape as well.
pub fn stats_close_on_escape(
    keys: Res<ButtonInput<KeyCode>>,
    mut overlay: ResMut<StatsOverlayState>,
) {
    if overlay.visible && keys.just_pressed(KeyCode::Escape) {
        overlay.visible = false;
    }
}
