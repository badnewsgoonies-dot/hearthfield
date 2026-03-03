# City Office Worker DLC - STATUS

Last updated: 2026-03-03
Current wave: Wave 1 - Playable Core Day Loop
Wave state: In progress (Wave 0 skeleton exists and compiles)

## Gate Dashboard

| Gate | Wave 1 target | Current evidence | Status |
|---|---|---|---|
| G1 Compile | `cargo check --manifest-path city_office_worker_dlc/Cargo.toml` passes | Passed on 2026-03-03 | PASS |
| G2 Scope | 0 out-of-scope edits after each lane run | `city_office_worker_dlc/tools/scope_guard.sh` added; lane enforcement must be verified per task | PARTIAL |
| G3 Loop | One full day playable + deterministic 3-day headless simulation | Current prototype supports single-day completion logic in `src/game/systems.rs`; no deterministic 3-day test yet | PARTIAL |
| G4 Contract | 0 contract drift vs `CONTRACT.md` for Wave 1 types/events/states | Current code still uses Wave 0 names (`DayClock`, `InboxState`, `DayStats`) and has no `OfficeGameState` wiring | FAIL |
| G5 Stability | 0 panics in autoplay/headless runs | `cargo test` passes but runs 0 tests (no coverage yet) | BLOCKED |
| G6 Quality | Failing tests: 0, clippy warnings: 0 | Tests exist only as empty baseline; clippy not yet executed for DLC crate | BLOCKED |

## Active Blockers

1. Contract drift: runtime model is not yet aligned with `CONTRACT.md` naming and state flow.
2. No deterministic headless simulation harness for the Wave 1 gate.
3. Gameplay systems are concentrated in one file (`src/game/systems.rs`), increasing lane merge risk.

## Immediate Next Actions

1. Integrator lane: introduce `OfficeGameState` and register state transitions (`Boot/MainMenu/InDay/DaySummary/Paused`).
2. Time lane: move day clock logic to a dedicated module and enforce single `EndDayRequested`/`DayAdvanced` flow.
3. Tasks lane: implement contract-shaped `TaskBoard` lifecycle (`accepted -> progressed -> completed/failed`) with unique `TaskId` handling.
4. Economy lane: apply salary/reputation/stress only in `DaySummary`, then rollover to next day.
5. Test lane: add deterministic 3-day headless test and 5-day no-panic autoplay test.
6. Tooling lane: run `city_office_worker_dlc/tools/scope_guard.sh` per lane and record evidence in this file.
