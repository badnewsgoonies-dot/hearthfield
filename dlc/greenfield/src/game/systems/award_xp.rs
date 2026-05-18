use bevy::prelude::*;
use crate::game::resources::LevelProgress;
use crate::game::events::ExperienceGainedEvent;

pub fn award_xp_system(mut events: EventReader<ExperienceGainedEvent>, mut state: ResMut<LevelProgress>) {
    for ev in events.read() {
        state.xp = state.xp.saturating_add(ev.amount);
        while state.xp >= 100 {
            state.xp -= 100;
            state.level = state.level.saturating_add(1);
        }
    }
}
