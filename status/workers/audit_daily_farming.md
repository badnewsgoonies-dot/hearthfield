# Gameplay Action Audit — Daily Loop & Farming
_Audited: 2026-03-03_

## DAILY LOOP

| # | Action | Status | Evidence | Issue |
|---|--------|--------|----------|-------|
| 1 | Wake up in house after loading a save | **YES** | `src/save/mod.rs:734-738` — `handle_load_request` sends `MapTransitionEvent` with saved `current_map` + `save_grid_x/y`; new-game spawn at `(8,8)` in `src/player/spawn.rs:6-7` | — |
| 2 | Walk out the front door of player house | **YES** | `src/world/maps.rs:700-706` — `MapTransition` at door rect `(7,15,2,1)` → `MapId::Farm (16,1)`; detected by `src/player/interaction.rs:118-135` | — |
| 3 | Walk to the farm | **YES** | `src/world/maps.rs:78-130` — Farm is 32×24 with walkable Grass base; no blocking tiles on open area | — |
| 4 | Walk to other maps (town, beach, forest, mine entrance) | **YES** | `src/player/interaction.rs:31-114` — edge transitions: Farm↔Town (south), Farm↔Forest (east), Farm↔Beach (west), Forest↔MineEntrance (north) | — |
| 5 | Enter buildings (general store, blacksmith, animal shop) | **YES** | `src/world/maps.rs:722-835` — each building has door `MapTransition` rect `(5,11,2,1)` firing back to Town spawn | — |
| 6 | Return home at end of day | **PARTIAL** | `src/player/interaction.rs:280-350` — only **pass-out** auto-returns player to bed `(12,4)` in PlayerHouse when stamina ≤ 0 at late hour | No manual "go home" shortcut; no forced return if player is still out at 2 AM with stamina remaining |
| 7 | Sleep in bed (advance to next day) | **YES** | `src/player/interact_dispatch.rs:118-136` — `InteractionKind::Bed` fires `DayEndEvent` when `calendar.hour >= 18` | — |
| 8 | Day-end processing fires (shipping bin sold, animals fed check, crop growth) | **YES** | `src/economy/shipping.rs:91-160`, `src/animals/day_end.rs:72-150`, `src/farming/events_handler.rs:85-130` — all three subscribe to `DayEndEvent` | — |

## FARMING

| # | Action | Status | Evidence | Issue |
|---|--------|--------|----------|-------|
| 9 | Equip hoe via keyboard | **PARTIAL** | `src/input/mod.rs:66-86` — digit keys 1-9 select hotbar slots; `src/player/tools.rs:6-32` cycles through `TOOL_ORDER` with Q/E | No single dedicated key to jump directly to Hoe; depends on hotbar slot assignment at runtime |
| 10 | Till a soil tile | **YES** | `src/farming/soil.rs:34` — Hoe use inserts `SoilState::Tilled` and calls `spawn_or_update_soil_entity()` | — |
| 11 | Plant a seed on tilled soil | **YES** | `src/farming/crops.rs:95-102` — `detect_seed_use()` checks `SoilState::Tilled/Watered`, fires `PlantSeedEvent`; `handle_plant_seed()` spawns crop at line 170-178 | — |
| 12 | Water the crop with watering can | **YES** | `src/farming/soil.rs:98-105` — `handle_watering_can_tool_use()` sets `SoilState::Watered` and `crop.watered_today = true` | — |
| 13 | Crop grows over multiple days | **YES** | `src/farming/events_handler.rs:122-127` + `src/farming/crops.rs:271-286` — `DayEndEvent` calls `advance_crop_growth()` advancing `days_in_stage` through growth stages | — |
| 14 | Harvest a mature crop (goes to inventory) | **YES** | `src/farming/harvest.rs:122-128` — `try_harvest_at()` checks `current_stage == growth_days.len()`, fires `ItemPickupEvent` with harvest item | — |
| 15 | Put crop in shipping bin | **YES** | `src/player/interact_dispatch.rs:62-77` — `InteractionKind::ShippingBin` fires `ShipItemEvent` when player holds an item and interacts with bin | — |
| 16 | Shipping bin pays gold at day end | **YES** | `src/economy/shipping.rs:91-141` — `process_shipping_bin_on_day_end()` totals bin value, fires `GoldChangeEvent` (line 138-140), clears bin (line 159) | — |

## TOOLS

| # | Action | Status | Evidence | Issue |
|---|--------|--------|----------|-------|
| 17 | Switch between tools (1-5 keys or scroll) | **YES** | `src/input/mod.rs:66-86` — digit keys set `input.tool_slot`; scroll (tool_next/tool_prev) cycles; `src/player/tools.rs:6-32` applies selection | — |
| 18 | Axe chops trees/stumps (drops wood) | **YES** | `src/world/objects.rs:226-227` — Tree/Stump require Axe; drop table lines 263-264 (Tree→4 wood+4 sap+1 hardwood; Stump→2 hardwood); `ItemPickupEvent` fired at line 480 | — |
| 19 | Pickaxe breaks farm rocks (drops stone) | **YES** | `src/world/objects.rs:230-231, 267-268` — Rock/LargeRock require Pickaxe; `src/mining/rock_breaking.rs:31-102` applies damage, fires `ItemPickupEvent` at line 86 | — |
| 20 | Scythe cuts grass/weeds | **YES** | `src/world/objects.rs:228, 266, 840-870` — Bush/Weed require Scythe; Bush drops 3 fiber+berry; Weed drops 1 fiber | — |
| 21 | Tool stamina cost deducted per use | **YES** | `src/player/mod.rs:160-169` — `stamina_cost()` returns per-tool cost (Axe=6.0, Pickaxe=6.0, Scythe=2.0); `src/player/tools.rs:70,103` fires `StaminaDrainEvent` | — |
| 22 | Stamina too low prevents tool use | **YES** | `src/player/tools.rs:72-78` — checks `player_state.stamina < cost`; plays error sound and returns early if insufficient | — |

## Summary

- **YES**: 19 / 22
- **PARTIAL**: 2 / 22 (actions 6 and 9)
- **NO**: 0 / 22

### PARTIAL details

**Action 6 – Return home:** Pass-out mechanic returns player to bed if stamina hits zero late at night, but there is no forced auto-return at a hard curfew (e.g. 2 AM) with stamina still remaining, and no manual fast-travel-home shortcut.

**Action 9 – Equip hoe:** Tools are equipped by hotbar slot number (1-9 keys) or Q/E cycling; the Hoe position depends on what the player has assigned. There is no fixed key that always selects the Hoe specifically.
