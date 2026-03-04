use bevy::prelude::*;

use crate::game::resources::{CareerProgression, DayClock, DayOutcome};
use crate::game::OfficeGameState;

use super::DaySummaryRoot;

#[derive(Component)]
pub(crate) struct ContinueButton;

/// Tracks whether we have already applied the summary data so we don't re-read
/// stale resources after rollover zeroes them.
#[derive(Resource, Default)]
pub struct DaySummarySnapshot {
    pub captured: bool,
    pub day_number: u32,
    pub completed_tasks: u32,
    pub failed_tasks: u32,
    pub salary_earned: i32,
    pub reputation_delta: i32,
    pub stress_delta: i32,
    pub level: u32,
    pub xp: u32,
    pub xp_threshold: u32,
}

pub fn spawn_day_summary(
    mut commands: Commands,
    outcome: Res<DayOutcome>,
    clock: Res<DayClock>,
    progression: Res<CareerProgression>,
    mut snapshot: ResMut<DaySummarySnapshot>,
) {
    // Capture snapshot before rollover clears values
    if !snapshot.captured {
        snapshot.captured = true;
        snapshot.day_number = clock.day_number;
        snapshot.completed_tasks = outcome.completed_tasks;
        snapshot.failed_tasks = outcome.failed_tasks;
        snapshot.salary_earned = outcome.salary_earned;
        snapshot.reputation_delta = outcome.reputation_delta;
        snapshot.stress_delta = outcome.stress_delta;
        snapshot.level = progression.level;
        snapshot.xp = progression.xp;
        snapshot.xp_threshold = progression.xp_for_next_level();
    }

    let snap = &*snapshot;

    let rep_text = if snap.reputation_delta >= 0 {
        format!("+{}", snap.reputation_delta)
    } else {
        format!("{}", snap.reputation_delta)
    };

    let stress_text = if snap.stress_delta >= 0 {
        format!("+{}", snap.stress_delta)
    } else {
        format!("{}", snap.stress_delta)
    };

    commands
        .spawn((
            DaySummaryRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: Val::Px(10.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.04, 0.04, 0.07, 0.95)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(format!("DAY {} COMPLETE", snap.day_number)),
                TextFont {
                    font_size: 36.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.85, 0.5)),
            ));

            parent.spawn(Node {
                height: Val::Px(12.0),
                ..default()
            });

            let lines = [
                format!(
                    "Tasks completed: {}  /  Tasks failed: {}",
                    snap.completed_tasks, snap.failed_tasks
                ),
                format!("Salary earned: ${}", snap.salary_earned),
                format!("Reputation change: {}", rep_text),
                format!("Stress change: {}", stress_text),
                format!(
                    "Level: {} (XP: {}/{})",
                    snap.level, snap.xp, snap.xp_threshold
                ),
            ];

            for line in &lines {
                parent.spawn((
                    Text::new(line.clone()),
                    TextFont {
                        font_size: 18.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.8, 0.8, 0.8)),
                ));
            }

            parent.spawn(Node {
                height: Val::Px(24.0),
                ..default()
            });

            // Continue button
            parent
                .spawn((
                    ContinueButton,
                    Button,
                    Node {
                        width: Val::Px(280.0),
                        height: Val::Px(48.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.2, 0.5, 0.3)),
                ))
                .with_children(|btn| {
                    btn.spawn((
                        Text::new(format!("Continue to Day {}", snap.day_number + 1)),
                        TextFont {
                            font_size: 22.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });
        });
}

pub fn despawn_day_summary(
    mut commands: Commands,
    query: Query<Entity, With<DaySummaryRoot>>,
    mut snapshot: ResMut<DaySummarySnapshot>,
) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
    snapshot.captured = false;
}

pub fn handle_day_summary_input(
    button_query: Query<&Interaction, (Changed<Interaction>, With<ContinueButton>)>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<OfficeGameState>>,
) {
    let mut should_continue = false;

    for interaction in &button_query {
        if *interaction == Interaction::Pressed {
            should_continue = true;
        }
    }

    if keyboard.just_pressed(KeyCode::Enter) || keyboard.just_pressed(KeyCode::Space) {
        should_continue = true;
    }

    if should_continue {
        next_state.set(OfficeGameState::InDay);
    }
}
