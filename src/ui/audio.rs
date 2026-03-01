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
        "axe_chop" => Some("audio/sfx/sfx_sounds_impact2.ogg"),
        "pickaxe_hit" => Some("audio/sfx/sfx_sounds_impact3.ogg"),
        "hoe_till" => Some("audio/sfx/sfx_sounds_impact1.ogg"),
        "water_splash" => Some("audio/sfx/sfx_sounds_interaction5.ogg"),
        "fishing_cast" => Some("audio/sfx/sfx_movement_jump1.ogg"),
        "tool_generic" => Some("audio/sfx/sfx_wpn_sword1.ogg"),
        "craft_success" => Some("audio/sfx/sfx_sounds_fanfare1.ogg"),
        "craft_fail" => Some("audio/sfx/sfx_sounds_error1.ogg"),
        "ui_deny" => Some("audio/sfx/sfx_sounds_error1.ogg"),
        "blacksmith_forge" => Some("audio/sfx/sfx_sounds_impact3.ogg"),
        "upgrade_complete" => Some("audio/sfx/sfx_sounds_powerup1.ogg"),
        "item_pickup" => Some("audio/sfx/sfx_coin_single1.ogg"),
        "shop_buy" => Some("audio/sfx/sfx_coin_cluster1.ogg"),
        "shop_sell" => Some("audio/sfx/sfx_coin_double1.ogg"),
        "shipping_bin" => Some("audio/sfx/sfx_coin_double1.ogg"),
        "eat" => Some("audio/sfx/sfx_coin_single1.ogg"),
        "harvest" => Some("audio/sfx/sfx_coin_single1.ogg"),
        "plant" => Some("audio/sfx/sfx_sounds_interaction5.ogg"),
        "fish_escape" => Some("audio/sfx/sfx_sounds_error1.ogg"),
        "thunder" => Some("audio/sfx/sfx_sounds_damage1.ogg"),
        "treasure_found" => Some("audio/sfx/sfx_sounds_fanfare1.ogg"),
        "rocks_broken" => Some("audio/sfx/sfx_sounds_impact5.ogg"),
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

/// Start background music when entering the Playing state, using the current season.
pub fn start_game_music(
    mut music_events: EventWriter<PlayMusicEvent>,
    music_state: Res<MusicState>,
    calendar: Res<Calendar>,
) {
    // Skip if music is already playing (avoids restart on Cutscene→Playing).
    if music_state.current_track.is_some() {
        return;
    }
    let track = match calendar.season {
        Season::Spring => "spring",
        Season::Summer => "summer",
        Season::Fall => "fall",
        Season::Winter => "winter",
    };
    music_events.send(PlayMusicEvent {
        track_id: track.to_string(),
        fade_in: true,
    });
}

/// Switch music when the season changes (only on the farm map).
pub fn switch_music_on_season_change(
    mut season_events: EventReader<SeasonChangeEvent>,
    mut music_events: EventWriter<PlayMusicEvent>,
    player_state: Res<PlayerState>,
) {
    for event in season_events.read() {
        let track = match event.new_season {
            Season::Spring => "spring",
            Season::Summer => "summer",
            Season::Fall => "fall",
            Season::Winter => "winter",
        };
        // Only switch if player is on the farm (other maps have their own music)
        if player_state.current_map == MapId::Farm {
            music_events.send(PlayMusicEvent {
                track_id: track.to_string(),
                fade_in: true,
            });
        }
    }
}

/// Switch music when the player transitions to a new map.
pub fn switch_music_on_map_change(
    mut map_events: EventReader<MapTransitionEvent>,
    mut music_events: EventWriter<PlayMusicEvent>,
    calendar: Res<Calendar>,
) {
    for event in map_events.read() {
        let track = match event.to_map {
            MapId::Farm => match calendar.season {
                Season::Spring => "spring",
                Season::Summer => "summer",
                Season::Fall => "fall",
                Season::Winter => "winter",
            },
            MapId::Town => "town",
            MapId::Mine => "mine",
            MapId::Forest => "forest",
            MapId::Beach => "beach",
            _ => continue,
        };
        music_events.send(PlayMusicEvent {
            track_id: track.to_string(),
            fade_in: true,
        });
    }
}

/// Start menu music when entering the MainMenu state.
pub fn start_menu_music(mut music_events: EventWriter<PlayMusicEvent>) {
    music_events.send(PlayMusicEvent {
        track_id: "menu".to_string(),
        fade_in: true,
    });
}
