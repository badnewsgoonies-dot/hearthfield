//! Shared type contract for Skywarden (Pilot Life Sim).
//!
//! All components, resources, events, and states shared across domains are
//! defined here. **Do not casually modify** — changes ripple across every domain.
//!
//! Domains communicate ONLY through types defined in this module.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ═══════════════════════════════════════════════════════════════════════════
// GAME STATE MACHINE
// ═══════════════════════════════════════════════════════════════════════════

/// Top-level game state.
///
/// ```text
/// Loading → MainMenu → Playing ⇄ Paused
///                         ↕
///   Dialogue / Shop / Flying / RadioComm / CrewLounge / Inventory / Cutscene / MissionBoard
/// ```
#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GameState {
    #[default]
    Loading,
    MainMenu,
    Playing,
    Paused,
    Dialogue,
    Shop,
    Flying,
    RadioComm,
    CrewLounge,
    Inventory,
    Cutscene,
    MissionBoard,
    LoadGame,
    Logbook,
    Profile,
    Achievements,
    Settings,
    MapView,
    Notifications,
    Tutorial,
    Intro,
    LoanOffice,
    InsuranceOffice,
    BusinessHQ,
}

// ═══════════════════════════════════════════════════════════════════════════
// CONSTANTS
// ═══════════════════════════════════════════════════════════════════════════

pub const TILE_SIZE: f32 = 16.0;
pub const PIXEL_SCALE: f32 = 3.0;
pub const PLAYER_SPEED: f32 = 80.0;
pub const FLIGHT_SPEED_BASE: f32 = 200.0;
pub const MAX_FUEL: f32 = 100.0;
pub const MAX_STAMINA: f32 = 100.0;
pub const DAY_LENGTH_SECS: f32 = 720.0; // 12 real minutes = 1 game day
pub const HOURS_IN_DAY: u32 = 24;
pub const WAKE_HOUR: u32 = 6;
pub const SLEEP_HOUR: u32 = 23;

// Z-layer constants for sprite ordering
pub const Z_GROUND: f32 = 0.0;
pub const Z_GROUND_DECOR: f32 = 1.0;
pub const Z_OBJECTS: f32 = 10.0;
pub const Z_PLAYER: f32 = 50.0;
pub const Z_WEATHER: f32 = 90.0;
pub const Z_UI_OVERLAY: f32 = 100.0;

// ═══════════════════════════════════════════════════════════════════════════
// CALENDAR & TIME
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Resource, Clone, Debug, Serialize, Deserialize)]
pub struct Calendar {
    pub day: u32,
    pub season: Season,
    pub year: u32,
    pub day_of_week: DayOfWeek,
    pub hour: u32,
    pub minute: u32,
    pub time_of_day_secs: f32,
    pub time_paused: bool,
}

impl Default for Calendar {
    fn default() -> Self {
        Self {
            day: 1,
            season: Season::Spring,
            year: 1,
            day_of_week: DayOfWeek::Monday,
            hour: WAKE_HOUR,
            minute: 0,
            time_of_day_secs: 0.0,
            time_paused: false,
        }
    }
}

impl Calendar {
    pub fn formatted_time(&self) -> String {
        let suffix = if self.hour < 12 { "AM" } else { "PM" };
        let display_hour = match self.hour {
            0 => 12,
            13..=23 => self.hour - 12,
            h => h,
        };
        format!("{display_hour}:{:02} {suffix}", self.minute)
    }

    pub fn formatted_date(&self) -> String {
        format!(
            "{:?} {} {}, Year {}",
            self.day_of_week, self.season, self.day, self.year
        )
    }

    pub fn total_days(&self) -> u32 {
        (self.year - 1) * 112 + self.season.index() * 28 + self.day
    }

    pub fn is_night(&self) -> bool {
        self.hour >= 20 || self.hour < 6
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Season {
    Spring,
    Summer,
    Fall,
    Winter,
}

impl Season {
    pub fn index(&self) -> u32 {
        match self {
            Season::Spring => 0,
            Season::Summer => 1,
            Season::Fall => 2,
            Season::Winter => 3,
        }
    }

    pub fn next(&self) -> Season {
        match self {
            Season::Spring => Season::Summer,
            Season::Summer => Season::Fall,
            Season::Fall => Season::Winter,
            Season::Winter => Season::Spring,
        }
    }
}

impl std::fmt::Display for Season {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DayOfWeek {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}

impl DayOfWeek {
    pub fn next(&self) -> DayOfWeek {
        match self {
            DayOfWeek::Monday => DayOfWeek::Tuesday,
            DayOfWeek::Tuesday => DayOfWeek::Wednesday,
            DayOfWeek::Wednesday => DayOfWeek::Thursday,
            DayOfWeek::Thursday => DayOfWeek::Friday,
            DayOfWeek::Friday => DayOfWeek::Saturday,
            DayOfWeek::Saturday => DayOfWeek::Sunday,
            DayOfWeek::Sunday => DayOfWeek::Monday,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// WEATHER (AVIATION)
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Weather {
    #[default]
    Clear,
    Cloudy,
    Rain,
    Storm,
    Fog,
    Snow,
    Windy,
}

impl Weather {
    pub fn flight_difficulty(&self) -> f32 {
        match self {
            Weather::Clear => 0.0,
            Weather::Cloudy => 0.1,
            Weather::Windy => 0.3,
            Weather::Rain => 0.4,
            Weather::Fog => 0.6,
            Weather::Snow => 0.7,
            Weather::Storm => 1.0,
        }
    }

    pub fn visibility_modifier(&self) -> f32 {
        match self {
            Weather::Clear => 1.0,
            Weather::Cloudy => 0.8,
            Weather::Windy => 0.9,
            Weather::Rain => 0.5,
            Weather::Fog => 0.2,
            Weather::Snow => 0.4,
            Weather::Storm => 0.3,
        }
    }

    pub fn is_flyable(&self) -> bool {
        !matches!(self, Weather::Storm)
    }
}

#[derive(Resource, Clone, Debug, Default, Serialize, Deserialize)]
pub struct WeatherState {
    pub current: Weather,
    pub forecast: Vec<Weather>,
    pub wind_speed_knots: f32,
    pub wind_direction_deg: f32,
    pub visibility_nm: f32,
    pub ceiling_ft: u32,
    pub turbulence_level: TurbulenceLevel,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum TurbulenceLevel {
    #[default]
    None,
    Light,
    Moderate,
    Severe,
}

// ═══════════════════════════════════════════════════════════════════════════
// LOCATIONS — AIRPORTS & MAPS
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AirportId {
    #[default]
    HomeBase, // Starter airport — small regional
    Windport,   // Coastal city
    Frostpeak,  // Mountain airport
    Sunhaven,   // Tropical resort
    Ironforge,  // Industrial hub
    Cloudmere,  // High-altitude
    Duskhollow, // Desert oasis
    Stormwatch, // Weather research station
    Grandcity,  // Major international hub
    Skyreach,   // Elite/endgame airport
}

impl AirportId {
    pub fn display_name(&self) -> &'static str {
        match self {
            AirportId::HomeBase => "Clearfield Regional",
            AirportId::Windport => "Windport International",
            AirportId::Frostpeak => "Frostpeak Alpine",
            AirportId::Sunhaven => "Sunhaven Coastal",
            AirportId::Ironforge => "Ironforge Industrial",
            AirportId::Cloudmere => "Cloudmere Heights",
            AirportId::Duskhollow => "Duskhollow Desert",
            AirportId::Stormwatch => "Stormwatch Research",
            AirportId::Grandcity => "Grand City International",
            AirportId::Skyreach => "Skyreach Elite",
        }
    }

    pub fn icao_code(&self) -> &'static str {
        match self {
            AirportId::HomeBase => "SWCR",
            AirportId::Windport => "SWWP",
            AirportId::Frostpeak => "SWFP",
            AirportId::Sunhaven => "SWSH",
            AirportId::Ironforge => "SWIF",
            AirportId::Cloudmere => "SWCM",
            AirportId::Duskhollow => "SWDH",
            AirportId::Stormwatch => "SWST",
            AirportId::Grandcity => "SWGC",
            AirportId::Skyreach => "SWSR",
        }
    }

    pub fn unlock_rank(&self) -> PilotRank {
        match self {
            AirportId::HomeBase => PilotRank::Student,
            AirportId::Windport => PilotRank::Private,
            AirportId::Frostpeak => PilotRank::Private,
            AirportId::Sunhaven => PilotRank::Commercial,
            AirportId::Ironforge => PilotRank::Commercial,
            AirportId::Cloudmere => PilotRank::Senior,
            AirportId::Duskhollow => PilotRank::Senior,
            AirportId::Stormwatch => PilotRank::Captain,
            AirportId::Grandcity => PilotRank::Captain,
            AirportId::Skyreach => PilotRank::Ace,
        }
    }
}

/// Map areas within each airport. Each airport has these zones.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MapZone {
    #[default]
    Terminal,
    Lounge,
    Hangar,
    Runway,
    ControlTower,
    CrewQuarters,
    Shop,
    CityStreet,
}

impl MapZone {
    pub fn is_indoor(&self) -> bool {
        matches!(
            self,
            MapZone::Terminal
                | MapZone::Lounge
                | MapZone::Hangar
                | MapZone::CrewQuarters
                | MapZone::Shop
        )
    }
}

/// Current location of the player.
#[derive(Resource, Clone, Debug, Serialize, Deserialize)]
pub struct PlayerLocation {
    pub airport: AirportId,
    pub zone: MapZone,
}

impl Default for PlayerLocation {
    fn default() -> Self {
        Self {
            airport: AirportId::HomeBase,
            zone: MapZone::Terminal,
        }
    }
}

/// Map transition data for zone-to-zone transitions.
#[derive(Clone, Debug)]
pub struct ZoneTransition {
    pub from_zone: MapZone,
    pub to_zone: MapZone,
    pub edge: MapEdge,
    pub spawn_x: i32,
    pub spawn_y: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MapEdge {
    North,
    South,
    East,
    West,
    Door,
}

/// Definition of a tile map zone.
pub struct ZoneMapDef {
    pub zone: MapZone,
    pub width: u32,
    pub height: u32,
    pub tiles: Vec<Vec<TileKind>>,
    pub transitions: Vec<ZoneTransition>,
    pub solid_tiles: Vec<(i32, i32)>,
}

// ═══════════════════════════════════════════════════════════════════════════
// TILES & WORLD
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TileKind {
    Floor,
    Wall,
    Runway,
    Taxiway,
    Grass,
    Tarmac,
    Water,
    Sand,
    Snow,
    Carpet,
    Metal,
    Window,
    Door,
    Void,
}

impl TileKind {
    pub fn is_solid(&self) -> bool {
        matches!(self, TileKind::Wall | TileKind::Water | TileKind::Void)
    }

    pub fn placeholder_color(&self) -> Color {
        match self {
            TileKind::Floor => Color::srgb(0.7, 0.7, 0.7),
            TileKind::Wall => Color::srgb(0.3, 0.3, 0.3),
            TileKind::Runway => Color::srgb(0.2, 0.2, 0.2),
            TileKind::Taxiway => Color::srgb(0.4, 0.4, 0.3),
            TileKind::Grass => Color::srgb(0.2, 0.6, 0.2),
            TileKind::Tarmac => Color::srgb(0.35, 0.35, 0.35),
            TileKind::Water => Color::srgb(0.1, 0.3, 0.7),
            TileKind::Sand => Color::srgb(0.8, 0.7, 0.4),
            TileKind::Snow => Color::srgb(0.9, 0.9, 0.95),
            TileKind::Carpet => Color::srgb(0.5, 0.1, 0.1),
            TileKind::Metal => Color::srgb(0.6, 0.6, 0.65),
            TileKind::Window => Color::srgb(0.5, 0.7, 0.9),
            TileKind::Door => Color::srgb(0.5, 0.3, 0.15),
            TileKind::Void => Color::srgb(0.0, 0.0, 0.0),
        }
    }
}

#[derive(Component)]
pub struct MapTile {
    pub kind: TileKind,
    pub grid_x: i32,
    pub grid_y: i32,
}

#[derive(Resource, Default)]
pub struct WorldMap {
    pub width: u32,
    pub height: u32,
    pub tiles: Vec<Vec<TileKind>>,
}

#[derive(Resource, Default)]
pub struct CollisionMap {
    pub initialised: bool,
    pub width: u32,
    pub height: u32,
    pub blocked: Vec<Vec<bool>>,
}

impl CollisionMap {
    pub fn is_blocked(&self, x: i32, y: i32) -> bool {
        if !self.initialised {
            return false;
        }
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            return true;
        }
        self.blocked[y as usize][x as usize]
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// WORLD OBJECTS & INTERACTABLES
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WorldObjectKind {
    // Airport furniture
    Chair,
    Table,
    Counter,
    VendingMachine,
    Bookshelf,
    Locker,
    Sofa,
    Bed,
    Plant,
    Monitor,
    // Airport-specific
    CheckInDesk,
    BaggageCarousel,
    BoardingGate,
    ControlPanel,
    RadarScreen,
    WeatherStation,
    FuelPump,
    Toolbox,
    Windsock,
    // Mission board
    MissionBoard,
    // Shop
    ShopDisplay,
    // Decorative
    Lamp,
    TrashCan,
    Flag,
    Clock,
    Sign,
}

#[derive(Component)]
pub struct WorldObject {
    pub kind: WorldObjectKind,
    pub grid_x: i32,
    pub grid_y: i32,
}

#[derive(Component)]
pub struct Interactable {
    pub prompt: String,
    pub range: f32,
}

impl Default for Interactable {
    fn default() -> Self {
        Self {
            prompt: "Interact".to_string(),
            range: 1.5,
        }
    }
}

#[derive(Resource, Default)]
pub struct InteractionClaimed(pub bool);

// ═══════════════════════════════════════════════════════════════════════════
// PLAYER & PILOT STATE
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Component)]
pub struct Player;

#[derive(Resource, Clone, Debug, Serialize, Deserialize)]
pub struct PilotState {
    pub name: String,
    pub rank: PilotRank,
    pub xp: u32,
    pub xp_to_next_rank: u32,
    pub total_flights: u32,
    pub total_flight_hours: f32,
    pub perfect_landings: u32,
    pub stamina: f32,
    pub max_stamina: f32,
    pub reputation: f32, // 0.0 - 100.0
    pub current_airport: AirportId,
    pub home_airport: AirportId,
    pub licenses: Vec<LicenseType>,
}

impl Default for PilotState {
    fn default() -> Self {
        Self {
            name: "Pilot".to_string(),
            rank: PilotRank::Student,
            xp: 0,
            xp_to_next_rank: 100,
            total_flights: 0,
            total_flight_hours: 0.0,
            perfect_landings: 0,
            stamina: MAX_STAMINA,
            max_stamina: MAX_STAMINA,
            reputation: 50.0,
            current_airport: AirportId::HomeBase,
            home_airport: AirportId::HomeBase,
            licenses: vec![LicenseType::SingleEngine],
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub enum PilotRank {
    Student,
    Private,
    Commercial,
    Senior,
    Captain,
    Ace,
}

impl PilotRank {
    pub fn display_name(&self) -> &'static str {
        match self {
            PilotRank::Student => "Student Pilot",
            PilotRank::Private => "Private Pilot",
            PilotRank::Commercial => "Commercial Pilot",
            PilotRank::Senior => "Senior First Officer",
            PilotRank::Captain => "Captain",
            PilotRank::Ace => "Ace Pilot",
        }
    }

    pub fn next(&self) -> Option<PilotRank> {
        match self {
            PilotRank::Student => Some(PilotRank::Private),
            PilotRank::Private => Some(PilotRank::Commercial),
            PilotRank::Commercial => Some(PilotRank::Senior),
            PilotRank::Senior => Some(PilotRank::Captain),
            PilotRank::Captain => Some(PilotRank::Ace),
            PilotRank::Ace => None,
        }
    }

    pub fn xp_required(&self) -> u32 {
        match self {
            PilotRank::Student => 0,
            PilotRank::Private => 100,
            PilotRank::Commercial => 350,
            PilotRank::Senior => 800,
            PilotRank::Captain => 1500,
            PilotRank::Ace => 3000,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LicenseType {
    SingleEngine,
    MultiEngine,
    Instrument,
    Turboprop,
    Jet,
    HeavyJet,
    Aerobatic,
}

#[derive(Resource, Default, Clone, Debug, Serialize, Deserialize)]
pub struct PlayerMovement {
    pub direction: Vec2,
    pub facing: Facing,
    pub is_moving: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum Facing {
    #[default]
    Down,
    Up,
    Left,
    Right,
}

#[derive(Resource, Default)]
pub struct GridPosition {
    pub x: i32,
    pub y: i32,
}

// ═══════════════════════════════════════════════════════════════════════════
// INPUT ABSTRACTION
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Resource, Default)]
pub struct PlayerInput {
    pub movement: Vec2,
    pub interact: bool,
    pub cancel: bool,
    pub pause: bool,
    pub inventory: bool,
    pub map_view: bool,
    pub radio: bool,
    pub throttle_up: bool,
    pub throttle_down: bool,
    pub yaw_left: bool,
    pub yaw_right: bool,
    pub flaps_toggle: bool,
    pub gear_toggle: bool,
    pub confirm: bool,
    pub tab_next: bool,
    pub tab_prev: bool,
    pub debug_overlay: bool,
    pub sprint: bool,
    pub menu_up: bool,
    pub menu_down: bool,
    pub menu_left: bool,
    pub menu_right: bool,
    pub menu_confirm: bool,
    pub menu_cancel: bool,
    pub hotbar_1: bool,
    pub hotbar_2: bool,
    pub hotbar_3: bool,
    pub hotbar_4: bool,
}

#[derive(Resource, Clone, Debug)]
pub struct KeyBindings {
    pub move_up: KeyCode,
    pub move_down: KeyCode,
    pub move_left: KeyCode,
    pub move_right: KeyCode,
    pub interact: KeyCode,
    pub cancel: KeyCode,
    pub pause: KeyCode,
    pub inventory: KeyCode,
    pub map_view: KeyCode,
    pub radio: KeyCode,
    pub sprint: KeyCode,
    pub throttle_up: KeyCode,
    pub throttle_down: KeyCode,
    pub yaw_left: KeyCode,
    pub yaw_right: KeyCode,
    pub flaps: KeyCode,
    pub gear: KeyCode,
    pub debug_overlay: KeyCode,
}

impl Default for KeyBindings {
    fn default() -> Self {
        Self {
            move_up: KeyCode::KeyW,
            move_down: KeyCode::KeyS,
            move_left: KeyCode::KeyA,
            move_right: KeyCode::KeyD,
            interact: KeyCode::KeyF,
            cancel: KeyCode::KeyQ,
            pause: KeyCode::Escape,
            inventory: KeyCode::KeyE,
            map_view: KeyCode::KeyM,
            radio: KeyCode::KeyR,
            sprint: KeyCode::ShiftLeft,
            throttle_up: KeyCode::KeyW,
            throttle_down: KeyCode::KeyS,
            yaw_left: KeyCode::KeyA,
            yaw_right: KeyCode::KeyD,
            flaps: KeyCode::KeyG,
            gear: KeyCode::KeyB,
            debug_overlay: KeyCode::F3,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum InputContext {
    #[default]
    Gameplay,
    Menu,
    Dialogue,
    Cockpit,
    Cutscene,
    Disabled,
}

#[derive(Resource, Default)]
pub struct InputState {
    pub context: InputContext,
}

// ═══════════════════════════════════════════════════════════════════════════
// INVENTORY & ITEMS
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Resource, Clone, Debug, Default, Serialize, Deserialize)]
pub struct Inventory {
    pub slots: Vec<InventorySlot>,
    pub max_slots: usize,
}

impl Inventory {
    pub fn new(max_slots: usize) -> Self {
        Self {
            slots: Vec::new(),
            max_slots,
        }
    }

    pub fn add_item(&mut self, item_id: &str, quantity: u32) -> bool {
        if let Some(slot) = self.slots.iter_mut().find(|s| s.item_id == item_id) {
            slot.quantity += quantity;
            return true;
        }
        if self.slots.len() < self.max_slots {
            self.slots.push(InventorySlot {
                item_id: item_id.to_string(),
                quantity,
            });
            return true;
        }
        false
    }

    pub fn remove_item(&mut self, item_id: &str, quantity: u32) -> bool {
        if let Some(slot) = self.slots.iter_mut().find(|s| s.item_id == item_id) {
            if slot.quantity >= quantity {
                slot.quantity -= quantity;
                if slot.quantity == 0 {
                    self.slots
                        .retain(|s| s.item_id != item_id || s.quantity > 0);
                }
                return true;
            }
        }
        false
    }

    pub fn has_item(&self, item_id: &str, quantity: u32) -> bool {
        self.slots
            .iter()
            .any(|s| s.item_id == item_id && s.quantity >= quantity)
    }

    pub fn count_item(&self, item_id: &str) -> u32 {
        self.slots
            .iter()
            .find(|s| s.item_id == item_id)
            .map_or(0, |s| s.quantity)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InventorySlot {
    pub item_id: String,
    pub quantity: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ItemDef {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: ItemCategory,
    pub price: u32,
    pub sell_price: u32,
    pub stackable: bool,
    pub max_stack: u32,
    pub icon_index: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ItemCategory {
    Food,
    Drink,
    Gift,
    Tool,
    Part, // Aircraft parts
    Fuel,
    Map,
    Document, // Licenses, manuals
    Cosmetic, // Pilot customization
    Collectible,
    KeyItem,
}

#[derive(Resource, Default, Clone, Debug, Serialize, Deserialize)]
pub struct ItemRegistry {
    pub items: HashMap<String, ItemDef>,
}

impl ItemRegistry {
    pub fn get(&self, id: &str) -> Option<&ItemDef> {
        self.items.get(id)
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// AIRCRAFT
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AircraftDef {
    pub id: String,
    pub name: String,
    pub class: AircraftClass,
    pub speed_knots: f32,
    pub range_nm: f32,
    pub fuel_capacity: f32,
    pub fuel_burn_rate: f32,
    pub passenger_capacity: u32,
    pub cargo_capacity_kg: f32,
    pub purchase_price: u32,
    pub maintenance_cost_per_flight: u32,
    pub required_license: LicenseType,
    pub required_rank: PilotRank,
    pub handling: f32,   // 0.0 = sluggish, 1.0 = responsive
    pub durability: f32, // max condition
    pub sprite_index: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AircraftClass {
    SingleProp,
    TwinProp,
    Turboprop,
    LightJet,
    MediumJet,
    HeavyJet,
    Cargo,
    Seaplane,
}

impl AircraftClass {
    pub fn display_name(&self) -> &'static str {
        match self {
            AircraftClass::SingleProp => "Single-Engine Prop",
            AircraftClass::TwinProp => "Twin-Engine Prop",
            AircraftClass::Turboprop => "Turboprop",
            AircraftClass::LightJet => "Light Jet",
            AircraftClass::MediumJet => "Medium Jet",
            AircraftClass::HeavyJet => "Heavy Jet",
            AircraftClass::Cargo => "Cargo Freighter",
            AircraftClass::Seaplane => "Seaplane",
        }
    }
}

#[derive(Resource, Default, Clone, Debug, Serialize, Deserialize)]
pub struct AircraftRegistry {
    pub aircraft: HashMap<String, AircraftDef>,
}

impl AircraftRegistry {
    pub fn get(&self, id: &str) -> Option<&AircraftDef> {
        self.aircraft.get(id)
    }
}

/// Player's fleet of owned aircraft.
#[derive(Resource, Clone, Debug, Default, Serialize, Deserialize)]
pub struct Fleet {
    pub aircraft: Vec<OwnedAircraft>,
    pub active_index: usize,
}

impl Fleet {
    pub fn active(&self) -> Option<&OwnedAircraft> {
        self.aircraft.get(self.active_index)
    }

    pub fn active_mut(&mut self) -> Option<&mut OwnedAircraft> {
        self.aircraft.get_mut(self.active_index)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OwnedAircraft {
    pub aircraft_id: String,
    pub nickname: String,
    pub condition: f32, // 0.0 = broken, 100.0 = perfect
    pub fuel: f32,
    pub total_flights: u32,
    pub customizations: Vec<String>,
}

// ═══════════════════════════════════════════════════════════════════════════
// FLIGHT STATE
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Resource, Clone, Debug, Default, Serialize, Deserialize)]
pub struct FlightState {
    pub phase: FlightPhase,
    pub origin: AirportId,
    pub destination: AirportId,
    pub altitude_ft: f32,
    pub speed_knots: f32,
    pub heading_deg: f32,
    pub fuel_remaining: f32,
    pub distance_remaining_nm: f32,
    pub distance_total_nm: f32,
    pub flight_time_secs: f32,
    pub throttle: f32, // 0.0 - 1.0
    pub flaps_deployed: bool,
    pub gear_down: bool,
    pub autopilot: bool,
    pub turbulence_shake: f32,
    pub passengers_happy: f32, // 0.0 - 100.0
    pub cargo_secure: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum FlightPhase {
    #[default]
    Idle,
    Preflight,
    Taxi,
    Takeoff,
    Climb,
    Cruise,
    Descent,
    Approach,
    Landing,
    Arrived,
    Emergency,
}

impl FlightPhase {
    pub fn display_name(&self) -> &'static str {
        match self {
            FlightPhase::Idle => "On Ground",
            FlightPhase::Preflight => "Preflight Check",
            FlightPhase::Taxi => "Taxiing",
            FlightPhase::Takeoff => "Takeoff",
            FlightPhase::Climb => "Climbing",
            FlightPhase::Cruise => "Cruising",
            FlightPhase::Descent => "Descending",
            FlightPhase::Approach => "Approach",
            FlightPhase::Landing => "Landing",
            FlightPhase::Arrived => "Arrived",
            FlightPhase::Emergency => "EMERGENCY",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PreflightChecklist {
    pub items: Vec<ChecklistItem>,
}

impl Default for PreflightChecklist {
    fn default() -> Self {
        Self {
            items: vec![
                ChecklistItem::new("Fuel levels", "Check fuel gauges"),
                ChecklistItem::new("Control surfaces", "Move yoke full range"),
                ChecklistItem::new("Instruments", "Verify all readings"),
                ChecklistItem::new("Weather brief", "Review current conditions"),
                ChecklistItem::new("Flight plan", "File with tower"),
                ChecklistItem::new("Passenger count", "Verify manifest"),
            ],
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChecklistItem {
    pub name: String,
    pub description: String,
    pub completed: bool,
}

impl ChecklistItem {
    pub fn new(name: &str, desc: &str) -> Self {
        Self {
            name: name.to_string(),
            description: desc.to_string(),
            completed: false,
        }
    }
}

/// Landing quality assessment.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum LandingGrade {
    Perfect,    // Butter landing
    Good,       // Smooth
    Acceptable, // Safe but bumpy
    Hard,       // Passengers complain
    Rough,      // Damage aircraft
}

impl LandingGrade {
    pub fn xp_bonus(&self) -> u32 {
        match self {
            LandingGrade::Perfect => 50,
            LandingGrade::Good => 25,
            LandingGrade::Acceptable => 10,
            LandingGrade::Hard => 0,
            LandingGrade::Rough => 0,
        }
    }

    pub fn reputation_change(&self) -> f32 {
        match self {
            LandingGrade::Perfect => 2.0,
            LandingGrade::Good => 1.0,
            LandingGrade::Acceptable => 0.0,
            LandingGrade::Hard => -1.0,
            LandingGrade::Rough => -3.0,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// MISSIONS
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MissionDef {
    pub id: String,
    pub title: String,
    pub description: String,
    pub mission_type: MissionType,
    pub origin: AirportId,
    pub destination: AirportId,
    pub reward_gold: u32,
    pub reward_xp: u32,
    pub time_limit_hours: Option<u32>,
    pub required_rank: PilotRank,
    pub required_aircraft_class: Option<AircraftClass>,
    pub passenger_count: u32,
    pub cargo_kg: f32,
    pub bonus_conditions: Vec<BonusCondition>,
    pub difficulty: MissionDifficulty,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MissionType {
    Passenger,
    Cargo,
    Medical,
    VIP,
    Charter,
    Training,
    AirShow,
    Rescue,
    Survey,
    Delivery,
}

impl MissionType {
    pub fn display_name(&self) -> &'static str {
        match self {
            MissionType::Passenger => "Passenger Flight",
            MissionType::Cargo => "Cargo Delivery",
            MissionType::Medical => "Medical Transport",
            MissionType::VIP => "VIP Charter",
            MissionType::Charter => "Private Charter",
            MissionType::Training => "Training Flight",
            MissionType::AirShow => "Air Show",
            MissionType::Rescue => "Rescue Operation",
            MissionType::Survey => "Aerial Survey",
            MissionType::Delivery => "Express Delivery",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MissionDifficulty {
    Easy,
    Medium,
    Hard,
    Expert,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum BonusCondition {
    PerfectLanding,
    OnTime,
    NoTurbulenceDamage,
    LowFuelUsage,
    NightFlight,
    BadWeatherFlight,
}

#[derive(Resource, Default, Clone, Debug, Serialize, Deserialize)]
pub struct MissionBoard {
    pub available: Vec<MissionDef>,
    pub active: Option<ActiveMission>,
    pub completed_ids: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ActiveMission {
    pub mission: MissionDef,
    pub accepted_day: u32,
    pub bonuses_met: Vec<bool>,
}

#[derive(Resource, Default, Clone, Debug, Serialize, Deserialize)]
pub struct MissionLog {
    pub completed: Vec<CompletedMission>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CompletedMission {
    pub mission_id: String,
    pub day_completed: u32,
    pub landing_grade: String,
    pub gold_earned: u32,
    pub xp_earned: u32,
    pub flight_time_minutes: f32,
}

// ═══════════════════════════════════════════════════════════════════════════
// CREW & NPC SYSTEM
// ═══════════════════════════════════════════════════════════════════════════

pub const CREW_IDS: &[&str] = &[
    "captain_elena",
    "copilot_marco",
    "navigator_yuki",
    "mechanic_hank",
    "attendant_sofia",
    "controller_raj",
    "instructor_chen",
    "veteran_pete",
    "rookie_alex",
    "charter_diana",
];

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CrewMemberDef {
    pub id: String,
    pub name: String,
    pub role: CrewRole,
    pub personality: String,
    pub favorite_gift: String,
    pub disliked_gift: String,
    pub home_airport: AirportId,
    pub dialogue_pool: Vec<String>,
    pub backstory: String,
    pub sprite_index: u32,
    pub tint_color: [f32; 3],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CrewRole {
    Captain,
    FirstOfficer,
    Navigator,
    FlightEngineer,
    Mechanic,
    FlightAttendant,
    AirTrafficController,
    Instructor,
    Dispatcher,
    Charter,
}

impl CrewRole {
    pub fn display_name(&self) -> &'static str {
        match self {
            CrewRole::Captain => "Captain",
            CrewRole::FirstOfficer => "First Officer",
            CrewRole::Navigator => "Navigator",
            CrewRole::FlightEngineer => "Flight Engineer",
            CrewRole::Mechanic => "Mechanic",
            CrewRole::FlightAttendant => "Flight Attendant",
            CrewRole::AirTrafficController => "ATC",
            CrewRole::Instructor => "Instructor",
            CrewRole::Dispatcher => "Dispatcher",
            CrewRole::Charter => "Charter Pilot",
        }
    }
}

#[derive(Component)]
pub struct CrewMember {
    pub id: String,
}

#[derive(Resource, Default, Clone, Debug, Serialize, Deserialize)]
pub struct CrewRegistry {
    pub members: HashMap<String, CrewMemberDef>,
}

#[derive(Resource, Default, Clone, Debug, Serialize, Deserialize)]
pub struct Relationships {
    pub friendship: HashMap<String, i32>, // -100 to 100
    pub gifts_given_today: HashMap<String, bool>,
}

impl Relationships {
    pub fn friendship_level(&self, npc_id: &str) -> i32 {
        self.friendship.get(npc_id).copied().unwrap_or(0)
    }

    pub fn add_friendship(&mut self, npc_id: &str, amount: i32) {
        let current = self.friendship.entry(npc_id.to_string()).or_insert(0);
        *current = (*current + amount).clamp(-100, 100);
    }

    pub fn friendship_tier(&self, npc_id: &str) -> &'static str {
        match self.friendship_level(npc_id) {
            -100..=-50 => "Hostile",
            -49..=-1 => "Cold",
            0..=24 => "Acquaintance",
            25..=49 => "Friendly",
            50..=74 => "Good Friend",
            75..=100 => "Best Friend",
            _ => "Unknown",
        }
    }
}

/// NPC schedule entry — where an NPC should be at a given time.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScheduleEntry {
    pub hour: u32,
    pub airport: AirportId,
    pub zone: MapZone,
    pub grid_x: i32,
    pub grid_y: i32,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct NpcSchedule {
    pub weekday: Vec<ScheduleEntry>,
    pub weekend: Vec<ScheduleEntry>,
}

// ═══════════════════════════════════════════════════════════════════════════
// DIALOGUE
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Resource, Default, Clone, Debug)]
pub struct DialogueState {
    pub active: bool,
    pub speaker_id: String,
    pub speaker_name: String,
    pub lines: Vec<DialogueLine>,
    pub current_line: usize,
    pub choices: Vec<DialogueChoice>,
}

#[derive(Clone, Debug)]
pub struct DialogueLine {
    pub text: String,
    pub speaker: String,
    pub emotion: Emotion,
}

#[derive(Clone, Debug)]
pub struct DialogueChoice {
    pub text: String,
    pub friendship_change: i32,
    pub next_line: Option<usize>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum Emotion {
    #[default]
    Neutral,
    Happy,
    Sad,
    Angry,
    Surprised,
    Excited,
    Worried,
    Proud,
}

// ═══════════════════════════════════════════════════════════════════════════
// ECONOMY
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Resource, Clone, Debug, Serialize, Deserialize)]
pub struct Gold {
    pub amount: u32,
}

impl Default for Gold {
    fn default() -> Self {
        Self { amount: 500 }
    }
}

#[derive(Resource, Default, Clone, Debug, Serialize, Deserialize)]
pub struct EconomyStats {
    pub total_earned: u32,
    pub total_spent: u32,
    pub flights_completed: u32,
    pub fuel_purchased: u32,
    pub repairs_paid: u32,
    pub gifts_given: u32,
    pub items_purchased: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ShopListing {
    pub item_id: String,
    pub price: u32,
    pub stock: Option<u32>, // None = infinite
}

#[derive(Resource, Default, Clone, Debug, Serialize, Deserialize)]
pub struct ActiveShop {
    pub name: String,
    pub listings: Vec<ShopListing>,
    pub is_open: bool,
}

// ═══════════════════════════════════════════════════════════════════════════
// ACHIEVEMENTS & PROGRESSION
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AchievementDef {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub icon_index: u32,
}

pub const ACHIEVEMENTS: &[AchievementDef] = &[
    AchievementDef {
        id: "first_flight",
        name: "Wheels Up",
        description: "Complete your first flight",
        icon_index: 0,
    },
    AchievementDef {
        id: "perfect_10",
        name: "Butter Landing x10",
        description: "Make 10 perfect landings",
        icon_index: 1,
    },
    AchievementDef {
        id: "all_airports",
        name: "Globe Trotter",
        description: "Visit every airport",
        icon_index: 2,
    },
    AchievementDef {
        id: "storm_pilot",
        name: "Storm Chaser",
        description: "Complete a flight in severe weather",
        icon_index: 3,
    },
    AchievementDef {
        id: "night_owl",
        name: "Night Owl",
        description: "Complete 5 night flights",
        icon_index: 4,
    },
    AchievementDef {
        id: "captain_rank",
        name: "Captain!",
        description: "Reach Captain rank",
        icon_index: 5,
    },
    AchievementDef {
        id: "ace_rank",
        name: "Ace of Aces",
        description: "Reach Ace rank",
        icon_index: 6,
    },
    AchievementDef {
        id: "millionaire",
        name: "Sky Millionaire",
        description: "Earn 1,000,000 gold total",
        icon_index: 7,
    },
    AchievementDef {
        id: "friend_all",
        name: "Crew Family",
        description: "Max friendship with all crew",
        icon_index: 8,
    },
    AchievementDef {
        id: "all_aircraft",
        name: "Collector",
        description: "Own every aircraft type",
        icon_index: 9,
    },
    AchievementDef {
        id: "cargo_king",
        name: "Cargo King",
        description: "Deliver 100 cargo missions",
        icon_index: 10,
    },
    AchievementDef {
        id: "medical_hero",
        name: "Lifesaver",
        description: "Complete 10 medical flights",
        icon_index: 11,
    },
    AchievementDef {
        id: "vip_service",
        name: "First Class",
        description: "Complete 20 VIP charters",
        icon_index: 12,
    },
    AchievementDef {
        id: "100_flights",
        name: "Century Pilot",
        description: "Complete 100 flights",
        icon_index: 13,
    },
    AchievementDef {
        id: "500_flights",
        name: "Sky Legend",
        description: "Complete 500 flights",
        icon_index: 14,
    },
    AchievementDef {
        id: "all_licenses",
        name: "Full Ticket",
        description: "Earn every license type",
        icon_index: 15,
    },
    AchievementDef {
        id: "speed_demon",
        name: "Speed Demon",
        description: "Fly at maximum speed in a heavy jet",
        icon_index: 16,
    },
    AchievementDef {
        id: "fuel_saver",
        name: "Eco Pilot",
        description: "Complete 10 flights under fuel budget",
        icon_index: 17,
    },
    AchievementDef {
        id: "no_damage",
        name: "Pristine Fleet",
        description: "Complete 50 flights with no aircraft damage",
        icon_index: 18,
    },
    AchievementDef {
        id: "rescue_5",
        name: "Guardian Angel",
        description: "Complete 5 rescue missions",
        icon_index: 19,
    },
];

#[derive(Resource, Default, Clone, Debug, Serialize, Deserialize)]
pub struct Achievements {
    pub unlocked: Vec<String>,
}

impl Achievements {
    pub fn is_unlocked(&self, id: &str) -> bool {
        self.unlocked.iter().any(|a| a == id)
    }

    pub fn unlock(&mut self, id: &str) -> bool {
        if !self.is_unlocked(id) {
            self.unlocked.push(id.to_string());
            true
        } else {
            false
        }
    }
}

#[derive(Resource, Default, Clone, Debug, Serialize, Deserialize)]
pub struct PlayStats {
    pub total_play_time_secs: f64,
    pub total_flights: u32,
    pub total_flight_hours: f32,
    pub total_distance_nm: f32,
    pub perfect_landings: u32,
    pub rough_landings: u32,
    pub missions_completed: u32,
    pub missions_failed: u32,
    pub gifts_given: u32,
    pub items_purchased: u32,
    pub airports_visited: Vec<AirportId>,
    pub aircraft_owned: Vec<String>,
}

// ═══════════════════════════════════════════════════════════════════════════
// EVENTS
// ═══════════════════════════════════════════════════════════════════════════

// --- Calendar ---
#[derive(Event, Clone, Debug)]
pub struct DayEndEvent;

#[derive(Event, Clone, Debug)]
pub struct SeasonChangeEvent {
    pub new_season: Season,
}

// --- Map/Zone transitions ---
#[derive(Event, Clone, Debug)]
pub struct ZoneTransitionEvent {
    pub to_airport: AirportId,
    pub to_zone: MapZone,
    pub to_x: i32,
    pub to_y: i32,
}

#[derive(Event, Clone, Debug)]
pub struct AirportArrivalEvent {
    pub airport: AirportId,
}

// --- Flight ---
#[derive(Event, Clone, Debug)]
pub struct FlightStartEvent {
    pub origin: AirportId,
    pub destination: AirportId,
    pub mission: Option<String>,
}

#[derive(Event, Clone, Debug)]
pub struct FlightCompleteEvent {
    pub origin: AirportId,
    pub destination: AirportId,
    pub landing_grade: String,
    pub flight_time_secs: f32,
    pub fuel_used: f32,
    pub xp_earned: u32,
    pub gold_earned: u32,
}

#[derive(Event, Clone, Debug)]
pub struct FlightPhaseChangeEvent {
    pub new_phase: FlightPhase,
}

#[derive(Event, Clone, Debug)]
pub struct EmergencyEvent {
    pub kind: EmergencyKind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EmergencyKind {
    EngineFailure,
    FuelLeak,
    HydraulicFailure,
    BirdStrike,
    LightningStrike,
}

// --- Economy ---
#[derive(Event, Clone, Debug)]
pub struct GoldChangeEvent {
    pub amount: i32,
    pub reason: String,
}

#[derive(Event, Clone, Debug)]
pub struct PurchaseEvent {
    pub item_id: String,
    pub price: u32,
}

// --- Crew & Dialogue ---
#[derive(Event, Clone, Debug)]
pub struct DialogueStartEvent {
    pub npc_id: String,
}

#[derive(Event, Clone, Debug)]
pub struct GiftGivenEvent {
    pub npc_id: String,
    pub item_id: String,
}

#[derive(Event, Clone, Debug)]
pub struct FriendshipChangeEvent {
    pub npc_id: String,
    pub amount: i32,
}

// --- Missions ---
#[derive(Event, Clone, Debug)]
pub struct MissionAcceptedEvent {
    pub mission_id: String,
}

#[derive(Event, Clone, Debug)]
pub struct MissionCompletedEvent {
    pub mission_id: String,
    pub gold_earned: u32,
    pub xp_earned: u32,
}

#[derive(Event, Clone, Debug)]
pub struct MissionFailedEvent {
    pub mission_id: String,
    pub reason: String,
}

// --- Items ---
#[derive(Event, Clone, Debug)]
pub struct ItemPickupEvent {
    pub item_id: String,
    pub quantity: u32,
}

// --- Player / Progression ---
#[derive(Event, Clone, Debug)]
pub struct RankUpEvent {
    pub new_rank: PilotRank,
}

#[derive(Event, Clone, Debug)]
pub struct LicenseEarnedEvent {
    pub license: LicenseType,
}

#[derive(Event, Clone, Debug)]
pub struct AchievementUnlockedEvent {
    pub achievement_id: String,
}

#[derive(Event, Clone, Debug)]
pub struct XpGainEvent {
    pub amount: u32,
    pub source: String,
}

// --- UI ---
#[derive(Event, Clone, Debug)]
pub struct ToastEvent {
    pub message: String,
    pub duration_secs: f32,
}

#[derive(Event, Clone, Debug)]
pub struct PlaySfxEvent {
    pub sfx_id: String,
}

#[derive(Event, Clone, Debug)]
pub struct PlayMusicEvent {
    pub track_id: String,
    pub fade_in: bool,
}

#[derive(Event, Clone, Debug)]
pub struct ScreenFadeEvent {
    pub fade_in: bool,
    pub duration_secs: f32,
}

// --- Weather ---
#[derive(Event, Clone, Debug)]
pub struct WeatherChangeEvent {
    pub new_weather: Weather,
}

// --- Save ---
#[derive(Event, Clone, Debug)]
pub struct SaveRequestEvent {
    pub slot: usize,
}

#[derive(Event, Clone, Debug)]
pub struct LoadRequestEvent {
    pub slot: usize,
}

#[derive(Event, Clone, Debug)]
pub struct SaveCompleteEvent;

#[derive(Event, Clone, Debug)]
pub struct LoadCompleteEvent;

// --- Cutscene ---
#[derive(Event, Clone, Debug)]
pub struct CutsceneStartEvent {
    pub cutscene_id: String,
}

// ═══════════════════════════════════════════════════════════════════════════
// CUTSCENE SYSTEM
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Clone, Debug)]
pub enum CutsceneStep {
    Dialogue {
        speaker: String,
        text: String,
    },
    Wait {
        seconds: f32,
    },
    Teleport {
        airport: AirportId,
        zone: MapZone,
        x: i32,
        y: i32,
    },
    FadeOut {
        duration: f32,
    },
    FadeIn {
        duration: f32,
    },
    PlayMusic {
        track_id: String,
    },
    PlaySfx {
        sfx_id: String,
    },
    SetWeather {
        weather: Weather,
    },
}

#[derive(Resource, Default)]
pub struct CutsceneQueue {
    pub pending: Vec<Vec<CutsceneStep>>,
}

#[derive(Resource, Default)]
pub struct ActiveCutscene {
    pub steps: Vec<CutsceneStep>,
    pub current_step: usize,
    pub timer: f32,
    pub waiting_for_input: bool,
}

// ═══════════════════════════════════════════════════════════════════════════
// UI STATE
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Resource, Default)]
pub struct UiFontHandle(pub Handle<Font>);

#[derive(Resource, Default)]
pub struct MenuTheme {
    pub bg_color: Color,
    pub text_color: Color,
    pub highlight_color: Color,
    pub border_color: Color,
}

#[derive(Resource, Default, Clone, Debug)]
pub struct DebugOverlay {
    pub visible: bool,
}

#[derive(Resource, Default)]
pub struct MusicState {
    pub current_track: String,
    pub entity: Option<Entity>,
}

// ═══════════════════════════════════════════════════════════════════════════
// SAVE SYSTEM TYPES
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SaveSlotInfo {
    pub slot: usize,
    pub pilot_name: String,
    pub rank: PilotRank,
    pub day: u32,
    pub season: Season,
    pub year: u32,
    pub play_time_secs: f64,
    pub airport: AirportId,
}

#[derive(Resource, Default)]
pub struct SaveSlots {
    pub slots: [Option<SaveSlotInfo>; 3],
}

#[derive(Resource, Default)]
pub struct SessionTimer {
    pub elapsed_secs: f64,
}

// ═══════════════════════════════════════════════════════════════════════════
// TUTORIAL / HINTS
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Resource, Default, Clone, Debug, Serialize, Deserialize)]
pub struct TutorialState {
    pub completed_hints: Vec<String>,
    pub active: bool,
}

#[derive(Event, Clone, Debug)]
pub struct HintEvent {
    pub hint_id: String,
    pub message: String,
}

// ═══════════════════════════════════════════════════════════════════════════
// CITY REGISTRY
// ═══════════════════════════════════════════════════════════════════════════

/// Registry of rich city data for each airport, populated by DataPlugin.
#[derive(Resource, Default, Clone, Debug)]
pub struct CityRegistry {
    pub cities: HashMap<AirportId, crate::data::cities::CityInfo>,
}

// ═══════════════════════════════════════════════════════════════════════════
// UTILITY FUNCTIONS
// ═══════════════════════════════════════════════════════════════════════════

/// Convert grid coordinates to world-space center of tile.
pub fn grid_to_world_center(grid_x: i32, grid_y: i32) -> Vec2 {
    Vec2::new(
        grid_x as f32 * TILE_SIZE + TILE_SIZE / 2.0,
        -(grid_y as f32 * TILE_SIZE + TILE_SIZE / 2.0),
    )
}

/// Convert world-space position to grid coordinates.
pub fn world_to_grid(world_pos: Vec2) -> (i32, i32) {
    let gx = (world_pos.x / TILE_SIZE).floor() as i32;
    let gy = (-world_pos.y / TILE_SIZE).floor() as i32;
    (gx, gy)
}

/// Distance between two airports in nautical miles.
pub fn airport_distance(from: AirportId, to: AirportId) -> f32 {
    // Hardcoded distance table — symmetric
    let key = if (from as u8) < (to as u8) {
        (from, to)
    } else {
        (to, from)
    };
    match key {
        (AirportId::HomeBase, AirportId::Windport) => 120.0,
        (AirportId::HomeBase, AirportId::Frostpeak) => 200.0,
        (AirportId::HomeBase, AirportId::Sunhaven) => 350.0,
        (AirportId::HomeBase, AirportId::Ironforge) => 180.0,
        (AirportId::HomeBase, AirportId::Cloudmere) => 400.0,
        (AirportId::HomeBase, AirportId::Duskhollow) => 500.0,
        (AirportId::HomeBase, AirportId::Stormwatch) => 600.0,
        (AirportId::HomeBase, AirportId::Grandcity) => 450.0,
        (AirportId::HomeBase, AirportId::Skyreach) => 800.0,
        (AirportId::Windport, AirportId::Sunhaven) => 250.0,
        (AirportId::Windport, AirportId::Frostpeak) => 300.0,
        (AirportId::Windport, AirportId::Ironforge) => 200.0,
        (AirportId::Windport, AirportId::Grandcity) => 350.0,
        (AirportId::Frostpeak, AirportId::Cloudmere) => 150.0,
        (AirportId::Frostpeak, AirportId::Stormwatch) => 400.0,
        (AirportId::Sunhaven, AirportId::Duskhollow) => 300.0,
        (AirportId::Sunhaven, AirportId::Grandcity) => 200.0,
        (AirportId::Ironforge, AirportId::Grandcity) => 250.0,
        (AirportId::Cloudmere, AirportId::Skyreach) => 350.0,
        (AirportId::Duskhollow, AirportId::Stormwatch) => 200.0,
        (AirportId::Grandcity, AirportId::Skyreach) => 500.0,
        (AirportId::Stormwatch, AirportId::Skyreach) => 300.0,
        _ => 400.0, // Default for unspecified routes
    }
}
