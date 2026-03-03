# City Office Worker DLC - TASKS (Wave 1)

Wave objective: deliver a stable playable core day loop aligned to `CONTRACT.md`.

## Lane Rules

1. Each lane may edit only its allowlisted files.
2. Cross-lane behavior changes are coordinated through events/resources defined in `CONTRACT.md`.
3. If a lane needs a file outside its allowlist, hand off to Integrator lane.
4. A lane is only done when its acceptance criteria and command checks pass.

## Lane Assignments

### Lane INT - Integrator and Contract Wiring

Allowlist:
- `city_office_worker_dlc/src/main.rs`
- `city_office_worker_dlc/src/game/mod.rs`
- `city_office_worker_dlc/src/game/resources.rs`
- `city_office_worker_dlc/src/game/events.rs`
- `city_office_worker_dlc/src/game/components.rs`
- `city_office_worker_dlc/src/game/systems.rs` (only until split is complete)

Acceptance criteria:
- `OfficeGameState` is wired and governs `InDay` vs `DaySummary` scheduling.
- Shared resource/event names used in runtime match Wave 1 contract vocabulary.
- System order remains deterministic and documented in code.
- `cargo check --manifest-path city_office_worker_dlc/Cargo.toml` passes.

### Lane TIME - Office Clock and Day Transitions

Allowlist:
- `city_office_worker_dlc/src/game/office_time/**`
- `city_office_worker_dlc/tests/wave1_time_state.rs`

Acceptance criteria:
- Day starts from configured start minute and ends at configured end minute.
- End-of-day trigger is emitted at most once per day.
- Clock progression is deterministic for fixed action sequence inputs.
- Time/state tests pass.

### Lane TASKS - Intake, Progress, Completion, Failure

Allowlist:
- `city_office_worker_dlc/src/game/tasks/**`
- `city_office_worker_dlc/tests/wave1_tasks.rs`

Acceptance criteria:
- Task intake generates contract task kinds/priorities with unique IDs.
- Task progress stays within `[0.0, 1.0]` and completion is `progress >= 1.0`.
- Missed deadlines produce task failure exactly once.
- Duplicate task IDs are rejected by tests.

### Lane ECON - Day Outcome, Salary, Reputation

Allowlist:
- `city_office_worker_dlc/src/game/economy/**`
- `city_office_worker_dlc/tests/wave1_economy.rs`

Acceptance criteria:
- End-of-day computes `DayOutcome` from completed/failed work.
- Salary/reputation/stress rollover is applied only in `DaySummary`.
- Stat bounds are enforced after every economy update.
- Economy tests pass.

### Lane UI - Task Board and End-of-Day Summary Presentation

Allowlist:
- `city_office_worker_dlc/src/game/ui/**`
- `city_office_worker_dlc/assets/ui/**` (if needed)

Acceptance criteria:
- Player can open/close task board and view active task state.
- End-of-day summary shows payout, completed, failed, and stat deltas.
- UI reflects current day/time/task pressure without panics.

### Lane TEST - Deterministic Headless Simulation and Tooling

Allowlist:
- `city_office_worker_dlc/tests/headless_wave1.rs`
- `city_office_worker_dlc/tests/common/**`
- `city_office_worker_dlc/tools/scope_guard.sh`

Acceptance criteria:
- Fixed-seed 3-day replay produces identical outcomes across runs.
- 5-day autoplay completes with panic count = 0.
- Scope guard script reports non-allowlisted edits and exits non-zero.
- `cargo test --manifest-path city_office_worker_dlc/Cargo.toml` passes.

## Wave 1 Exit Criteria (All Lanes)

1. G1/G3/G4/G5 gates in `STATUS.md` are all PASS.
2. No lane retains out-of-allowlist edits at handoff.
3. Contract drift list is empty for Wave 1 surface area.
