# Rotation Ledger

This log tracks long-horizon rolling rotations toward OES-v1.

## R0 - Parity Baseline (Completed 2026-03-03)

Goals:
1. Define measurable parity targets against origin game.
2. Capture baseline engineering metrics.

Delivered:
1. `research/parity_baseline_matrix.md` with line/test/gate deltas.
2. OES-v1 done-state criteria recorded and frozen.

Gate result: PASS.

## R1 - Systems Module Split (Completed 2026-03-03)

Goals:
1. Decompose monolithic `src/game/systems.rs` into owned lane modules.
2. Preserve behavior while reducing integration risk and merge pressure.

Delivered:
1. Modular systems package at `src/game/systems/`:
   - `bootstrap.rs`
   - `input.rs`
   - `tasks.rs`
   - `interruptions.rs`
   - `day_cycle.rs`
   - `visuals.rs`
   - `task_board.rs`
   - `tests.rs`
2. Re-export facade via `src/game/systems/mod.rs`.

Gate result: PASS (`cargo check`, `cargo test`, `cargo clippy -D warnings`).

## R2 - Deterministic Simulation Foundation (Completed 2026-03-03)

Goals:
1. Add fixed-seed replay validation.
2. Add seeded autoplay no-panic coverage.

Delivered:
1. Deterministic replay test: `fixed_seed_three_day_replay_is_deterministic`.
2. Seeded autoplay test: `five_day_seeded_autoplay_completes_without_panic`.
3. Test count raised from 6 to 8.

Gate result: PASS.

## R3 - Task Lifecycle and Save Skeleton (Completed 2026-03-03)

Goals:
1. Add explicit task lifecycle event surface (`TaskAccepted`, `TaskProgressed`, `TaskCompleted`, `TaskFailed`).
2. Enforce task terminal-state invariants in code/tests.
3. Land persistence skeleton with identity-preserving snapshot round-trip behavior.

Delivered:
1. Lifecycle events added in `src/game/events.rs` and wired in plugin.
2. `TaskBoard` lifecycle helpers added (`complete_task`, `fail_task`, disjoint terminal protections).
3. Day-end flow emits `TaskFailed` for unresolved tasks.
4. Snapshot module added at `src/game/save/mod.rs` with capture/serialize/deserialize/apply helpers.
5. DaySummary persistence hook added (`persist_day_summary_snapshot`).
6. Tests added:
   - `completed_task_cannot_fail_later_same_day`
   - `snapshot_roundtrip_preserves_task_ids_and_midday_load_no_regen`
7. Test count raised from 8 to 10.

Gate result: PASS (`cargo fmt`, `cargo check`, `cargo test`, `cargo clippy -D warnings`).

## R4 - Durable Save Slots and Migration Stub (Completed 2026-03-03)

Goals:
1. Promote snapshot skeleton to durable save-slot file persistence.
2. Add explicit load flow that restores day/task/worker identity without regeneration drift.
3. Add schema migration stub to support legacy payload forward-compatibility.

Delivered:
1. Save runtime resources/events added:
   - `SaveSlotConfig`
   - `SaveSlotRequest`
   - `LoadSlotRequest`
2. Save store expanded with `last_io_error` and `last_loaded_slot` tracking.
3. File-backed save/load helpers landed in `src/game/save/mod.rs`:
   - `write_snapshot_to_slot`
   - `read_snapshot_from_slot`
   - `migrate_snapshot_json` (`v0 -> v1`, `v1` passthrough)
4. Runtime handlers wired via plugin update chain:
   - `handle_save_slot_requests`
   - `handle_load_slot_requests`
5. New regression tests:
   - `save_slot_roundtrip_persists_snapshot_payload`
   - `migrate_v0_snapshot_to_v1_preserves_core_fields_and_ids`
   - `load_slot_request_restores_state_without_task_regeneration_drift`
6. Test count raised from 10 to 13.

Gate result: PASS (`cargo fmt`, `cargo check`, `cargo test`, `cargo clippy -D warnings`).

## R5 - Task/Economy Semantics + Process Hardening (In Progress 2026-03-03)

Scope:
1. Add richer task progress/deadline failure semantics and deterministic coverage.
2. Expand economy/progression depth while preserving deterministic replay expectations.
3. Produce social/progression parity decomposition packet for subsequent waves.

Entry conditions:
1. R0-R4 artifacts present and green.
2. No open regressions in day-state transitions, replay, or snapshot identity tests.

Orchestration adjustments adopted from `research/r5_origin_commit_patterns.md`:
1. Keep vertical slices small (target `<=20` files and `~1,200` insertions).
2. Block `WIP` commits on integration branches.
3. Require contract/resource/event changes to land with wiring and deterministic/headless tests in the same PR.
4. Keep infra/build changes isolated from gameplay/content slices.
5. Reserve a hardening checkpoint before the next feature burst.

Current status:
1. Task/deadline semantics, save-drift hardening, and economy/progression hooks are landed (progress deltas, deadline failure handling, load-state reconciliation, slot ownership semantics, terminal-set normalization hardening, streak/burnout salary modifiers, XP leveling, and auto-perk assignment).
2. Startup-first reliability hardening landed: `setup_scene` is idempotent for early-frame singleton entities (camera/worker/inbox), reducing first-seconds drift/soft-lock risk from duplicate spawns.
3. Gate checkpoint PASS on 2026-03-03 (`cargo fmt`, `cargo check`, `cargo test`, `cargo clippy -D warnings`) with 23 passing tests.
4. Remaining R5 work is parity decomposition packet publication and content-scale expansion on top of stabilized semantics.
