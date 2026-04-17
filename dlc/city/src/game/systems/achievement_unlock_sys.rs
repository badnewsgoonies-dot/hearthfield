use bevy::prelude::*;
use crate::game::resources::AchievementState;
use crate::game::events::AchievementUnlockedEvent;

pub fn achievement_unlock_system(mut state: ResMut<AchievementState>, mut reader: EventReader<AchievementUnlockedEvent>) {
    for _ev in reader.read() {
        state.unlocked = state.unlocked.saturating_add(1);
    }

}
