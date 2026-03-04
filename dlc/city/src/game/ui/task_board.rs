use bevy::prelude::*;

use crate::game::resources::{format_clock, TaskBoard, TaskKind, TaskPriority};

use super::TaskBoardRoot;

#[derive(Component)]
pub(crate) struct TaskBoardContent;

pub fn spawn_task_board(mut commands: Commands) {
    commands
        .spawn((
            TaskBoardRoot,
            Node {
                position_type: PositionType::Absolute,
                right: Val::Px(8.0),
                top: Val::Px(90.0),
                width: Val::Px(260.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(4.0),
                padding: UiRect::all(Val::Px(8.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.6)),
            GlobalZIndex(1),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("TASK BOARD"),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.8, 0.4)),
            ));

            parent.spawn((
                TaskBoardContent,
                Text::new(""),
                TextFont {
                    font_size: 13.0,
                    ..default()
                },
                TextColor(Color::srgb(0.75, 0.75, 0.75)),
            ));
        });
}

pub fn despawn_task_board(mut commands: Commands, query: Query<Entity, With<TaskBoardRoot>>) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}

fn kind_label(kind: TaskKind) -> &'static str {
    match kind {
        TaskKind::DataEntry => "DataEntry",
        TaskKind::Filing => "Filing",
        TaskKind::EmailTriage => "Email",
        TaskKind::PermitReview => "Permit",
    }
}

fn priority_label(priority: TaskPriority) -> &'static str {
    match priority {
        TaskPriority::Low => "LOW",
        TaskPriority::Medium => "MED",
        TaskPriority::High => "HIGH",
        TaskPriority::Critical => "CRIT",
    }
}

pub fn update_task_board(
    board: Res<TaskBoard>,
    mut content_q: Query<(&mut Text, &mut TextColor), With<TaskBoardContent>>,
) {
    let Ok((mut text, mut color)) = content_q.get_single_mut() else {
        return;
    };

    let mut buf = String::new();

    for task in &board.active {
        let pct = (task.progress * 100.0) as u32;
        let deadline_str = format_clock(task.deadline_minute as u32);
        buf.push_str(&format!(
            "{} [{}] {}% Due:{}\n",
            kind_label(task.kind),
            priority_label(task.priority),
            pct,
            deadline_str,
        ));
    }

    buf.push_str(&format!(
        "\n{} done / {} failed",
        board.completed_today.len(),
        board.failed_today.len()
    ));

    **text = buf;
    color.0 = Color::srgb(0.75, 0.75, 0.75);
}
