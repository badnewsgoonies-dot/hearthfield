use bevy::prelude::*;
use crate::game::resources::GameScore;

pub fn increment_score_system(mut game_score: ResMut<GameScore>) {
    game_score.total += 1;
}
