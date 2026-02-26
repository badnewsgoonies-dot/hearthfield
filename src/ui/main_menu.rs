use bevy::prelude::*;
use crate::shared::*;

// ═══════════════════════════════════════════════════════════════════════
// MARKER COMPONENTS
// ═══════════════════════════════════════════════════════════════════════

#[derive(Component)]
pub struct MainMenuRoot;

#[derive(Component)]
pub struct MainMenuItem {
    pub index: usize,
}

#[derive(Component)]
pub struct MainMenuItemText {
    pub index: usize,
}

/// Tracks main menu selection
#[derive(Resource)]
pub struct MainMenuState {
    pub cursor: usize,
}

const MAIN_MENU_OPTIONS: &[&str] = &["New Game", "Load Game", "Quit"];

// ═══════════════════════════════════════════════════════════════════════
// SPAWN / DESPAWN
// ═══════════════════════════════════════════════════════════════════════

pub fn spawn_main_menu(mut commands: Commands) {
    commands.insert_resource(MainMenuState { cursor: 0 });

    commands
        .spawn((
            MainMenuRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(30.0),
                ..default()
            },
            BackgroundColor(Color::srgb(0.12, 0.18, 0.08)),
        ))
        .with_children(|parent| {
            // Game title
            parent.spawn((
                Text::new("HEARTHFIELD"),
                TextFont {
                    font_size: 52.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.9, 0.5)),
            ));

            // Subtitle
            parent.spawn((
                Text::new("A Farming & Life Simulator"),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.8, 0.6)),
            ));

            // Menu options container
            parent
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        row_gap: Val::Px(8.0),
                        ..default()
                    },
                ))
                .with_children(|menu| {
                    for (i, label) in MAIN_MENU_OPTIONS.iter().enumerate() {
                        menu.spawn((
                                MainMenuItem { index: i },
                                Node {
                                    width: Val::Px(240.0),
                                    height: Val::Px(42.0),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    border: UiRect::all(Val::Px(2.0)),
                                    ..default()
                                },
                                BackgroundColor(Color::srgba(0.15, 0.12, 0.08, 0.9)),
                                BorderColor(Color::srgba(0.4, 0.35, 0.25, 0.7)),
                            ))
                            .with_children(|item| {
                                item.spawn((
                                    MainMenuItemText { index: i },
                                    Text::new(*label),
                                    TextFont {
                                        font_size: 20.0,
                                        ..default()
                                    },
                                    TextColor(Color::WHITE),
                                ));
                            });
                    }
                });

            // Version text
            parent.spawn((
                Text::new("v0.1.0 - Early Development"),
                TextFont {
                    font_size: 11.0,
                    ..default()
                },
                TextColor(Color::srgb(0.4, 0.45, 0.35)),
            ));
        });
}

pub fn despawn_main_menu(
    mut commands: Commands,
    query: Query<Entity, With<MainMenuRoot>>,
) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
    commands.remove_resource::<MainMenuState>();
}

// ═══════════════════════════════════════════════════════════════════════
// UPDATE / INTERACTION
// ═══════════════════════════════════════════════════════════════════════

pub fn update_main_menu_visuals(
    state: Option<Res<MainMenuState>>,
    mut query: Query<(&MainMenuItem, &mut BackgroundColor, &mut BorderColor)>,
) {
    let Some(state) = state else { return };
    for (item, mut bg, mut border) in &mut query {
        if item.index == state.cursor {
            *bg = BackgroundColor(Color::srgba(0.25, 0.22, 0.12, 0.95));
            *border = BorderColor(Color::srgb(1.0, 0.84, 0.0));
        } else {
            *bg = BackgroundColor(Color::srgba(0.15, 0.12, 0.08, 0.9));
            *border = BorderColor(Color::srgba(0.4, 0.35, 0.25, 0.7));
        }
    }
}

pub fn main_menu_navigation(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut state: Option<ResMut<MainMenuState>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut app_exit: EventWriter<AppExit>,
) {
    let Some(ref mut state) = state else { return };

    if keyboard.just_pressed(KeyCode::ArrowDown) {
        if state.cursor < MAIN_MENU_OPTIONS.len() - 1 {
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
                // New Game — data is already loaded, go straight to Playing
                next_state.set(GameState::Playing);
            }
            1 => {
                // Load Game — for now, same as New Game
                next_state.set(GameState::Playing);
            }
            2 => {
                // Quit
                app_exit.send(AppExit::Success);
            }
            _ => {}
        }
    }
}
