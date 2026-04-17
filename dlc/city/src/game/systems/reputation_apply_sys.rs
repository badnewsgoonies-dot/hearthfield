use bevy::prelude::*;
use crate::game::resources::ReputationState;
use crate::game::events::ReputationGainedEvent;

pub fn reputation_apply_system(mut rep: ResMut<ReputationState>, mut reader: EventReader<ReputationGainedEvent>) {
    for _ev in reader.read() {
        rep.score = rep.score.saturating_add(10);
    }

}
