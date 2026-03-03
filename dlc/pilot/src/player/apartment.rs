//! Pilot apartment/quarters — rest, interact with furniture, morning routine.

use bevy::prelude::*;
use crate::shared::*;
use std::collections::HashMap;

// ═══════════════════════════════════════════════════════════════════════════
// DATA TYPES
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FurnitureKind {
    Bed,
    Desk,
    Bookshelf,
    Radio,
    Couch,
    Kitchenette,
    Wardrobe,
    Trophy,
}

impl FurnitureKind {
    pub fn display_name(&self) -> &'static str {
        match self {
            FurnitureKind::Bed => "Bed",
            FurnitureKind::Desk => "Desk",
            FurnitureKind::Bookshelf => "Bookshelf",
            FurnitureKind::Radio => "Radio",
            FurnitureKind::Couch => "Couch",
            FurnitureKind::Kitchenette => "Kitchenette",
            FurnitureKind::Wardrobe => "Wardrobe",
            FurnitureKind::Trophy => "Trophy Case",
        }
    }
}

#[derive(Clone, Debug)]
pub struct FurnitureItem {
    pub kind: FurnitureKind,
    pub tier: u8, // 0 = basic, 1 = upgraded, 2 = premium
    pub grid_x: i32,
    pub grid_y: i32,
}

impl FurnitureItem {
    pub fn upgrade_cost(&self) -> u32 {
        match (self.kind, self.tier) {
            (FurnitureKind::Bed, 0) => 500,
            (FurnitureKind::Bed, 1) => 1500,
            (FurnitureKind::Desk, 0) => 400,
            (FurnitureKind::Desk, 1) => 1200,
            (FurnitureKind::Radio, 0) => 300,
            (FurnitureKind::Radio, 1) => 800,
            (FurnitureKind::Bookshelf, 0) => 350,
            (FurnitureKind::Bookshelf, 1) => 900,
            _ => 0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DecorationKind {
    Poster,
    PlantPot,
    ModelAircraft,
    Certificate,
    WorldMap,
    Rug,
    Lamp,
}

/// Apartment state — persistent across days.
#[derive(Resource, Clone, Debug)]
pub struct ApartmentState {
    pub furniture: Vec<FurnitureItem>,
    pub decorations: Vec<DecorationKind>,
    pub comfort_level: f32, // 0.0 – 100.0
    pub is_home_base: bool,
    pub morning_routine_done: bool,
}

impl Default for ApartmentState {
    fn default() -> Self {
        Self {
            furniture: vec![
                FurnitureItem { kind: FurnitureKind::Bed, tier: 0, grid_x: 2, grid_y: 1 },
                FurnitureItem { kind: FurnitureKind::Desk, tier: 0, grid_x: 5, grid_y: 1 },
                FurnitureItem { kind: FurnitureKind::Bookshelf, tier: 0, grid_x: 7, grid_y: 1 },
                FurnitureItem { kind: FurnitureKind::Radio, tier: 0, grid_x: 5, grid_y: 3 },
            ],
            decorations: vec![DecorationKind::Lamp],
            comfort_level: 30.0,
            is_home_base: true,
            morning_routine_done: false,
        }
    }
}

impl ApartmentState {
    pub fn recalculate_comfort(&mut self) {
        let furniture_score: f32 = self
            .furniture
            .iter()
            .map(|f| (f.tier as f32 + 1.0) * 5.0)
            .sum();
        let decoration_score = self.decorations.len() as f32 * 3.0;
        self.comfort_level = (furniture_score + decoration_score).min(100.0);
    }

    pub fn stamina_recovery_bonus(&self) -> f32 {
        let bed_tier = self
            .furniture
            .iter()
            .find(|f| f.kind == FurnitureKind::Bed)
            .map_or(0, |f| f.tier);
        match bed_tier {
            0 => 0.0,
            1 => 15.0,
            _ => 30.0,
        }
    }

    pub fn desk_tier(&self) -> u8 {
        self.furniture
            .iter()
            .find(|f| f.kind == FurnitureKind::Desk)
            .map_or(0, |f| f.tier)
    }

    pub fn radio_tier(&self) -> u8 {
        self.furniture
            .iter()
            .find(|f| f.kind == FurnitureKind::Radio)
            .map_or(0, |f| f.tier)
    }

    pub fn hotel_room(airport: AirportId) -> Self {
        Self {
            furniture: vec![
                FurnitureItem { kind: FurnitureKind::Bed, tier: 0, grid_x: 2, grid_y: 1 },
                FurnitureItem { kind: FurnitureKind::Desk, tier: 0, grid_x: 4, grid_y: 1 },
            ],
            decorations: Vec::new(),
            comfort_level: 20.0,
            is_home_base: false,
            morning_routine_done: false,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// SYSTEMS
// ═══════════════════════════════════════════════════════════════════════════

/// Interact with apartment furniture based on player proximity and input.
pub fn interact_furniture(
    player_input: Res<PlayerInput>,
    grid_pos: Res<GridPosition>,
    mut apartment: ResMut<ApartmentState>,
    mut pilot_state: ResMut<PilotState>,
    mut toast_events: EventWriter<ToastEvent>,
    mut day_end_events: EventWriter<DayEndEvent>,
    mut save_events: EventWriter<SaveRequestEvent>,
    location: Res<PlayerLocation>,
    mut claimed: ResMut<InteractionClaimed>,
    calendar: Res<Calendar>,
) {
    if !player_input.interact || claimed.0 || location.zone != MapZone::CrewQuarters {
        return;
    }

    let px = grid_pos.x;
    let py = grid_pos.y;

    for item in &apartment.furniture {
        let dx = (px - item.grid_x).abs();
        let dy = (py - item.grid_y).abs();
        if dx > 1 || dy > 1 {
            continue;
        }

        claimed.0 = true;

        match item.kind {
            FurnitureKind::Bed => {
                if calendar.hour >= 20 || calendar.hour < 4 {
                    let recovery = MAX_STAMINA + apartment.stamina_recovery_bonus();
                    pilot_state.stamina = recovery.min(pilot_state.max_stamina);
                    save_events.send(SaveRequestEvent { slot: 0 });
                    day_end_events.send(DayEndEvent);
                    toast_events.send(ToastEvent {
                        message: "Good night... zzz".to_string(),
                        duration_secs: 2.0,
                    });
                } else {
                    toast_events.send(ToastEvent {
                        message: "It's too early to sleep.".to_string(),
                        duration_secs: 2.0,
                    });
                }
            }
            FurnitureKind::Desk => {
                let msg = if apartment.desk_tier() >= 1 {
                    "Checking flight logs and upcoming missions..."
                } else {
                    "Reviewing today's flight schedule."
                };
                toast_events.send(ToastEvent {
                    message: msg.to_string(),
                    duration_secs: 3.0,
                });
            }
            FurnitureKind::Bookshelf => {
                toast_events.send(ToastEvent {
                    message: "Reading the pilot operations manual...".to_string(),
                    duration_secs: 3.0,
                });
            }
            FurnitureKind::Radio => {
                let msg = if apartment.radio_tier() >= 1 {
                    "Aviation news: weather front approaching from the west."
                } else {
                    "Listening to local aviation radio chatter."
                };
                toast_events.send(ToastEvent {
                    message: msg.to_string(),
                    duration_secs: 3.0,
                });
            }
            _ => {
                toast_events.send(ToastEvent {
                    message: format!("You look at the {}.", item.kind.display_name()),
                    duration_secs: 2.0,
                });
            }
        }
        return;
    }
}

/// Morning routine — reset daily flags on day start.
pub fn morning_routine(
    mut day_end_events: EventReader<DayEndEvent>,
    mut apartment: ResMut<ApartmentState>,
    mut toast_events: EventWriter<ToastEvent>,
) {
    for _ev in day_end_events.read() {
        apartment.morning_routine_done = false;
        apartment.recalculate_comfort();
        toast_events.send(ToastEvent {
            message: "☀ A new day begins. Check the mission board!".to_string(),
            duration_secs: 3.0,
        });
    }
}
