use super::UiFontHandle;
use crate::npcs::definitions::npc_sprite_file;
use crate::npcs::spawning::NpcSpriteData;
use crate::shared::*;
use bevy::prelude::*;

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
    /// How many characters of the current line have been revealed (typewriter).
    pub chars_revealed: usize,
    /// Accumulated fractional characters for smooth typewriter pacing.
    pub char_accumulator: f32,
}

/// Typewriter text speed in characters per second.
const TYPEWRITER_SPEED: f32 = 30.0;

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
            chars_revealed: 0,
            char_accumulator: 0.0,
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
    npc_registry: Res<NpcRegistry>,
) {
    // Start with empty text — the typewriter system will reveal characters.
    let _first_line_full = ui_state
        .as_ref()
        .and_then(|s| s.lines.first())
        .cloned()
        .unwrap_or_else(|| "...".to_string());
    let first_line = String::new();

    let npc_name = ui_state
        .as_ref()
        .map(|s| {
            npc_registry
                .npcs
                .get(&s.npc_id)
                .map(|def| def.name.clone())
                .unwrap_or_else(|| s.npc_id.clone())
        })
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
                        let npc_id_str = ui_state.as_ref().map(|s| s.npc_id.as_str()).unwrap_or("");
                        let portrait_image = npc_sprites
                            .images
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
                                    index: ui_state
                                        .as_ref()
                                        .and_then(|s| s.portrait_index)
                                        .unwrap_or(0)
                                        as usize,
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
                        .spawn((Node {
                            flex_direction: FlexDirection::Column,
                            flex_grow: 1.0,
                            justify_content: JustifyContent::SpaceBetween,
                            ..default()
                        },))
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

pub fn despawn_dialogue_box(mut commands: Commands, query: Query<Entity, With<DialogueBoxRoot>>) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
    commands.remove_resource::<DialogueUiState>();
}

// ═══════════════════════════════════════════════════════════════════════
// INTERACTION — advance through dialogue lines
// ═══════════════════════════════════════════════════════════════════════

/// Typewriter system: reveals characters of the current dialogue line at
/// ~30 chars/sec. Runs every frame while in Dialogue state.
pub fn typewriter_update(
    time: Res<Time>,
    mut ui_state: Option<ResMut<DialogueUiState>>,
    mut text_query: Query<&mut Text, With<DialogueText>>,
    mut prompt_query: Query<&mut Text, (With<DialoguePrompt>, Without<DialogueText>)>,
) {
    let Some(ref mut state) = ui_state else {
        return;
    };
    let Some(full_line) = state.lines.get(state.current_line).cloned() else {
        return;
    };
    let total_chars = full_line.chars().count();

    if state.chars_revealed >= total_chars {
        // Already fully revealed — show the appropriate prompt.
        let is_last = state.current_line >= state.lines.len() - 1;
        for mut text in &mut prompt_query {
            if is_last {
                **text = "[F / Space] Close".to_string();
            } else {
                **text = "[F / Space] Continue".to_string();
            }
        }
        return;
    }

    // Show "..." prompt while text is still typing
    for mut text in &mut prompt_query {
        **text = "...".to_string();
    }

    // Advance typewriter
    state.char_accumulator += TYPEWRITER_SPEED * time.delta_secs();
    let new_chars = state.char_accumulator as usize;
    if new_chars > 0 {
        state.chars_revealed = (state.chars_revealed + new_chars).min(total_chars);
        state.char_accumulator -= new_chars as f32;

        let partial: String = full_line.chars().take(state.chars_revealed).collect();
        for mut text in &mut text_query {
            **text = partial.clone();
        }
    }
}

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

    let Some(ref mut state) = ui_state else {
        return;
    };

    // If typewriter hasn't finished, skip to full line first.
    let current_full = state
        .lines
        .get(state.current_line)
        .cloned()
        .unwrap_or_default();
    let total_chars = current_full.chars().count();
    if state.chars_revealed < total_chars {
        state.chars_revealed = total_chars;
        state.char_accumulator = 0.0;
        for mut text in &mut text_query {
            **text = current_full.clone();
        }
        return;
    }

    // Move to next line
    state.current_line += 1;

    if state.current_line >= state.lines.len() {
        // End dialogue
        end_event.send(DialogueEndEvent);
        if cutscene_queue.active {
            next_state.set(GameState::Cutscene);
        } else {
            next_state.set(GameState::Playing);
        }
        return;
    }

    // Reset typewriter for new line
    state.chars_revealed = 0;
    state.char_accumulator = 0.0;

    // Clear the text — typewriter_update will fill it in
    for mut text in &mut text_query {
        **text = String::new();
    }

    let is_last = state.current_line >= state.lines.len() - 1;
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
pub fn handle_dialogue_end(mut events: EventReader<DialogueEndEvent>) {
    for _ev in events.read() {
        info!("[UI/Dialogue] DialogueEndEvent received — dialogue cleanup complete.");
    }
}
