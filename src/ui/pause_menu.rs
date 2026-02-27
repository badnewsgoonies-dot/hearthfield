use super::UiFontHandle;
use super::menu_kit::{self, MenuAssets, set_button_visual};
use crate::save::{ActiveSaveSlot, SaveCompleteEvent, SaveRequestEvent};
use crate::shared::*;
use bevy::prelude::*;

// ═══════════════════════════════════════════════════════════════════════
// MARKER COMPONENTS
// ═══════════════════════════════════════════════════════════════════════

#[derive(Component)]
pub struct PauseMenuRoot;

/// Tracks pause menu selection
#[derive(Resource)]
pub struct PauseMenuState {
    pub cursor: usize,
    pub status_message: String,
}

#[derive(Component)]
pub struct PauseMenuStatusText;

const PAUSE_OPTIONS: &[&str] = &["Resume", "Save Game", "Quit to Menu"];

// ═══════════════════════════════════════════════════════════════════════
// SPAWN / DESPAWN
// ═══════════════════════════════════════════════════════════════════════

pub fn spawn_pause_menu(
    mut commands: Commands,
    font_handle: Res<UiFontHandle>,
    assets: Res<MenuAssets>,
    theme: Res<MenuTheme>,
) {
    commands.insert_resource(PauseMenuState {
        cursor: 0,
        status_message: String::new(),
    });

    let font = font_handle.0.clone();

    commands
        .spawn((
            PauseMenuRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(theme.bg_overlay),
        ))
        .with_children(|parent| {
            // Panel
            parent
                .spawn((
                    Node {
                        width: Val::Px(theme.panel_width),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        padding: UiRect::all(Val::Px(theme.panel_padding)),
                        row_gap: Val::Px(theme.panel_gap),
                        border: UiRect::all(Val::Px(theme.panel_border_width)),
                        ..default()
                    },
                    BackgroundColor(theme.panel_bg),
                    BorderColor(theme.panel_border),
                ))
                .with_children(|panel| {
                    // Title
                    menu_kit::spawn_menu_title(panel, "PAUSED", &theme, &font);

                    // Menu items — atlas-backed buttons matching main menu
                    for (i, label) in PAUSE_OPTIONS.iter().enumerate() {
                        menu_kit::spawn_menu_button(panel, i, label, &assets, &theme, &font);
                    }

                    // Status text
                    panel.spawn((
                        PauseMenuStatusText,
                        Text::new(""),
                        TextFont {
                            font: font.clone(),
                            font_size: 12.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.95, 0.75, 0.45)),
                    ));

                    // Hint
                    menu_kit::spawn_menu_footer(
                        panel,
                        "Up/Down: Select | Enter: Confirm | Esc: Resume",
                        &theme,
                        &font,
                    );
                });
        });
}

pub fn despawn_pause_menu(mut commands: Commands, query: Query<Entity, With<PauseMenuRoot>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
    commands.remove_resource::<PauseMenuState>();
}

// ═══════════════════════════════════════════════════════════════════════
// UPDATE / INTERACTION
// ═══════════════════════════════════════════════════════════════════════

pub fn update_pause_menu_visuals(
    state: Option<Res<PauseMenuState>>,
    mut query: Query<(&MenuItem, &mut ImageNode)>,
    mut status_query: Query<&mut Text, With<PauseMenuStatusText>>,
) {
    let Some(state) = state else { return };
    for (item, mut image_node) in &mut query {
        set_button_visual(&mut image_node, item.index == state.cursor);
    }

    let mut text = status_query.single_mut();
    text.0 = state.status_message.clone();
}

pub fn pause_menu_navigation(
    action: Res<MenuAction>,
    mut state: Option<ResMut<PauseMenuState>>,
    mut next_state: ResMut<NextState<GameState>>,
    active_slot: Res<ActiveSaveSlot>,
    mut save_writer: EventWriter<SaveRequestEvent>,
) {
    let Some(ref mut state) = state else { return };

    // Pointer hover → set cursor
    if let Some(idx) = action.set_cursor {
        if idx < PAUSE_OPTIONS.len() {
            state.cursor = idx;
        }
    }

    if action.move_down {
        if state.cursor < PAUSE_OPTIONS.len() - 1 {
            state.cursor += 1;
        }
    }
    if action.move_up {
        if state.cursor > 0 {
            state.cursor -= 1;
        }
    }

    if action.activate {
        match state.cursor {
            0 => {
                // Resume
                next_state.set(GameState::Playing);
            }
            1 => {
                let slot = active_slot.slot;
                state.status_message = format!("Saving Slot {}...", slot + 1);
                save_writer.send(SaveRequestEvent { slot });
            }
            2 => {
                // Quit to menu
                next_state.set(GameState::MainMenu);
            }
            _ => {}
        }
    }

    // Escape also resumes
    if action.cancel {
        next_state.set(GameState::Playing);
    }
}

pub fn handle_save_complete_in_pause_menu(
    mut complete_events: EventReader<SaveCompleteEvent>,
    mut state: Option<ResMut<PauseMenuState>>,
) {
    let Some(ref mut state) = state else { return };

    for ev in complete_events.read() {
        if ev.success {
            state.status_message = format!("Saved to Slot {}.", ev.slot + 1);
        } else {
            state.status_message = ev
                .error_message
                .clone()
                .unwrap_or_else(|| "Save failed.".to_string());
        }
    }
}
