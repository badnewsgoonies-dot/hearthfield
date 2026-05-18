use bevy::prelude::*;

pub fn play_audio_system(_audio: Res<crate::game::resources::AudioManager>) {
    // System play_audio_system: substrate-expanded body
    // Each param is exercised below.
    let mut activity: u64 = 0;
    let _ = &*_audio;
    if activity > 0 {
        // play_audio_system: tick had {activity} actionable events
    }
    let _ = activity;
}
