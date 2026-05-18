use bevy::prelude::*;
use crate::game::resources::*;
use crate::game::events::*;

pub fn tick_advance_system(
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


