use bevy::prelude::*;
use crate::shared::*;

// ═══════════════════════════════════════════════════════════════════════
// MUSIC STATE — tracks the currently playing music entity
// ═══════════════════════════════════════════════════════════════════════

#[derive(Resource, Default)]
pub struct MusicState {
    pub current_track: Option<Entity>,
    pub current_track_id: String,
}

// ═══════════════════════════════════════════════════════════════════════
// SFX PATH MAPPING
// ═══════════════════════════════════════════════════════════════════════

/// Maps SFX IDs (sent by other domains) to actual audio file paths.
fn sfx_path(sfx_id: &str) -> Option<&'static str> {
    match sfx_id {
        "sfx_hoe" | "sfx_axe" | "sfx_pickaxe" | "sfx_scythe" => {
            Some("audio/sfx/sfx_sounds_impact1.ogg")
        }
        "sfx_water" => Some("audio/sfx/sfx_sounds_interaction5.ogg"),
        "sfx_cast" => Some("audio/sfx/sfx_movement_jump1.ogg"),
        "chop" => Some("audio/sfx/sfx_sounds_impact2.ogg"),
        "rock_hit" => Some("audio/sfx/sfx_sounds_impact3.ogg"),
        "swish" => Some("audio/sfx/sfx_wpn_sword1.ogg"),
        "hit" => Some("audio/sfx/sfx_damage_hit1.ogg"),
        "object_break" => Some("audio/sfx/sfx_sounds_impact5.ogg"),
        "pickup" => Some("audio/sfx/sfx_coin_single1.ogg"),
        "menu_move" => Some("audio/sfx/sfx_menu_move1.ogg"),
        "menu_select" => Some("audio/sfx/sfx_menu_select1.ogg"),
        "purchase" => Some("audio/sfx/sfx_coin_cluster1.ogg"),
        "sell" => Some("audio/sfx/sfx_coin_double1.ogg"),
        "door" => Some("audio/sfx/sfx_movement_dooropen1.ogg"),
        "footstep" => Some("audio/sfx/sfx_movement_footsteps1a.ogg"),
        "error" => Some("audio/sfx/sfx_sounds_error1.ogg"),
        "fanfare" => Some("audio/sfx/sfx_sounds_fanfare1.ogg"),
        "powerup" => Some("audio/sfx/sfx_sounds_powerup1.ogg"),
        "damage" => Some("audio/sfx/sfx_sounds_damage1.ogg"),
        _ => None,
    }
}

// ═══════════════════════════════════════════════════════════════════════
// MUSIC PATH MAPPING
// ═══════════════════════════════════════════════════════════════════════

/// Maps music track IDs to actual audio file paths.
fn music_path(track_id: &str) -> Option<&'static str> {
    match track_id {
        "farm" | "spring" => Some("audio/music/pixel_1.ogg"),
        "summer" => Some("audio/music/pixel_2.ogg"),
        "fall" => Some("audio/music/pixel_3.ogg"),
        "winter" => Some("audio/music/pixel_4.ogg"),
        "town" => Some("audio/music/pixel_5.ogg"),
        "mine" => Some("audio/music/pixel_6.ogg"),
        "forest" => Some("audio/music/pixel_7.ogg"),
        "beach" => Some("audio/music/pixel_8.ogg"),
        "menu" => Some("audio/music/pixel_9.ogg"),
        "night" => Some("audio/music/pixel_10.ogg"),
        "festival" => Some("audio/music/pixel_11.ogg"),
        "credits" => Some("audio/music/pixel_12.ogg"),
        _ => None,
    }
}

// ═══════════════════════════════════════════════════════════════════════
// SYSTEMS
// ═══════════════════════════════════════════════════════════════════════

/// Listen for PlaySfxEvent and spawn one-shot audio sources that auto-despawn.
pub fn handle_play_sfx(
    mut events: EventReader<PlaySfxEvent>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    for event in events.read() {
        if let Some(path) = sfx_path(&event.sfx_id) {
            commands.spawn((
                AudioPlayer::new(asset_server.load(path)),
                PlaybackSettings::DESPAWN,
            ));
        }
    }
}

/// Listen for PlayMusicEvent, stop the current music track, and start a new one.
pub fn handle_play_music(
    mut events: EventReader<PlayMusicEvent>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut music_state: ResMut<MusicState>,
) {
    for event in events.read() {
        // Stop current track if playing
        if let Some(entity) = music_state.current_track {
            commands.entity(entity).despawn_recursive();
        }

        if let Some(path) = music_path(&event.track_id) {
            let entity = commands
                .spawn((
                    AudioPlayer::new(asset_server.load(path)),
                    PlaybackSettings::LOOP,
                ))
                .id();
            music_state.current_track = Some(entity);
            music_state.current_track_id = event.track_id.clone();
        } else {
            music_state.current_track = None;
            music_state.current_track_id.clear();
        }
    }
}

/// Start background music when entering the Playing state.
pub fn start_game_music(
    mut music_events: EventWriter<PlayMusicEvent>,
    music_state: Res<MusicState>,
) {
    // Skip if music is already playing (avoids restart on Cutscene→Playing).
    if music_state.current_track.is_some() {
        return;
    }
    music_events.send(PlayMusicEvent {
        track_id: "farm".to_string(),
        fade_in: true,
    });
}

/// Start menu music when entering the MainMenu state.
pub fn start_menu_music(mut music_events: EventWriter<PlayMusicEvent>) {
    music_events.send(PlayMusicEvent {
        track_id: "menu".to_string(),
        fade_in: true,
    });
}
