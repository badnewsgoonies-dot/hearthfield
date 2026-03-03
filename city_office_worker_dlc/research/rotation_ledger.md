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

## Next Rotation (R4 Planned)

Scope:
1. Promote snapshot skeleton to durable save-slot file flow + migration stubs.
2. Add richer task progress/deadline failure semantics and deterministic coverage.
3. Produce social/progression parity decomposition packet for subsequent waves.

Entry conditions:
1. R0-R3 artifacts present and green.
2. No open regressions in day-state transitions, replay, or snapshot identity tests.
