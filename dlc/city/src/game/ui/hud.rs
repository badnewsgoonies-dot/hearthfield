use bevy::prelude::*;

use crate::game::events::RelationshipMilestone;
use crate::game::resources::{
    format_clock, CareerProgression, CoworkerRole, DayClock, InboxState, MilestoneKind,
    OfficeRules, PlayerMindState, SocialGraphState, ToastState, UnlockCatalogState, WorkerStats,
};

use super::{HudCoworkerContent, HudCoworkerPanel, HudRoot, ToastRoot};

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

#[derive(Component)]
pub(crate) struct HudLevelText;

#[derive(Component)]
pub(crate) struct HudStreakText;

#[derive(Component)]
pub(crate) struct HudUnlocksText;

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
                        // Career readout
                        panel.spawn((
                            HudLevelText,
                            Text::new("Lv.1 \u{25a0}\u{25a0}\u{25a1}\u{25a1}\u{25a1}\u{25a1}\u{25a1}\u{25a1}\u{25a1}\u{25a1} 0 XP"),
                            TextFont {
                                font_size: 14.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.9, 0.8, 0.5)),
                        ));
                        panel.spawn((
                            HudStreakText,
                            Text::new("Streak: 0 | Burnout: 0"),
                            TextFont {
                                font_size: 14.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.7, 0.7, 0.7)),
                        ));
                        // Unlock catalog
                        panel.spawn((
                            HudUnlocksText,
                            Text::new("[ ] Quick Coffee  [ ] Efficient Processing"),
                            TextFont {
                                font_size: 13.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.5, 0.5, 0.5)),
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
                        // Coworker panel header
                        panel.spawn((
                            Text::new("COWORKERS"),
                            TextFont {
                                font_size: 14.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.8, 0.8, 0.8)),
                        ));
                        panel.spawn((
                            HudCoworkerPanel,
                            HudCoworkerContent,
                            Text::new(""),
                            TextFont {
                                font_size: 13.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.7, 0.7, 0.7)),
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
    mut energy_q: Query<(&mut Text, &mut TextColor), (With<HudEnergyText>, Without<HudTimeText>)>,
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

#[allow(clippy::type_complexity)]
pub fn update_career_hud(
    progression: Res<CareerProgression>,
    mut level_q: Query<&mut Text, (With<HudLevelText>, Without<HudStreakText>)>,
    mut streak_q: Query<&mut Text, (With<HudStreakText>, Without<HudLevelText>)>,
) {
    if let Ok(mut text) = level_q.get_single_mut() {
        let threshold = progression.xp_for_next_level();
        let filled = if threshold == 0 {
            0
        } else {
            ((progression.xp as f32 / threshold as f32) * 10.0).floor() as usize
        };
        let filled = filled.min(10);
        let empty = 10 - filled;
        let bar: String = "\u{25a0}".repeat(filled) + &"\u{25a1}".repeat(empty);
        **text = format!("Lv.{} {} {} XP", progression.level, bar, progression.xp);
    }

    if let Ok(mut text) = streak_q.get_single_mut() {
        **text = format!(
            "Streak: {} | Burnout: {}",
            progression.success_streak, progression.burnout_days
        );
    }
}

#[allow(clippy::type_complexity)]
pub fn update_unlocks_hud(
    unlocks: Res<UnlockCatalogState>,
    mut unlocks_q: Query<(&mut Text, &mut TextColor), With<HudUnlocksText>>,
) {
    if let Ok((mut text, mut color)) = unlocks_q.get_single_mut() {
        let check = |unlocked: bool, label: &str| -> String {
            if unlocked {
                format!("[\u{2713}] {label}")
            } else {
                format!("[ ] {label}")
            }
        };
        let line1 = format!(
            "{}  {}",
            check(unlocks.quick_coffee, "Quick Coffee"),
            check(unlocks.efficient_processing, "Efficient Processing")
        );
        let line2 = format!(
            "{}  {}",
            check(unlocks.conflict_training, "Conflict Training"),
            check(unlocks.escalation_license, "Escalation License")
        );
        **text = format!("{line1}\n{line2}");
        // Use green-ish if any unlocked, else dark gray
        if unlocks.unlocked_count() > 0 {
            color.0 = Color::srgb(0.4, 0.8, 0.4);
        } else {
            color.0 = Color::srgb(0.5, 0.5, 0.5);
        }
    }
}

fn role_short_label(role: &CoworkerRole) -> &'static str {
    match role {
        CoworkerRole::Manager => "Mgr",
        CoworkerRole::Clerk => "Clerk",
        CoworkerRole::Analyst => "Analyst",
        CoworkerRole::Coordinator => "Coord",
        CoworkerRole::Intern => "Intern",
    }
}

pub fn update_coworker_panel(
    social: Res<SocialGraphState>,
    mut coworker_q: Query<&mut Text, With<HudCoworkerContent>>,
) {
    if let Ok(mut text) = coworker_q.get_single_mut() {
        let mut lines = Vec::new();
        for profile in &social.profiles {
            let role_label = role_short_label(&profile.role);
            lines.push(format!(
                "{} ({})  \u{2665} {}  \u{2605} {}",
                profile.codename, role_label, profile.affinity, profile.trust
            ));
        }
        **text = lines.join("\n");
    }
}

pub fn spawn_toast(mut commands: Commands) {
    commands
        .spawn((
            ToastRoot,
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Px(40.0),
                top: Val::Percent(40.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::NONE),
            Visibility::Hidden,
            PickingBehavior::IGNORE,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(""),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 1.0, 0.6)),
            ));
        });
}

pub fn despawn_toast(mut commands: Commands, query: Query<Entity, With<ToastRoot>>) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}

pub fn update_toast(
    mut toast: ResMut<ToastState>,
    mut milestone_reader: EventReader<RelationshipMilestone>,
    mut toast_q: Query<(&mut Visibility, &Children), With<ToastRoot>>,
    mut text_q: Query<&mut Text, Without<ToastRoot>>,
) {
    // Process new milestones
    for event in milestone_reader.read() {
        let message = match event.milestone {
            MilestoneKind::Friendly => format!("{} warms up to you!", event.coworker_name),
            MilestoneKind::Trusted => {
                format!("{} is starting to trust you.", event.coworker_name)
            }
            MilestoneKind::CloseFriend => {
                format!("You and {} are close friends now!", event.coworker_name)
            }
            MilestoneKind::DeepTrust => {
                format!("{} trusts you completely.", event.coworker_name)
            }
            MilestoneKind::Rival => format!("Things are tense with {}...", event.coworker_name),
            MilestoneKind::Distrusted => {
                format!("{} doesn't trust you anymore.", event.coworker_name)
            }
        };
        toast.message = message;
        toast.remaining_ticks = 180;
    }

    // Update visibility and text
    if let Ok((mut visibility, children)) = toast_q.get_single_mut() {
        if toast.remaining_ticks > 0 {
            *visibility = Visibility::Visible;
            toast.remaining_ticks -= 1;
            for &child in children.iter() {
                if let Ok(mut text) = text_q.get_mut(child) {
                    **text = toast.message.clone();
                }
            }
        } else {
            *visibility = Visibility::Hidden;
        }
    }
}
