use bevy::prelude::*;
use crate::game::resources::ReputationState;

pub fn reputation_tier_system(mut rep: ResMut<ReputationState>) {
    let new_tier = (rep.score / 200).max(0) as u8;
    rep.tier = new_tier;

}
