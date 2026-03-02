# Gold Double-Deduction Fix

## Status: DONE

## Problem
Tool and building upgrades deducted gold twice:
1. Direct mutation: `player_state.gold -= cost`
2. `GoldChangeEvent` send → processed by `apply_gold_changes`

## Changes Made

### `src/economy/blacksmith.rs` — `handle_upgrade_request`
- **Removed** `player_state.gold -= gold_cost;` (line 166)
- Kept `gold_writer.send(GoldChangeEvent { ... })` as the sole deduction path

### `src/economy/buildings.rs` — `handle_building_upgrade_request`
- **Removed** `player_state.gold = player_state.gold.saturating_sub(gold_cost);` (line 127)
- Kept `gold_writer.send(GoldChangeEvent { ... })` as the sole deduction path

## Notes
- Gold sufficiency guards (`player_state.gold < gold_cost`) were left intact — they correctly *read* gold without mutating it.
- `cargo check` passes for Rust source; the only build error is a missing system `alsa` library (environment issue, unrelated to this fix).
