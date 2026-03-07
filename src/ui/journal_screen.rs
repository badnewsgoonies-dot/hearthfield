use super::UiFontHandle;
use crate::shared::*;
use bevy::prelude::*;

// ═══════════════════════════════════════════════════════════════════════
// MARKER COMPONENTS
// ═══════════════════════════════════════════════════════════════════════

#[derive(Component)]
pub struct JournalScreenRoot;

#[derive(Component)]
pub struct QuestListItem {
    #[allow(dead_code)]
    pub index: usize,
}

#[derive(Component)]
pub struct QuestListItemBg {
    pub index: usize,
}

#[derive(Component)]
pub struct QuestDetailPanel;

// ═══════════════════════════════════════════════════════════════════════
// RESOURCE
// ═══════════════════════════════════════════════════════════════════════

#[derive(Resource, Default)]
pub struct JournalUiState {
    pub cursor: usize,
    #[allow(dead_code)]
    pub quest_ids: Vec<String>,
}

// ═══════════════════════════════════════════════════════════════════════
// SPAWN / DESPAWN
// ═══════════════════════════════════════════════════════════════════════

pub fn spawn_journal_screen(
    mut commands: Commands,
    font_handle: Res<UiFontHandle>,
    quest_log: Res<QuestLog>,
) {
    let font = font_handle.0.clone();

    let quest_ids: Vec<String> = quest_log.active.iter().map(|q| q.id.clone()).collect();
    commands.insert_resource(JournalUiState {
        cursor: 0,
        quest_ids,
    });

    commands
        .spawn((
            JournalScreenRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.5)),
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        width: Val::Px(620.0),
                        height: Val::Px(480.0),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        padding: UiRect::all(Val::Px(16.0)),
                        row_gap: Val::Px(10.0),
                        border: UiRect::all(Val::Px(3.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.12, 0.1, 0.08, 0.95)),
                    BorderColor(Color::srgb(0.5, 0.4, 0.25)),
                ))
                .with_children(|panel| {
                    // Title
                    panel.spawn((
                        Text::new("QUEST LOG"),
                        TextFont {
                            font: font.clone(),
                            font_size: 22.0,
                            ..default()
                        },
                        TextColor(Color::srgb(1.0, 0.9, 0.6)),
                    ));

                    // Hint text
                    panel.spawn((
                        Text::new("W/S or Arrows: Navigate | J/Esc: Close"),
                        TextFont {
                            font: font.clone(),
                            font_size: 12.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.6, 0.6, 0.6)),
                    ));

                    // Quest list area
                    panel
                        .spawn((Node {
                            width: Val::Percent(100.0),
                            flex_direction: FlexDirection::Column,
                            flex_grow: 1.0,
                            row_gap: Val::Px(4.0),
                            overflow: Overflow::clip_y(),
                            ..default()
                        },))
                        .with_children(|list| {
                            if quest_log.active.is_empty() {
                                list.spawn((
                                    Text::new("No active quests."),
                                    TextFont {
                                        font: font.clone(),
                                        font_size: 14.0,
                                        ..default()
                                    },
                                    TextColor(Color::srgb(0.6, 0.6, 0.6)),
                                ));
                            } else {
                                for (i, quest) in quest_log.active.iter().enumerate() {
                                    let progress = format_objective(&quest.objective);
                                    let from_text = format!("From: {}", quest.giver);
                                    list.spawn((
                                        QuestListItemBg { index: i },
                                        QuestListItem { index: i },
                                        Node {
                                            width: Val::Percent(100.0),
                                            flex_direction: FlexDirection::Column,
                                            padding: UiRect::all(Val::Px(6.0)),
                                            border: UiRect::all(Val::Px(2.0)),
                                            ..default()
                                        },
                                        BackgroundColor(Color::srgba(0.2, 0.17, 0.14, 0.9)),
                                        BorderColor(Color::srgba(0.4, 0.35, 0.3, 0.7)),
                                    ))
                                    .with_children(|row| {
                                        // Quest title
                                        row.spawn((
                                            Text::new(quest.title.clone()),
                                            TextFont {
                                                font: font.clone(),
                                                font_size: 14.0,
                                                ..default()
                                            },
                                            TextColor(Color::srgb(1.0, 0.9, 0.7)),
                                        ));
                                        // From NPC
                                        row.spawn((
                                            Text::new(from_text),
                                            TextFont {
                                                font: font.clone(),
                                                font_size: 11.0,
                                                ..default()
                                            },
                                            TextColor(Color::srgb(0.7, 0.7, 0.7)),
                                        ));
                                        // Progress
                                        row.spawn((
                                            Text::new(progress),
                                            TextFont {
                                                font: font.clone(),
                                                font_size: 11.0,
                                                ..default()
                                            },
                                            TextColor(Color::srgb(0.6, 0.85, 0.6)),
                                        ));
                                    });
                                }
                            }
                        });

                    // Divider
                    panel.spawn((
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Px(2.0),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.5, 0.4, 0.25)),
                    ));

                    // Detail panel
                    panel
                        .spawn((
                            QuestDetailPanel,
                            Node {
                                width: Val::Percent(100.0),
                                flex_direction: FlexDirection::Column,
                                padding: UiRect::all(Val::Px(8.0)),
                                row_gap: Val::Px(4.0),
                                min_height: Val::Px(80.0),
                                ..default()
                            },
                            BackgroundColor(Color::srgba(0.08, 0.07, 0.06, 0.9)),
                        ))
                        .with_children(|detail| {
                            detail.spawn((
                                Text::new("DETAILS"),
                                TextFont {
                                    font: font.clone(),
                                    font_size: 13.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(1.0, 0.9, 0.6)),
                            ));
                            if let Some(quest) = quest_log.active.first() {
                                spawn_detail_children(detail, quest, &font);
                            } else {
                                detail.spawn((
                                    Text::new("Select a quest to see details."),
                                    TextFont {
                                        font: font.clone(),
                                        font_size: 12.0,
                                        ..default()
                                    },
                                    TextColor(Color::srgb(0.6, 0.6, 0.6)),
                                ));
                            }
                        });
                });
        });
}

fn spawn_detail_children(parent: &mut ChildBuilder, quest: &Quest, font: &Handle<Font>) {
    parent.spawn((
        Text::new(quest.description.clone()),
        TextFont {
            font: font.clone(),
            font_size: 12.0,
            ..default()
        },
        TextColor(Color::srgb(0.85, 0.85, 0.85)),
    ));
    parent.spawn((
        Text::new(format!("Reward: {}g", quest.reward_gold)),
        TextFont {
            font: font.clone(),
            font_size: 11.0,
            ..default()
        },
        TextColor(Color::srgb(1.0, 0.85, 0.4)),
    ));
    if let Some(days) = quest.days_remaining {
        parent.spawn((
            Text::new(format!("Days remaining: {}", days)),
            TextFont {
                font: font.clone(),
                font_size: 11.0,
                ..default()
            },
            TextColor(Color::srgb(0.9, 0.5, 0.5)),
        ));
    }
}

pub fn despawn_journal_screen(
    mut commands: Commands,
    query: Query<Entity, With<JournalScreenRoot>>,
) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
    commands.remove_resource::<JournalUiState>();
}

// ═══════════════════════════════════════════════════════════════════════
// UPDATE SYSTEMS
// ═══════════════════════════════════════════════════════════════════════

pub fn update_quest_display(
    mut commands: Commands,
    quest_log: Res<QuestLog>,
    ui_state: Option<Res<JournalUiState>>,
    font_handle: Res<UiFontHandle>,
    detail_query: Query<Entity, With<QuestDetailPanel>>,
) {
    if !quest_log.is_changed() {
        return;
    }
    let Some(ui_state) = ui_state else { return };
    let font = font_handle.0.clone();
    let cursor = ui_state.cursor;
    let selected = quest_log.active.get(cursor);
    for entity in &detail_query {
        commands.entity(entity).despawn_descendants();
        commands.entity(entity).with_children(|detail| {
            detail.spawn((
                Text::new("DETAILS"),
                TextFont {
                    font: font.clone(),
                    font_size: 13.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.9, 0.6)),
            ));
            if let Some(quest) = selected {
                spawn_detail_children(detail, quest, &font);
            } else {
                detail.spawn((
                    Text::new("Select a quest to see details."),
                    TextFont {
                        font: font.clone(),
                        font_size: 12.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.6, 0.6, 0.6)),
                ));
            }
        });
    }
}

pub fn update_cursor_highlight(
    ui_state: Option<Res<JournalUiState>>,
    mut item_query: Query<(&QuestListItemBg, &mut BackgroundColor, &mut BorderColor)>,
) {
    let Some(ui_state) = ui_state else { return };
    for (item, mut bg, mut border) in &mut item_query {
        if item.index == ui_state.cursor {
            *bg = BackgroundColor(Color::srgba(0.35, 0.3, 0.2, 0.95));
            *border = BorderColor(Color::srgb(1.0, 0.84, 0.0));
        } else {
            *bg = BackgroundColor(Color::srgba(0.2, 0.17, 0.14, 0.9));
            *border = BorderColor(Color::srgba(0.4, 0.35, 0.3, 0.7));
        }
    }
}

pub fn journal_navigation(
    action: Res<MenuAction>,
    mut commands: Commands,
    mut ui_state: Option<ResMut<JournalUiState>>,
    quest_log: Res<QuestLog>,
    font_handle: Res<UiFontHandle>,
    detail_query: Query<Entity, With<QuestDetailPanel>>,
) {
    let Some(ref mut ui_state) = ui_state else {
        return;
    };
    let count = quest_log.active.len();
    if count == 0 {
        return;
    }

    let mut moved = false;
    if action.move_up && ui_state.cursor > 0 {
        ui_state.cursor -= 1;
        moved = true;
    }
    if action.move_down && ui_state.cursor + 1 < count {
        ui_state.cursor += 1;
        moved = true;
    }

    if moved {
        let font = font_handle.0.clone();
        let cursor = ui_state.cursor;
        let selected = quest_log.active.get(cursor);
        for entity in &detail_query {
            commands.entity(entity).despawn_descendants();
            commands.entity(entity).with_children(|detail| {
                detail.spawn((
                    Text::new("DETAILS"),
                    TextFont {
                        font: font.clone(),
                        font_size: 13.0,
                        ..default()
                    },
                    TextColor(Color::srgb(1.0, 0.9, 0.6)),
                ));
                if let Some(quest) = selected {
                    spawn_detail_children(detail, quest, &font);
                } else {
                    detail.spawn((
                        Text::new("Select a quest to see details."),
                        TextFont {
                            font: font.clone(),
                            font_size: 12.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.6, 0.6, 0.6)),
                    ));
                }
            });
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// HELPERS
// ═══════════════════════════════════════════════════════════════════════

fn format_objective(objective: &QuestObjective) -> String {
    match objective {
        QuestObjective::Deliver {
            item_id,
            quantity,
            delivered,
        } => {
            format!("Deliver: {}/{} {}", delivered, quantity, item_id)
        }
        QuestObjective::Catch { fish_id, delivered } => {
            if *delivered {
                format!("Catch: {} ✓", fish_id)
            } else {
                format!("Catch: {}", fish_id)
            }
        }
        QuestObjective::Harvest {
            crop_id,
            quantity,
            harvested,
        } => {
            format!("Harvest: {}/{} {}", harvested, quantity, crop_id)
        }
        QuestObjective::Mine {
            item_id,
            quantity,
            collected,
        } => {
            format!("Mine: {}/{} {}", collected, quantity, item_id)
        }
        QuestObjective::Talk { npc_name, talked } => {
            if *talked {
                format!("Talk to {} ✓", npc_name)
            } else {
                format!("Talk to {}", npc_name)
            }
        }
        QuestObjective::Slay {
            monster_kind,
            quantity,
            slain,
        } => {
            format!("Slay: {}/{} {}", slain, quantity, monster_kind)
        }
    }
}
