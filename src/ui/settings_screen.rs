use super::UiFontHandle;
use crate::shared::*;
use bevy::prelude::*;

// ═══════════════════════════════════════════════════════════════════════
// MARKER COMPONENTS
// ═══════════════════════════════════════════════════════════════════════

#[derive(Component)]
pub struct SettingsScreenRoot;

#[derive(Component)]
pub struct VolumeValueText;

/// Tracks overlay visibility (toggled by F4 during Playing).
#[derive(Resource, Default)]
pub struct SettingsOverlayState {
    pub visible: bool,
}

/// Placeholder volume setting (0-100).
#[derive(Resource)]
pub struct AudioVolume {
    pub level: u8,
}

impl Default for AudioVolume {
    fn default() -> Self {
        Self { level: 80 }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// TOGGLE
// ═══════════════════════════════════════════════════════════════════════

pub fn toggle_settings_overlay(
    keys: Res<ButtonInput<KeyCode>>,
    mut overlay: ResMut<SettingsOverlayState>,
) {
    if keys.just_pressed(KeyCode::F4) {
        overlay.visible = !overlay.visible;
    }
}

// ═══════════════════════════════════════════════════════════════════════
// LIFECYCLE — reactive spawn/despawn based on SettingsOverlayState
// ═══════════════════════════════════════════════════════════════════════

pub fn update_settings_lifecycle(
    mut commands: Commands,
    overlay: Res<SettingsOverlayState>,
    font_handle: Res<UiFontHandle>,
    volume: Res<AudioVolume>,
    bindings: Res<KeyBindings>,
    existing: Query<Entity, With<SettingsScreenRoot>>,
) {
    let ui_exists = !existing.is_empty();

    if overlay.visible && !ui_exists {
        spawn_settings_screen(&mut commands, &font_handle, &volume, &bindings);
    } else if !overlay.visible && ui_exists {
        for entity in &existing {
            commands.entity(entity).despawn_recursive();
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// SPAWN
// ═══════════════════════════════════════════════════════════════════════

fn spawn_settings_screen(
    commands: &mut Commands,
    font_handle: &UiFontHandle,
    volume: &AudioVolume,
    bindings: &KeyBindings,
) {
    let font = font_handle.0.clone();

    let keybind_rows: Vec<(&str, String)> = vec![
        ("Move Up", format!("{:?}", bindings.move_up)),
        ("Move Down", format!("{:?}", bindings.move_down)),
        ("Move Left", format!("{:?}", bindings.move_left)),
        ("Move Right", format!("{:?}", bindings.move_right)),
        ("Interact", format!("{:?}", bindings.interact)),
        ("Tool Use", format!("{:?}", bindings.tool_use)),
        ("Inventory", format!("{:?}", bindings.open_inventory)),
        ("Crafting", format!("{:?}", bindings.open_crafting)),
        ("Map", format!("{:?}", bindings.open_map)),
        ("Journal", format!("{:?}", bindings.open_journal)),
        (
            "Relationships",
            format!("{:?}", bindings.open_relationships),
        ),
        ("Pause", format!("{:?}", bindings.pause)),
    ];

    commands
        .spawn((
            SettingsScreenRoot,
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
                        width: Val::Px(480.0),
                        height: Val::Px(520.0),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        padding: UiRect::all(Val::Px(16.0)),
                        row_gap: Val::Px(6.0),
                        border: UiRect::all(Val::Px(3.0)),
                        overflow: Overflow::clip_y(),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.12, 0.1, 0.1, 0.97)),
                    BorderColor(Color::srgb(0.5, 0.4, 0.4)),
                ))
                .with_children(|panel| {
                    // Title
                    panel.spawn((
                        Text::new("SETTINGS"),
                        TextFont {
                            font: font.clone(),
                            font_size: 22.0,
                            ..default()
                        },
                        TextColor(Color::srgb(1.0, 0.9, 0.85)),
                    ));

                    // Hint
                    panel.spawn((
                        Text::new("F4 / Esc: Close | Left/Right: Volume"),
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
                        BackgroundColor(Color::srgb(0.5, 0.4, 0.4)),
                    ));

                    // ─── Audio volume section ───
                    panel.spawn((
                        Text::new("AUDIO"),
                        TextFont {
                            font: font.clone(),
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(Color::srgb(1.0, 0.85, 0.7)),
                    ));

                    panel
                        .spawn(Node {
                            width: Val::Percent(100.0),
                            flex_direction: FlexDirection::Row,
                            justify_content: JustifyContent::SpaceBetween,
                            align_items: AlignItems::Center,
                            padding: UiRect::axes(Val::Px(12.0), Val::Px(4.0)),
                            ..default()
                        })
                        .with_children(|row| {
                            row.spawn((
                                Text::new("Volume"),
                                TextFont {
                                    font: font.clone(),
                                    font_size: 14.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.85, 0.85, 0.85)),
                            ));
                            // Volume bar visualization
                            let bar = build_volume_bar(volume.level);
                            row.spawn((
                                VolumeValueText,
                                Text::new(bar),
                                TextFont {
                                    font: font.clone(),
                                    font_size: 14.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(1.0, 0.9, 0.5)),
                            ));
                        });

                    // Divider
                    panel.spawn((
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Px(2.0),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.5, 0.4, 0.4)),
                    ));

                    // ─── Keybinds section ───
                    panel.spawn((
                        Text::new("KEYBINDS"),
                        TextFont {
                            font: font.clone(),
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(Color::srgb(1.0, 0.85, 0.7)),
                    ));

                    for (action_name, key_name) in &keybind_rows {
                        panel
                            .spawn(Node {
                                width: Val::Percent(100.0),
                                flex_direction: FlexDirection::Row,
                                justify_content: JustifyContent::SpaceBetween,
                                padding: UiRect::axes(Val::Px(12.0), Val::Px(3.0)),
                                ..default()
                            })
                            .with_children(|row| {
                                row.spawn((
                                    Text::new(*action_name),
                                    TextFont {
                                        font: font.clone(),
                                        font_size: 12.0,
                                        ..default()
                                    },
                                    TextColor(Color::srgb(0.8, 0.8, 0.8)),
                                ));
                                row.spawn((
                                    Text::new(key_name.clone()),
                                    TextFont {
                                        font: font.clone(),
                                        font_size: 12.0,
                                        ..default()
                                    },
                                    TextColor(Color::srgb(0.6, 0.8, 1.0)),
                                ));
                            });
                    }
                });
        });
}

// ═══════════════════════════════════════════════════════════════════════
// VOLUME ADJUSTMENT
// ═══════════════════════════════════════════════════════════════════════

pub fn settings_volume_input(
    keys: Res<ButtonInput<KeyCode>>,
    overlay: Res<SettingsOverlayState>,
    mut volume: ResMut<AudioVolume>,
    mut text_query: Query<&mut Text, With<VolumeValueText>>,
) {
    if !overlay.visible {
        return;
    }

    let mut changed = false;
    if keys.just_pressed(KeyCode::ArrowRight) && volume.level < 100 {
        volume.level = (volume.level + 10).min(100);
        changed = true;
    }
    if keys.just_pressed(KeyCode::ArrowLeft) && volume.level > 0 {
        volume.level = volume.level.saturating_sub(10);
        changed = true;
    }

    if changed {
        let bar = build_volume_bar(volume.level);
        for mut text in &mut text_query {
            **text = bar.clone();
        }
    }
}

/// Close overlay on Escape as well.
pub fn settings_close_on_escape(
    keys: Res<ButtonInput<KeyCode>>,
    mut overlay: ResMut<SettingsOverlayState>,
) {
    if overlay.visible && keys.just_pressed(KeyCode::Escape) {
        overlay.visible = false;
    }
}

// ═══════════════════════════════════════════════════════════════════════
// HELPERS
// ═══════════════════════════════════════════════════════════════════════

fn build_volume_bar(level: u8) -> String {
    let filled = (level / 10) as usize;
    let empty = 10usize.saturating_sub(filled);
    format!("[{}{}] {}%", "|".repeat(filled), "-".repeat(empty), level)
}
