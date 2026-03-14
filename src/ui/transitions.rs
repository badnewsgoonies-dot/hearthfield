use crate::shared::*;
use crate::save::{LoadCompleteEvent, LoadRequestEvent};
use bevy::prelude::*;

/// Marker for the screen fade overlay
#[derive(Component)]
pub struct ScreenFadeOverlay;

#[derive(Clone, Copy)]
pub enum ScreenFadeTint {
    MapTransition,
    SaveLoad,
}

/// Resource that drives fade in/out
#[derive(Resource)]
pub struct ScreenFade {
    /// Current opacity 0.0 (transparent) to 1.0 (opaque black)
    pub alpha: f32,
    /// Target opacity
    pub target_alpha: f32,
    /// Speed of fade (alpha units per second)
    pub speed: f32,
    /// Whether a fade is actively running
    pub active: bool,
    /// Seconds to hold at full black before fading back in
    pub hold_timer: f32,
    /// The color treatment for the current fade.
    pub tint: ScreenFadeTint,
    /// Marks that the next map transition came from a load handoff.
    pub pending_save_load_handoff: bool,
}

/// Fade speed for map transitions: 1.0 / 0.42s = ~2.38 alpha/s.
const MAP_TRANSITION_FADE_SPEED: f32 = 1.0 / 0.42;
/// Fade speed for save/load handoffs: 1.0 / 0.58s = ~1.72 alpha/s.
const SAVE_LOAD_FADE_SPEED: f32 = 1.0 / 0.58;
const MAP_TRANSITION_HOLD_TIME: f32 = 0.16;
const SAVE_LOAD_HOLD_TIME: f32 = 0.28;

impl Default for ScreenFade {
    fn default() -> Self {
        Self {
            alpha: 0.0,
            target_alpha: 0.0,
            speed: MAP_TRANSITION_FADE_SPEED,
            active: false,
            hold_timer: 0.0,
            tint: ScreenFadeTint::MapTransition,
            pending_save_load_handoff: false,
        }
    }
}

/// Spawn the fade overlay (always present but invisible)
pub fn spawn_fade_overlay(mut commands: Commands) {
    commands.insert_resource(ScreenFade::default());

    commands.spawn((
        ScreenFadeOverlay,
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.0)),
        GlobalZIndex(100), // on top of everything
        PickingBehavior::IGNORE,
    ));
}

/// Listen for map transition events and trigger a fade
pub fn trigger_fade_on_transition(
    mut load_requests: EventReader<LoadRequestEvent>,
    mut load_completions: EventReader<LoadCompleteEvent>,
    mut events: EventReader<MapTransitionEvent>,
    mut fade: ResMut<ScreenFade>,
) {
    if load_requests.read().next().is_some() {
        fade.pending_save_load_handoff = true;
    }

    for completion in load_completions.read() {
        if !completion.success {
            fade.pending_save_load_handoff = false;
        }
    }

    for _event in events.read() {
        fade.target_alpha = 1.0;
        if fade.pending_save_load_handoff {
            fade.speed = SAVE_LOAD_FADE_SPEED;
            fade.hold_timer = SAVE_LOAD_HOLD_TIME;
            fade.tint = ScreenFadeTint::SaveLoad;
            fade.pending_save_load_handoff = false;
        } else {
            fade.speed = MAP_TRANSITION_FADE_SPEED;
            fade.hold_timer = MAP_TRANSITION_HOLD_TIME;
            fade.tint = ScreenFadeTint::MapTransition;
        }
        fade.active = true;
    }
}

/// Animate the fade overlay
pub fn update_fade(
    time: Res<Time>,
    mut fade: ResMut<ScreenFade>,
    mut query: Query<&mut BackgroundColor, With<ScreenFadeOverlay>>,
) {
    if !fade.active {
        return;
    }

    let dt = time.delta_secs();
    let diff = fade.target_alpha - fade.alpha;

    if diff.abs() < 0.01 {
        fade.alpha = fade.target_alpha;
        // If we've faded to black, hold briefly then fade back in
        if fade.target_alpha >= 0.99 {
            if fade.hold_timer > 0.0 {
                fade.hold_timer -= dt;
            } else {
                fade.target_alpha = 0.0;
            }
        } else {
            fade.active = false;
        }
    } else {
        fade.alpha += diff.signum() * fade.speed * dt;
        fade.alpha = fade.alpha.clamp(0.0, 1.0);
    }

    for mut bg in &mut query {
        let color = match fade.tint {
            ScreenFadeTint::MapTransition => Color::srgba(0.0, 0.0, 0.0, fade.alpha),
            ScreenFadeTint::SaveLoad => Color::srgba(0.16, 0.11, 0.06, fade.alpha),
        };
        *bg = BackgroundColor(color);
    }
}
