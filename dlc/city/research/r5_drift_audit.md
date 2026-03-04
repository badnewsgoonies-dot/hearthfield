# R5 Drift/Regression Audit

Date: 2026-03-03

## Scope
Reviewed:
- `src/game/save/mod.rs`
- `src/game/resources.rs`
- `src/game/systems/tasks.rs`
- `src/game/systems/day_cycle.rs`
- `src/game/systems/task_board.rs`
- `src/game/systems/tests.rs`
- `src/game/mod.rs`

Validation run:
- `cargo check --manifest-path Cargo.toml` (pass)
- `cargo test --manifest-path Cargo.toml` (pass, 15/15)

## Confirmed Invariants

1. Deterministic simulation ordering is preserved in runtime wiring.
- Evidence: ordered `OfficeSimSet` chain and in-state `InDay` chain in `src/game/mod.rs:87-133`.

2. Task terminal-state exclusivity is enforced for normal flow.
- Evidence: `TaskBoard::complete_task` and `TaskBoard::fail_task` reject terminal conflicts in `src/game/resources.rs:157-202`.
- Coverage: `completed_task_cannot_fail_later_same_day` in `src/game/systems/tests.rs:604-613`.

3. Overdue deadline failures are single-fire and do not re-enter active pool.
- Evidence: deadline handler in `src/game/systems/tasks.rs:108-139`.
- Coverage: `overdue_tasks_fail_once_and_do_not_reenter_active_pool` in `src/game/systems/tests.rs:616-647`.

4. Mid-day snapshot identity invariants (TaskId preservation/no regeneration drift) remain intact.
- Evidence: snapshot capture/apply path in `src/game/save/mod.rs:179-355`.
- Coverage: `snapshot_roundtrip_preserves_task_ids_and_midday_load_no_regen` in `src/game/systems/tests.rs:650-724`.

5. Durable save slot and migration baseline still hold.
- Evidence: slot I/O + migration in `src/game/save/mod.rs:238-288` and `src/game/save/mod.rs:357-511`.
- Coverage: `save_slot_roundtrip_persists_snapshot_payload` (`src/game/systems/tests.rs:727-750`), `migrate_v0_snapshot_to_v1_preserves_core_fields_and_ids` (`src/game/systems/tests.rs:753-795`), `load_slot_request_restores_state_without_task_regeneration_drift` (`src/game/systems/tests.rs:798-927`).

6. End-of-day transition remains debounced.
- Evidence: `check_end_of_day` + `finalize_end_day_request` in `src/game/systems/day_cycle.rs:60-161`.
- Coverage: `end_day_request_advances_once_and_emits_summary_once` (`src/game/systems/tests.rs:447-480`) and `duplicate_end_day_requests_are_debounced` (`src/game/systems/tests.rs:497-522`).

## Potential Regressions (Drift Risk)

1. High: Loading an ended-day snapshot can leave simulation in a non-progressing `InDay` state.
- Why: `apply_snapshot` restores `clock.ended` directly (`src/game/save/mod.rs:321-324`), but load flow does not reconcile `OfficeGameState` (`src/game/save/mod.rs:483-510`).
- Impact path: In `InDay`, most handlers short-circuit when `clock.ended` is true (`src/game/systems/tasks.rs:45-47`, `src/game/systems/day_cycle.rs:66-68`), potentially creating a soft-lock until external state change.

2. High: Load flow does not restore day-level counters/context used for summary/economy.
- Why: Snapshot schema excludes `DayStats` and interruption backlog (`src/game/save/mod.rs:47-56`), and load only syncs mind stress/focus/reputation + worker energy (`src/game/save/mod.rs:416-422`).
- Drift effect: `build_day_outcome` and `EndOfDayEvent` consume `DayStats` (`src/game/systems/day_cycle.rs:15-31`, `src/game/systems/day_cycle.rs:107-125`), so post-load outcomes can diverge from loaded task/time state.

3. Medium: Active save slot ownership can drift from user intent.
- Why: Day-summary persistence always writes `config.active_slot` (`src/game/save/mod.rs:445-447`), but save/load requests do not update it (`src/game/save/mod.rs:457-511`).
- Risk: Player loads/saves to slot N, then day-summary autosave still targets default slot 0.

4. Medium: Task terminal disjointness is not hardened against malformed/corrupt snapshot payloads.
- Why: `TaskBoard::normalize` de-duplicates within each list but does not resolve overlap between `completed_today` and `failed_today` (`src/game/resources.rs:216-223`).
- Risk: malformed data can violate outcome accounting assumptions (`completed_tasks` and `failed_tasks` both count same ID).

5. Low-Medium: Default save directory path is CWD-sensitive.
- Why: default `save_dir` is relative (`city_office_worker_dlc/saves`) in `src/game/save/mod.rs:23`.
- Risk: running from different working directories changes persistence location, increasing support/debug drift.

## Missing Tests

1. Load of `day_ended=true` snapshot should transition to valid playable state (or DaySummary state) without soft-lock.
- Gap: no test currently loads a day-ended snapshot then verifies state/controls recovery.

2. Load should restore/normalize day-level counters and interruption backlog.
- Gap: `load_slot_request_restores_state_without_task_regeneration_drift` does not assert `DayStats` or `PlayerMindState.pending_interruptions` restoration (`src/game/systems/tests.rs:798-927`).

3. Autosave slot targeting semantics after explicit slot load/save.
- Gap: no test asserts whether `active_slot` follows latest user slot request before `persist_day_summary_snapshot`.

4. Malformed snapshot hygiene for terminal sets.
- Gap: no test for snapshot where the same `TaskId` appears in both `completed_today` and `failed_today`.

5. Save/load error-path behavior hardening.
- Gap: no explicit tests for missing file, unreadable file, unsupported `schema_version`, or invalid task kind/priority string with expected `last_io_error` behavior.

## Highest-Priority Fixes

1. Fix load-state reconciliation (P0).
- On load, if snapshot has `day_ended=true`, reconcile app state explicitly (e.g., enter `DaySummary` or perform safe rollover/transition path) instead of leaving `InDay` + `ended=true`.

2. Expand persistence schema for run-consistency (P0).
- Add `DayStats` and interruption backlog to snapshot + migration path; restore them during load to avoid post-load summary/economy drift.

3. Make slot ownership explicit (P1).
- Update `SaveSlotConfig.active_slot` on successful save/load requests, and test that day-summary persistence writes to intended slot.

4. Harden task terminal invariants on load (P1).
- Enforce cross-set disjointness (`completed_today` ∩ `failed_today` = ∅) during normalization or reject malformed snapshot.

5. Add regression tests for the above before further R5 scope (P1).
- Prioritize tests for day-ended load recovery and full post-load consistency (clock/inbox/task board/stats/mind/career).
