//! Tutorial system — guides new players through first steps at Windridge Aviation.

use bevy::prelude::*;
use std::collections::HashSet;

use crate::shared::*;

// ─── Tutorial Step Definitions ───────────────────────────────────────────

/// Each discrete tutorial milestone the player can reach.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TutorialStep {
    Welcome,
    Movement,
    Interaction,
    MissionBoard,
    AcceptMission,
    Preflight,
    Takeoff,
    InFlight,
    Landing,
    PostFlight,
    Exploration,
    CrewInteraction,
    ShopVisit,
    Complete,
}

impl TutorialStep {
    /// The contextual hint message shown for this step.
    pub fn hint_message(&self) -> &'static str {
        match self {
            TutorialStep::Welcome => {
                "Welcome to Windridge Aviation! Use WASD to walk around the terminal."
            }
            TutorialStep::Movement => {
                "Great! Head to the Mission Board (the glowing panel) and press F to interact."
            }
            TutorialStep::Interaction => {
                "You can interact with objects and people by pressing F when nearby."
            }
            TutorialStep::MissionBoard => {
                "This is your mission board. Select a mission and press F to accept it."
            }
            TutorialStep::AcceptMission => {
                "Mission accepted! Head to the Hangar to begin your preflight checks."
            }
            TutorialStep::Preflight => {
                "Press F at each station to verify your aircraft systems."
            }
            TutorialStep::Takeoff => {
                "Hold W to increase throttle. Watch your speed \u{2014} rotate at V1!"
            }
            TutorialStep::InFlight => {
                "Use A/D to adjust heading. Watch your fuel gauge and navigation."
            }
            TutorialStep::Landing => {
                "Reduce throttle and align with the runway. Smooth landings earn bonus XP!"
            }
            TutorialStep::PostFlight => {
                "Flight complete! Check your logbook to see your stats."
            }
            TutorialStep::Exploration => {
                "Explore the city! Visit shops, talk to crew, find collectibles."
            }
            TutorialStep::CrewInteraction => {
                "Talk to crew members to build friendships. Give gifts they like!"
            }
            TutorialStep::ShopVisit => {
                "Buy supplies, gifts, and aircraft parts at the shop."
            }
            TutorialStep::Complete => "Tutorial complete! The sky is yours, pilot.",
        }
    }

    /// The natural next step, if any.
    fn next(&self) -> Option<TutorialStep> {
        match self {
            TutorialStep::Welcome => Some(TutorialStep::Movement),
            TutorialStep::Movement => Some(TutorialStep::Interaction),
            TutorialStep::Interaction => Some(TutorialStep::MissionBoard),
            TutorialStep::MissionBoard => Some(TutorialStep::AcceptMission),
            TutorialStep::AcceptMission => Some(TutorialStep::Preflight),
            TutorialStep::Preflight => Some(TutorialStep::Takeoff),
            TutorialStep::Takeoff => Some(TutorialStep::InFlight),
            TutorialStep::InFlight => Some(TutorialStep::Landing),
            TutorialStep::Landing => Some(TutorialStep::PostFlight),
            TutorialStep::PostFlight => Some(TutorialStep::Exploration),
            TutorialStep::Exploration => Some(TutorialStep::CrewInteraction),
            TutorialStep::CrewInteraction => Some(TutorialStep::ShopVisit),
            TutorialStep::ShopVisit => Some(TutorialStep::Complete),
            TutorialStep::Complete => None,
        }
    }
}

// ─── Tutorial Tracker Resource ───────────────────────────────────────────

/// Tracks which tutorial steps the player has completed and the currently
/// active step. Lives alongside the shared `TutorialState` which is
/// serialised in save files.
#[derive(Resource)]
pub struct TutorialTracker {
    pub current_step: TutorialStep,
    pub completed_steps: HashSet<TutorialStep>,
    pub show_hints: bool,
    pub hint_timer: Timer,
}

impl Default for TutorialTracker {
    fn default() -> Self {
        Self {
            current_step: TutorialStep::Welcome,
            completed_steps: HashSet::new(),
            show_hints: true,
            hint_timer: Timer::from_seconds(1.0, TimerMode::Once),
        }
    }
}

impl TutorialTracker {
    /// Mark the current step done and advance to the next one.
    pub fn advance(&mut self) {
        self.completed_steps.insert(self.current_step);
        if let Some(next) = self.current_step.next() {
            self.current_step = next;
            self.hint_timer.reset();
        } else {
            self.current_step = TutorialStep::Complete;
        }
    }

    pub fn is_complete(&self) -> bool {
        self.current_step == TutorialStep::Complete
            && self.completed_steps.contains(&TutorialStep::Complete)
    }

    pub fn skip(&mut self) {
        self.current_step = TutorialStep::Complete;
        self.completed_steps.insert(TutorialStep::Complete);
        self.show_hints = false;
    }
}

// ─── Components ──────────────────────────────────────────────────────────

#[derive(Component)]
pub struct TutorialHintBox;

// ─── Systems ─────────────────────────────────────────────────────────────

/// Watches game conditions and auto-advances the tutorial when milestones
/// are reached (e.g. player moved → Movement done).
pub fn advance_tutorial(
    mut tracker: ResMut<TutorialTracker>,
    player_movement: Res<PlayerMovement>,
    location: Res<PlayerLocation>,
    mission_board: Res<MissionBoard>,
    flight_state: Res<FlightState>,
    pilot_state: Res<PilotState>,
    settings: Res<crate::ui::settings::GameSettings>,
) {
    if !tracker.show_hints || tracker.is_complete() {
        return;
    }
    if !settings.tutorial_tips {
        tracker.skip();
        return;
    }

    let should_advance = match tracker.current_step {
        TutorialStep::Welcome => player_movement.is_moving,
        TutorialStep::Movement => {
            player_movement.is_moving
                && !tracker.completed_steps.contains(&TutorialStep::Movement)
        }
        TutorialStep::Interaction => location.zone == MapZone::Terminal,
        TutorialStep::MissionBoard => mission_board.available.len() > 0,
        TutorialStep::AcceptMission => mission_board.active.is_some(),
        TutorialStep::Preflight => {
            location.zone == MapZone::Hangar
                && flight_state.phase == FlightPhase::Preflight
        }
        TutorialStep::Takeoff => flight_state.phase == FlightPhase::Takeoff,
        TutorialStep::InFlight => {
            flight_state.phase == FlightPhase::Cruise
                || flight_state.phase == FlightPhase::Climb
        }
        TutorialStep::Landing => flight_state.phase == FlightPhase::Arrived,
        TutorialStep::PostFlight => pilot_state.total_flights >= 1,
        TutorialStep::Exploration => location.zone == MapZone::CityStreet,
        TutorialStep::CrewInteraction => location.zone == MapZone::CrewQuarters,
        TutorialStep::ShopVisit => location.zone == MapZone::Shop,
        TutorialStep::Complete => false,
    };

    if should_advance {
        tracker.advance();
    }
}

/// Sends a `HintEvent` for the current tutorial step so the toast / HUD
/// system can render it. Only fires once per step (debounced via timer).
pub fn show_tutorial_hint(
    mut tracker: ResMut<TutorialTracker>,
    time: Res<Time>,
    mut hint_events: EventWriter<HintEvent>,
    settings: Res<crate::ui::settings::GameSettings>,
) {
    if !tracker.show_hints || tracker.is_complete() || !settings.tutorial_tips {
        return;
    }

    tracker.hint_timer.tick(time.delta());
    if tracker.hint_timer.just_finished() {
        hint_events.send(HintEvent {
            hint_id: format!("tutorial_{:?}", tracker.current_step),
            message: tracker.current_step.hint_message().to_string(),
        });
    }
}

/// Spawn the tutorial hint overlay box anchored to the top of the screen.
pub fn spawn_tutorial_hint_box(mut commands: Commands, font: Res<UiFontHandle>) {
    let text_style = TextFont {
        font: font.0.clone(),
        font_size: 14.0,
        ..default()
    };

    commands.spawn((
        TutorialHintBox,
        Text::new(""),
        text_style,
        TextColor(Color::srgb(1.0, 1.0, 0.85)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(12.0),
            left: Val::Percent(20.0),
            right: Val::Percent(20.0),
            padding: UiRect::all(Val::Px(8.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
    ));
}

/// Updates the tutorial hint box text to reflect the current step.
pub fn update_tutorial_hint_box(
    tracker: Res<TutorialTracker>,
    mut query: Query<(&mut Text, &mut Node), With<TutorialHintBox>>,
) {
    for (mut text, mut node) in &mut query {
        if tracker.is_complete() || !tracker.show_hints {
            node.display = Display::None;
        } else {
            node.display = Display::Flex;
            **text = tracker.current_step.hint_message().to_string();
        }
    }
}

/// Despawn the tutorial hint box.
pub fn despawn_tutorial_hint_box(
    mut commands: Commands,
    query: Query<Entity, With<TutorialHintBox>>,
) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}

/// Allow the player to skip the entire tutorial.
pub fn handle_skip_tutorial(
    input: Res<PlayerInput>,
    mut tracker: ResMut<TutorialTracker>,
    mut tutorial_state: ResMut<TutorialState>,
) {
    if input.cancel && !tracker.is_complete() {
        tracker.skip();
        tutorial_state.active = false;
        tutorial_state
            .completed_hints
            .push("tutorial_skipped".to_string());
    }
}
