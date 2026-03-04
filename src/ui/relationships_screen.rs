use bevy::prelude::*;
use crate::shared::*;
use super::UiFontHandle;

// ═══════════════════════════════════════════════════════════════════════
// MARKER COMPONENTS
// ═══════════════════════════════════════════════════════════════════════

#[derive(Component)]
pub struct RelationshipsScreenRoot;

#[derive(Component)]
pub struct RelNpcRow {
    #[allow(dead_code)]
    pub index: usize,
}

#[derive(Component)]
pub struct RelNpcRowBg {
    pub index: usize,
}

#[derive(Component)]
pub struct RelDetailPanel;

// ═══════════════════════════════════════════════════════════════════════
// RESOURCE
// ═══════════════════════════════════════════════════════════════════════

#[derive(Resource, Default)]
pub struct RelationshipsUiState {
    pub cursor: usize,
    pub npc_ids: Vec<NpcId>,
}

// ═══════════════════════════════════════════════════════════════════════
// SPAWN / DESPAWN
// ═══════════════════════════════════════════════════════════════════════

pub fn spawn_relationships_screen(
    mut commands: Commands,
    font_handle: Res<UiFontHandle>,
    npc_registry: Res<NpcRegistry>,
    relationships: Res<Relationships>,
) {
    let font = font_handle.0.clone();

    // Collect NPCs sorted by name for stable ordering
    let mut npc_ids: Vec<NpcId> = npc_registry.npcs.keys().cloned().collect();
    npc_ids.sort();

    commands.insert_resource(RelationshipsUiState { cursor: 0, npc_ids: npc_ids.clone() });

    commands
        .spawn((
            RelationshipsScreenRoot,
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
                        width: Val::Px(640.0),
                        height: Val::Px(520.0),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        padding: UiRect::all(Val::Px(16.0)),
                        row_gap: Val::Px(10.0),
                        border: UiRect::all(Val::Px(3.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.1, 0.08, 0.12, 0.97)),
                    BorderColor(Color::srgb(0.55, 0.35, 0.7)),
                ))
                .with_children(|panel| {
                    // Title
                    panel.spawn((
                        Text::new("RELATIONSHIPS"),
                        TextFont {
                            font: font.clone(),
                            font_size: 22.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.95, 0.8, 1.0)),
                    ));

                    // Hint
                    panel.spawn((
                        Text::new("W/S or Arrows: Navigate | R/Esc: Close"),
                        TextFont {
                            font: font.clone(),
                            font_size: 12.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.6, 0.6, 0.6)),
                    ));

                    // NPC list
                    panel
                        .spawn((
                            Node {
                                width: Val::Percent(100.0),
                                flex_direction: FlexDirection::Column,
                                flex_grow: 1.0,
                                row_gap: Val::Px(3.0),
                                overflow: Overflow::clip_y(),
                                ..default()
                            },
                        ))
                        .with_children(|list| {
                            if npc_ids.is_empty() {
                                list.spawn((
                                    Text::new("No NPCs found."),
                                    TextFont { font: font.clone(), font_size: 14.0, ..default() },
                                    TextColor(Color::srgb(0.6, 0.6, 0.6)),
                                ));
                            } else {
                                for (i, npc_id) in npc_ids.iter().enumerate() {
                                    let Some(npc_def) = npc_registry.npcs.get(npc_id) else { continue };
                                    let hearts = relationships.hearts(npc_id);
                                    let heart_str = format_hearts(hearts);
                                    let row_text = format!("{:<16} {}  {}/10", npc_def.name, heart_str, hearts);

                                    list.spawn((
                                        RelNpcRowBg { index: i },
                                        RelNpcRow { index: i },
                                        Node {
                                            width: Val::Percent(100.0),
                                            padding: UiRect::axes(Val::Px(8.0), Val::Px(5.0)),
                                            border: UiRect::all(Val::Px(2.0)),
                                            ..default()
                                        },
                                        BackgroundColor(Color::srgba(0.18, 0.14, 0.22, 0.9)),
                                        BorderColor(Color::srgba(0.45, 0.35, 0.55, 0.6)),
                                    ))
                                    .with_children(|row| {
                                        row.spawn((
                                            Text::new(row_text),
                                            TextFont { font: font.clone(), font_size: 14.0, ..default() },
                                            TextColor(Color::srgb(0.95, 0.9, 1.0)),
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
                        BackgroundColor(Color::srgb(0.55, 0.35, 0.7)),
                    ));

                    // Detail panel
                    panel
                        .spawn((
                            RelDetailPanel,
                            Node {
                                width: Val::Percent(100.0),
                                flex_direction: FlexDirection::Column,
                                padding: UiRect::all(Val::Px(10.0)),
                                row_gap: Val::Px(4.0),
                                min_height: Val::Px(90.0),
                                ..default()
                            },
                            BackgroundColor(Color::srgba(0.07, 0.05, 0.1, 0.95)),
                        ))
                        .with_children(|detail| {
                            let first_id = npc_ids.first();
                            spawn_detail_children(detail, first_id, &npc_registry, &relationships, &font);
                        });
                });
        });
}

pub fn despawn_relationships_screen(
    mut commands: Commands,
    query: Query<Entity, With<RelationshipsScreenRoot>>,
) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
    commands.remove_resource::<RelationshipsUiState>();
}

// ═══════════════════════════════════════════════════════════════════════
// UPDATE SYSTEMS
// ═══════════════════════════════════════════════════════════════════════

pub fn relationships_navigation(
    action: Res<MenuAction>,
    mut commands: Commands,
    mut ui_state: Option<ResMut<RelationshipsUiState>>,
    npc_registry: Res<NpcRegistry>,
    relationships: Res<Relationships>,
    font_handle: Res<UiFontHandle>,
    detail_query: Query<Entity, With<RelDetailPanel>>,
) {
    let Some(ref mut ui_state) = ui_state else { return };
    let count = ui_state.npc_ids.len();
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
        let selected_id = ui_state.npc_ids.get(ui_state.cursor);
        for entity in &detail_query {
            commands.entity(entity).despawn_descendants();
            commands.entity(entity).with_children(|detail| {
                spawn_detail_children(detail, selected_id, &npc_registry, &relationships, &font);
            });
        }
    }
}

pub fn update_relationships_cursor(
    ui_state: Option<Res<RelationshipsUiState>>,
    mut row_query: Query<(&RelNpcRowBg, &mut BackgroundColor, &mut BorderColor)>,
) {
    let Some(ui_state) = ui_state else { return };
    for (row, mut bg, mut border) in &mut row_query {
        if row.index == ui_state.cursor {
            *bg = BackgroundColor(Color::srgba(0.35, 0.25, 0.45, 0.95));
            *border = BorderColor(Color::srgb(0.9, 0.6, 1.0));
        } else {
            *bg = BackgroundColor(Color::srgba(0.18, 0.14, 0.22, 0.9));
            *border = BorderColor(Color::srgba(0.45, 0.35, 0.55, 0.6));
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// HELPERS
// ═══════════════════════════════════════════════════════════════════════

fn format_hearts(hearts: u8) -> String {
    let filled = hearts as usize;
    let empty = (MAX_HEARTS as usize).saturating_sub(filled);
    format!("{}{}", "♥".repeat(filled), "♡".repeat(empty))
}

fn spawn_detail_children(
    parent: &mut ChildBuilder,
    npc_id: Option<&NpcId>,
    npc_registry: &NpcRegistry,
    relationships: &Relationships,
    font: &Handle<Font>,
) {
    let Some(id) = npc_id else {
        parent.spawn((
            Text::new("Select an NPC to see details."),
            TextFont { font: font.clone(), font_size: 13.0, ..default() },
            TextColor(Color::srgb(0.6, 0.6, 0.6)),
        ));
        return;
    };

    let Some(def) = npc_registry.npcs.get(id) else {
        return;
    };

    let hearts = relationships.hearts(id);
    let points = relationships.friendship.get(id).copied().unwrap_or(0);

    // NPC name header
    parent.spawn((
        Text::new(format!("SELECTED: {}", def.name)),
        TextFont { font: font.clone(), font_size: 14.0, ..default() },
        TextColor(Color::srgb(0.95, 0.8, 1.0)),
    ));

    // Birthday
    parent.spawn((
        Text::new(format!("Birthday: {:?} {}", def.birthday_season, def.birthday_day)),
        TextFont { font: font.clone(), font_size: 12.0, ..default() },
        TextColor(Color::srgb(0.85, 0.85, 0.85)),
    ));

    // Friendship
    parent.spawn((
        Text::new(format!("Friendship: {} pts ({}/10 hearts)", points, hearts)),
        TextFont { font: font.clone(), font_size: 12.0, ..default() },
        TextColor(Color::srgb(1.0, 0.7, 0.7)),
    ));

    // Loved gifts
    let loved: Vec<String> = def.gift_preferences.iter()
        .filter(|(_, pref)| matches!(pref, GiftPreference::Loved))
        .map(|(item_id, _)| item_id.clone())
        .collect();
    if !loved.is_empty() {
        let mut sorted = loved;
        sorted.sort();
        parent.spawn((
            Text::new(format!("Loves: {}", sorted.join(", "))),
            TextFont { font: font.clone(), font_size: 12.0, ..default() },
            TextColor(Color::srgb(1.0, 0.85, 0.4)),
        ));
    }

    // Marriageable
    if def.is_marriageable {
        parent.spawn((
            Text::new("Marriageable"),
            TextFont { font: font.clone(), font_size: 11.0, ..default() },
            TextColor(Color::srgb(0.6, 0.9, 0.6)),
        ));
    }
}
