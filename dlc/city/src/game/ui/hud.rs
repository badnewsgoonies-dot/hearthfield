use bevy::prelude::*;

use crate::game::resources::{
    format_clock, DayClock, InboxState, OfficeRules, PlayerMindState, WorkerStats,
};

use super::HudRoot;

#[derive(Component)]
pub(crate) struct HudTimeText;

#[derive(Component)]
pub(crate) struct HudEnergyText;

#[derive(Component)]
pub(crate) struct HudStressText;

#[derive(Component)]
pub(crate) struct HudFocusText;

#[derive(Component)]
pub(crate) struct HudMoneyText;

#[derive(Component)]
pub(crate) struct HudReputationText;

#[derive(Component)]
pub(crate) struct HudInboxText;

#[derive(Component)]
pub(crate) struct HudInterruptionsText;

pub fn spawn_hud(mut commands: Commands) {
    commands
        .spawn((
            HudRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::SpaceBetween,
                ..default()
            },
            // fully transparent so game shows through
            BackgroundColor(Color::NONE),
            // Don't block clicks on the game world
            PickingBehavior::IGNORE,
        ))
        .with_children(|parent| {
            // === Top bar ===
            parent
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(36.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
                ))
                .with_children(|bar| {
                    bar.spawn((
                        HudTimeText,
                        Text::new("Day 1 -- 09:00"),
                        TextFont {
                            font_size: 22.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });

            // === Middle row: left stats + right inbox ===
            parent
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::SpaceBetween,
                        align_items: AlignItems::FlexStart,
                        padding: UiRect::all(Val::Px(8.0)),
                        ..default()
                    },
                    PickingBehavior::IGNORE,
                ))
                .with_children(|row| {
                    // Left stats panel
                    row.spawn((
                        Node {
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(4.0),
                            padding: UiRect::all(Val::Px(6.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.55)),
                    ))
                    .with_children(|panel| {
                        panel.spawn((
                            HudEnergyText,
                            Text::new("Energy: 100/100"),
                            TextFont {
                                font_size: 16.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.3, 0.9, 0.3)),
                        ));
                        panel.spawn((
                            HudStressText,
                            Text::new("Stress: 18/100"),
                            TextFont {
                                font_size: 16.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.3, 0.9, 0.3)),
                        ));
                        panel.spawn((
                            HudFocusText,
                            Text::new("Focus: 76/100"),
                            TextFont {
                                font_size: 16.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.5, 0.7, 1.0)),
                        ));
                        panel.spawn((
                            HudMoneyText,
                            Text::new("Money: $0"),
                            TextFont {
                                font_size: 16.0,
                                ..default()
                            },
                            TextColor(Color::srgb(1.0, 0.85, 0.3)),
                        ));
                        panel.spawn((
                            HudReputationText,
                            Text::new("Reputation: 0"),
                            TextFont {
                                font_size: 16.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.7, 0.7, 0.9)),
                        ));
                    });

                    // Right panel: inbox + interruptions
                    row.spawn((
                        Node {
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(4.0),
                            padding: UiRect::all(Val::Px(6.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.55)),
                    ))
                    .with_children(|panel| {
                        panel.spawn((
                            HudInboxText,
                            Text::new("Inbox: 12/12"),
                            TextFont {
                                font_size: 16.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.9, 0.8, 0.4)),
                        ));
                        panel.spawn((
                            HudInterruptionsText,
                            Text::new(""),
                            TextFont {
                                font_size: 16.0,
                                ..default()
                            },
                            TextColor(Color::srgb(1.0, 0.3, 0.3)),
                        ));
                    });
                });

            // === Bottom bar: key hints ===
            parent
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(28.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
                ))
                .with_children(|bar| {
                    bar.spawn((
                        Text::new(
                            "P:Process  C:Coffee  N:Wait  I:Interrupt  1:Calm  2:Panic  M:Manager  H:Help",
                        ),
                        TextFont {
                            font_size: 13.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.7, 0.7, 0.7)),
                    ));
                });
        });
}

pub fn despawn_hud(mut commands: Commands, query: Query<Entity, With<HudRoot>>) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}

fn stat_color(value: i32, max: i32, invert: bool) -> Color {
    let ratio = if max == 0 {
        0.0
    } else {
        (value as f32 / max as f32).clamp(0.0, 1.0)
    };
    let r = if invert { ratio } else { 1.0 - ratio };
    let g = if invert { 1.0 - ratio } else { ratio };
    Color::srgb(r, g, 0.2)
}

#[allow(clippy::type_complexity, clippy::too_many_arguments)]
pub fn update_hud(
    clock: Res<DayClock>,
    stats: Res<WorkerStats>,
    rules: Res<OfficeRules>,
    inbox: Res<InboxState>,
    mind: Res<PlayerMindState>,
    mut time_q: Query<(&mut Text, &mut TextColor), With<HudTimeText>>,
    mut energy_q: Query<
        (&mut Text, &mut TextColor),
        (With<HudEnergyText>, Without<HudTimeText>),
    >,
    mut stress_q: Query<
        (&mut Text, &mut TextColor),
        (
            With<HudStressText>,
            Without<HudTimeText>,
            Without<HudEnergyText>,
        ),
    >,
    mut focus_q: Query<
        (&mut Text, &mut TextColor),
        (
            With<HudFocusText>,
            Without<HudTimeText>,
            Without<HudEnergyText>,
            Without<HudStressText>,
        ),
    >,
    mut money_q: Query<
        &mut Text,
        (
            With<HudMoneyText>,
            Without<HudTimeText>,
            Without<HudEnergyText>,
            Without<HudStressText>,
            Without<HudFocusText>,
        ),
    >,
    mut rep_q: Query<
        &mut Text,
        (
            With<HudReputationText>,
            Without<HudTimeText>,
            Without<HudEnergyText>,
            Without<HudStressText>,
            Without<HudFocusText>,
            Without<HudMoneyText>,
        ),
    >,
    mut inbox_q: Query<
        &mut Text,
        (
            With<HudInboxText>,
            Without<HudTimeText>,
            Without<HudEnergyText>,
            Without<HudStressText>,
            Without<HudFocusText>,
            Without<HudMoneyText>,
            Without<HudReputationText>,
        ),
    >,
    mut intr_q: Query<
        &mut Text,
        (
            With<HudInterruptionsText>,
            Without<HudTimeText>,
            Without<HudEnergyText>,
            Without<HudStressText>,
            Without<HudFocusText>,
            Without<HudMoneyText>,
            Without<HudReputationText>,
            Without<HudInboxText>,
        ),
    >,
) {
    if let Ok((mut text, mut color)) = time_q.get_single_mut() {
        **text = format!(
            "Day {} -- {}",
            clock.day_number,
            format_clock(clock.current_minute)
        );
        color.0 = Color::WHITE;
    }

    if let Ok((mut text, mut color)) = energy_q.get_single_mut() {
        **text = format!("Energy: {}/{}", stats.energy, rules.max_energy);
        color.0 = stat_color(stats.energy, rules.max_energy, false);
    }

    if let Ok((mut text, mut color)) = stress_q.get_single_mut() {
        **text = format!("Stress: {}/{}", stats.stress, rules.max_stress);
        color.0 = stat_color(stats.stress, rules.max_stress, true);
    }

    if let Ok((mut text, mut color)) = focus_q.get_single_mut() {
        **text = format!("Focus: {}/{}", stats.focus, rules.max_focus);
        color.0 = stat_color(stats.focus, rules.max_focus, false);
    }

    if let Ok(mut text) = money_q.get_single_mut() {
        **text = format!("Money: ${}", stats.money);
    }

    if let Ok(mut text) = rep_q.get_single_mut() {
        **text = format!("Reputation: {}", stats.reputation);
    }

    if let Ok(mut text) = inbox_q.get_single_mut() {
        **text = format!(
            "Inbox: {}/{}",
            inbox.remaining_items, rules.starting_inbox_items
        );
    }

    if let Ok(mut text) = intr_q.get_single_mut() {
        if mind.pending_interruptions > 0 {
            **text = format!("Pending interruptions: {}", mind.pending_interruptions);
        } else {
            **text = String::new();
        }
    }
}
