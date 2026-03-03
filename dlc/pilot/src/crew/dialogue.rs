//! Dialogue system for crew interactions.

use bevy::prelude::*;
use crate::shared::*;

pub fn handle_dialogue_start(
    mut events: EventReader<DialogueStartEvent>,
    crew_registry: Res<CrewRegistry>,
    relationships: Res<Relationships>,
    mut dialogue_state: ResMut<DialogueState>,
    mut next_state: ResMut<NextState<GameState>>,
    calendar: Res<Calendar>,
) {
    for ev in events.read() {
        let Some(member) = crew_registry.members.get(&ev.npc_id) else {
            continue;
        };

        let friendship = relationships.friendship_level(&ev.npc_id);
        let _tier = relationships.friendship_tier(&ev.npc_id);

        // Select dialogue based on friendship and time
        let greeting = if friendship >= 50 {
            format!("Hey, partner! Great to see you. {}", member.personality)
        } else if friendship >= 25 {
            format!("Hi there! {} How's flying?", member.name)
        } else if friendship >= 0 {
            format!("Oh, hello. I'm {}. {}", member.name, member.role.display_name())
        } else {
            format!("...{} here.", member.name)
        };

        let time_line = if calendar.is_night() {
            "Working the night shift, huh?".to_string()
        } else if calendar.hour < 10 {
            "Early morning flight today?".to_string()
        } else {
            "How's the weather out there?".to_string()
        };

        let backstory_line = if friendship >= 40 {
            member.backstory.clone()
        } else {
            String::new()
        };

        let mut lines = vec![
            DialogueLine {
                text: greeting,
                speaker: member.name.clone(),
                emotion: if friendship >= 25 {
                    Emotion::Happy
                } else {
                    Emotion::Neutral
                },
            },
            DialogueLine {
                text: time_line,
                speaker: member.name.clone(),
                emotion: Emotion::Neutral,
            },
        ];

        if !backstory_line.is_empty() {
            lines.push(DialogueLine {
                text: backstory_line,
                speaker: member.name.clone(),
                emotion: Emotion::Proud,
            });
        }

        // Add random pool dialogue
        if !member.dialogue_pool.is_empty() {
            let idx = (calendar.total_days() as usize + ev.npc_id.len()) % member.dialogue_pool.len();
            lines.push(DialogueLine {
                text: member.dialogue_pool[idx].clone(),
                speaker: member.name.clone(),
                emotion: Emotion::Neutral,
            });
        }

        *dialogue_state = DialogueState {
            active: true,
            speaker_id: ev.npc_id.clone(),
            speaker_name: member.name.clone(),
            lines,
            current_line: 0,
            choices: vec![],
        };

        next_state.set(GameState::Dialogue);
    }
}

pub fn advance_dialogue(
    input: Res<PlayerInput>,
    mut dialogue_state: ResMut<DialogueState>,
    mut next_state: ResMut<NextState<GameState>>,
    mut friendship_events: EventWriter<FriendshipChangeEvent>,
) {
    if !input.confirm && !input.cancel {
        return;
    }

    if input.cancel {
        dialogue_state.active = false;
        next_state.set(GameState::Playing);
        return;
    }

    dialogue_state.current_line += 1;
    if dialogue_state.current_line >= dialogue_state.lines.len() {
        // Conversation complete — small friendship boost
        friendship_events.send(FriendshipChangeEvent {
            npc_id: dialogue_state.speaker_id.clone(),
            amount: 2,
        });
        dialogue_state.active = false;
        next_state.set(GameState::Playing);
    }
}
