# Worker Report: fix-city-task-progress

## Files Modified
- `dlc/city/src/game/systems/tasks.rs` (line 24): formula change
- `dlc/city/src/game/systems/tests.rs` (line 335): test progress seed value

## What Was Implemented
Fixed the task progress formula (Bug B001):
- `progress_delta_for_task`: base factor `0.52 → 0.28`, clamp `[0.12, 1.0] → [0.08, 0.55]`
- Updated test `process_event_updates_energy_inbox_and_clock`: starting progress `0.5 → 0.75` (0.75 + 0.28 = 1.03 ≥ 1.0, task completes in 1 action as the test expects)

## Quantitative Targets Hit
- Low priority, max focus: `0.28 * 1.0 * 1.0 = 0.28` → ~4 actions to complete ✓
- Critical priority, decent focus (76/66): `0.28 * (76/66) * 0.62 ≈ 0.20` → ~5 actions ✓
- Single-action cap: 0.55 (no task completes in 1 action unless starting near completion) ✓

## Validation Results
- `cargo check`: PASS
- `cargo test`: PASS (40/40)
- `cargo clippy -- -D warnings`: PASS
