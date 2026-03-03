# City Office Worker DLC - TASKS (Rolling Rotations)

Objective: run long-horizon rotating lanes until OES-v1 parity is reached.

## Lane Rules

1. Each lane edits only allowlisted files.
2. All shared vocabulary must match `CONTRACT.md`.
3. Integrator lane owns merge, gate runs, and status updates.
4. A lane is done only when acceptance criteria and quality gates pass.

## Active Rotation: R4 (Planned)

### Lane INT - Integrator

Allowlist:
- `city_office_worker_dlc/src/main.rs`
- `city_office_worker_dlc/src/game/mod.rs`
- `city_office_worker_dlc/src/game/resources.rs`
- `city_office_worker_dlc/src/game/events.rs`
- `city_office_worker_dlc/src/game/components.rs`
- `city_office_worker_dlc/src/game/systems/**`
- `city_office_worker_dlc/src/game/save/**`
- `city_office_worker_dlc/STATUS.md`
- `city_office_worker_dlc/DECISIONS.md`

Acceptance criteria:
1. No contract drift between systems, resources, events, and save schema.
2. System ordering remains deterministic.
3. Gate suite is green before handoff.

### Lane SAVE - Durable Save Slots

Allowlist:
- `city_office_worker_dlc/src/game/save/**`
- `city_office_worker_dlc/src/game/resources.rs`
- `city_office_worker_dlc/src/game/systems/day_cycle.rs`
- `city_office_worker_dlc/src/game/systems/tests.rs`

Acceptance criteria:
1. Save slot write/read flow persists snapshot JSON outside volatile memory.
2. Load path restores exact `TaskId` identities and mid-day task board state.
3. Versioned schema/migration stub is present for forward compatibility.

### Lane TASK - Progression/Deadline Semantics

Allowlist:
- `city_office_worker_dlc/src/game/systems/tasks.rs`
- `city_office_worker_dlc/src/game/systems/day_cycle.rs`
- `city_office_worker_dlc/src/game/events.rs`
- `city_office_worker_dlc/src/game/resources.rs`
- `city_office_worker_dlc/src/game/systems/tests.rs`

Acceptance criteria:
1. Task progress can advance by non-trivial deltas and complete deterministically.
2. Deadline-breach path emits `TaskFailed` without violating terminal disjointness.
3. Invariant tests cover completion/failure exclusivity and repeated-event safety.

### Lane INV - Investigation/Audit

Allowlist:
- `city_office_worker_dlc/research/**`
- `city_office_worker_dlc/STATUS.md`

Acceptance criteria:
1. Publish social/progression parity packet with executable checks.
2. Update parity delta report with next highest-impact blockers.
3. Record waivers with owners and follow-up tasks.

## Rotation Exit Criteria

1. `cargo check --manifest-path city_office_worker_dlc/Cargo.toml` passes.
2. `cargo test --manifest-path city_office_worker_dlc/Cargo.toml` passes.
3. `cargo clippy --manifest-path city_office_worker_dlc/Cargo.toml --all-targets -- -D warnings` passes.
4. Scope guard confirms allowlist compliance.
