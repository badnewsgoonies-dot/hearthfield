use bevy::prelude::*;

use crate::game::save::SaveSlotRequest;
use crate::game::OfficeGameState;

use super::PauseMenuRoot;

#[derive(Component)]
pub(crate) struct ResumeButton;

#[derive(Component)]
pub(crate) struct SaveButton;

#[derive(Component)]
pub(crate) struct QuitToMenuButton;

pub fn spawn_pause_menu(mut commands: Commands) {
    commands
        .spawn((
            PauseMenuRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: Val::Px(16.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.65)),
            GlobalZIndex(10),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("PAUSED"),
                TextFont {
                    font_size: 42.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            parent.spawn(Node {
                height: Val::Px(20.0),
                ..default()
            });

            // Resume
            parent
                .spawn((
                    ResumeButton,
                    Button,
                    Node {
                        width: Val::Px(220.0),
                        height: Val::Px(48.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.2, 0.5, 0.3)),
                ))
                .with_children(|btn| {
                    btn.spawn((
                        Text::new("Resume"),
                        TextFont {
                            font_size: 22.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });

            // Save Game
            parent
                .spawn((
                    SaveButton,
                    Button,
                    Node {
                        width: Val::Px(220.0),
                        height: Val::Px(48.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.3, 0.3, 0.5)),
                ))
                .with_children(|btn| {
                    btn.spawn((
                        Text::new("Save Game"),
                        TextFont {
                            font_size: 22.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });

            // Quit to Menu
            parent
                .spawn((
                    QuitToMenuButton,
                    Button,
                    Node {
                        width: Val::Px(220.0),
                        height: Val::Px(48.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.5, 0.2, 0.2)),
                ))
                .with_children(|btn| {
                    btn.spawn((
                        Text::new("Quit to Menu"),
                        TextFont {
                            font_size: 22.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });
        });
}

pub fn despawn_pause_menu(mut commands: Commands, query: Query<Entity, With<PauseMenuRoot>>) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}

pub fn handle_pause_input(
    resume_query: Query<&Interaction, (Changed<Interaction>, With<ResumeButton>)>,
    save_query: Query<&Interaction, (Changed<Interaction>, With<SaveButton>)>,
    quit_query: Query<&Interaction, (Changed<Interaction>, With<QuitToMenuButton>)>,
    mut next_state: ResMut<NextState<OfficeGameState>>,
    mut save_writer: EventWriter<SaveSlotRequest>,
) {
    for interaction in &resume_query {
        if *interaction == Interaction::Pressed {
            next_state.set(OfficeGameState::InDay);
        }
    }

    for interaction in &save_query {
        if *interaction == Interaction::Pressed {
            save_writer.send(SaveSlotRequest { slot: 0 });
            next_state.set(OfficeGameState::InDay);
        }
    }

    for interaction in &quit_query {
        if *interaction == Interaction::Pressed {
            next_state.set(OfficeGameState::MainMenu);
        }
    }
}
