//! Full-screen mission board — lists available missions with accept button.

use bevy::prelude::*;
use crate::shared::*;

#[derive(Component)]
pub struct MissionScreenRoot;

#[derive(Component)]
pub struct MissionEntry(pub usize);

pub fn spawn_mission_screen(
    mut commands: Commands,
    font: Res<UiFontHandle>,
    board: Res<MissionBoard>,
) {
    let title_style = TextFont {
        font: font.0.clone(),
        font_size: 24.0,
        ..default()
    };
    let item_style = TextFont {
        font: font.0.clone(),
        font_size: 14.0,
        ..default()
    };
    let btn_style = TextFont {
        font: font.0.clone(),
        font_size: 13.0,
        ..default()
    };

    commands
        .spawn((
            MissionScreenRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(24.0)),
                row_gap: Val::Px(8.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.05, 0.05, 0.1, 0.95)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("MISSION BOARD"),
                title_style,
                TextColor(Color::srgb(0.9, 0.8, 0.3)),
            ));

            for (i, mission) in board.available.iter().enumerate() {
                parent
                    .spawn((
                        MissionEntry(i),
                        Node {
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            column_gap: Val::Px(12.0),
                            padding: UiRect::all(Val::Px(6.0)),
                            width: Val::Percent(80.0),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.15, 0.15, 0.2, 0.9)),
                    ))
                    .with_children(|row| {
                        let info = format!(
                            "{} — {} → {} | {} gold | {} XP",
                            mission.title,
                            mission.origin.display_name(),
                            mission.destination.display_name(),
                            mission.reward_gold,
                            mission.reward_xp,
                        );
                        row.spawn((
                            Text::new(info),
                            item_style.clone(),
                            TextColor(Color::WHITE),
                            Node {
                                flex_grow: 1.0,
                                ..default()
                            },
                        ));
                        row.spawn((
                            Button,
                            MissionEntry(i),
                            Node {
                                padding: UiRect::axes(Val::Px(14.0), Val::Px(4.0)),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.2, 0.5, 0.2)),
                        ))
                        .with_children(|btn| {
                            btn.spawn((
                                Text::new("Accept"),
                                btn_style.clone(),
                                TextColor(Color::WHITE),
                            ));
                        });
                    });
            }

            if board.available.is_empty() {
                parent.spawn((
                    Text::new("No missions available."),
                    item_style,
                    TextColor(Color::srgb(0.6, 0.6, 0.6)),
                ));
            }
        });
}

pub fn despawn_mission_screen(
    mut commands: Commands,
    query: Query<Entity, With<MissionScreenRoot>>,
) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}

pub fn handle_mission_screen_input(
    interaction_q: Query<(&Interaction, &MissionEntry), Changed<Interaction>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    board: Res<MissionBoard>,
    mut mission_events: EventWriter<MissionAcceptedEvent>,
    mut next_state: ResMut<NextState<GameState>>,
    mut toast_events: EventWriter<ToastEvent>,
) {
    for (interaction, entry) in &interaction_q {
        if *interaction == Interaction::Pressed {
            if let Some(mission) = board.available.get(entry.0) {
                mission_events.send(MissionAcceptedEvent {
                    mission_id: mission.id.clone(),
                });
                toast_events.send(ToastEvent {
                    message: format!("Mission accepted: {}", mission.title),
                    duration_secs: 3.0,
                });
                next_state.set(GameState::Playing);
            }
        }
    }

    if keyboard.just_pressed(KeyCode::Escape) {
        next_state.set(GameState::Playing);
    }
}
