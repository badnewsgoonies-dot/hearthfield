use bevy::prelude::*;
use crate::game::events::{GameLoadedEvent, EnemySpawnedEvent};
use hearthfield::shared::Calendar;

pub fn emit_game_loaded_tick_system(
    mut emitted: Local<bool>,
    mut loaded_writer: EventWriter<GameLoadedEvent>,
    mut spawn_writer: EventWriter<EnemySpawnedEvent>,
    calendar: Option<Res<Calendar>>,
) {
    if *emitted { return; }
    *emitted = true;
    loaded_writer.send(GameLoadedEvent);
    spawn_writer.send(EnemySpawnedEvent { at_x: 150.0, at_y: 0.0 });
    spawn_writer.send(EnemySpawnedEvent { at_x: -150.0, at_y: 100.0 });
    // I4/I6: integration — read the shared Calendar resource to
    // log the in-universe date when the Greenfield run starts.
    if let Some(cal) = calendar {
        info!(
            "greenfield run starting on year {} {:?} day {} ({:?})",
            cal.year, cal.season, cal.day, cal.weather,
        );
    } else {
        info!("greenfield run starting (no shared Calendar registered yet)");
    }
}
