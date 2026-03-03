# City Office Worker DLC - STATUS

Last updated: 2026-03-03  
Current rotation: R2 - Deterministic Foundation  
Rotation state: Completed (R0 baseline + R1 module split + R2 deterministic tests)

## OES-v1 Target

OES-v1 (Origin-Equivalent State v1) means architecture parity, 200+ tests, deterministic replay/autoplay coverage, and green quality gates (`check`/`test`/`clippy -D warnings`).

## Rotation Gate Dashboard

| Gate | Rotation target | Current evidence | Status |
|---|---|---|---|
| G0 Baseline | Measurable origin-vs-DLC parity profile frozen | `research/parity_baseline_matrix.md` | PASS |
| G1 Module Topology | Monolithic systems split into lane modules | `src/game/systems/{bootstrap,input,tasks,interruptions,day_cycle,visuals,task_board}.rs` | PASS |
| G2 Event Backbone | `EndDayRequested -> DayAdvanced { new_day_index }` remains deterministic | Existing day-transition tests remain green | PASS |
| G3 Determinism Replay | Fixed-seed 3-day replay stable across runs | `fixed_seed_three_day_replay_is_deterministic` | PASS |
| G4 Seeded Endurance | 5-day seeded autoplay completes without panic | `five_day_seeded_autoplay_completes_without_panic` | PASS |
| G5 Quality | `fmt/check/test/clippy -D warnings` all pass | All commands passed locally on 2026-03-03 | PASS |

## Current Snapshot

1. Source lines (`src/**/*.rs`): 1,978.
2. Test count (`cargo test -- --list`): 8.
3. Clippy strictness: PASS at `-D warnings` in DLC.

## Remaining Blockers Toward OES-v1

1. Domain breadth gap vs origin remains large (single `game` domain vs origin multi-domain tree).
2. Persistence contract is still pending (`OfficeClock`/`TaskBoard` round-trip and mid-day load invariants).
3. Task lifecycle invariants are only partially expressed in tests.
4. Content scale and social/progression depth are still early-stage.

## Next Rotation (R3)

1. Task lifecycle event chain and invariant hardening.
2. Save/load skeleton for task identity round-trip.
3. Expand deterministic suite around completion/failure exclusivity.

Evidence index:
- `research/parity_baseline_matrix.md`
- `research/rotation_ledger.md`
- `research/wave3_audit_checklist.md`
- `research/wave3_audit_report.md`
