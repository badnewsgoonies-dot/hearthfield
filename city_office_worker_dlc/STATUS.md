# City Office Worker DLC - STATUS

Last updated: 2026-03-03  
Current rotation: R3 - Lifecycle and Save Skeleton  
Rotation state: Completed (task lifecycle events + snapshot round-trip invariants)

## OES-v1 Target

OES-v1 (Origin-Equivalent State v1) means architecture parity, 200+ tests, deterministic replay/autoplay coverage, persistence invariants, and green quality gates (`check`/`test`/`clippy -D warnings`).

## Rotation Gate Dashboard

| Gate | Rotation target | Current evidence | Status |
|---|---|---|---|
| G0 Baseline | Measurable origin-vs-DLC parity profile frozen | `research/parity_baseline_matrix.md` | PASS |
| G1 Module Topology | Monolithic systems split into lane modules | `src/game/systems/{bootstrap,input,tasks,interruptions,day_cycle,visuals,task_board}.rs` | PASS |
| G2 Event Backbone | `EndDayRequested -> DayAdvanced { new_day_index }` remains deterministic | Day transition tests remain green | PASS |
| G3 Determinism Replay | Fixed-seed 3-day replay stable across runs | `fixed_seed_three_day_replay_is_deterministic` | PASS |
| G4 Seeded Endurance | 5-day seeded autoplay completes without panic | `five_day_seeded_autoplay_completes_without_panic` | PASS |
| G5 Task Lifecycle | Completed tasks cannot fail later; terminal sets stay deterministic | `completed_task_cannot_fail_later_same_day` + `TaskBoard` lifecycle helpers | PASS |
| G6 Save Skeleton | Snapshot serialize/deserialize/apply preserves `TaskId` identities without regeneration | `snapshot_roundtrip_preserves_task_ids_and_midday_load_no_regen` + `game/save/mod.rs` | PASS |
| G7 Quality | `fmt/check/test/clippy -D warnings` all pass | All commands passed locally on 2026-03-03 | PASS |

## Current Snapshot

1. Source lines (`src/**/*.rs`): 2,413.
2. Test count (`cargo test -- --list`): 10.
3. Clippy strictness: PASS at `-D warnings` in DLC.

## Remaining Blockers Toward OES-v1

1. Domain breadth gap vs origin remains large (single `game` domain vs origin multi-domain tree).
2. Persistence is currently in-memory snapshot skeleton only (no durable slot/filesystem flow or migration chain).
3. Task lifecycle events are emitted but still need richer progression/deadline semantics and content-driven task generation.
4. Content scale and social/progression depth are still early-stage.

## Next Rotation (R4)

1. Promote save skeleton to durable slot-based persistence (`save slot`, `load slot`, version migration stubs).
2. Expand task lifecycle with non-trivial progress deltas/deadline failure paths and explicit invariant tests.
3. Add parity packet for social/progression domain decomposition.

Evidence index:
- `research/parity_baseline_matrix.md`
- `research/rotation_ledger.md`
- `research/wave3_audit_checklist.md`
- `research/wave3_audit_report.md`
