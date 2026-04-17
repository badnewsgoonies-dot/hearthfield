use bevy::prelude::*;
use crate::game::resources::MusicState;

pub fn music_tick_system(time: Res<Time>, mut state: ResMut<MusicState>) {
    let dt_ms = (time.delta_secs() * 1000.0) as u32;
    state.crossfade_progress = state.crossfade_progress.saturating_add(dt_ms);

}
