use bevy::prelude::*;

use crate::game::components::{InboxAvatar, OfficeWorker, WorkerAvatar};
use crate::game::resources::{
    format_clock, DayClock, DayStats, InboxState, OfficeEconomyRules, OfficeRules,
    PlayerCareerState, PlayerMindState, SocialGraphState, TaskBoard,
};
use crate::game::OfficeGameState;

use super::task_board::seed_task_board;

fn retain_single_entity<I>(entities: I, commands: &mut Commands) -> Option<Entity>
where
    I: IntoIterator<Item = Entity>,
{
    let mut iter = entities.into_iter();
    let primary = iter.next();
    for duplicate in iter {
        commands.entity(duplicate).despawn_recursive();
    }
    primary
}

pub fn boot_to_main_menu(mut next_state: ResMut<NextState<OfficeGameState>>) {
    next_state.set(OfficeGameState::MainMenu);
}

pub fn toggle_pause(
    keyboard: Res<ButtonInput<KeyCode>>,
    game_state: Res<State<OfficeGameState>>,
    mut next_state: ResMut<NextState<OfficeGameState>>,
) {
    if !keyboard.just_pressed(KeyCode::Escape) {
        return;
    }

    match game_state.get() {
        OfficeGameState::InDay => next_state.set(OfficeGameState::Paused),
        OfficeGameState::Paused => next_state.set(OfficeGameState::InDay),
        _ => {}
    }
}

#[allow(clippy::too_many_arguments)]
pub fn setup_scene(
    mut commands: Commands,
    rules: Res<OfficeRules>,
    mut economy: ResMut<OfficeEconomyRules>,
    mut inbox: ResMut<InboxState>,
    mut clock: ResMut<DayClock>,
    mut mind: ResMut<PlayerMindState>,
    mut career: ResMut<PlayerCareerState>,
    mut social: ResMut<SocialGraphState>,
    mut stats: ResMut<DayStats>,
    task_board: Option<ResMut<TaskBoard>>,
    worker_entities: Query<Entity, With<OfficeWorker>>,
    inbox_entities: Query<Entity, With<InboxAvatar>>,
    camera_entities: Query<Entity, With<Camera2d>>,
) {
    economy.normalize();

    inbox.remaining_items = rules.starting_inbox_items;
    clock.current_minute = rules.day_start_minute;
    clock.ended = false;
    mind.stress = rules.starting_stress.clamp(0, rules.max_stress);
    mind.focus = rules.starting_focus.clamp(0, rules.max_focus);
    mind.pending_interruptions = 0;
    career.reputation = rules
        .starting_reputation
        .clamp(-rules.max_reputation, rules.max_reputation);
    social.normalize();
    stats.processed_items = 0;
    stats.coffee_breaks = 0;
    stats.wait_actions = 0;
    stats.failed_process_attempts = 0;
    stats.interruptions_triggered = 0;
    stats.calm_responses = 0;
    stats.panic_responses = 0;
    stats.manager_checkins = 0;
    stats.coworker_helps = 0;

    let seeded_board = seed_task_board(
        clock.day_number,
        inbox.remaining_items,
        rules.day_end_minute,
    );
    if let Some(mut existing_task_board) = task_board {
        *existing_task_board = seeded_board;
    } else {
        commands.insert_resource(seeded_board);
    }

    if retain_single_entity(camera_entities.iter(), &mut commands).is_none() {
        commands.spawn((Camera2d,));
    }

    if let Some(entity) = retain_single_entity(worker_entities.iter(), &mut commands) {
        commands.entity(entity).insert((
            OfficeWorker {
                energy: rules.max_energy,
            },
            WorkerAvatar,
            Sprite::from_color(Color::srgb(0.2, 0.8, 0.4), Vec2::new(130.0, 130.0)),
            Transform::from_xyz(-180.0, 0.0, 0.0),
        ));
    } else {
        commands.spawn((
            OfficeWorker {
                energy: rules.max_energy,
            },
            WorkerAvatar,
            Sprite::from_color(Color::srgb(0.2, 0.8, 0.4), Vec2::new(130.0, 130.0)),
            Transform::from_xyz(-180.0, 0.0, 0.0),
        ));
    }

    if let Some(entity) = retain_single_entity(inbox_entities.iter(), &mut commands) {
        commands.entity(entity).insert((
            InboxAvatar,
            Sprite::from_color(Color::srgb(0.8, 0.75, 0.2), Vec2::new(120.0, 120.0)),
            Transform::from_xyz(190.0, 0.0, 0.0),
        ));
    } else {
        commands.spawn((
            InboxAvatar,
            Sprite::from_color(Color::srgb(0.8, 0.75, 0.2), Vec2::new(120.0, 120.0)),
            Transform::from_xyz(190.0, 0.0, 0.0),
        ));
    }

    info!(
        "Day {} starts at {} with {} inbox items. Stress: {}, focus: {}, reputation: {}.",
        clock.day_number,
        format_clock(clock.current_minute),
        inbox.remaining_items,
        mind.stress,
        mind.focus,
        career.reputation
    );
}
