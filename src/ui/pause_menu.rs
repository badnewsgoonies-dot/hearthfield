use bevy::prelude::*;
use crate::shared::*;
use super::UiFontHandle;

// ═══════════════════════════════════════════════════════════════════════
// MARKER COMPONENTS
// ═══════════════════════════════════════════════════════════════════════

#[derive(Component)]
pub struct PauseMenuRoot;

#[derive(Component)]
pub struct PauseMenuItem {
    pub index: usize,
}

#[derive(Component)]
pub struct PauseMenuItemText {
    pub index: usize,
}

/// Tracks pause menu selection
#[derive(Resource)]
pub struct PauseMenuState {
    pub cursor: usize,
}

const PAUSE_OPTIONS: &[&str] = &["Resume", "Save Game", "Quit to Menu"];

// ═══════════════════════════════════════════════════════════════════════
// SPAWN / DESPAWN
// ═══════════════════════════════════════════════════════════════════════

pub fn spawn_pause_menu(mut commands: Commands, font_handle: Res<UiFontHandle>) {
    commands.insert_resource(PauseMenuState { cursor: 0 });

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
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.6)),
        ))
        .with_children(|parent| {
            // Panel
            parent
                .spawn((
                    Node {
                        width: Val::Px(300.0),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        padding: UiRect::all(Val::Px(24.0)),
                        row_gap: Val::Px(12.0),
                        border: UiRect::all(Val::Px(3.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.1, 0.08, 0.06, 0.95)),
                    BorderColor(Color::srgb(0.5, 0.4, 0.25)),
                ))
                .with_children(|panel| {
                    // Title
                    panel.spawn((
                        Text::new("PAUSED"),
                        TextFont {
                            font: font.clone(),
                            font_size: 28.0,
                            ..default()
                        },
                        TextColor(Color::srgb(1.0, 0.9, 0.6)),
                    ));

                    // Menu items
                    for (i, label) in PAUSE_OPTIONS.iter().enumerate() {
                        panel
                            .spawn((
                                PauseMenuItem { index: i },
                                Node {
                                    width: Val::Px(220.0),
                                    height: Val::Px(36.0),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    border: UiRect::all(Val::Px(2.0)),
                                    ..default()
                                },
                                BackgroundColor(Color::srgba(0.2, 0.17, 0.14, 0.8)),
                                BorderColor(Color::srgba(0.4, 0.35, 0.3, 0.6)),
                            ))
                            .with_children(|item| {
                                item.spawn((
                                    PauseMenuItemText { index: i },
                                    Text::new(*label),
                                    TextFont {
                                        font: font.clone(),
                                        font_size: 18.0,
                                        ..default()
                                    },
                                    TextColor(Color::WHITE),
                                ));
                            });
                    }

                    // Hint
                    panel.spawn((
                        Text::new("Up/Down: Select | Enter: Confirm | Esc: Resume"),
                        TextFont {
                            font: font.clone(),
                            font_size: 10.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.5, 0.5, 0.5)),
                    ));
                });
        });
}

pub fn despawn_pause_menu(
    mut commands: Commands,
    query: Query<Entity, With<PauseMenuRoot>>,
) {
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
    mut query: Query<(&PauseMenuItem, &mut BackgroundColor, &mut BorderColor)>,
) {
    let Some(state) = state else { return };
    for (item, mut bg, mut border) in &mut query {
        if item.index == state.cursor {
            *bg = BackgroundColor(Color::srgba(0.35, 0.3, 0.2, 0.95));
            *border = BorderColor(Color::srgb(1.0, 0.84, 0.0));
        } else {
            *bg = BackgroundColor(Color::srgba(0.2, 0.17, 0.14, 0.8));
            *border = BorderColor(Color::srgba(0.4, 0.35, 0.3, 0.6));
        }
    }
}

pub fn pause_menu_navigation(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut state: Option<ResMut<PauseMenuState>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut app_exit: EventWriter<AppExit>,
) {
    let Some(ref mut state) = state else { return };

    if keyboard.just_pressed(KeyCode::ArrowDown) {
        if state.cursor < PAUSE_OPTIONS.len() - 1 {
            state.cursor += 1;
        }
    }
    if keyboard.just_pressed(KeyCode::ArrowUp) {
        if state.cursor > 0 {
            state.cursor -= 1;
        }
    }

    if keyboard.just_pressed(KeyCode::Enter) {
        match state.cursor {
            0 => {
                // Resume
                next_state.set(GameState::Playing);
            }
            1 => {
                // Save — we don't directly call save logic here; the save domain
                // should listen for a save trigger. For now, resume after "saving".
                // In a full implementation, we'd emit a SaveGameEvent.
                // For now, just resume.
                next_state.set(GameState::Playing);
            }
            2 => {
                // Quit to menu
                next_state.set(GameState::MainMenu);
            }
            _ => {}
        }
    }

    // Escape also resumes
    if keyboard.just_pressed(KeyCode::Escape) {
        next_state.set(GameState::Playing);
    }
}
