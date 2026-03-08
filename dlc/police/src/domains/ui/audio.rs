use bevy::audio::{AudioPlayer, AudioSink, AudioSinkPlayback, PlaybackSettings, Volume};
use bevy::prelude::*;
use bevy::time::TimerMode;

use crate::shared::{
    CaseAssignedEvent, CaseFailedEvent, CaseSolvedEvent, DispatchCallEvent, EvidenceCollectedEvent,
    GameState, MapId, MapTransitionEvent, PatrolState, PlayMusicEvent, PlaySfxEvent, Player,
    PlayerMovement, PlayerState, PromotionEvent, ShiftClock, ShiftEndEvent, ToastEvent,
    UpdatePhase,
};

const MUSIC_FADE_SPEED: f32 = 0.8;
const AMBIENT_FADE_SPEED: f32 = 1.4;
const MUSIC_VOLUME: f32 = 0.38;
const CLOCK_TICK_VOLUME: f32 = 0.10;
const RADIO_STATIC_VOLUME: f32 = 0.07;
const SILENT_VOLUME_THRESHOLD: f32 = 0.005;
const FOOTSTEP_WALK_INTERVAL_SECS: f32 = 0.42;
const FOOTSTEP_RUN_INTERVAL_SECS: f32 = 0.28;
const FOOTSTEP_DISTANCE_THRESHOLD: f32 = 0.4;
const CLOCK_TICK_PATH: &str = "audio/ambient/clock_ticking.ogg";
const RADIO_STATIC_PATH: &str = "audio/ambient/radio_static.ogg";

pub(super) fn build_audio(app: &mut App) {
    app.add_event::<CaseAssignedEvent>()
        .add_event::<EvidenceCollectedEvent>()
        .add_event::<DispatchCallEvent>()
        .add_event::<ShiftEndEvent>()
        .add_event::<PromotionEvent>()
        .add_event::<CaseSolvedEvent>()
        .add_event::<CaseFailedEvent>()
        .add_event::<MapTransitionEvent>()
        .add_event::<PlayMusicEvent>()
        .add_event::<PlaySfxEvent>()
        .add_event::<ToastEvent>()
        .init_resource::<ActiveMusicState>()
        .init_resource::<FootstepCadence>()
        .add_systems(Startup, spawn_ambient_loops)
        .add_systems(
            Update,
            (
                emit_toast_pop_sfx,
                emit_case_assigned_sfx,
                emit_evidence_collected_sfx,
                emit_dispatch_call_sfx,
                emit_shift_end_sfx,
                emit_promotion_sfx,
                emit_case_solved_sfx,
                emit_case_failed_sfx,
                queue_transition_audio,
                emit_footsteps,
            )
                .in_set(UpdatePhase::Reactions),
        )
        .add_systems(
            Update,
            (
                sync_ambient_targets,
                apply_looped_audio_fades,
                play_requested_sfx,
                handle_music_requests,
            )
                .in_set(UpdatePhase::Presentation),
        );
}

pub(super) fn music_for_map(map_id: MapId) -> &'static str {
    match map_id {
        MapId::PrecinctInterior => "precinct_theme",
        MapId::PrecinctExterior
        | MapId::ResidentialNorth
        | MapId::ResidentialSouth
        | MapId::Highway => "patrol_theme",
        MapId::Downtown | MapId::IndustrialDistrict | MapId::PlayerApartment => "downtown_ambient",
        MapId::ForestPark | MapId::CrimeSceneTemplate | MapId::Hospital | MapId::CourtHouse => {
            "tension_theme"
        }
    }
}

#[derive(Resource, Default)]
struct ActiveMusicState {
    current_track: Option<String>,
}

#[derive(Resource)]
struct FootstepCadence {
    timer: Timer,
    last_position: Option<Vec2>,
}

impl Default for FootstepCadence {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(FOOTSTEP_WALK_INTERVAL_SECS, TimerMode::Repeating),
            last_position: None,
        }
    }
}

#[derive(Component, Clone, Copy, PartialEq, Eq)]
enum LoopAudioChannel {
    Music,
    ClockTick,
    RadioStatic,
}

#[derive(Component)]
struct ManagedLoopAudio {
    channel: LoopAudioChannel,
    current_volume: f32,
    target_volume: f32,
    fade_speed: f32,
    despawn_when_silent: bool,
}

fn spawn_ambient_loops(
    mut commands: Commands,
    asset_server: Option<Res<AssetServer>>,
    existing: Query<&ManagedLoopAudio>,
) {
    if existing.iter().next().is_some() {
        return;
    }

    let Some(asset_server) = asset_server else {
        return;
    };

    spawn_looped_audio(
        &mut commands,
        &asset_server,
        LoopAudioChannel::ClockTick,
        CLOCK_TICK_PATH,
        0.0,
        false,
    );
    spawn_looped_audio(
        &mut commands,
        &asset_server,
        LoopAudioChannel::RadioStatic,
        RADIO_STATIC_PATH,
        0.0,
        false,
    );
}

fn play_requested_sfx(
    mut commands: Commands,
    asset_server: Option<Res<AssetServer>>,
    mut sfx_events: EventReader<PlaySfxEvent>,
) {
    let Some(asset_server) = asset_server else {
        return;
    };

    for event in sfx_events.read() {
        let Some((path, volume)) = sfx_asset(event.name.as_str()) else {
            warn!("Missing SFX mapping for '{}'", event.name);
            continue;
        };

        commands.spawn((
            AudioPlayer::new(asset_server.load(path)),
            PlaybackSettings::DESPAWN.with_volume(Volume::new(volume)),
        ));
    }
}

fn handle_music_requests(
    mut commands: Commands,
    asset_server: Option<Res<AssetServer>>,
    mut music_events: EventReader<PlayMusicEvent>,
    mut active_music: ResMut<ActiveMusicState>,
    mut looped_audio: Query<(Entity, &mut ManagedLoopAudio), Without<AudioSink>>,
    mut looped_audio_with_sink: Query<(Entity, &mut ManagedLoopAudio), With<AudioSink>>,
) {
    let Some(asset_server) = asset_server else {
        return;
    };

    let Some((track_name, looping)) = music_events
        .read()
        .filter_map(|event| {
            canonical_music_name(event.name.as_str()).map(|name| (name, event.looping))
        })
        .last()
    else {
        return;
    };

    if active_music.current_track.as_deref() == Some(track_name) {
        for (_, mut audio) in &mut looped_audio {
            if audio.channel == LoopAudioChannel::Music {
                audio.target_volume = MUSIC_VOLUME;
                audio.despawn_when_silent = false;
            }
        }
        for (_, mut audio) in &mut looped_audio_with_sink {
            if audio.channel == LoopAudioChannel::Music {
                audio.target_volume = MUSIC_VOLUME;
                audio.despawn_when_silent = false;
            }
        }
        return;
    }

    for (_, mut audio) in &mut looped_audio {
        if audio.channel != LoopAudioChannel::Music {
            continue;
        }

        audio.target_volume = 0.0;
        audio.fade_speed = MUSIC_FADE_SPEED;
        audio.despawn_when_silent = true;
    }
    for (_, mut audio) in &mut looped_audio_with_sink {
        if audio.channel != LoopAudioChannel::Music {
            continue;
        }

        audio.target_volume = 0.0;
        audio.fade_speed = MUSIC_FADE_SPEED;
        audio.despawn_when_silent = true;
    }

    let Some(path) = music_asset(track_name) else {
        warn!("Missing music mapping for '{}'", track_name);
        return;
    };

    active_music.current_track = Some(track_name.to_string());
    let settings = if looping {
        PlaybackSettings::LOOP
    } else {
        PlaybackSettings::ONCE
    }
    .with_volume(Volume::ZERO);

    commands.spawn((
        AudioPlayer::new(asset_server.load(path)),
        settings,
        ManagedLoopAudio {
            channel: LoopAudioChannel::Music,
            current_volume: 0.0,
            target_volume: MUSIC_VOLUME,
            fade_speed: MUSIC_FADE_SPEED,
            despawn_when_silent: false,
        },
    ));
}

fn apply_looped_audio_fades(
    time: Res<Time>,
    mut commands: Commands,
    mut looped_audio: Query<(Entity, &mut ManagedLoopAudio, Option<&AudioSink>)>,
) {
    for (entity, mut audio, sink) in &mut looped_audio {
        audio.current_volume = move_towards(
            audio.current_volume,
            audio.target_volume,
            audio.fade_speed * time.delta_secs(),
        );

        if let Some(sink) = sink {
            sink.set_volume(audio.current_volume);

            if audio.target_volume <= SILENT_VOLUME_THRESHOLD
                && audio.current_volume <= SILENT_VOLUME_THRESHOLD
            {
                sink.pause();
            } else if sink.is_paused() {
                sink.play();
            }
        }

        if audio.despawn_when_silent
            && audio.target_volume <= SILENT_VOLUME_THRESHOLD
            && audio.current_volume <= SILENT_VOLUME_THRESHOLD
        {
            commands.entity(entity).despawn_recursive();
        }
    }
}

fn sync_ambient_targets(
    state: Res<State<GameState>>,
    clock: Res<ShiftClock>,
    patrol_state: Res<PatrolState>,
    player_state: Res<PlayerState>,
    mut looped_audio: Query<&mut ManagedLoopAudio>,
) {
    let player_can_hear_world = !matches!(state.get(), GameState::Loading | GameState::MainMenu);
    let clock_target = if player_can_hear_world && clock.on_duty && !clock.time_paused {
        CLOCK_TICK_VOLUME
    } else {
        0.0
    };
    let radio_target = if player_can_hear_world
        && clock.on_duty
        && !clock.time_paused
        && player_state.position_map == MapId::PrecinctInterior
        && patrol_state.current_dispatch.is_none()
    {
        RADIO_STATIC_VOLUME
    } else {
        0.0
    };

    for mut audio in &mut looped_audio {
        match audio.channel {
            LoopAudioChannel::ClockTick => {
                audio.target_volume = clock_target;
                audio.fade_speed = AMBIENT_FADE_SPEED;
            }
            LoopAudioChannel::RadioStatic => {
                audio.target_volume = radio_target;
                audio.fade_speed = AMBIENT_FADE_SPEED;
            }
            LoopAudioChannel::Music => {}
        }
    }
}

fn emit_toast_pop_sfx(
    mut toast_events: EventReader<ToastEvent>,
    mut sfx_events: EventWriter<PlaySfxEvent>,
) {
    if toast_events.read().next().is_some() {
        sfx_events.send(named_sfx("toast_pop"));
    }
}

fn emit_case_assigned_sfx(
    mut case_events: EventReader<CaseAssignedEvent>,
    mut sfx_events: EventWriter<PlaySfxEvent>,
) {
    if case_events.read().next().is_some() {
        sfx_events.send(named_sfx("case_assigned"));
    }
}

fn emit_evidence_collected_sfx(
    mut evidence_events: EventReader<EvidenceCollectedEvent>,
    mut sfx_events: EventWriter<PlaySfxEvent>,
) {
    if evidence_events.read().next().is_some() {
        sfx_events.send(named_sfx("evidence_collected"));
    }
}

fn emit_dispatch_call_sfx(
    mut dispatch_events: EventReader<DispatchCallEvent>,
    mut sfx_events: EventWriter<PlaySfxEvent>,
) {
    if dispatch_events.read().next().is_some() {
        sfx_events.send(named_sfx("dispatch_call"));
    }
}

fn emit_shift_end_sfx(
    mut shift_end_events: EventReader<ShiftEndEvent>,
    mut sfx_events: EventWriter<PlaySfxEvent>,
) {
    if shift_end_events.read().next().is_some() {
        sfx_events.send(named_sfx("shift_end"));
    }
}

fn emit_promotion_sfx(
    mut promotion_events: EventReader<PromotionEvent>,
    mut sfx_events: EventWriter<PlaySfxEvent>,
) {
    if promotion_events.read().next().is_some() {
        sfx_events.send(named_sfx("promotion"));
    }
}

fn emit_case_solved_sfx(
    mut case_events: EventReader<CaseSolvedEvent>,
    mut sfx_events: EventWriter<PlaySfxEvent>,
) {
    if case_events.read().next().is_some() {
        sfx_events.send(named_sfx("case_solved"));
    }
}

fn emit_case_failed_sfx(
    mut case_events: EventReader<CaseFailedEvent>,
    mut sfx_events: EventWriter<PlaySfxEvent>,
) {
    if case_events.read().next().is_some() {
        sfx_events.send(named_sfx("case_failed"));
    }
}

fn queue_transition_audio(
    mut transition_events: EventReader<MapTransitionEvent>,
    mut music_events: EventWriter<PlayMusicEvent>,
    mut sfx_events: EventWriter<PlaySfxEvent>,
) {
    for event in transition_events.read() {
        if event.from == event.to {
            continue;
        }

        sfx_events.send(named_sfx("door_open"));
        music_events.send(PlayMusicEvent {
            name: music_for_map(event.to).to_string(),
            looping: true,
        });
    }
}

fn emit_footsteps(
    time: Res<Time>,
    state: Res<State<GameState>>,
    mut cadence: ResMut<FootstepCadence>,
    player_query: Query<(&Transform, &PlayerMovement), With<Player>>,
    mut sfx_events: EventWriter<PlaySfxEvent>,
) {
    if *state.get() != GameState::Playing {
        cadence.timer.reset();
        cadence.last_position = None;
        return;
    }

    let Ok((transform, movement)) = player_query.get_single() else {
        cadence.timer.reset();
        cadence.last_position = None;
        return;
    };

    let position = transform.translation.truncate();
    let Some(last_position) = cadence.last_position.replace(position) else {
        cadence.timer.reset();
        return;
    };

    let distance = position.distance(last_position);
    if distance < FOOTSTEP_DISTANCE_THRESHOLD {
        cadence.timer.reset();
        return;
    }

    cadence
        .timer
        .set_duration(std::time::Duration::from_secs_f32(if movement.is_running {
            FOOTSTEP_RUN_INTERVAL_SECS
        } else {
            FOOTSTEP_WALK_INTERVAL_SECS
        }));
    cadence.timer.tick(time.delta());

    if cadence.timer.just_finished() {
        sfx_events.send(named_sfx("footstep"));
    }
}

fn spawn_looped_audio(
    commands: &mut Commands,
    asset_server: &AssetServer,
    channel: LoopAudioChannel,
    path: &'static str,
    target_volume: f32,
    despawn_when_silent: bool,
) {
    commands.spawn((
        AudioPlayer::new(asset_server.load(path)),
        PlaybackSettings::LOOP.with_volume(Volume::ZERO),
        ManagedLoopAudio {
            channel,
            current_volume: 0.0,
            target_volume,
            fade_speed: AMBIENT_FADE_SPEED,
            despawn_when_silent,
        },
    ));
}

fn sfx_asset(name: &str) -> Option<(&'static str, f32)> {
    match canonical_sfx_name(name)? {
        "menu_click" => Some(("audio/sfx/menu_click.ogg", 0.45)),
        "case_assigned" => Some(("audio/sfx/case_assigned.ogg", 0.55)),
        "evidence_collected" => Some(("audio/sfx/evidence_collected.ogg", 0.52)),
        "dispatch_call" => Some(("audio/sfx/dispatch_call.ogg", 0.60)),
        "toast_pop" => Some(("audio/sfx/toast_pop.ogg", 0.35)),
        "door_open" => Some(("audio/sfx/door_open.ogg", 0.50)),
        "footstep" => Some(("audio/sfx/footstep.ogg", 0.28)),
        "coffee_pour" => Some(("audio/sfx/coffee_pour.ogg", 0.42)),
        "shift_end" => Some(("audio/sfx/shift_end.ogg", 0.58)),
        "promotion" => Some(("audio/sfx/promotion.ogg", 0.62)),
        "case_solved" => Some(("audio/sfx/case_solved.ogg", 0.62)),
        "case_failed" => Some(("audio/sfx/case_failed.ogg", 0.60)),
        _ => None,
    }
}

fn canonical_sfx_name(name: &str) -> Option<&'static str> {
    match name {
        "menu_click" | "ui_confirm" => Some("menu_click"),
        "case_assigned" => Some("case_assigned"),
        "evidence_collected" => Some("evidence_collected"),
        "dispatch_call" => Some("dispatch_call"),
        "toast_pop" => Some("toast_pop"),
        "door_open" => Some("door_open"),
        "footstep" => Some("footstep"),
        "coffee_pour" => Some("coffee_pour"),
        "shift_end" => Some("shift_end"),
        "promotion" => Some("promotion"),
        "case_solved" => Some("case_solved"),
        "case_failed" => Some("case_failed"),
        _ => None,
    }
}

fn music_asset(name: &str) -> Option<&'static str> {
    match canonical_music_name(name)? {
        "precinct_theme" => Some("audio/music/precinct_theme.ogg"),
        "patrol_theme" => Some("audio/music/patrol_theme.ogg"),
        "downtown_ambient" => Some("audio/music/downtown_ambient.ogg"),
        "tension_theme" => Some("audio/music/tension_theme.ogg"),
        _ => None,
    }
}

fn canonical_music_name(name: &str) -> Option<&'static str> {
    match name {
        "precinct_theme" | "main_menu_theme" => Some("precinct_theme"),
        "patrol_theme" | "patrol_shift_theme" => Some("patrol_theme"),
        "downtown_ambient" => Some("downtown_ambient"),
        "tension_theme" => Some("tension_theme"),
        _ => None,
    }
}

fn named_sfx(name: &str) -> PlaySfxEvent {
    PlaySfxEvent {
        name: name.to_string(),
    }
}

fn move_towards(current: f32, target: f32, max_delta: f32) -> f32 {
    if (target - current).abs() <= max_delta {
        target
    } else if target > current {
        current + max_delta
    } else {
        current - max_delta
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use bevy::ecs::event::Events;
    use bevy::state::app::StatesPlugin;
    use bevy::time::TimeUpdateStrategy;
    use std::time::Duration;

    fn build_test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(StatesPlugin);
        app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
            0.25,
        )));
        app.init_state::<GameState>();
        app.configure_sets(
            Update,
            (
                UpdatePhase::Input,
                UpdatePhase::Intent,
                UpdatePhase::Simulation,
                UpdatePhase::Reactions,
                UpdatePhase::Presentation,
            )
                .chain(),
        );
        app.init_resource::<ShiftClock>()
            .init_resource::<PatrolState>()
            .init_resource::<PlayerState>()
            .add_event::<CaseAssignedEvent>()
            .add_event::<EvidenceCollectedEvent>()
            .add_event::<DispatchCallEvent>()
            .add_event::<ShiftEndEvent>()
            .add_event::<PromotionEvent>()
            .add_event::<CaseSolvedEvent>()
            .add_event::<CaseFailedEvent>()
            .add_event::<MapTransitionEvent>()
            .add_event::<PlayMusicEvent>()
            .add_event::<PlaySfxEvent>()
            .add_event::<ToastEvent>()
            .init_resource::<NextState<GameState>>();
        build_audio(&mut app);

        app
    }

    fn enter_playing(app: &mut App) {
        app.world_mut()
            .resource_mut::<NextState<GameState>>()
            .set(GameState::Playing);
        app.update();
        app.update();
    }

    #[test]
    fn map_music_follows_destination() {
        assert_eq!(music_for_map(MapId::PrecinctInterior), "precinct_theme");
        assert_eq!(music_for_map(MapId::PrecinctExterior), "patrol_theme");
        assert_eq!(music_for_map(MapId::Downtown), "downtown_ambient");
        assert_eq!(music_for_map(MapId::CrimeSceneTemplate), "tension_theme");
    }

    #[test]
    fn map_transition_emits_music_and_door_audio() {
        let mut app = build_test_app();
        enter_playing(&mut app);

        app.world_mut().send_event(MapTransitionEvent {
            from: MapId::PrecinctInterior,
            to: MapId::Downtown,
        });
        app.update();

        let music_events = app.world().resource::<Events<PlayMusicEvent>>();
        let mut music_reader = music_events.get_cursor();
        let requested_tracks: Vec<_> = music_reader
            .read(music_events)
            .map(|event| event.name.clone())
            .collect();
        assert!(requested_tracks.contains(&"downtown_ambient".to_string()));

        let sfx_events = app.world().resource::<Events<PlaySfxEvent>>();
        let mut sfx_reader = sfx_events.get_cursor();
        let requested_sfx: Vec<_> = sfx_reader
            .read(sfx_events)
            .map(|event| event.name.clone())
            .collect();
        assert!(requested_sfx.contains(&"door_open".to_string()));
    }

    #[test]
    fn footsteps_emit_when_player_is_moving() {
        let mut app = build_test_app();
        enter_playing(&mut app);

        let player_entity = app
            .world_mut()
            .spawn((
                Player,
                PlayerMovement {
                    speed: 80.0,
                    facing: crate::shared::Facing::Down,
                    is_running: false,
                },
                Transform::default(),
                GlobalTransform::default(),
            ))
            .id();

        app.update();

        app.world_mut()
            .entity_mut(player_entity)
            .get_mut::<Transform>()
            .expect("player transform should exist")
            .translation
            .x = 1.0;

        app.update();
        app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
            0.5,
        )));
        app.world_mut()
            .entity_mut(player_entity)
            .get_mut::<Transform>()
            .expect("player transform should exist")
            .translation
            .x = 2.0;
        app.update();

        let sfx_events = app.world().resource::<Events<PlaySfxEvent>>();
        let mut reader = sfx_events.get_cursor();
        let emitted: Vec<_> = reader
            .read(sfx_events)
            .map(|event| event.name.clone())
            .collect();
        assert!(emitted.contains(&"footstep".to_string()));
    }

    #[test]
    fn idle_dispatch_enables_radio_static_target() {
        let mut app = build_test_app();
        enter_playing(&mut app);

        app.world_mut().spawn(ManagedLoopAudio {
            channel: LoopAudioChannel::ClockTick,
            current_volume: 0.0,
            target_volume: 0.0,
            fade_speed: 0.0,
            despawn_when_silent: false,
        });
        let radio_entity = app
            .world_mut()
            .spawn(ManagedLoopAudio {
                channel: LoopAudioChannel::RadioStatic,
                current_volume: 0.0,
                target_volume: 0.0,
                fade_speed: 0.0,
                despawn_when_silent: false,
            })
            .id();

        {
            let mut player_state = app.world_mut().resource_mut::<PlayerState>();
            player_state.position_map = MapId::PrecinctInterior;
        }

        app.update();

        let radio = app
            .world()
            .entity(radio_entity)
            .get::<ManagedLoopAudio>()
            .expect("radio loop should exist");
        assert!(radio.target_volume > 0.0);
    }
}
