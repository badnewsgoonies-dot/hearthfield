# City Office Worker DLC - TASKS (Rolling Rotations)

Objective: run long-horizon rotating lanes until OES-v1 parity is reached.

## Lane Rules

1. Each lane edits only allowlisted files.
2. All shared vocabulary must match `CONTRACT.md`.
3. Integrator lane owns merge, gate runs, and status updates.
4. A lane is done only when acceptance criteria and quality gates pass.

## Process Guardrails (R5+)

1. Start each rotation with contract/resource/event delta before broad feature slices when schema changes are needed.
2. Keep commit slices reviewable (target `<=20` files and `~1,200` insertions); split before merge if larger.
3. No `WIP` commits on integration branches.
4. Any contract delta must ship with wiring updates and at least one deterministic/headless regression in the same PR.
5. Keep infra/build changes separate from gameplay/content changes.

## Last Completed Rotation: R5

Delivered:
1. Durable save/load and migration hardening completed, including load-state reconciliation and terminal-set hygiene.
2. Task progression/deadline semantics completed with deterministic invariant coverage.
3. Economy/progression hooks landed (streak/burnout salary modifiers, XP/leveling, auto-perk progression).
4. Startup-first reliability hardening completed (idempotent first-seconds scene setup).
5. Social/progression parity packet published for R6 planning.

## Active Rotation: R6 (In Progress)

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

### Lane ECON - Economy/Progression Depth

Allowlist:
- `city_office_worker_dlc/src/game/resources.rs`
- `city_office_worker_dlc/src/game/systems/day_cycle.rs`
- `city_office_worker_dlc/src/game/systems/tests.rs`
- `city_office_worker_dlc/src/game/events.rs`

Acceptance criteria:
1. Salary/penalty/reputation deltas are tunable and deterministic under replay.
2. Progression hooks can be extended without breaking save compatibility.
3. Regression tests cover reward and penalty edge cases.

### Lane SOC - Social State and Scenarios

Allowlist:
- `city_office_worker_dlc/src/game/resources.rs`
- `city_office_worker_dlc/src/game/events.rs`
- `city_office_worker_dlc/src/game/systems/interruptions.rs`
- `city_office_worker_dlc/src/game/systems/day_cycle.rs`
- `city_office_worker_dlc/src/game/systems/tests.rs`

Acceptance criteria:
1. Persistent coworker/manager relationship state exists with bounded normalization.
2. Deterministic social scenario templates are selected by seed/day and affect outcomes.
3. Save/load roundtrip preserves social state without replay drift.

### Lane PROG - Unlock Catalog

Allowlist:
- `city_office_worker_dlc/src/game/resources.rs`
- `city_office_worker_dlc/src/game/systems/day_cycle.rs`
- `city_office_worker_dlc/src/game/save/mod.rs`
- `city_office_worker_dlc/src/game/systems/tests.rs`

Acceptance criteria:
1. Unlock table maps progression milestones to deterministic gameplay benefits.
2. Unlock state is persisted and migration-safe.
3. Deterministic tests verify unlock timing for fixed seeds/scripts.

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
