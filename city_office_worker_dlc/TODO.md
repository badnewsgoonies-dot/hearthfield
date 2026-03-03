# City Office Worker DLC - Master TODO

This is the single long-horizon execution list to reach **OES-v1** (Origin-Equivalent State v1).

## OES-v1 Definition

- [ ] Architecture parity across core domains (`time`, `tasks`, `interruptions`, `economy`, `social`, `save`, `ui`, `world`).
- [ ] 200+ automated tests with deterministic multi-day replay and autoplay coverage.
- [ ] `cargo check`, `cargo test`, `cargo clippy --all-targets -- -D warnings` all pass.
- [ ] Persistence invariants hold (exact `TaskId` round-trip, no mid-day task regeneration).
- [ ] Rotation evidence and ADR trail are up to date.

## Rotation Checklist (Full List)

- [x] **R0**: Parity baseline matrix and OES-v1 target freeze.
- [x] **R1**: Systems module split (`src/game/systems/**`) and ownership boundaries.
- [x] **R2**: Deterministic foundation (`fixed_seed_three_day_replay_is_deterministic`, `five_day_seeded_autoplay_completes_without_panic`).
- [x] **R3**: Task lifecycle events + snapshot save skeleton + identity round-trip tests.

- [x] **R4**: Durable save slots + load flow + schema migration stubs.
- [ ] **R5**: Rich task progression/deadline semantics with full terminal-state invariants.
- [ ] **R6**: Economy/progression depth (salary curves, penalties, upgrades/perks).
- [ ] **R7**: Social domain decomposition (coworkers, manager arcs, affinity/progression hooks).
- [ ] **R8**: UI/HUD expansion for summary, task board, progression readability.
- [ ] **R9**: Content pack scaling (task templates, interruption scenarios, progression content).
- [ ] **R10**: World/navigation layer parity for office map transitions and interactables.
- [ ] **R11**: Multi-day endurance expansion (10/20/30-day deterministic and no-panic runs).
- [ ] **R12**: Balancing pass with deterministic replay baselines locked.
- [ ] **R13**: Performance/memory sanity and regression hardening.
- [ ] **R14**: Final parity audit packet and release-candidate gate.

## Process Guardrails (R5+)

- [x] Commit guardrails frozen from origin-pattern audit (`<=20` files, `~1,200` insertions target).
- [ ] Keep each merge as a vertical slice (contract/resource/event delta + wiring + deterministic/headless tests).
- [ ] Reject `WIP` commits on integration branches.
- [ ] Keep infra/build commits separate from gameplay/content commits.
- [ ] Close each rotation with a hardening checkpoint and deterministic test expansion.

## Gate Runbook (Every Rotation)

- [x] `cargo fmt --manifest-path city_office_worker_dlc/Cargo.toml --all`
- [x] `cargo check --manifest-path city_office_worker_dlc/Cargo.toml`
- [x] `cargo test --manifest-path city_office_worker_dlc/Cargo.toml`
- [x] `cargo clippy --manifest-path city_office_worker_dlc/Cargo.toml --all-targets -- -D warnings`

## Current Next-Up

- [x] Continue **R5** slice A: richer task progression deltas + deterministic assertions.
- [x] Continue **R5** slice B: deadline-breach failure path + terminal exclusivity coverage.
- [x] Continue **R5** slice C: economy/progression hook expansion with replay-safe tests.
- [ ] Continue **R5** slice D: publish social/progression parity decomposition packet for **R6** planning.
- [ ] Continue **R5** slice E: expand task/economy content templates on top of stabilized semantics.
