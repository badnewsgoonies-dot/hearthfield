use bevy::prelude::*;
use std::collections::HashMap;

use crate::shared::*;
use super::UiFontHandle;
use super::transitions::ScreenFade;

// ═══════════════════════════════════════════════════════════════════════
// COMPONENTS & RESOURCES
// ═══════════════════════════════════════════════════════════════════════

/// Marker for text overlays spawned by ShowText steps.
#[derive(Component)]
pub struct CutsceneTextOverlay;

/// Arbitrary string flags set by cutscene scripts.
#[derive(Resource, Debug, Clone, Default)]
pub struct CutsceneFlags {
    pub flags: HashMap<String, bool>,
}

// ═══════════════════════════════════════════════════════════════════════
// STATE TRANSITIONS
// ═══════════════════════════════════════════════════════════════════════

pub fn on_enter_cutscene(mut queue: ResMut<CutsceneQueue>) {
    queue.active = true;
    queue.step_timer = 0.0;
}

pub fn on_exit_cutscene(
    mut commands: Commands,
    mut queue: ResMut<CutsceneQueue>,
    overlay_query: Query<Entity, With<CutsceneTextOverlay>>,
) {
    queue.active = false;
    // Despawn any lingering text overlays.
    for entity in &overlay_query {
        commands.entity(entity).despawn_recursive();
    }
}

/// Runs on OnEnter(Playing). If the cutscene queue was pre-populated
/// (e.g. by the intro sequence from the main menu), immediately
/// transition to Cutscene state. The farm/player/HUD all spawned this
/// frame behind the ScreenFade overlay, so the player sees nothing.
pub fn start_pending_cutscene(
    queue: Res<CutsceneQueue>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if !queue.steps.is_empty() {
        next_state.set(GameState::Cutscene);
    }
}

// ═══════════════════════════════════════════════════════════════════════
// CUTSCENE RUNNER — processes one step per frame
// ═══════════════════════════════════════════════════════════════════════

/// The main cutscene driver. Reads the front of the queue, executes
/// the current step, and pops it when complete. Processes exactly ONE
/// step per frame to avoid race conditions with dialogue handoffs.
pub fn run_cutscene_queue(
    time: Res<Time>,
    mut commands: Commands,
    mut queue: ResMut<CutsceneQueue>,
    mut fade: ResMut<ScreenFade>,
    mut next_state: ResMut<NextState<GameState>>,
    mut dialogue_start: EventWriter<DialogueStartEvent>,
    mut music_writer: EventWriter<PlayMusicEvent>,
    mut sfx_writer: EventWriter<PlaySfxEvent>,
    mut map_transition: EventWriter<MapTransitionEvent>,
    mut flags: ResMut<CutsceneFlags>,
    player_input: Res<PlayerInput>,
    npc_registry: Res<NpcRegistry>,
    font_handle: Res<UiFontHandle>,
    overlay_query: Query<Entity, With<CutsceneTextOverlay>>,
    dialogue_ui: Option<Res<super::dialogue_box::DialogueUiState>>,
) {
    if queue.steps.is_empty() {
        // Queue drained — return to Playing.
        queue.active = false;
        next_state.set(GameState::Playing);
        return;
    }

    let dt = time.delta_secs();
    let step = queue.steps.front().unwrap().clone();

    match step {
        CutsceneStep::FadeOut(duration) => {
            let speed = if duration > 0.0 { 1.0 / duration } else { 100.0 };
            fade.target_alpha = 1.0;
            fade.speed = speed;
            fade.active = true;

            if fade.alpha >= 0.99 {
                queue.steps.pop_front();
                queue.step_timer = 0.0;
            }
        }

        CutsceneStep::FadeIn(duration) => {
            let speed = if duration > 0.0 { 1.0 / duration } else { 100.0 };
            fade.target_alpha = 0.0;
            fade.speed = speed;
            fade.active = true;

            if fade.alpha <= 0.01 {
                queue.steps.pop_front();
                queue.step_timer = 0.0;
            }
        }

        CutsceneStep::Wait(secs) => {
            queue.step_timer += dt;
            if queue.step_timer >= secs {
                queue.steps.pop_front();
                queue.step_timer = 0.0;
            }
        }

        CutsceneStep::ShowText(ref text, secs) => {
            // Spawn overlay on first frame of this step.
            if overlay_query.is_empty() {
                let font = font_handle.0.clone();
                let display_text = text.clone();
                commands.spawn((
                    CutsceneTextOverlay,
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        position_type: PositionType::Absolute,
                        ..default()
                    },
                    GlobalZIndex(99),
                    PickingBehavior::IGNORE,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new(display_text),
                        TextFont {
                            font,
                            font_size: 24.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                        Node {
                            max_width: Val::Px(600.0),
                            ..default()
                        },
                        PickingBehavior::IGNORE,
                    ));
                });
            }

            queue.step_timer += dt;

            // Allow skipping text cards with Space.
            let skip = player_input.skip_cutscene && queue.step_timer > 0.3;

            if queue.step_timer >= secs || skip {
                // Despawn the overlay.
                for entity in &overlay_query {
                    commands.entity(entity).despawn_recursive();
                }
                queue.steps.pop_front();
                queue.step_timer = 0.0;
            }
        }

        CutsceneStep::Teleport(map_id) => {
            map_transition.send(MapTransitionEvent {
                to_map: map_id,
                to_x: 10,
                to_y: 10,
            });
            queue.steps.pop_front();
            queue.step_timer = 0.0;
        }

        CutsceneStep::PlayBgm(ref track) => {
            music_writer.send(PlayMusicEvent {
                track_id: track.clone(),
                fade_in: true,
            });
            queue.steps.pop_front();
            queue.step_timer = 0.0;
        }

        CutsceneStep::PlaySfx(ref sfx) => {
            sfx_writer.send(PlaySfxEvent {
                sfx_id: sfx.clone(),
            });
            queue.steps.pop_front();
            queue.step_timer = 0.0;
        }

        CutsceneStep::SetFlag(ref key, val) => {
            flags.flags.insert(key.clone(), val);
            queue.steps.pop_front();
            queue.step_timer = 0.0;
        }

        CutsceneStep::StartDialogue(ref npc_id) => {
            // Look up NPC in registry for default dialogue.
            if let Some(npc_def) = npc_registry.npcs.get(npc_id) {
                dialogue_start.send(DialogueStartEvent {
                    npc_id: npc_id.clone(),
                    lines: npc_def.default_dialogue.clone(),
                    portrait_index: Some(npc_def.portrait_index),
                });
            }
            queue.steps.pop_front();
            queue.step_timer = 0.0;
        }

        CutsceneStep::StartDialogueCustom { ref npc_id, ref lines, portrait_index } => {
            dialogue_start.send(DialogueStartEvent {
                npc_id: npc_id.clone(),
                lines: lines.clone(),
                portrait_index,
            });
            queue.steps.pop_front();
            queue.step_timer = 0.0;
        }

        CutsceneStep::WaitForDialogueEnd => {
            // Complete when the dialogue UI has been despawned
            // (DialogueUiState resource is removed on dialogue exit).
            // On the first frame after StartDialogue, the DialogueUiState
            // may not exist yet (listen_for_dialogue_start hasn't run),
            // so we also check step_timer > 0 to skip the initial frame.
            queue.step_timer += dt;
            let dialogue_closed = dialogue_ui.is_none() && queue.step_timer > 0.1;
            if dialogue_closed {
                queue.steps.pop_front();
                queue.step_timer = 0.0;
            }
        }
    }
}
