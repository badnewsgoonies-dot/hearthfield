use bevy::prelude::*;
use crate::game::resources::*;
use crate::game::events::*;

/// Reads `QuestAcceptedEvent` and marks a quest as accepted.
/// In our model, quests posted via `post_daily_quests` are already in
/// `QuestLog.active`, so this is a confirmation/no-op if already active.
/// If a UI later separates "posted" from "accepted", this system would
/// move the quest between lists.
pub fn quest_accept_system(
    mut accepted_events: EventReader<QuestAcceptedEvent>,
    quest_log: Res<QuestLog>,
    mut toast_writer: EventWriter<ToastEvent>,
) {
    for event in accepted_events.read() {
        // Check if quest is already in active list
        if let Some(quest) = quest_log.active.iter().find(|q| q.id == event.quest_id) {
            toast_writer.send(ToastEvent {
                message: format!("Quest accepted: {}", quest.title),
                duration_secs: 3.0,
            });
        }
    }
}


