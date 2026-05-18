use bevy::prelude::*;
use crate::game::events::{GameLoadedEvent, EnemySpawnedEvent};

pub fn emit_game_loaded_tick_system(mut emitted: Local<bool>, mut loaded_writer: EventWriter<GameLoadedEvent>, mut spawn_writer: EventWriter<EnemySpawnedEvent>) {
    if *emitted { return; }
    *emitted = true;
    loaded_writer.send(GameLoadedEvent);
    spawn_writer.send(EnemySpawnedEvent { at_x: 150.0, at_y: 0.0 });
    spawn_writer.send(EnemySpawnedEvent { at_x: -150.0, at_y: 100.0 });
}
