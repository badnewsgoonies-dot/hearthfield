//! SFX and music handlers — reads PlaySfxEvent / PlayMusicEvent.

use crate::shared::*;
use crate::ui::settings::VolumeSettings;
use bevy::audio::Volume;
use bevy::prelude::*;

fn sfx_path(id: &str) -> Option<&'static str> {
    match id {
        "engine_start" => Some("audio/sfx/engine_start.ogg"),
        "engine_stop" => Some("audio/sfx/engine_stop.ogg"),
        "takeoff" => Some("audio/sfx/takeoff.ogg"),
        "landing" => Some("audio/sfx/landing.ogg"),
        "gear_toggle" => Some("audio/sfx/gear_toggle.ogg"),
        "flaps" => Some("audio/sfx/flaps.ogg"),
        "radio_beep" => Some("audio/sfx/radio_beep.ogg"),
        "coin" => Some("audio/sfx/coin.ogg"),
        "click" => Some("audio/sfx/click.ogg"),
        "alert" => Some("audio/sfx/alert.ogg"),
        "whoosh" => Some("audio/sfx/whoosh.ogg"),
        "achievement" => Some("audio/sfx/achievement.ogg"),
        _ => None,
    }
}

fn music_path(id: &str) -> Option<&'static str> {
    match id {
        "title" => Some("audio/music/title.ogg"),
        "hangar" => Some("audio/music/hangar.ogg"),
        "terminal" => Some("audio/music/terminal.ogg"),
        "flying_calm" => Some("audio/music/flying_calm.ogg"),
        "flying_tense" => Some("audio/music/flying_tense.ogg"),
        "night" => Some("audio/music/night.ogg"),
        "shop" => Some("audio/music/shop.ogg"),
        "mission_complete" => Some("audio/music/mission_complete.ogg"),
        "storm" => Some("audio/music/storm.ogg"),
        "credits" => Some("audio/music/credits.ogg"),
        _ => None,
    }
}

pub fn handle_play_sfx(
    mut events: EventReader<PlaySfxEvent>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    volume_settings: Res<VolumeSettings>,
) {
    for evt in events.read() {
        if let Some(path) = sfx_path(&evt.sfx_id) {
            commands.spawn((
                AudioPlayer::new(asset_server.load(path)),
                PlaybackSettings::DESPAWN.with_volume(Volume::new(volume_settings.sfx_volume)),
            ));
        }
    }
}

pub fn handle_play_music(
    mut events: EventReader<PlayMusicEvent>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut music_state: ResMut<MusicState>,
    volume_settings: Res<VolumeSettings>,
) {
    for evt in events.read() {
        // Stop current track
        if let Some(entity) = music_state.entity {
            commands.entity(entity).despawn_recursive();
        }

        if let Some(path) = music_path(&evt.track_id) {
            let entity = commands
                .spawn((
                    AudioPlayer::new(asset_server.load(path)),
                    PlaybackSettings::LOOP.with_volume(Volume::new(volume_settings.music_volume)),
                ))
                .id();
            music_state.current_track = evt.track_id.clone();
            music_state.entity = Some(entity);
        }
    }
}
