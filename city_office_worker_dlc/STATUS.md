# City Office Worker DLC - STATUS

Last updated: 2026-03-03  
Current rotation: R5 - Task/Economy Semantics + Parity Decomposition  
Rotation state: In progress (task/deadline semantics and drift hardening slices landed; broader economy/content parity still pending)

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
| G7 Durable Save Slots | Snapshot persistence survives filesystem roundtrip and load requests restore run state | `save_slot_roundtrip_persists_snapshot_payload` + `load_slot_request_restores_state_without_task_regeneration_drift` | PASS |
| G8 Migration Stub | Legacy v0 payloads migrate into v1 schema | `migrate_v0_snapshot_to_v1_preserves_core_fields_and_ids` | PASS |
| G9 Quality | `fmt/check/test/clippy -D warnings` all pass | Full gate run passed on 2026-03-03 after R5 drift-hardening slice | PASS |

## Current Snapshot

1. Source lines (`src/**/*.rs`): 3,585.
2. Test count (`cargo test -- --list`): 19.
3. Clippy strictness: PASS at `-D warnings` in DLC.

## Remaining Blockers Toward OES-v1

1. Domain breadth gap vs origin remains large (single `game` domain vs origin multi-domain tree).
2. Task lifecycle events are emitted but still need richer progression/deadline semantics and content-driven task generation.
3. Economy/progression, social arcs, and content breadth still trail origin parity targets.
4. Content scale and social/progression depth are still early-stage.

## R5 Focus (In Progress)

1. Expand task lifecycle with non-trivial progress deltas/deadline failure paths and explicit invariant tests.
2. Add richer economy/progression scaffolding tied to deterministic replay baselines.
3. Publish social/progression parity packet for downstream decomposition rotations.

## Process Guardrails (Active for R5)

1. Keep slices small (target `<=20` files and `~1,200` insertions per commit/PR).
2. Disallow `WIP` commits on integration branches.
3. Couple contract deltas with wiring updates and deterministic/headless tests in the same slice.

Evidence index:
- `research/parity_baseline_matrix.md`
- `research/r5_origin_commit_patterns.md`
- `research/rotation_ledger.md`
- `research/wave3_audit_checklist.md`
- `research/wave3_audit_report.md`
