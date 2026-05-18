use bevy::prelude::*;
use crate::game::resources::*;
use crate::game::events::*;

/// Runs in PostUpdate while Playing. If trigger_sleep or tick_time
/// populated the cutscene queue this frame, activate it and transition
/// to Cutscene. This runs AFTER all Update systems have processed
/// their DayEndEvents, preventing the race where Playing-gated readers
/// miss the event because the state changed too early.
pub fn checkpoint_activate_system(
    mut queue: ResMut<CutsceneQueue>,
    mut next_state: ResMut<NextState<GameState>>,
    mut fade: ResMut<ScreenFade>,
) {
    if !queue.steps.is_empty() && !queue.active {
        queue.active = true;
        queue.step_timer = 0.0;
        fade.active = true;
        next_state.set(GameState::Cutscene);
    }
}


