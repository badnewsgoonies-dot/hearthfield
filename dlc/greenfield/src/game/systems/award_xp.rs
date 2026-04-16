use bevy::prelude::*;
use crate::game::resources::LevelProgress;

pub fn award_xp_system(mut level_progress: ResMut<LevelProgress>) {
    level_progress.xp += 1;
}
