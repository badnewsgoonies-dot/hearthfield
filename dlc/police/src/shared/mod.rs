//! Shared components, resources, events, and states for Precinct.
//!
//! This is the type contract. Every domain plugin imports from here.
//! No domain imports from any other domain directly.
//!
//! Decision: all IDs are String (not numeric).
//! Why: stable serialization, save migration, cross-domain merge behavior.
//! Tempting alternative: numeric IDs.
//! Consequence: coercion bugs, key mismatches, brittle parsing.
//! Drift cue: workers using increment logic, parseInt, or local numeric aliases.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

// ═══════════════════════════════════════════════════════════════════════
// GAME STATE — top-level state machine
// ═══════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, States, Default)]
pub enum GameState {
    #[default]
    Loading,
    MainMenu,
    Playing,
    Paused,
    Dialogue,
    Interrogation,
    EvidenceExam,
    CaseFile,
    SkillTree,
    CareerView,
    MapView,
    ShiftSummary,
    Precinct,
}

// ═══════════════════════════════════════════════════════════════════════
// UPDATE PHASE ORDERING
// ═══════════════════════════════════════════════════════════════════════

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum UpdatePhase {
    Input,
    Intent,
    Simulation,
    Reactions,
    Presentation,
}

// ═══════════════════════════════════════════════════════════════════════
// CALENDAR & SHIFTS
// ═══════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub enum Rank {
    PatrolOfficer,
    Detective,
    Sergeant,
    Lieutenant,
}

impl Rank {
    pub fn index(self) -> usize {
        match self {
            Rank::PatrolOfficer => 0,
            Rank::Detective => 1,
            Rank::Sergeant => 2,
            Rank::Lieutenant => 3,
        }
    }

    pub fn salary(self) -> i32 {
        match self {
            Rank::PatrolOfficer => PATROL_SALARY,
            Rank::Detective => DETECTIVE_SALARY,
            Rank::Sergeant => SERGEANT_SALARY,
            Rank::Lieutenant => LIEUTENANT_SALARY,
        }
    }

    pub fn display_name(self) -> &'static str {
        match self {
            Rank::PatrolOfficer => "Patrol Officer",
            Rank::Detective => "Detective",
            Rank::Sergeant => "Sergeant",
            Rank::Lieutenant => "Lieutenant",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ShiftType {
    Morning,   // 06:00–14:00
    Afternoon, // 14:00–22:00
    Night,     // 22:00–06:00 (Detective+ only)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DayOfWeek {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Weather {
    Clear,
    Rainy,
    Foggy,
    Snowy,
}

#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct ShiftClock {
    pub shift_number: u32,        // total shifts worked (1-indexed)
    pub day: u32,                 // current day (1-indexed)
    pub day_of_week: DayOfWeek,
    pub hour: u8,                 // 0–23
    pub minute: u8,               // 0–59
    pub shift_type: ShiftType,
    pub on_duty: bool,
    pub weather: Weather,
    pub rank: Rank,
    pub time_scale: f32,          // game-minutes per real-second
    pub time_paused: bool,
    pub elapsed_real_seconds: f32,
}

impl Default for ShiftClock {
    fn default() -> Self {
        Self {
            shift_number: 1,
            day: 1,
            day_of_week: DayOfWeek::Monday,
            hour: 6,
            minute: 0,
            shift_type: ShiftType::Morning,
            on_duty: true,
            weather: Weather::Clear,
            rank: Rank::PatrolOfficer,
            time_scale: TIME_SCALE,
            time_paused: false,
            elapsed_real_seconds: 0.0,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// PLAYER
// ═══════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Facing {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Equipment {
    Badge,
    Sidearm,
    Flashlight,
    Radio,
    Notebook,
    Handcuffs,
    ForensicKit,
    Camera,
}

#[derive(Component, Debug)]
pub struct Player;

#[derive(Component, Debug)]
pub struct PlayerMovement {
    pub speed: f32,
    pub facing: Facing,
    pub is_running: bool,
}

impl Default for PlayerMovement {
    fn default() -> Self {
        Self {
            speed: 80.0,
            facing: Facing::Down,
            is_running: false,
        }
    }
}

#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct PlayerState {
    pub fatigue: f32,
    pub stress: f32,
    pub gold: i32,
    pub equipped: Vec<Equipment>,
    pub position_map: MapId,
    pub position_x: f32,
    pub position_y: f32,
}

impl Default for PlayerState {
    fn default() -> Self {
        Self {
            fatigue: MAX_FATIGUE,
            stress: 0.0,
            gold: 200,
            equipped: vec![Equipment::Badge, Equipment::Radio, Equipment::Notebook],
            position_map: MapId::PrecinctInterior,
            position_x: 0.0,
            position_y: 0.0,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// INVENTORY & ITEMS
// ═══════════════════════════════════════════════════════════════════════

pub type ItemId = String;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ItemCategory {
    Evidence,
    PersonalItem,
    Consumable,
    Equipment,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemDef {
    pub id: ItemId,
    pub name: String,
    pub category: ItemCategory,
    pub description: String,
    pub sell_price: i32,
    pub stackable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventorySlot {
    pub item_id: ItemId,
    pub quantity: u32,
}

#[derive(Resource, Debug, Clone, Serialize, Deserialize, Default)]
pub struct Inventory {
    pub evidence_slots: Vec<Option<InventorySlot>>,   // 12 slots
    pub personal_slots: Vec<Option<InventorySlot>>,    // 6 slots
}

// ═══════════════════════════════════════════════════════════════════════
// CASES
// ═══════════════════════════════════════════════════════════════════════

pub type CaseId = String;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CaseStatus {
    New,
    Active,
    Investigating,
    EvidenceComplete,
    Interrogating,
    Solved,
    Cold,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaseDef {
    pub id: CaseId,
    pub name: String,
    pub description: String,
    pub rank_required: Rank,
    pub evidence_required: Vec<EvidenceId>,
    pub witnesses: Vec<NpcId>,
    pub suspects: Vec<NpcId>,
    pub scenes: Vec<MapId>,
    pub time_limit_shifts: Option<u8>,
    pub reward_xp: u32,
    pub reward_reputation: i32,
    pub reward_gold: i32,
    pub difficulty: u8, // 1–10
    pub is_major: bool, // scripted narrative case
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveCase {
    pub case_id: CaseId,
    pub status: CaseStatus,
    pub evidence_collected: Vec<EvidenceId>,
    pub witnesses_interviewed: HashSet<NpcId>,
    pub suspects_interrogated: HashSet<NpcId>,
    pub shifts_elapsed: u8,
    pub notes: Vec<String>,
}

#[derive(Resource, Debug, Clone, Serialize, Deserialize, Default)]
pub struct CaseBoard {
    pub available: Vec<CaseId>,
    pub active: Vec<ActiveCase>,
    pub solved: Vec<CaseId>,
    pub cold: Vec<CaseId>,
    pub failed: Vec<CaseId>,
    pub total_cases_solved: u32,
}

// ═══════════════════════════════════════════════════════════════════════
// EVIDENCE
// ═══════════════════════════════════════════════════════════════════════

pub type EvidenceId = String;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EvidenceCategory {
    Physical,
    Documentary,
    Testimonial,
    Forensic,
    Environmental,
    Circumstantial,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EvidenceProcessingState {
    Raw,
    Processing,
    Analyzed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidencePiece {
    pub id: EvidenceId,
    pub name: String,
    pub category: EvidenceCategory,
    pub description: String,
    pub quality: f32,              // 0.0–1.0
    pub linked_case: Option<CaseId>,
    pub processing_state: EvidenceProcessingState,
    pub collected_shift: u32,
    pub collected_map: MapId,
}

#[derive(Resource, Debug, Clone, Serialize, Deserialize, Default)]
pub struct EvidenceLocker {
    pub pieces: Vec<EvidencePiece>,
}

// ═══════════════════════════════════════════════════════════════════════
// NPCs
// ═══════════════════════════════════════════════════════════════════════

pub type NpcId = String;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NpcRole {
    Captain,
    Partner,
    Colleague,
    Mentor,
    Mayor,
    MedicalExaminer,
    Informant,
    Priest,
    Tipster,
    Journalist,
    ExCon,
    PublicDefender,
    Witness,
    Suspect,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NpcDef {
    pub id: NpcId,
    pub name: String,
    pub role: NpcRole,
    pub default_map: MapId,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NpcRelationship {
    pub npc_id: NpcId,
    pub trust: i32,       // -100 to +100
    pub pressure: i32,    // 0 to 100
    pub favors_done: u32,
    pub dialogue_flags: HashSet<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleEntry {
    pub hour: u8,
    pub map_id: MapId,
    pub x: f32,
    pub y: f32,
}

#[derive(Resource, Debug, Clone, Serialize, Deserialize, Default)]
pub struct NpcRegistry {
    pub definitions: HashMap<NpcId, NpcDef>,
    pub relationships: HashMap<NpcId, NpcRelationship>,
    pub schedules: HashMap<NpcId, Vec<ScheduleEntry>>,
}

#[derive(Component, Debug)]
pub struct Npc {
    pub id: NpcId,
}

// ═══════════════════════════════════════════════════════════════════════
// PARTNER ARC
// ═══════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PartnerStage {
    Stranger,
    UneasyPartners,
    WorkingRapport,
    TrustedPartners,
    BestFriends,
}

#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct PartnerArc {
    pub stage: PartnerStage,
    pub events_triggered: HashSet<String>,
}

impl Default for PartnerArc {
    fn default() -> Self {
        Self {
            stage: PartnerStage::Stranger,
            events_triggered: HashSet::new(),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// ECONOMY
// ═══════════════════════════════════════════════════════════════════════

#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct Economy {
    pub reputation: i32,           // -100 to +100
    pub department_budget: i32,    // shared resource for equipment
    pub weekly_expenses: i32,
    pub total_earned: i32,
}

impl Default for Economy {
    fn default() -> Self {
        Self {
            reputation: 0,
            department_budget: 500,
            weekly_expenses: 120, // rent 100 + maintenance 20
            total_earned: 0,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// SKILLS
// ═══════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SkillTree {
    Investigation,
    Interrogation,
    Patrol,
    Leadership,
}

#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct Skills {
    pub total_xp: u32,
    pub available_points: u32,
    pub investigation_level: u8,   // 0–5
    pub interrogation_level: u8,   // 0–5
    pub patrol_level: u8,          // 0–5
    pub leadership_level: u8,      // 0–5
}

impl Default for Skills {
    fn default() -> Self {
        Self {
            total_xp: 0,
            available_points: 0,
            investigation_level: 0,
            interrogation_level: 0,
            patrol_level: 0,
            leadership_level: 0,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// PATROL & DISPATCH
// ═══════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DispatchEventKind {
    TrafficStop,
    NoiseComplaint,
    ShoplifterInProgress,
    DomesticDisturbance,
    SuspiciousVehicle,
    OfficerNeedsBackup,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DispatchCall {
    pub kind: DispatchEventKind,
    pub map_id: MapId,
    pub description: String,
    pub fatigue_cost: f32,
    pub stress_cost: f32,
    pub xp_reward: u32,
    pub may_generate_evidence: bool,
}

#[derive(Resource, Debug, Clone, Serialize, Deserialize, Default)]
pub struct PatrolState {
    pub fuel: f32,
    pub calls_responded: u32,
    pub calls_ignored: u32,
    pub current_dispatch: Option<DispatchCall>,
}

// ═══════════════════════════════════════════════════════════════════════
// MAPS
// ═══════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MapId {
    PrecinctInterior,
    PrecinctExterior,
    Downtown,
    ResidentialNorth,
    ResidentialSouth,
    IndustrialDistrict,
    Highway,
    ForestPark,
    CrimeSceneTemplate,
    Hospital,
    CourtHouse,
    PlayerApartment,
}

impl MapId {
    pub fn display_name(self) -> &'static str {
        match self {
            MapId::PrecinctInterior => "Precinct",
            MapId::PrecinctExterior => "Precinct Parking",
            MapId::Downtown => "Downtown",
            MapId::ResidentialNorth => "Residential North",
            MapId::ResidentialSouth => "Residential South",
            MapId::IndustrialDistrict => "Industrial District",
            MapId::Highway => "Highway",
            MapId::ForestPark => "Forest Park",
            MapId::CrimeSceneTemplate => "Crime Scene",
            MapId::Hospital => "Hospital",
            MapId::CourtHouse => "Court House",
            MapId::PlayerApartment => "Your Apartment",
        }
    }

    pub fn dispatch_rate_modifier(self) -> f32 {
        match self {
            MapId::Downtown => 1.5,
            MapId::IndustrialDistrict => 1.2,
            MapId::ResidentialNorth | MapId::ResidentialSouth => 1.0,
            MapId::Highway => 0.8,
            MapId::ForestPark => 0.5,
            _ => 0.0, // interior maps don't generate dispatch
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TileKind {
    Floor,
    Wall,
    Door,
    Sidewalk,
    Road,
    Grass,
    Water,
    CrimeTape,  // restricted area
    Interactable,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct GridPosition {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapTransition {
    pub from_map: MapId,
    pub from_x: i32,
    pub from_y: i32,
    pub to_map: MapId,
    pub to_x: i32,
    pub to_y: i32,
}

// ═══════════════════════════════════════════════════════════════════════
// EVENTS — cross-domain communication
// ═══════════════════════════════════════════════════════════════════════

#[derive(Event, Debug)]
pub struct ShiftEndEvent {
    pub shift_number: u32,
    pub cases_progressed: u32,
    pub evidence_collected: u32,
    pub xp_earned: u32,
}

#[derive(Event, Debug)]
pub struct CaseAssignedEvent {
    pub case_id: CaseId,
}

#[derive(Event, Debug)]
pub struct CaseSolvedEvent {
    pub case_id: CaseId,
    pub xp_reward: u32,
    pub gold_reward: i32,
    pub reputation_reward: i32,
}

#[derive(Event, Debug)]
pub struct CaseFailedEvent {
    pub case_id: CaseId,
    pub reason: String,
}

#[derive(Event, Debug)]
pub struct EvidenceCollectedEvent {
    pub evidence_id: EvidenceId,
    pub case_id: CaseId,
    pub quality: f32,
}

#[derive(Event, Debug)]
pub struct EvidenceProcessedEvent {
    pub evidence_id: EvidenceId,
}

#[derive(Event, Debug)]
pub struct InterrogationStartEvent {
    pub npc_id: NpcId,
    pub case_id: CaseId,
}

#[derive(Event, Debug)]
pub struct InterrogationEndEvent {
    pub npc_id: NpcId,
    pub case_id: CaseId,
    pub confession: bool,
}

#[derive(Event, Debug)]
pub struct DispatchCallEvent {
    pub call: DispatchCall,
}

#[derive(Event, Debug)]
pub struct DispatchResolvedEvent {
    pub kind: DispatchEventKind,
    pub xp_earned: u32,
}

#[derive(Event, Debug)]
pub struct PromotionEvent {
    pub new_rank: Rank,
}

#[derive(Event, Debug)]
pub struct NpcTrustChangeEvent {
    pub npc_id: NpcId,
    pub trust_delta: i32,
    pub pressure_delta: i32,
}

#[derive(Event, Debug)]
pub struct DialogueStartEvent {
    pub npc_id: NpcId,
    pub context: String,
}

#[derive(Event, Debug)]
pub struct DialogueEndEvent;

#[derive(Event, Debug)]
pub struct MapTransitionEvent {
    pub from: MapId,
    pub to: MapId,
}

#[derive(Event, Debug)]
pub struct FatigueChangeEvent {
    pub delta: f32,
}

#[derive(Event, Debug)]
pub struct StressChangeEvent {
    pub delta: f32,
}

#[derive(Event, Debug)]
pub struct GoldChangeEvent {
    pub amount: i32,
    pub reason: String,
}

#[derive(Event, Debug)]
pub struct XpGainedEvent {
    pub amount: u32,
    pub source: String,
}

#[derive(Event, Debug)]
pub struct SkillPointSpentEvent {
    pub tree: SkillTree,
    pub new_level: u8,
}

#[derive(Event, Debug)]
pub struct PlaySfxEvent {
    pub name: String,
}

#[derive(Event, Debug)]
pub struct PlayMusicEvent {
    pub name: String,
    pub looping: bool,
}

#[derive(Event, Debug)]
pub struct ToastEvent {
    pub message: String,
    pub duration_secs: f32,
}

#[derive(Event, Debug, Default)]
pub struct SaveRequestEvent;

#[derive(Event, Debug, Default)]
pub struct LoadRequestEvent {
    pub slot: u8,
}

// ═══════════════════════════════════════════════════════════════════════
// INPUT ABSTRACTION
// ═══════════════════════════════════════════════════════════════════════

#[derive(Resource, Debug, Clone, Default)]
pub struct PlayerInput {
    pub move_dir: Vec2,
    pub interact: bool,
    pub cancel: bool,
    pub run: bool,
    pub menu: bool,
    pub notebook: bool,
    pub radio: bool,
}

#[derive(Resource, Debug, Clone, Default)]
pub struct InputContext {
    pub in_dialogue: bool,
    pub in_menu: bool,
    pub in_interrogation: bool,
}

// ═══════════════════════════════════════════════════════════════════════
// CONSTANTS — frozen, workers must use these exactly
// ═══════════════════════════════════════════════════════════════════════

pub const TILE_SIZE: f32 = 16.0;
pub const PIXEL_SCALE: f32 = 3.0;
pub const SCREEN_WIDTH: f32 = 960.0;
pub const SCREEN_HEIGHT: f32 = 540.0;

pub const MAX_FATIGUE: f32 = 100.0;
pub const MAX_STRESS: f32 = 100.0;
pub const MAX_TRUST: i32 = 100;
pub const MIN_TRUST: i32 = -100;
pub const MAX_PRESSURE: i32 = 100;
pub const MAX_REPUTATION: i32 = 100;
pub const MIN_REPUTATION: i32 = -100;

pub const TIME_SCALE: f32 = 2.0;
pub const SHIFT_DURATION_HOURS: u8 = 8;
pub const SHIFTS_PER_WEEK: u8 = 3;

pub const DISPATCH_BASE_RATE: f32 = 0.15;
pub const DISPATCH_NIGHT_MODIFIER: f32 = 1.5;

pub const EVIDENCE_BASE_QUALITY: f32 = 0.5;
pub const EVIDENCE_SKILL_BONUS: f32 = 0.05;
pub const EVIDENCE_MAX_QUALITY: f32 = 0.95;
pub const EVIDENCE_WEATHER_PENALTY: f32 = 0.1;

pub const PATROL_SALARY: i32 = 80;
pub const DETECTIVE_SALARY: i32 = 120;
pub const SERGEANT_SALARY: i32 = 160;
pub const LIEUTENANT_SALARY: i32 = 200;
pub const CASE_CLOSE_BONUS_MULTIPLIER: i32 = 25;
pub const FAILED_CASE_PENALTY: i32 = 50;

pub const PROMOTION_DETECTIVE_XP: u32 = 200;
pub const PROMOTION_DETECTIVE_CASES: u32 = 3;
pub const PROMOTION_DETECTIVE_REP: i32 = 10;
pub const PROMOTION_SERGEANT_XP: u32 = 500;
pub const PROMOTION_SERGEANT_CASES: u32 = 8;
pub const PROMOTION_SERGEANT_REP: i32 = 25;
pub const PROMOTION_LIEUTENANT_XP: u32 = 1000;
pub const PROMOTION_LIEUTENANT_CASES: u32 = 16;
pub const PROMOTION_LIEUTENANT_REP: i32 = 50;

pub const XP_PER_EVIDENCE: u32 = 5;
pub const XP_PER_INTERROGATION: u32 = 20;
pub const XP_PER_PATROL_EVENT: u32 = 10;
pub const XP_PER_FAVOR: u32 = 8;
pub const XP_CASE_MULTIPLIER: u32 = 15;
pub const SKILL_POINT_INTERVAL: u32 = 100;

pub const COFFEE_FATIGUE_RESTORE: f32 = 25.0;
pub const COFFEE_STRESS_RELIEF: f32 = 5.0;
pub const COFFEE_TIME_COST_MINUTES: u32 = 15;
pub const MEAL_FATIGUE_RESTORE: f32 = 50.0;
pub const MEAL_STRESS_RELIEF: f32 = 10.0;
pub const MEAL_TIME_COST_MINUTES: u32 = 30;

pub const FUEL_MAX: f32 = 100.0;
pub const FUEL_COST_PER_TRIP: f32 = 15.0;

pub const MAX_ACTIVE_CASES: usize = 3;
