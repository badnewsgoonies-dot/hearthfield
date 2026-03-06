//! Cutscene / event sequence runner — plays scripted CutsceneStep sequences.

use crate::shared::*;
use bevy::prelude::*;

// ─── Pre-built Cutscene Constructors ─────────────────────────────────────

/// Opening cutscene when the player first starts the game.
pub fn intro_cutscene() -> Vec<CutsceneStep> {
    vec![
        CutsceneStep::FadeOut { duration: 0.5 },
        CutsceneStep::PlayMusic {
            track_id: "mus_intro".into(),
        },
        CutsceneStep::FadeIn { duration: 1.0 },
        CutsceneStep::Dialogue {
            speaker: "Narrator".into(),
            text: "You arrive at Windridge Aviation Academy, ready to start your pilot career..."
                .into(),
        },
        CutsceneStep::Wait { seconds: 1.5 },
        CutsceneStep::Dialogue {
            speaker: "Narrator".into(),
            text: "The smell of jet fuel and fresh tarmac greets you as you step off the bus."
                .into(),
        },
        CutsceneStep::Wait { seconds: 1.0 },
        CutsceneStep::Dialogue {
            speaker: "Captain Elena".into(),
            text: "Welcome, cadet! I'm Captain Elena. I'll be your flight instructor.".into(),
        },
        CutsceneStep::Dialogue {
            speaker: "Captain Elena".into(),
            text: "Let me show you around Clearfield Regional. Follow me to the terminal.".into(),
        },
        CutsceneStep::FadeOut { duration: 0.5 },
        CutsceneStep::Teleport {
            airport: AirportId::HomeBase,
            zone: MapZone::Terminal,
            x: 8,
            y: 10,
        },
        CutsceneStep::FadeIn { duration: 0.5 },
    ]
}

/// Celebration after the player completes their first solo flight.
pub fn first_solo_cutscene() -> Vec<CutsceneStep> {
    vec![
        CutsceneStep::PlaySfx {
            sfx_id: "sfx_applause".into(),
        },
        CutsceneStep::PlayMusic {
            track_id: "mus_celebration".into(),
        },
        CutsceneStep::Dialogue {
            speaker: "Captain Elena".into(),
            text: "Congratulations! You just completed your first solo flight!".into(),
        },
        CutsceneStep::Dialogue {
            speaker: "Captain Elena".into(),
            text: "I knew you had it in you. The sky is your office now, pilot.".into(),
        },
        CutsceneStep::Dialogue {
            speaker: "Mechanic Hank".into(),
            text: "Not bad for a greenhorn! Your landing was... mostly straight.".into(),
        },
        CutsceneStep::Wait { seconds: 1.0 },
        CutsceneStep::Dialogue {
            speaker: "Narrator".into(),
            text: "Your first solo flight is logged. A milestone you'll never forget.".into(),
        },
    ]
}

/// Ceremony cutscene when the player earns a new rank.
pub fn rank_up_cutscene(rank: PilotRank) -> Vec<CutsceneStep> {
    let rank_name = rank.display_name();
    vec![
        CutsceneStep::FadeOut { duration: 0.3 },
        CutsceneStep::PlayMusic {
            track_id: "mus_rank_up".into(),
        },
        CutsceneStep::FadeIn { duration: 0.5 },
        CutsceneStep::PlaySfx {
            sfx_id: "sfx_fanfare".into(),
        },
        CutsceneStep::Dialogue {
            speaker: "Flight Director".into(),
            text: format!(
                "Attention! By the authority of Windridge Aviation, you are hereby promoted to {}.",
                rank_name
            ),
        },
        CutsceneStep::Wait { seconds: 1.0 },
        CutsceneStep::Dialogue {
            speaker: "Flight Director".into(),
            text: "Your dedication to the skies has earned this honour. Fly safe, pilot.".into(),
        },
        CutsceneStep::PlaySfx {
            sfx_id: "sfx_applause".into(),
        },
        CutsceneStep::Wait { seconds: 1.5 },
        CutsceneStep::FadeOut { duration: 0.5 },
        CutsceneStep::FadeIn { duration: 0.5 },
    ]
}

/// Dramatic aftermath when the player survives an in-flight emergency.
pub fn emergency_survived_cutscene() -> Vec<CutsceneStep> {
    vec![
        CutsceneStep::SetWeather {
            weather: Weather::Clear,
        },
        CutsceneStep::PlayMusic {
            track_id: "mus_relief".into(),
        },
        CutsceneStep::Dialogue {
            speaker: "ATC".into(),
            text: "All stations, aircraft is down safely. Emergency services standing down.".into(),
        },
        CutsceneStep::Wait { seconds: 1.0 },
        CutsceneStep::Dialogue {
            speaker: "Captain Elena".into(),
            text: "That was incredible flying. You kept your cool under pressure.".into(),
        },
        CutsceneStep::Dialogue {
            speaker: "Mechanic Hank".into(),
            text: "I'll check the aircraft over. You go sit down \u{2014} you've earned it.".into(),
        },
        CutsceneStep::Dialogue {
            speaker: "Narrator".into(),
            text: "You survived. The crew looks at you with newfound respect.".into(),
        },
    ]
}

/// Montage cutscene after visiting every airport.
pub fn all_airports_cutscene() -> Vec<CutsceneStep> {
    vec![
        CutsceneStep::FadeOut { duration: 0.5 },
        CutsceneStep::PlayMusic {
            track_id: "mus_epic".into(),
        },
        CutsceneStep::FadeIn { duration: 0.8 },
        CutsceneStep::Dialogue {
            speaker: "Narrator".into(),
            text: "From the alpine runways of Frostpeak to the shimmering coast of Sunhaven..."
                .into(),
        },
        CutsceneStep::Wait { seconds: 1.5 },
        CutsceneStep::Dialogue {
            speaker: "Narrator".into(),
            text: "From the forges of Ironforge to the elite strips of Skyreach...".into(),
        },
        CutsceneStep::Wait { seconds: 1.5 },
        CutsceneStep::Dialogue {
            speaker: "Narrator".into(),
            text: "You've flown them all. Every airport, every runway, every horizon.".into(),
        },
        CutsceneStep::Dialogue {
            speaker: "Narrator".into(),
            text: "The skies have no more secrets from you, Ace.".into(),
        },
        CutsceneStep::FadeOut { duration: 1.0 },
        CutsceneStep::FadeIn { duration: 0.5 },
    ]
}

// ─── Components ──────────────────────────────────────────────────────────

#[derive(Component)]
pub struct CutsceneOverlay;

#[derive(Component)]
pub struct CutsceneTextBox;

// ─── Systems ─────────────────────────────────────────────────────────────

/// Spawns the cutscene UI overlay (dark background + text area).
pub fn spawn_cutscene_overlay(mut commands: Commands, font: Res<UiFontHandle>) {
    let text_style = TextFont {
        font: font.0.clone(),
        font_size: 16.0,
        ..default()
    };

    commands
        .spawn((
            CutsceneOverlay,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::FlexEnd,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(24.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.85)),
        ))
        .with_children(|parent| {
            parent.spawn((
                CutsceneTextBox,
                Text::new(""),
                text_style,
                TextColor(Color::srgb(1.0, 1.0, 1.0)),
                Node {
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(48.0),
                    left: Val::Px(48.0),
                    right: Val::Px(48.0),
                    padding: UiRect::all(Val::Px(16.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.6)),
            ));
        });
}

/// Despawn cutscene UI.
pub fn despawn_cutscene_overlay(
    mut commands: Commands,
    query: Query<Entity, With<CutsceneOverlay>>,
) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}

/// Main cutscene runner — processes the current `ActiveCutscene` step and
/// advances to the next on completion or player input.
#[allow(clippy::too_many_arguments)]
pub fn run_cutscene(
    time: Res<Time>,
    mut active: ResMut<ActiveCutscene>,
    mut cutscene_queue: ResMut<CutsceneQueue>,
    input: Res<PlayerInput>,
    mut text_query: Query<&mut Text, With<CutsceneTextBox>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut screen_fade: EventWriter<ScreenFadeEvent>,
    mut play_music: EventWriter<PlayMusicEvent>,
    mut play_sfx: EventWriter<PlaySfxEvent>,
    mut weather_change: EventWriter<WeatherChangeEvent>,
    _day_end: EventWriter<DayEndEvent>,
    mut zone_transition: EventWriter<ZoneTransitionEvent>,
) {
    if active.steps.is_empty() {
        // Try to start the next queued cutscene.
        if let Some(steps) = cutscene_queue.pending.pop() {
            active.steps = steps;
            active.current_step = 0;
            active.timer = 0.0;
            active.waiting_for_input = false;
        } else {
            next_state.set(GameState::Playing);
            return;
        }
    }

    if active.current_step >= active.steps.len() {
        // Cutscene finished.
        active.steps.clear();
        active.current_step = 0;
        return;
    }

    let step = active.steps[active.current_step].clone();

    // If waiting for player input to advance dialogue, check confirm.
    if active.waiting_for_input {
        if input.confirm || input.interact {
            active.waiting_for_input = false;
            active.current_step += 1;
            active.timer = 0.0;
        }
        return;
    }

    match step {
        CutsceneStep::Dialogue { speaker, text } => {
            let display = format!("[{}]: {}", speaker, text);
            for mut txt in &mut text_query {
                **txt = display.clone();
            }
            active.waiting_for_input = true;
        }
        CutsceneStep::Wait { seconds } => {
            active.timer += time.delta_secs();
            if active.timer >= seconds {
                active.current_step += 1;
                active.timer = 0.0;
            }
        }
        CutsceneStep::FadeOut { duration } => {
            screen_fade.send(ScreenFadeEvent {
                fade_in: false,
                duration_secs: duration,
            });
            active.current_step += 1;
        }
        CutsceneStep::FadeIn { duration } => {
            screen_fade.send(ScreenFadeEvent {
                fade_in: true,
                duration_secs: duration,
            });
            active.current_step += 1;
        }
        CutsceneStep::PlayMusic { track_id } => {
            play_music.send(PlayMusicEvent {
                track_id,
                fade_in: true,
            });
            active.current_step += 1;
        }
        CutsceneStep::PlaySfx { sfx_id } => {
            play_sfx.send(PlaySfxEvent { sfx_id });
            active.current_step += 1;
        }
        CutsceneStep::SetWeather { weather } => {
            weather_change.send(WeatherChangeEvent {
                new_weather: weather,
            });
            active.current_step += 1;
        }
        CutsceneStep::Teleport {
            airport,
            zone,
            x,
            y,
        } => {
            zone_transition.send(ZoneTransitionEvent {
                to_airport: airport,
                to_zone: zone,
                to_x: x,
                to_y: y,
            });
            active.current_step += 1;
        }
    }
}

/// Player can press Escape to skip the current cutscene entirely.
pub fn skip_cutscene(
    input: Res<PlayerInput>,
    mut active: ResMut<ActiveCutscene>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if input.cancel && !active.steps.is_empty() {
        active.steps.clear();
        active.current_step = 0;
        active.timer = 0.0;
        active.waiting_for_input = false;
        next_state.set(GameState::Playing);
    }
}

/// Listens for `CutsceneStartEvent` and queues the appropriate cutscene.
pub fn handle_cutscene_start(
    mut events: EventReader<CutsceneStartEvent>,
    mut cutscene_queue: ResMut<CutsceneQueue>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for evt in events.read() {
        let script = match evt.cutscene_id.as_str() {
            "intro" => Some(intro_cutscene()),
            "first_solo" => Some(first_solo_cutscene()),
            "rank_student" => Some(rank_up_cutscene(PilotRank::Student)),
            "rank_private" => Some(rank_up_cutscene(PilotRank::Private)),
            "rank_commercial" => Some(rank_up_cutscene(PilotRank::Commercial)),
            "rank_senior" => Some(rank_up_cutscene(PilotRank::Senior)),
            "rank_captain" => Some(rank_up_cutscene(PilotRank::Captain)),
            "rank_ace" => Some(rank_up_cutscene(PilotRank::Ace)),
            "emergency_survived" => Some(emergency_survived_cutscene()),
            "all_airports" => Some(all_airports_cutscene()),
            _ => None,
        };
        if let Some(steps) = script {
            cutscene_queue.pending.push(steps);
            next_state.set(GameState::Cutscene);
        }
    }
}

/// Fires rank-up cutscenes automatically when a `RankUpEvent` is received.
pub fn trigger_rank_cutscene(
    mut rank_events: EventReader<RankUpEvent>,
    mut cutscene_events: EventWriter<CutsceneStartEvent>,
) {
    for evt in rank_events.read() {
        let id = match evt.new_rank {
            PilotRank::Student => "rank_student",
            PilotRank::Private => "rank_private",
            PilotRank::Commercial => "rank_commercial",
            PilotRank::Senior => "rank_senior",
            PilotRank::Captain => "rank_captain",
            PilotRank::Ace => "rank_ace",
        };
        cutscene_events.send(CutsceneStartEvent {
            cutscene_id: id.into(),
        });
    }
}
