//! Lifeline — hospital shift sim DLC for Hearthfield.
//!
//! Topology mirrors precinct: 12 domain plugins that own their own
//! state; a frozen `shared` contract that carries cross-domain types;
//! briefcase transforms bulk-fill structural boilerplate via anchor
//! comments in each domain.

pub mod domains;
pub mod shared;

use bevy::prelude::*;
use shared::GameState;

pub struct LifelinePlugin;

impl Plugin for LifelinePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>()
            .add_plugins((
                domains::calendar::DomainPlugin,
                domains::player::DomainPlugin,
                domains::world::DomainPlugin,
                domains::ui::DomainPlugin,
                domains::patients::DomainPlugin,
                domains::diagnostics::DomainPlugin,
                domains::rounds::DomainPlugin,
                domains::pharmacy::DomainPlugin,
                domains::skills::DomainPlugin,
                domains::economy::DomainPlugin,
                domains::npcs::DomainPlugin,
                domains::save::DomainPlugin,
            ));
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(LifelinePlugin)
        .run();
}
