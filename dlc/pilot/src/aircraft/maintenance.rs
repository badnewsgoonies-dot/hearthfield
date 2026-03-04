//! Aircraft maintenance — per-component tracking, wear, repair actions, scheduling.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use crate::shared::*;

/// Individual component condition within an aircraft.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ComponentCondition {
    pub engine: f32,
    pub airframe: f32,
    pub avionics: f32,
    pub landing_gear: f32,
    pub tires: f32,
    pub brakes: f32,
}

impl Default for ComponentCondition {
    fn default() -> Self {
        Self { engine: 100.0, airframe: 100.0, avionics: 100.0, landing_gear: 100.0, tires: 100.0, brakes: 100.0 }
    }
}

impl ComponentCondition {
    pub fn overall(&self) -> f32 {
        (self.engine + self.airframe + self.avionics + self.landing_gear + self.tires + self.brakes) / 6.0
    }

    pub fn weakest(&self) -> f32 {
        self.engine.min(self.airframe).min(self.avionics)
            .min(self.landing_gear).min(self.tires).min(self.brakes)
    }

    pub fn apply_flight_wear(&mut self, landing_grade: &str, avg_throttle: f32) {
        let engine_wear = 0.5 + avg_throttle * 1.5;
        self.engine = (self.engine - engine_wear).max(0.0);
        self.airframe = (self.airframe - 0.3).max(0.0);
        self.avionics = (self.avionics - 0.1).max(0.0);

        let landing_wear = match landing_grade {
            "Perfect" => 0.5,
            "Good" => 1.0,
            "Acceptable" => 2.0,
            "Hard" => 5.0,
            "Rough" => 10.0,
            _ => 2.0,
        };
        self.landing_gear = (self.landing_gear - landing_wear * 0.8).max(0.0);
        self.tires = (self.tires - landing_wear).max(0.0);
        self.brakes = (self.brakes - landing_wear * 0.6).max(0.0);
    }
}

#[derive(Resource, Default, Clone, Debug, Serialize, Deserialize)]
pub struct MaintenanceTracker {
    pub conditions: std::collections::HashMap<String, ComponentCondition>,
    pub log: Vec<MaintenanceLogEntry>,
    pub last_annual: std::collections::HashMap<String, u32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MaintenanceLogEntry {
    pub aircraft: String,
    pub day: u32,
    pub action: MaintenanceAction,
    pub cost: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MaintenanceAction {
    Inspection,
    RepairEngine,
    RepairAirframe,
    RepairAvionics,
    RepairLandingGear,
    ReplaceTires,
    ReplaceBrakes,
    Overhaul,
}

impl MaintenanceAction {
    pub fn display_name(&self) -> &'static str {
        match self {
            MaintenanceAction::Inspection => "Inspection",
            MaintenanceAction::RepairEngine => "Engine Repair",
            MaintenanceAction::RepairAirframe => "Airframe Repair",
            MaintenanceAction::RepairAvionics => "Avionics Repair",
            MaintenanceAction::RepairLandingGear => "Landing Gear Repair",
            MaintenanceAction::ReplaceTires => "Tire Replacement",
            MaintenanceAction::ReplaceBrakes => "Brake Replacement",
            MaintenanceAction::Overhaul => "Full Overhaul",
        }
    }

    pub fn base_cost(&self) -> u32 {
        match self {
            MaintenanceAction::Inspection => 50,
            MaintenanceAction::RepairEngine => 300,
            MaintenanceAction::RepairAirframe => 250,
            MaintenanceAction::RepairAvionics => 200,
            MaintenanceAction::RepairLandingGear => 150,
            MaintenanceAction::ReplaceTires => 80,
            MaintenanceAction::ReplaceBrakes => 100,
            MaintenanceAction::Overhaul => 1000,
        }
    }
}

pub fn perform_repair(
    tracker: &mut MaintenanceTracker,
    aircraft_nick: &str,
    action: MaintenanceAction,
    gold: &mut Gold,
    day: u32,
) -> Result<u32, &'static str> {
    let cost = action.base_cost();
    if gold.amount < cost { return Err("Not enough gold"); }

    let cond = tracker.conditions.entry(aircraft_nick.to_string()).or_default();
    match action {
        MaintenanceAction::Inspection => {
            tracker.last_annual.insert(aircraft_nick.to_string(), day);
        }
        MaintenanceAction::RepairEngine => cond.engine = 100.0,
        MaintenanceAction::RepairAirframe => cond.airframe = 100.0,
        MaintenanceAction::RepairAvionics => cond.avionics = 100.0,
        MaintenanceAction::RepairLandingGear => cond.landing_gear = 100.0,
        MaintenanceAction::ReplaceTires => cond.tires = 100.0,
        MaintenanceAction::ReplaceBrakes => cond.brakes = 100.0,
        MaintenanceAction::Overhaul => {
            *cond = ComponentCondition::default();
            tracker.last_annual.insert(aircraft_nick.to_string(), day);
        }
    }

    gold.amount -= cost;
    tracker.log.push(MaintenanceLogEntry {
        aircraft: aircraft_nick.to_string(), day, action, cost,
    });
    Ok(cost)
}

pub fn needs_annual_inspection(tracker: &MaintenanceTracker, aircraft_nick: &str, current_day: u32) -> bool {
    let last = tracker.last_annual.get(aircraft_nick).copied().unwrap_or(0);
    current_day.saturating_sub(last) >= 28
}

pub fn is_grounded(tracker: &MaintenanceTracker, aircraft_nick: &str, current_day: u32) -> bool {
    if let Some(cond) = tracker.conditions.get(aircraft_nick) {
        if cond.weakest() < 30.0 { return true; }
    }
    needs_annual_inspection(tracker, aircraft_nick, current_day)
}

pub fn check_maintenance(
    fleet: Res<Fleet>,
    aircraft_registry: Res<AircraftRegistry>,
    mut toast_events: EventWriter<ToastEvent>,
    tracker: Res<MaintenanceTracker>,
    calendar: Res<Calendar>,
) {
    let _ = &aircraft_registry;
    for ac in &fleet.aircraft {
        let day = calendar.total_days();
        if is_grounded(&tracker, &ac.nickname, day) {
            toast_events.send(ToastEvent {
                message: format!("🚫 {} is grounded! Visit hangar for maintenance.", ac.nickname),
                duration_secs: 5.0,
            });
            continue;
        }
        if let Some(cond) = tracker.conditions.get(&ac.nickname) {
            if cond.engine < 40.0 {
                toast_events.send(ToastEvent {
                    message: format!("⚠ {} engine condition: {:.0}%", ac.nickname, cond.engine),
                    duration_secs: 3.0,
                });
            }
            if cond.tires < 40.0 {
                toast_events.send(ToastEvent {
                    message: format!("⚠ {} tires worn: {:.0}%", ac.nickname, cond.tires),
                    duration_secs: 3.0,
                });
            }
        }
        if needs_annual_inspection(&tracker, &ac.nickname, day) {
            toast_events.send(ToastEvent {
                message: format!("📋 {} overdue for annual inspection!", ac.nickname),
                duration_secs: 4.0,
            });
        }
    }
}

/// Full repair shortcut (original API).
pub fn repair_aircraft(
    fleet: &mut Fleet,
    gold: &mut Gold,
    registry: &AircraftRegistry,
) -> Result<u32, &'static str> {
    let aircraft = fleet.active_mut().ok_or("No active aircraft")?;
    let def = registry.get(&aircraft.aircraft_id).ok_or("Unknown aircraft")?;
    let repair_amount = 100.0 - aircraft.condition;
    let cost = (repair_amount * def.maintenance_cost_per_flight as f32 / 10.0) as u32;
    if gold.amount < cost { return Err("Not enough gold"); }
    gold.amount -= cost;
    aircraft.condition = 100.0;
    Ok(cost)
}
