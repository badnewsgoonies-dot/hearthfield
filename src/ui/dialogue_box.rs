use bevy::prelude::*;
use crate::shared::*;
use crate::npcs::spawning::NpcSpriteData;
use crate::npcs::definitions::npc_sprite_file;
use super::UiFontHandle;

// ═══════════════════════════════════════════════════════════════════════
// MARKER COMPONENTS
// ═══════════════════════════════════════════════════════════════════════

#[derive(Component)]
pub struct DialogueBoxRoot;

#[derive(Component)]
pub struct DialoguePortrait;

#[derive(Component)]
pub struct DialogueNpcName;

#[derive(Component)]
pub struct DialogueText;

#[derive(Component)]
pub struct DialoguePrompt;

/// Tracks dialogue state within the UI
#[derive(Resource)]
pub struct DialogueUiState {
    pub npc_id: NpcId,
    pub lines: Vec<String>,
    pub current_line: usize,
    pub portrait_index: Option<u32>,
}

// ═══════════════════════════════════════════════════════════════════════
// EVENT LISTENER — detect DialogueStartEvent and open dialogue
// ═══════════════════════════════════════════════════════════════════════

pub fn listen_for_dialogue_start(
    mut commands: Commands,
    mut events: EventReader<DialogueStartEvent>,
    mut next_state: ResMut<NextState<GameState>>,
    existing: Query<Entity, With<DialogueBoxRoot>>,
) {
    for event in events.read() {
        // Don't open if already open
        if !existing.is_empty() {
            continue;
        }

        commands.insert_resource(DialogueUiState {
            npc_id: event.npc_id.clone(),
            lines: event.lines.clone(),
            current_line: 0,
            portrait_index: event.portrait_index,
        });

        next_state.set(GameState::Dialogue);
    }
}

// ═══════════════════════════════════════════════════════════════════════
// SPAWN / DESPAWN
// ═══════════════════════════════════════════════════════════════════════

pub fn spawn_dialogue_box(
    mut commands: Commands,
    ui_state: Option<Res<DialogueUiState>>,
    font_handle: Res<UiFontHandle>,
    asset_server: Res<AssetServer>,
    npc_sprites: Res<NpcSpriteData>,
) {
    let first_line = ui_state
        .as_ref()
        .and_then(|s| s.lines.first())
        .cloned()
        .unwrap_or_else(|| "...".to_string());

    let npc_name = ui_state
        .as_ref()
        .map(|s| s.npc_id.clone())
        .unwrap_or_else(|| "???".to_string());

    let font = font_handle.0.clone();
    let dialog_bg = asset_server.load("ui/dialog_box_big.png");

    commands
        .spawn((
            DialogueBoxRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::FlexEnd,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                padding: UiRect::bottom(Val::Px(20.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.3)),
        ))
        .with_children(|parent| {
            // Dialogue panel at bottom — uses dialog_box_big.png as background
            parent
                .spawn((
                    Node {
                        width: Val::Px(700.0),
                        height: Val::Px(150.0),
                        flex_direction: FlexDirection::Row,
                        padding: UiRect::all(Val::Px(16.0)),
                        column_gap: Val::Px(16.0),
                        ..default()
                    },
                    ImageNode {
                        image: dialog_bg,
                        // The image will be stretched to fill the node dimensions
                        ..default()
                    },
                ))
                .with_children(|panel| {
                    // Left: NPC portrait using per-NPC spritesheet
                    if npc_sprites.loaded {
                        let npc_id_str = ui_state.as_ref()
                            .map(|s| s.npc_id.as_str())
                            .unwrap_or("");
                        let portrait_image = npc_sprites.images
                            .get(npc_id_str)
                            .cloned()
                            .unwrap_or_else(|| asset_server.load(npc_sprite_file(npc_id_str)));
                        panel.spawn((
                            DialoguePortrait,
                            Node {
                                width: Val::Px(80.0),
                                height: Val::Px(80.0),
                                ..default()
                            },
                            ImageNode {
                                image: portrait_image,
                                texture_atlas: Some(TextureAtlas {
                                    layout: npc_sprites.layout.clone(),
                                    index: 0, // front-facing standing frame
                                }),
                                ..default()
                            },
                        ));
                    } else {
                        panel.spawn((
                            DialoguePortrait,
                            Node {
                                width: Val::Px(80.0),
                                height: Val::Px(80.0),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.3, 0.5, 0.7)),
                        ));
                    }

                    // Right: Text area
                    panel
                        .spawn((
                            Node {
                                flex_direction: FlexDirection::Column,
                                flex_grow: 1.0,
                                justify_content: JustifyContent::SpaceBetween,
                                ..default()
                            },
                        ))
                        .with_children(|text_area| {
                            // NPC name
                            text_area.spawn((
                                DialogueNpcName,
                                Text::new(npc_name),
                                TextFont {
                                    font: font.clone(),
                                    font_size: 18.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(1.0, 0.9, 0.6)),
                            ));

                            // Dialogue text
                            text_area.spawn((
                                DialogueText,
                                Text::new(first_line),
                                TextFont {
                                    font: font.clone(),
                                    font_size: 16.0,
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                            ));

                            // Continue prompt
                            text_area.spawn((
                                DialoguePrompt,
                                Text::new("[F / Space] Continue"),
                                TextFont {
                                    font: font.clone(),
                                    font_size: 12.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.6, 0.6, 0.6)),
                            ));
                        });
                });
        });
}

pub fn despawn_dialogue_box(
    mut commands: Commands,
    query: Query<Entity, With<DialogueBoxRoot>>,
) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
    commands.remove_resource::<DialogueUiState>();
}

// ═══════════════════════════════════════════════════════════════════════
// INTERACTION — advance through dialogue lines
// ═══════════════════════════════════════════════════════════════════════

pub fn advance_dialogue(
    player_input: Res<PlayerInput>,
    mut ui_state: Option<ResMut<DialogueUiState>>,
    mut text_query: Query<&mut Text, With<DialogueText>>,
    mut prompt_query: Query<&mut Text, (With<DialoguePrompt>, Without<DialogueText>)>,
    mut next_state: ResMut<NextState<GameState>>,
    mut end_event: EventWriter<DialogueEndEvent>,
    cutscene_queue: Res<CutsceneQueue>,
) {
    if !player_input.interact {
        return;
    }

    let Some(ref mut state) = ui_state else { return };

    state.current_line += 1;

    if state.current_line >= state.lines.len() {
        // End dialogue
        end_event.send(DialogueEndEvent);
        // Return to Cutscene if a cutscene is in progress, else Playing.
        if cutscene_queue.active {
            next_state.set(GameState::Cutscene);
        } else {
            next_state.set(GameState::Playing);
        }
        return;
    }

    // Show next line
    let line = state.lines[state.current_line].clone();
    let is_last = state.current_line >= state.lines.len() - 1;

    for mut text in &mut text_query {
        **text = line.clone();
    }

    for mut text in &mut prompt_query {
        if is_last {
            **text = "[F / Space] Close".to_string();
        } else {
            **text = "[F / Space] Continue".to_string();
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// DIALOGUE END HANDLER — drains DialogueEndEvent to prevent stale events
// ═══════════════════════════════════════════════════════════════════════

/// Reads `DialogueEndEvent` to ensure the event bus is drained each frame.
/// Without this reader, unread events would persist and potentially cause
/// stale state in subsequent frames.
pub fn handle_dialogue_end(
    mut events: EventReader<DialogueEndEvent>,
) {
    for _ev in events.read() {
        info!("[UI/Dialogue] DialogueEndEvent received — dialogue cleanup complete.");
    }
}
