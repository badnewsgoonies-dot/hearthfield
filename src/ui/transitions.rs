use crate::shared::*;
use bevy::prelude::*;

/// Marker for the screen fade overlay
#[derive(Component)]
pub struct ScreenFadeOverlay;

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
}

/// Default fade speed: 1.0 / 0.3s = ~3.33 alpha/s for 0.3s transitions.
const FADE_SPEED: f32 = 1.0 / 0.3;

impl Default for ScreenFade {
    fn default() -> Self {
        Self {
            alpha: 0.0,
            target_alpha: 0.0,
            speed: FADE_SPEED,
            active: false,
            hold_timer: 0.0,
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
    mut events: EventReader<MapTransitionEvent>,
    mut fade: ResMut<ScreenFade>,
) {
    for _event in events.read() {
        fade.target_alpha = 1.0;
        fade.speed = 2.5;
        fade.hold_timer = 0.1;
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
        *bg = BackgroundColor(Color::srgba(0.0, 0.0, 0.0, fade.alpha));
    }
}
