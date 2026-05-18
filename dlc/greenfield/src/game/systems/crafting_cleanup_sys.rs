use bevy::prelude::*;
use crate::game::events::RecipeUnlockedEvent;

pub fn crafting_cleanup_system(mut events: EventReader<RecipeUnlockedEvent>) {
    let _drained = events.read().count();
}
