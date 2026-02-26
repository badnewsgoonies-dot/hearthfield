use bevy::prelude::*;

pub struct DataPlugin;

impl Plugin for DataPlugin {
    fn build(&self, _app: &mut App) {
        // TODO: Worker will replace — loads all registries in OnEnter(GameState::Loading)
    }
}
