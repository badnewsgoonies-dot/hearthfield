# City Office Worker DLC - Type Contract Bible

This file defines the implementation contract for the prototype. Treat names and semantics here as the shared vocabulary across workers.

## Contract-First Rule

Before any new implementation wave:
1. Update this file first.
2. Freeze names for that wave (`states`, `resources`, `events`, `components`, invariants).
3. Dispatch workers only after freeze.
4. If implementation reveals missing contract surface, pause lanes, amend contract, then resume.

This mirrors the successful early Hearthfield pattern: contract/stub first, then parallel waves, then hardening.

## 1) App States

```rust
#[derive(States, Debug, Clone, Eq, PartialEq, Hash, Default)]
pub enum OfficeGameState {
    #[default]
    Boot,
    MainMenu,
    InDay,
    DaySummary,
    Paused,
}
```

Rules:
- `InDay` runs gameplay simulation.
- `DaySummary` is the only state allowed to apply salary/reputation rollover.

## 2) Core Resources

```rust
#[derive(Resource, Debug, Clone)]
pub struct OfficeClock {
    pub day_index: u32,          // starts at 1
    pub minute_of_day: u16,      // [0..1439]
    pub day_start_minute: u16,   // default 510 (08:30)
    pub day_end_minute: u16,     // default 1080 (18:00)
    pub time_scale: f32,         // sim minutes per real second
    pub paused: bool,
}

#[derive(Resource, Debug, Clone)]
pub struct WorkerStats {
    pub energy: i32,             // clamp [0..100]
    pub stress: i32,             // clamp [0..100]
    pub focus: i32,              // clamp [0..100]
    pub money: i32,
    pub reputation: i32,         // clamp [-100..100]
}

#[derive(Resource, Debug, Clone, Default)]
pub struct TaskBoard {
    pub active: Vec<OfficeTask>,
    pub completed_today: Vec<TaskId>,
    pub failed_today: Vec<TaskId>,
}

#[derive(Resource, Debug, Clone)]
pub struct OfficeRunConfig {
    pub seed: u64,
    pub max_tasks_per_day: u8,
    pub interruption_chance_per_hour: f32,
}

#[derive(Resource, Debug, Clone, Default)]
pub struct DayOutcome {
    pub salary_earned: i32,
    pub reputation_delta: i32,
    pub stress_delta: i32,
    pub completed_tasks: u32,
    pub failed_tasks: u32,
}
```

Rules:
- `WorkerStats` clamps must be enforced by a dedicated normalization system.
- `TaskBoard.active` must not contain duplicate `TaskId` values.

## 3) Core Value Types

```rust
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct TaskId(pub u64);

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum TaskKind {
    DataEntry,
    Filing,
    EmailTriage,
    PermitReview,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum TaskPriority {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone)]
pub struct OfficeTask {
    pub id: TaskId,
    pub kind: TaskKind,
    pub priority: TaskPriority,
    pub required_focus: i32,
    pub stress_impact: i32,
    pub reward_money: i32,
    pub reward_reputation: i32,
    pub deadline_minute: u16,
    pub progress: f32,           // [0.0..1.0]
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum InterruptionKind {
    ManagerRequest,
    EmergencyMeeting,
    SystemOutage,
    CoworkerHelp,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ChoiceId {
    A,
    B,
    C,
}
```

Rules:
- `OfficeTask.progress` must remain in `[0.0, 1.0]`.
- A task is complete exactly when `progress >= 1.0`.

## 4) Components

```rust
#[derive(Component)]
pub struct PlayerOfficeWorker;

#[derive(Component)]
pub struct OfficeDesk;

#[derive(Component)]
pub struct Interactable {
    pub interaction_id: &'static str,
}

#[derive(Component)]
pub struct NpcCoworker {
    pub npc_id: u32,
    pub role: NpcRole,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum NpcRole {
    Manager,
    Clerk,
    Analyst,
}
```

## 5) Events

```rust
#[derive(Event, Debug, Clone, Copy)]
pub struct TaskAccepted {
    pub task_id: TaskId,
}

#[derive(Event, Debug, Clone, Copy)]
pub struct TaskProgressed {
    pub task_id: TaskId,
    pub delta: f32,
}

#[derive(Event, Debug, Clone, Copy)]
pub struct TaskCompleted {
    pub task_id: TaskId,
}

#[derive(Event, Debug, Clone, Copy)]
pub struct TaskFailed {
    pub task_id: TaskId,
}

#[derive(Event, Debug, Clone, Copy)]
pub struct InterruptionTriggered {
    pub kind: InterruptionKind,
}

#[derive(Event, Debug, Clone, Copy)]
pub struct InterruptionChoiceMade {
    pub kind: InterruptionKind,
    pub choice: ChoiceId,
}

#[derive(Event, Debug, Clone, Copy)]
pub struct EndDayRequested;

#[derive(Event, Debug, Clone, Copy)]
pub struct DayAdvanced {
    pub new_day_index: u32,
}
```

Event flow:
- `TaskProgressed` -> (`TaskCompleted` or keep active).
- Deadline breach -> `TaskFailed`.
- `EndDayRequested` -> evaluate board + stats -> emit `DayAdvanced`.

## 6) System Sets and Ordering

```rust
#[derive(SystemSet, Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum OfficeSimSet {
    Input,
    Time,
    TaskGeneration,
    TaskResolution,
    Interruptions,
    Economy,
    StateTransitions,
    Ui,
}
```

Required order inside `Update` while in `OfficeGameState::InDay`:

1. `Input`
2. `Time`
3. `TaskGeneration`
4. `TaskResolution`
5. `Interruptions`
6. `Economy`
7. `StateTransitions`
8. `Ui`

## 7) Persistence Contract

Persist these at minimum:
- `OfficeClock.day_index`
- `WorkerStats`
- `TaskBoard.active` (including progress/deadlines)
- `OfficeRunConfig.seed`

Save/load invariants:
- Round-trip must preserve exact `TaskId` values.
- Loading mid-day must not regenerate already active tasks.

## 8) Invariants and Test Expectations

Required invariant tests:
- Stat clamping never exceeds allowed bounds.
- Duplicate task IDs are rejected.
- A completed task cannot emit `TaskFailed` later in same day.
- `DayAdvanced` fires at most once per day.

Headless simulation expectations:
- 3-day deterministic replay with fixed seed yields identical day outcomes.
- No panic during 5-day autoplay.

## 9) Change Protocol

Before implementation changes that alter shared behavior:
1. Update this contract first.
2. Record rationale in `DECISIONS.md`.
3. Update any affected tests.

No worker should introduce alternate names for contract concepts without updating this file.

## 10) R5 Addendum (Economy + Startup Hardening)

Additional frozen resources:

```rust
#[derive(Resource, Debug, Clone)]
pub struct OfficeEconomyRules {
    pub base_salary_per_task: i32,
    pub failure_penalty_per_task: i32,
    pub level_salary_bonus: i32,
    pub streak_bonus_per_day: i32,
    pub max_streak_bonus_days: u32,
    pub burnout_stress_threshold: i32,
    pub burnout_salary_penalty: i32,
    pub xp_per_completed_task: u32,
    pub xp_penalty_per_failed_task: u32,
    pub xp_per_manager_checkin: u32,
    pub xp_per_coworker_help: u32,
    pub reputation_per_diplomacy_perk: i32,
    pub stress_relief_per_resilience_perk: i32,
    pub process_energy_discount_per_efficiency_perk: i32,
    pub max_perk_level: u8,
}

#[derive(Resource, Debug, Clone)]
pub struct CareerProgression {
    pub level: u32,
    pub xp: u32,
    pub success_streak: u32,
    pub burnout_days: u32,
    pub efficiency_perk: u8,
    pub resilience_perk: u8,
    pub diplomacy_perk: u8,
}
```

Additional invariants:
1. `setup_scene` must be idempotent for first-seconds singleton entities (`Camera2d`, worker, inbox avatar).
2. Task-board seed generation must provide deterministic content variety (all task kinds and priority tiers across a normal workday board).
3. Day outcome preview and rollover must remain deterministic for fixed seed and fixed action script.
4. Save/load must round-trip progression state (`CareerProgression`) without schema drift.

## 11) R6 Addendum (Social Graph + Scenario Determinism)

Additional frozen resources:

```rust
#[derive(Resource, Debug, Clone)]
pub struct SocialGraphState {
    pub profiles: Vec<CoworkerProfile>,
    pub scenario_cursor: u32,
}

#[derive(Debug, Clone)]
pub struct CoworkerProfile {
    pub id: u8,
    pub codename: String,
    pub role: CoworkerRole,
    pub affinity: i32,
    pub trust: i32,
}
```

Additional invariants:
1. `SocialGraphState` must normalize to unique profile IDs and clamp affinity/trust to `[-100, 100]`.
2. Interruption social scenarios must be deterministic for fixed `seed + day + cursor`.
3. Save/load roundtrip must preserve social graph values and scenario cursor.

## 12) R6 Addendum (Progression Unlock Catalog)

Additional frozen resource:

```rust
#[derive(Resource, Debug, Clone, PartialEq, Eq, Default)]
pub struct UnlockCatalogState {
    pub quick_coffee: bool,          // unlock at level >= 2
    pub efficient_processing: bool,  // unlock at level >= 3
    pub conflict_training: bool,     // unlock at level >= 4
    pub escalation_license: bool,    // unlock at level >= 5
}
```

Additional invariants:
1. Unlock booleans must be deterministic functions of progression milestones (`sync_with_progression`) and remain replay-stable for fixed day scripts.
2. Save/load roundtrip must preserve unlock catalog values without schema drift.
3. Unlock gameplay effects are contractually active in core loop systems:
   - `efficient_processing` scales task progress delta.
   - `quick_coffee` reduces coffee action duration with a floor.
   - `conflict_training` improves calm-resolution focus/stress deltas.
   - `escalation_license` adds day-outcome reputation bonus.
