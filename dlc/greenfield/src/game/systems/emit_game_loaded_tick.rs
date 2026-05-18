use bevy::prelude::*;

pub fn emit_game_loaded_tick_system(mut emitted: Local<bool>, mut writer: EventWriter<crate::game::events::GameLoadedEvent>) {
    if *emitted { return; }
    *emitted = true;
    writer.send(crate::game::events::GameLoadedEvent);
}
