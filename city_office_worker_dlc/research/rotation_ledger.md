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

## Next Rotation (R3 Planned)

Scope:
1. Task lifecycle hardening (`TaskAccepted`, `TaskProgressed`, `TaskCompleted`, `TaskFailed`) as first-class flow.
2. Completion/failure exclusivity + terminal disjointness invariants.
3. Save/load skeleton for task identity round-trip.

Entry conditions:
1. R0-R2 artifacts present and green.
2. No open regression in day-state transitions.
