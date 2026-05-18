use bevy::prelude::*;
use crate::game::resources::*;
use crate::game::events::*;

/// Listen for map transition events and trigger a fade
pub fn combat_initiate_system(
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


