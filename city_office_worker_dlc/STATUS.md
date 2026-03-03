# City Office Worker DLC - STATUS

Last updated: 2026-03-03  
Current rotation: R6 - Social/Progression Expansion  
Rotation state: In progress (R5 complete; R6 social-state/content/unlock decomposition queued)

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
| G9 Quality | `fmt/check/test/clippy -D warnings` all pass | Full gate run passed on 2026-03-03 after R5 economy/startup slice | PASS |
| G10 First-Seconds Stability | Startup scene remains idempotent, preventing duplicate singleton entities in early frames | `setup_scene_is_idempotent_for_first_seconds_entities` | PASS |
| G11 Content Pack Scaling | Seeded task board now rotates multi-kind/multi-priority templates with day-scaling reward/focus curves | `seeded_task_board_content_pack_has_kind_and_priority_variety` + `seeded_task_board_scales_task_economy_with_day_progression` | PASS |

## Current Snapshot

1. Source lines (`src/**/*.rs`): 4,256.
2. Test count (`cargo test -- --list`): 25.
3. Clippy strictness: PASS at `-D warnings` in DLC.

## Remaining Blockers Toward OES-v1

1. Domain breadth gap vs origin remains large (single `game` domain vs origin multi-domain tree).
2. Social arcs and progression breadth still trail origin parity targets.
3. Content scale beyond task templates (events/dialogue/scenario branches) remains early-stage.
4. World/navigation parity is still early-stage.

## R6 Focus (In Progress)

1. Land persistent coworker/manager relationship state and deterministic social scenario packs.
2. Add explicit progression unlock catalog with save/load and replay invariants.
3. Expand endurance and balancing coverage while preserving deterministic replay baselines.

## Process Guardrails (Active for R5)

1. Keep slices small (target `<=20` files and `~1,200` insertions per commit/PR).
2. Disallow `WIP` commits on integration branches.
3. Couple contract deltas with wiring updates and deterministic/headless tests in the same slice.

Evidence index:
- `research/parity_baseline_matrix.md`
- `research/r5_origin_commit_patterns.md`
- `research/r5_social_progression_parity_packet.md`
- `research/rotation_ledger.md`
- `research/wave3_audit_checklist.md`
- `research/wave3_audit_report.md`
