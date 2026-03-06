use bevy::prelude::*;

use crate::game::save::LoadSlotRequest;
use crate::game::OfficeGameState;

use super::MainMenuRoot;

#[derive(Component)]
pub(crate) struct NewGameButton;

#[derive(Component)]
pub(crate) struct LoadGameButton;

#[derive(Component)]
pub(crate) struct QuitButton;

#[derive(Component)]
pub(crate) struct MenuButtonLabel;

fn button_idle_color(kind: &str) -> Color {
    match kind {
        "new" => Color::srgb(0.18, 0.34, 0.24),
        "load" => Color::srgb(0.20, 0.24, 0.38),
        "quit" => Color::srgb(0.34, 0.16, 0.18),
        _ => Color::srgb(0.2, 0.2, 0.2),
    }
}

fn button_hover_color(kind: &str) -> Color {
    match kind {
        "new" => Color::srgb(0.28, 0.48, 0.34),
        "load" => Color::srgb(0.30, 0.36, 0.52),
        "quit" => Color::srgb(0.48, 0.24, 0.28),
        _ => Color::srgb(0.32, 0.32, 0.32),
    }
}

fn button_pressed_color(kind: &str) -> Color {
    match kind {
        "new" => Color::srgb(0.84, 0.74, 0.36),
        "load" => Color::srgb(0.80, 0.70, 0.34),
        "quit" => Color::srgb(0.74, 0.56, 0.30),
        _ => Color::srgb(0.84, 0.74, 0.36),
    }
}

pub fn spawn_main_menu(mut commands: Commands) {
    commands
        .spawn((
            MainMenuRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: Val::Px(14.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.05, 0.05, 0.08, 1.0)),
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new("CITY OFFICE WORKER"),
                TextFont {
                    font_size: 48.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.85, 0.6)),
            ));

            // Subtitle
            parent.spawn((
                Text::new("A Hearthfield DLC"),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::srgb(0.6, 0.6, 0.6)),
            ));

            parent.spawn((
                Text::new("Inbox triage, interruptions, office politics, and survival."),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(0.72, 0.72, 0.76)),
            ));

            // Spacer
            parent.spawn(Node {
                height: Val::Px(24.0),
                ..default()
            });

            // New Game button
            parent
                .spawn((
                    NewGameButton,
                    Button,
                    Node {
                        width: Val::Px(240.0),
                        height: Val::Px(52.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(button_idle_color("new")),
                ))
                .with_children(|btn| {
                    btn.spawn((
                        MenuButtonLabel,
                        Text::new("New Game"),
                        TextFont {
                            font_size: 22.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });

            // Load Game button
            parent
                .spawn((
                    LoadGameButton,
                    Button,
                    Node {
                        width: Val::Px(240.0),
                        height: Val::Px(52.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(button_idle_color("load")),
                ))
                .with_children(|btn| {
                    btn.spawn((
                        MenuButtonLabel,
                        Text::new("Load Game"),
                        TextFont {
                            font_size: 22.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });

            // Quit button
            parent
                .spawn((
                    QuitButton,
                    Button,
                    Node {
                        width: Val::Px(240.0),
                        height: Val::Px(52.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(button_idle_color("quit")),
                ))
                .with_children(|btn| {
                    btn.spawn((
                        MenuButtonLabel,
                        Text::new("Quit"),
                        TextFont {
                            font_size: 22.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });

            parent.spawn(Node {
                height: Val::Px(10.0),
                ..default()
            });

            parent.spawn((
                Text::new("Click a button to start a shift or resume Slot 1."),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.66, 0.66, 0.70)),
            ));
        });
}

pub fn despawn_main_menu(mut commands: Commands, query: Query<Entity, With<MainMenuRoot>>) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}

#[allow(clippy::type_complexity)]
pub fn handle_main_menu_input(
    mut new_game_query: Query<
        (&Interaction, &mut BackgroundColor, &Children),
        (Changed<Interaction>, With<NewGameButton>),
    >,
    mut load_game_query: Query<
        (&Interaction, &mut BackgroundColor, &Children),
        (Changed<Interaction>, With<LoadGameButton>),
    >,
    mut quit_query: Query<
        (&Interaction, &mut BackgroundColor, &Children),
        (Changed<Interaction>, With<QuitButton>),
    >,
    mut label_query: Query<&mut TextColor, With<MenuButtonLabel>>,
    mut next_state: ResMut<NextState<OfficeGameState>>,
    mut load_writer: EventWriter<LoadSlotRequest>,
    mut exit_writer: EventWriter<AppExit>,
) {
    for (interaction, mut bg, children) in &mut new_game_query {
        bg.0 = match *interaction {
            Interaction::Pressed => button_pressed_color("new"),
            Interaction::Hovered => button_hover_color("new"),
            Interaction::None => button_idle_color("new"),
        };
        let text_color = if *interaction == Interaction::Pressed {
            Color::srgb(0.10, 0.10, 0.14)
        } else {
            Color::WHITE
        };
        for child in children.iter() {
            if let Ok(mut label) = label_query.get_mut(*child) {
                label.0 = text_color;
            }
        }
        if *interaction == Interaction::Pressed {
            next_state.set(OfficeGameState::InDay);
        }
    }

    for (interaction, mut bg, children) in &mut load_game_query {
        bg.0 = match *interaction {
            Interaction::Pressed => button_pressed_color("load"),
            Interaction::Hovered => button_hover_color("load"),
            Interaction::None => button_idle_color("load"),
        };
        let text_color = if *interaction == Interaction::Pressed {
            Color::srgb(0.10, 0.10, 0.14)
        } else {
            Color::WHITE
        };
        for child in children.iter() {
            if let Ok(mut label) = label_query.get_mut(*child) {
                label.0 = text_color;
            }
        }
        if *interaction == Interaction::Pressed {
            load_writer.send(LoadSlotRequest { slot: 0 });
            next_state.set(OfficeGameState::InDay);
        }
    }

    for (interaction, mut bg, children) in &mut quit_query {
        bg.0 = match *interaction {
            Interaction::Pressed => button_pressed_color("quit"),
            Interaction::Hovered => button_hover_color("quit"),
            Interaction::None => button_idle_color("quit"),
        };
        let text_color = if *interaction == Interaction::Pressed {
            Color::srgb(0.10, 0.10, 0.14)
        } else {
            Color::WHITE
        };
        for child in children.iter() {
            if let Ok(mut label) = label_query.get_mut(*child) {
                label.0 = text_color;
            }
        }
        if *interaction == Interaction::Pressed {
            exit_writer.send(AppExit::Success);
        }
    }
}
