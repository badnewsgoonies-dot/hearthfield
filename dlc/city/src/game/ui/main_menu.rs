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
                row_gap: Val::Px(16.0),
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

            // Spacer
            parent.spawn(Node {
                height: Val::Px(32.0),
                ..default()
            });

            // New Game button
            parent
                .spawn((
                    NewGameButton,
                    Button,
                    Node {
                        width: Val::Px(220.0),
                        height: Val::Px(50.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.2, 0.5, 0.3)),
                ))
                .with_children(|btn| {
                    btn.spawn((
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
                        width: Val::Px(220.0),
                        height: Val::Px(50.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.3, 0.3, 0.5)),
                ))
                .with_children(|btn| {
                    btn.spawn((
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
                        width: Val::Px(220.0),
                        height: Val::Px(50.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.5, 0.2, 0.2)),
                ))
                .with_children(|btn| {
                    btn.spawn((
                        Text::new("Quit"),
                        TextFont {
                            font_size: 22.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });
        });
}

pub fn despawn_main_menu(mut commands: Commands, query: Query<Entity, With<MainMenuRoot>>) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}

pub fn handle_main_menu_input(
    new_game_query: Query<&Interaction, (Changed<Interaction>, With<NewGameButton>)>,
    load_game_query: Query<&Interaction, (Changed<Interaction>, With<LoadGameButton>)>,
    quit_query: Query<&Interaction, (Changed<Interaction>, With<QuitButton>)>,
    mut next_state: ResMut<NextState<OfficeGameState>>,
    mut load_writer: EventWriter<LoadSlotRequest>,
    mut exit_writer: EventWriter<AppExit>,
) {
    for interaction in &new_game_query {
        if *interaction == Interaction::Pressed {
            next_state.set(OfficeGameState::InDay);
        }
    }

    for interaction in &load_game_query {
        if *interaction == Interaction::Pressed {
            load_writer.send(LoadSlotRequest { slot: 0 });
            next_state.set(OfficeGameState::InDay);
        }
    }

    for interaction in &quit_query {
        if *interaction == Interaction::Pressed {
            exit_writer.send(AppExit::Success);
        }
    }
}
