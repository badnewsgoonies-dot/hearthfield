//! Fleet management — buying, selling, hangar assignment, depreciation, starter aircraft.

use bevy::prelude::*;
use crate::shared::*;

fn hangar_capacity(airport: AirportId) -> usize {
    match airport {
        AirportId::HomeBase => 3,
        AirportId::Windport => 4,
        AirportId::Frostpeak => 2,
        AirportId::Sunhaven => 3,
        AirportId::Ironforge => 5,
        AirportId::Cloudmere => 2,
        AirportId::Duskhollow => 3,
        AirportId::Stormwatch => 2,
        AirportId::Grandcity => 6,
        AirportId::Skyreach => 4,
    }
}

#[derive(Resource, Default, Clone, Debug)]
pub struct HangarAssignments {
    pub assignments: std::collections::HashMap<String, AirportId>,
}

impl HangarAssignments {
    pub fn aircraft_at(&self, airport: AirportId) -> Vec<String> {
        self.assignments.iter()
            .filter(|(_, &ap)| ap == airport)
            .map(|(nick, _)| nick.clone())
            .collect()
    }

    pub fn space_available(&self, airport: AirportId) -> bool {
        self.aircraft_at(airport).len() < hangar_capacity(airport)
    }

    pub fn assign(&mut self, nickname: &str, airport: AirportId) -> bool {
        if !self.space_available(airport) { return false; }
        self.assignments.insert(nickname.to_string(), airport);
        true
    }
}

pub fn aircraft_value(ac: &OwnedAircraft, registry: &AircraftRegistry) -> u32 {
    let base = registry.get(&ac.aircraft_id).map_or(0, |d| d.purchase_price);
    let depreciation = (base as f32 * 0.02 * ac.total_flights as f32) as u32;
    base.saturating_sub(depreciation).max(base / 5)
}

pub fn purchase_aircraft(
    fleet: &mut Fleet,
    gold: &mut Gold,
    registry: &AircraftRegistry,
    hangars: &mut HangarAssignments,
    aircraft_id: &str,
    nickname: &str,
    airport: AirportId,
) -> Result<u32, &'static str> {
    let def = registry.get(aircraft_id).ok_or("Unknown aircraft type")?;
    if gold.amount < def.purchase_price { return Err("Not enough gold"); }
    if !hangars.space_available(airport) { return Err("No hangar space at this airport"); }

    gold.amount -= def.purchase_price;
    let owned = OwnedAircraft {
        aircraft_id: aircraft_id.to_string(),
        nickname: nickname.to_string(),
        condition: 100.0,
        fuel: def.fuel_capacity,
        total_flights: 0,
        customizations: Vec::new(),
    };
    fleet.aircraft.push(owned);
    hangars.assign(nickname, airport);
    Ok(def.purchase_price)
}

pub fn sell_aircraft(
    fleet: &mut Fleet,
    gold: &mut Gold,
    registry: &AircraftRegistry,
    hangars: &mut HangarAssignments,
    index: usize,
) -> Result<u32, &'static str> {
    if index >= fleet.aircraft.len() { return Err("Invalid aircraft index"); }
    if fleet.aircraft.len() <= 1 { return Err("Cannot sell your last aircraft"); }
    let ac = &fleet.aircraft[index];
    let value = aircraft_value(ac, registry);
    let nick = ac.nickname.clone();

    gold.amount += value;
    hangars.assignments.remove(&nick);
    fleet.aircraft.remove(index);

    if fleet.active_index >= fleet.aircraft.len() {
        fleet.active_index = fleet.aircraft.len().saturating_sub(1);
    }
    Ok(value)
}

pub struct FleetStats {
    pub total_value: u32,
    pub total_hours: f32,
    pub total_flights: u32,
    pub aircraft_count: usize,
    pub avg_condition: f32,
}

pub fn compute_fleet_stats(fleet: &Fleet, registry: &AircraftRegistry, pilot: &PilotState) -> FleetStats {
    let total_value: u32 = fleet.aircraft.iter().map(|ac| aircraft_value(ac, registry)).sum();
    let total_flights: u32 = fleet.aircraft.iter().map(|ac| ac.total_flights).sum();
    let avg_condition = if fleet.aircraft.is_empty() {
        0.0
    } else {
        fleet.aircraft.iter().map(|ac| ac.condition).sum::<f32>() / fleet.aircraft.len() as f32
    };
    FleetStats {
        total_value,
        total_hours: pilot.total_flight_hours,
        total_flights,
        aircraft_count: fleet.aircraft.len(),
        avg_condition,
    }
}

pub fn create_starter_fleet() -> (Fleet, HangarAssignments) {
    let starter = OwnedAircraft {
        aircraft_id: "cessna_172".to_string(),
        nickname: "Old Faithful".to_string(),
        condition: 65.0,
        fuel: 30.0,
        total_flights: 47,
        customizations: Vec::new(),
    };
    let fleet = Fleet { aircraft: vec![starter], active_index: 0 };
    let mut hangars = HangarAssignments::default();
    hangars.assign("Old Faithful", AirportId::HomeBase);
    (fleet, hangars)
}

pub fn handle_flight_complete_aircraft(
    mut flight_complete_events: EventReader<FlightCompleteEvent>,
    mut fleet: ResMut<Fleet>,
    mut maintenance_tracker: ResMut<super::maintenance::MaintenanceTracker>,
    mut toast_events: EventWriter<ToastEvent>,
) {
    for ev in flight_complete_events.read() {
        if let Some(aircraft) = fleet.active_mut() {
            aircraft.total_flights += 1;
            aircraft.fuel -= ev.fuel_used.min(aircraft.fuel);

            // Delegate condition tracking to maintenance system (single authority)
            let cond = maintenance_tracker.conditions
                .entry(aircraft.nickname.clone())
                .or_default();
            cond.apply_flight_wear(&ev.landing_grade, 0.7); // avg throttle estimate
            let overall = cond.overall();

            // Sync the top-level condition from the maintenance tracker
            aircraft.condition = overall;

            if overall < 20.0 {
                toast_events.send(ToastEvent {
                    message: format!("⚠ {} needs maintenance! Condition: {:.0}%", aircraft.nickname, overall),
                    duration_secs: 4.0,
                });
            }
        }
    }
}
