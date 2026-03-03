//! Game intro / new-game sequence — studio logo, title, character creation,
//! backstory narration, then transition to Playing.

use bevy::prelude::*;
use crate::shared::*;

// ─── Intro Phase ─────────────────────────────────────────────────────────

/// Phases of the intro sequence.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum IntroPhase {
    #[default]
    StudioLogo,
    TitleReveal,
    CharacterCreate,
    BackstoryNarration,
    ArrivalAtWindridge,
}

impl IntroPhase {
    fn next(&self) -> Option<IntroPhase> {
        match self {
            IntroPhase::StudioLogo => Some(IntroPhase::TitleReveal),
            IntroPhase::TitleReveal => Some(IntroPhase::CharacterCreate),
            IntroPhase::CharacterCreate => Some(IntroPhase::BackstoryNarration),
            IntroPhase::BackstoryNarration => Some(IntroPhase::ArrivalAtWindridge),
            IntroPhase::ArrivalAtWindridge => None,
        }
    }
}

// ─── Intro State Resource ────────────────────────────────────────────────

/// Resource tracking the current intro phase, timers, and character choices.
#[derive(Resource)]
pub struct IntroState {
    pub phase: IntroPhase,
    pub phase_timer: Timer,
    pub player_name: String,
    pub portrait_color: Color,
    pub done: bool,
}

impl Default for IntroState {
    fn default() -> Self {
        Self {
            phase: IntroPhase::StudioLogo,
            phase_timer: Timer::from_seconds(3.0, TimerMode::Once),
            player_name: "Ace".to_string(),
            portrait_color: Color::srgb(0.3, 0.5, 0.8),
            done: false,
        }
    }
}

// ─── Portrait colour palette ─────────────────────────────────────────────

const PORTRAIT_COLORS: &[(Color, &str)] = &[
    (Color::srgb(0.3, 0.5, 0.8), "Sky Blue"),
    (Color::srgb(0.8, 0.3, 0.3), "Sunset Red"),
    (Color::srgb(0.3, 0.7, 0.4), "Forest Green"),
    (Color::srgb(0.7, 0.5, 0.2), "Gold"),
    (Color::srgb(0.5, 0.3, 0.7), "Violet"),
    (Color::srgb(0.2, 0.2, 0.2), "Charcoal"),
];

// ─── Components ──────────────────────────────────────────────────────────

#[derive(Component)]
pub struct IntroOverlay;

#[derive(Component)]
pub struct IntroText;

#[derive(Component)]
pub struct CharCreateNameLabel;

#[derive(Component)]
pub struct CharCreateColorLabel;

// ─── Backstory text ──────────────────────────────────────────────────────

const BACKSTORY_LINES: &[&str] = &[
    "You've always dreamed of the open sky...",
    "While other kids played in the park, you watched planes trace white lines across the blue.",
    "Now, with a one-way ticket and a heart full of ambition,",
    "you're headed to Windridge Aviation Academy.",
    "Your journey as a pilot begins today.",
];

// ─── Systems ─────────────────────────────────────────────────────────────

/// Spawn the intro overlay UI.
pub fn spawn_intro(mut commands: Commands, font: Res<UiFontHandle>) {
    let text_style = TextFont {
        font: font.0.clone(),
        font_size: 28.0,
        ..default()
    };

    commands
        .spawn((
            IntroOverlay,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(12.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 1.0)),
        ))
        .with_children(|parent| {
            parent.spawn((
                IntroText,
                Text::new("Skywarden Studios Presents"),
                text_style.clone(),
                TextColor(Color::srgb(0.8, 0.8, 0.8)),
            ));
        });
}

/// Despawn the intro overlay.
pub fn despawn_intro(mut commands: Commands, query: Query<Entity, With<IntroOverlay>>) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}

/// Main intro runner — sequences through intro phases. Timed phases auto-
/// advance; interactive phases require player input.
#[allow(clippy::too_many_arguments)]
pub fn run_intro(
    time: Res<Time>,
    mut intro: ResMut<IntroState>,
    input: Res<PlayerInput>,
    mut text_query: Query<&mut Text, With<IntroText>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut pilot: ResMut<PilotState>,
    mut screen_fade: EventWriter<ScreenFadeEvent>,
    mut play_music: EventWriter<PlayMusicEvent>,
) {
    if intro.done {
        return;
    }

    intro.phase_timer.tick(time.delta());

    match intro.phase {
        IntroPhase::StudioLogo => {
            for mut txt in &mut text_query {
                **txt = "Skywarden Studios Presents".to_string();
            }
            if intro.phase_timer.just_finished() || input.confirm || input.interact {
                advance_intro_phase(&mut intro);
                play_music.send(PlayMusicEvent {
                    track_id: "mus_title".into(),
                    fade_in: true,
                });
            }
        }
        IntroPhase::TitleReveal => {
            for mut txt in &mut text_query {
                **txt = "~ S K Y W A R D E N ~\nPilot Life Simulator".to_string();
            }
            if intro.phase_timer.just_finished() || input.confirm || input.interact {
                advance_intro_phase(&mut intro);
            }
        }
        IntroPhase::CharacterCreate => {
            let color_index = PORTRAIT_COLORS
                .iter()
                .position(|(c, _)| *c == intro.portrait_color)
                .unwrap_or(0);
            let color_name = PORTRAIT_COLORS[color_index].1;
            let display = format!(
                "Create Your Pilot\n\nName: {}\nUniform Color: {}\n\n[Press ENTER to confirm]",
                intro.player_name, color_name
            );
            for mut txt in &mut text_query {
                **txt = display.clone();
            }

            // Cycle colour with left/right
            if input.menu_right {
                let next = (color_index + 1) % PORTRAIT_COLORS.len();
                intro.portrait_color = PORTRAIT_COLORS[next].0;
            }
            if input.menu_left && color_index > 0 {
                intro.portrait_color = PORTRAIT_COLORS[color_index - 1].0;
            }

            if input.confirm || input.interact {
                pilot.name = intro.player_name.clone();
                advance_intro_phase(&mut intro);
            }
        }
        IntroPhase::BackstoryNarration => {
            let elapsed = intro.phase_timer.elapsed_secs();
            let line_index = (elapsed / 3.0).floor() as usize;
            let line = BACKSTORY_LINES
                .get(line_index)
                .unwrap_or(BACKSTORY_LINES.last().unwrap());
            for mut txt in &mut text_query {
                **txt = line.to_string();
            }
            if intro.phase_timer.just_finished() || input.confirm || input.interact {
                advance_intro_phase(&mut intro);
                screen_fade.send(ScreenFadeEvent {
                    fade_in: false,
                    duration_secs: 1.0,
                });
            }
        }
        IntroPhase::ArrivalAtWindridge => {
            for mut txt in &mut text_query {
                **txt = "Welcome to Windridge Aviation Academy.".to_string();
            }
            if intro.phase_timer.just_finished() || input.confirm || input.interact {
                intro.done = true;
                screen_fade.send(ScreenFadeEvent {
                    fade_in: true,
                    duration_secs: 0.5,
                });
                next_state.set(GameState::Playing);
            }
        }
    }
}

fn advance_intro_phase(intro: &mut IntroState) {
    if let Some(next) = intro.phase.next() {
        intro.phase = next;
        let dur = match next {
            IntroPhase::StudioLogo => 3.0,
            IntroPhase::TitleReveal => 4.0,
            IntroPhase::CharacterCreate => 999.0, // waits for input
            IntroPhase::BackstoryNarration => 15.0,
            IntroPhase::ArrivalAtWindridge => 4.0,
        };
        intro.phase_timer = Timer::from_seconds(dur, TimerMode::Once);
    }
}
