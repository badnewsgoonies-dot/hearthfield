use bevy::prelude::*;
use crate::game::events::MusicTrackStartedEvent;

pub fn music_tick_system(mut events: EventReader<MusicTrackStartedEvent>) {
    let _drained = events.read().count();
}
