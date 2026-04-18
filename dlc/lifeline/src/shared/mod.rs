//! Lifeline — shared contract. FROZEN: domain crates import from here;
//! none may modify this file. Briefcase transforms that splice new
//! events/components/resources into this file are allowed (they use
//! structural anchors at end-of-file).

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

// ── constants ─────────────────────────────────────────────────────────

pub const TILE_SIZE: f32 = 16.0;
pub const PIXEL_SCALE: f32 = 3.0;
pub const MAX_STAMINA: i32 = 100;
pub const MAX_TRUST: i32 = 100;
pub const MIN_TRUST: i32 = -100;
pub const MAX_PRESSURE: i32 = 100;
pub const XP_PER_DIAGNOSTIC: u32 = 5;
pub const XP_PER_DISCHARGE: u32 = 20;

// ── state ─────────────────────────────────────────────────────────────

#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Default, Serialize, Deserialize)]
pub enum GameState {
    #[default]
    Boot,
    MainMenu,
    OnShift,
    ShiftSummary,
    Paused,
    Dialogue,
}

#[derive(SystemSet, Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum UpdatePhase {
    Input,
    Intent,
    Simulation,
    Reactions,
    Presentation,
}

// ── rank progression (career tier analog of police Rank) ─────────────

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum Rank {
    Intern,
    Resident,
    Attending,
    ChiefOfMedicine,
}

impl Default for Rank {
    fn default() -> Self {
        Self::Intern
    }
}

// ── shift clock ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum ShiftType {
    Morning,
    Afternoon,
    Night,
}

impl Default for ShiftType {
    fn default() -> Self {
        Self::Morning
    }
}

#[derive(Resource, Debug, Clone, Default, Serialize, Deserialize)]
pub struct ShiftClock {
    pub day: u32,
    pub shift_number: u32,
    pub shift_type: ShiftType,
    pub hour: u8,
    pub minute: u8,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum DayOfWeek {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}

impl Default for DayOfWeek {
    fn default() -> Self {
        Self::Monday
    }
}

// ── map / world ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum MapId {
    LobbyReception,
    EmergencyRoom,
    IcuWard,
    MedicalWard,
    SurgicalWard,
    OperatingRoom,
    Pharmacy,
    Laboratory,
    Cafeteria,
    LockerRoom,
    Rooftop,
    Parking,
}

impl MapId {
    pub fn display_name(self) -> &'static str {
        match self {
            Self::LobbyReception => "Lobby",
            Self::EmergencyRoom => "ER",
            Self::IcuWard => "ICU",
            Self::MedicalWard => "Medical Ward",
            Self::SurgicalWard => "Surgical Ward",
            Self::OperatingRoom => "OR",
            Self::Pharmacy => "Pharmacy",
            Self::Laboratory => "Lab",
            Self::Cafeteria => "Cafeteria",
            Self::LockerRoom => "Lockers",
            Self::Rooftop => "Rooftop",
            Self::Parking => "Parking",
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Default, Serialize, Deserialize)]
pub struct GridPosition {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Default, Serialize, Deserialize)]
pub enum Facing {
    Left,
    Right,
    Up,
    #[default]
    Down,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Default, Serialize, Deserialize)]
pub enum Weather {
    #[default]
    Clear,
    Rain,
    Snow,
    Fog,
}

// ── player ────────────────────────────────────────────────────────────

#[derive(Resource, Debug, Clone, Default, Serialize, Deserialize)]
pub struct PlayerState {
    pub rank: Rank,
    pub xp: u32,
    pub position_map: Option<MapId>,
    pub position_x: f32,
    pub position_y: f32,
    pub stamina: i32,
    pub facing: Facing,
}

#[derive(Resource, Debug, Clone, Default)]
pub struct PlayerInput {
    pub move_x: f32,
    pub move_y: f32,
    pub interact_pressed: bool,
    pub cancel_pressed: bool,
}

// ── patient (case analog) ─────────────────────────────────────────────

pub type PatientId = String;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum PatientAcuity {
    /// Stable, can wait.
    Routine,
    /// Needs attention same shift.
    Urgent,
    /// Actively deteriorating.
    Critical,
    /// Recovering; discharge soon.
    Recovering,
    /// End-of-life care path.
    Palliative,
}

impl Default for PatientAcuity {
    fn default() -> Self {
        Self::Routine
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum PatientStatus {
    Admitted,
    Diagnosing,
    Treating,
    Observing,
    Discharged,
    Deceased,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Patient {
    pub id: PatientId,
    pub name: String,
    pub complaint: String,
    pub acuity: PatientAcuity,
    pub rank_required: Rank,
    pub diagnostics_required: Vec<DiagnosticId>,
    pub ward: MapId,
    pub shifts_remaining: Option<u8>,
    pub reward_xp: u32,
    pub reward_reputation: i32,
    pub is_major: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivePatient {
    pub patient_id: PatientId,
    pub status: PatientStatus,
    pub diagnostics_collected: Vec<DiagnosticId>,
    pub interventions_applied: HashSet<String>,
    pub shifts_elapsed: u8,
    pub notes: Vec<String>,
}

#[derive(Resource, Debug, Clone, Default, Serialize, Deserialize)]
pub struct PatientBoard {
    pub available: Vec<PatientId>,
    pub active: Vec<ActivePatient>,
    pub discharged: Vec<PatientId>,
}

// ── diagnostics (evidence analog) ─────────────────────────────────────

pub type DiagnosticId = String;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum DiagnosticKind {
    Vitals,
    BloodPanel,
    Imaging,
    Biopsy,
    Interview,
    Observation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnostic {
    pub id: DiagnosticId,
    pub kind: DiagnosticKind,
    pub description: String,
    pub patient_id: PatientId,
    pub quality: f32,
}

#[derive(Resource, Debug, Clone, Default, Serialize, Deserialize)]
pub struct DiagnosticQueue {
    pub pending: Vec<Diagnostic>,
    pub completed: Vec<Diagnostic>,
}

// ── npcs ──────────────────────────────────────────────────────────────

pub type NpcId = String;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum NpcRole {
    ChiefOfStaff,
    SeniorDoctor,
    Colleague,
    Nurse,
    Pharmacist,
    Administrator,
    Patient,
    Family,
    Specialist,
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
pub struct ScheduleEntry {
    pub hour: u8,
    pub map_id: MapId,
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NpcRelationship {
    pub npc_id: NpcId,
    pub trust: i32,
    pub pressure: i32,
    pub favors_done: i32,
    pub dialogue_flags: HashSet<String>,
}

#[derive(Resource, Debug, Clone, Default, Serialize, Deserialize)]
pub struct NpcRegistry {
    pub definitions: HashMap<NpcId, NpcDef>,
    pub relationships: HashMap<NpcId, NpcRelationship>,
    pub schedules: HashMap<NpcId, Vec<ScheduleEntry>>,
}

#[derive(Component, Debug, Clone)]
pub struct Npc {
    pub id: NpcId,
}

/// Partner arc: the resident who shadows the player early career and
/// graduates to peer-attending by mid-game. Direct analog of precinct's
/// PartnerArc / Vasquez.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum MentorStage {
    Cool,
    Cordial,
    Respected,
    Trusted,
    Indispensable,
}

impl Default for MentorStage {
    fn default() -> Self {
        Self::Cool
    }
}

#[derive(Resource, Debug, Clone, Default, Serialize, Deserialize)]
pub struct MentorArc {
    pub stage: MentorStage,
    pub events_triggered: HashSet<String>,
}

// ── economy ───────────────────────────────────────────────────────────

#[derive(Resource, Debug, Clone, Default, Serialize, Deserialize)]
pub struct Economy {
    pub salary_per_shift: i32,
    pub wallet: i32,
    pub supply_budget: i32,
}

// ── skills ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum SkillKind {
    Diagnostics,
    Surgery,
    BedsideManner,
    Pharmacology,
    Research,
}

#[derive(Resource, Debug, Clone, Default, Serialize, Deserialize)]
pub struct Skills {
    pub levels: HashMap<String, u32>,
}

// ── events (contract surface; briefcase may append new ones via anchor) ─

#[derive(Event, Debug, Clone)]
pub struct PatientAssignedEvent {
    pub patient_id: PatientId,
}

#[derive(Event, Debug, Clone)]
pub struct PatientDischargedEvent {
    pub patient_id: PatientId,
    pub xp_reward: u32,
    pub reputation_reward: i32,
}

#[derive(Event, Debug, Clone)]
pub struct PatientDeclineEvent {
    pub patient_id: PatientId,
    pub reason: String,
}

#[derive(Event, Debug, Clone)]
pub struct DiagnosticCollectedEvent {
    pub patient_id: PatientId,
    pub diagnostic_id: DiagnosticId,
}

#[derive(Event, Debug, Clone)]
pub struct ShiftStartEvent {
    pub shift_number: u32,
}

#[derive(Event, Debug, Clone)]
pub struct ShiftEndEvent {
    pub shift_number: u32,
}

#[derive(Event, Debug, Clone)]
pub struct MapTransitionEvent {
    pub from: MapId,
    pub to: MapId,
}

#[derive(Event, Debug, Clone)]
pub struct DialogueStartEvent {
    pub npc_id: NpcId,
    pub context: Option<String>,
}

#[derive(Event, Debug, Clone)]
pub struct DialogueEndEvent {
    pub npc_id: NpcId,
}

#[derive(Event, Debug, Clone)]
pub struct NpcTrustChangeEvent {
    pub npc_id: NpcId,
    pub trust_delta: i32,
    pub pressure_delta: i32,
}

#[derive(Event, Debug, Clone)]
pub struct ToastEvent {
    pub message: String,
    pub duration_secs: f32,
}

#[derive(Event, Debug, Clone)]
pub struct XpGainedEvent {
    pub amount: u32,
    pub source: String,
}

// ── briefcase anchor (DO NOT REMOVE) ──────────────────────────────────
// Structural transforms (hearthfield_add_event, hearthfield_add_resource,
// hearthfield_add_component) splice at end-of-file. Keep this marker at
// the bottom so the anchor stays findable.
