use bevy::prelude::*;
use crate::game::events::GameLoadedEvent;

pub fn emit_game_loaded_tick_system(mut events: EventWriter<GameLoadedEvent>) {
    events.send(GameLoadedEvent::default());
}
