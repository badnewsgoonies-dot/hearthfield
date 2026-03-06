pub mod achievements;
pub mod aircraft;
pub mod aircraft_specs;
pub mod airport_layouts;
pub mod cities;
pub mod crew;
pub mod dialogue_lines;
pub mod events_calendar;
pub mod items;
pub mod missions;
pub mod routes;
pub mod shops;
pub mod tips;
pub mod weather_patterns;

use bevy::prelude::*;

use crate::shared::*;

pub struct DataPlugin;

impl Plugin for DataPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Loading), load_all_data);
    }
}

fn load_all_data(
    mut item_registry: ResMut<ItemRegistry>,
    mut aircraft_registry: ResMut<AircraftRegistry>,
    mut crew_registry: ResMut<CrewRegistry>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    items::populate_items(&mut item_registry);
    aircraft::populate_aircraft(&mut aircraft_registry);
    crew::populate_crew(&mut crew_registry);

    info!(
        "Data loaded: {} items, {} aircraft, {} crew",
        item_registry.items.len(),
        aircraft_registry.aircraft.len(),
        crew_registry.members.len(),
    );

    next_state.set(GameState::MainMenu);
}
