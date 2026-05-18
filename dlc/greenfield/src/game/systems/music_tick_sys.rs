use bevy::prelude::*;
use crate::game::resources::*;
use crate::game::events::*;

/// Advance the crossfade state machine each frame.
pub fn music_tick_system(
    mut music_fade: ResMut<MusicFade>,
    mut music_state: ResMut<MusicState>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    sinks: Query<&AudioSink>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();
    match music_fade.phase.clone() {
        FadePhase::Idle => {}
        FadePhase::FadingOut { mut timer, pending_track } => {
            timer += dt;
            let progress = (timer / FADE_DURATION).min(1.0);
            let vol = 1.0 - progress;
            // Adjust volume of current track
            if let Some(entity) = music_state.current_track {
                if let Ok(sink) = sinks.get(entity) {
                    sink.set_volume(vol);
                }
            }
            if timer >= FADE_DURATION {
                // Fade-out complete: despawn old track, spawn new one at 0 volume
                if let Some(entity) = music_state.current_track {
                    commands.entity(entity).despawn_recursive();
                    music_state.current_track = None;
                }
                spawn_music_silent(&mut commands, &asset_server, &mut music_state, &pending_track);
                music_fade.phase = FadePhase::FadingIn { timer: 0.0 };
            } else {
                music_fade.phase = FadePhase::FadingOut { timer, pending_track };
            }
        }
        FadePhase::FadingIn { mut timer } => {
            timer += dt;
            let progress = (timer / FADE_DURATION).min(1.0);
            // Adjust volume of new track
            if let Some(entity) = music_state.current_track {
                if let Ok(sink) = sinks.get(entity) {
                    sink.set_volume(progress);
                }
            }
            if timer >= FADE_DURATION {
                // Fade-in complete: restore full volume and return to idle
                if let Some(entity) = music_state.current_track {
                    if let Ok(sink) = sinks.get(entity) {
                        sink.set_volume(1.0);
                    }
                }
                music_fade.phase = FadePhase::Idle;
            } else {
                music_fade.phase = FadePhase::FadingIn { timer };
            }
        }
    }
}


