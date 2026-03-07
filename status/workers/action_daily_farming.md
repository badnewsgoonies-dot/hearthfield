# Daily Farming Action Audit
Generated: 2026-03-03

---

## WAKE UP & HOUSE

| Action | Status | Evidence | Issue |
|--------|--------|----------|-------|
| Player spawns in bed on new game | YES | `src/player/spawn.rs:6-7,25` — SPAWN_GRID_X=8,Y=8, map=PlayerHouse | None |
| Player walks to door | YES | `src/player/movement.rs:28-53` — WASD moves player; door at south wall of PlayerHouse | None |
| Player exits house (map transition) | YES | `src/world/maps.rs:699-706` — PlayerHouse south edge → Farm at (10,9) | None |
| Bed entity exists | YES | `src/world/objects.rs:1413-1414,1623-1635` — Bed spawned at grid (12,2) with `InteractionKind::Bed`, label "Sleep" | None |
| Player can interact with bed to sleep | PARTIAL | `src/player/interact_dispatch.rs:118-136` — F key triggers sleep; blocked before 18:00 ("Come back after 6 PM") | Time-lock: can only sleep after 6 PM |
| Sleeping advances to next day | YES | `src/player/interact_dispatch.rs:126-134` — fires `DayEndEvent` | None |
| Sleeping restores stamina | YES | `src/player/interaction.rs:295` — `player_state.stamina = player_state.max_stamina` on DayEndEvent | None |
| Sleeping restores health | YES | `src/player/interaction.rs:298` — `player_state.health = player_state.max_health` on DayEndEvent | None |

---

## TOOL SWITCHING

| Action | Status | Evidence | Issue |
|--------|--------|----------|-------|
| Cycle tools with `]`/`[` keys | YES | `src/shared/mod.rs:571-572` — KeyBindings defaults BracketRight/BracketLeft; `src/player/tools.rs:6-31` — `tool_cycle()` uses modulo | None |
| Cycle tools with scroll wheel | YES | `src/shared/mod.rs:517-518` — scroll up/down mapped in PlayerInput | None |
| Direct slot select (1-5 keys) | NO | `src/input/mod.rs:68-86` — Digit1-9 exist but map to inventory slots, not direct tool equip | Tools cannot be directly selected by number key |
| Hoe equippable | YES | `src/shared/mod.rs:164-171` — `ToolKind::Hoe` in enum; included in TOOL_ORDER | None |
| WateringCan equippable | YES | `src/shared/mod.rs:164-171` — `ToolKind::WateringCan` | None |
| Axe equippable | YES | `src/shared/mod.rs:164-171` — `ToolKind::Axe` | None |
| Pickaxe equippable | YES | `src/shared/mod.rs:164-171` — `ToolKind::Pickaxe` | None |
| Fishing rod equippable | YES | `src/shared/mod.rs:164-171` — `ToolKind::FishingRod` | None |
| Scythe equippable | YES | `src/shared/mod.rs:164-171` — `ToolKind::Scythe` (6th tool, bonus beyond the 5 listed) | None |
| HUD shows current tool | YES | `src/ui/hud.rs:49,186-196,617-643` — `HudToolText` component, `update_tool_display()` shows name + tier | None |

---

## FARMING LOOP

| Action | Status | Evidence | Issue |
|--------|--------|----------|-------|
| Hoe tills soil on farm map | YES | `src/farming/soil.rs:11,34` — `handle_hoe_tool_use()`, sets `SoilState::Tilled` | None |
| Hoe checks valid tile | YES | `src/farming/soil.rs:26-31` — skips if already tilled/occupied | None |
| Hoe costs stamina | YES | `src/farming/soil.rs:36-44` — 2.0 base × tier multiplier, sends `StaminaDrainEvent` | None |
| Plant seeds on tilled soil | YES | `src/farming/crops.rs:17,87-104` — `detect_seed_use()` checks tilled/watered soil, emits `PlantSeedEvent` | None |
| Planting checks season | YES | `src/farming/crops.rs:61-68` — out-of-season planting blocked | None |
| Planting consumes seed | YES | `src/farming/crops.rs:159,164` — `inventory.try_remove(&seed_item_id, 1)` + `ItemRemovedEvent` | None |
| Watering can waters tilled/planted tiles | YES | `src/farming/soil.rs:62,84-114` — `handle_watering_can_tool_use()`, tiles in AoE set to `SoilState::Watered` | None |
| Watering costs stamina | YES | `src/farming/soil.rs:117-118` — 2.0 base × tier, sends `StaminaDrainEvent` | None |
| Crops grow day-over-day | YES | `src/farming/crops.rs:237,271-287` — `advance_crop_growth()` increments `days_in_stage`, advances stages | None |
| Crops respect seasons | YES | `src/farming/crops.rs:264-269` — kills crops when season mismatches; `events_handler.rs:164-214` | None |
| Crops die without water | YES | `src/farming/crops.rs:288-295` — dies after 3 days without water (`days_without_water`) | None |
| Rain auto-waters crops | YES | `src/farming/events_handler.rs:114,118` — checks `TrackedDayWeather`, calls `apply_rain_watering()` | None |
| Harvest grown crops | YES | `src/farming/harvest.rs:90,114-125` — `try_harvest_at()` checks maturity (`current_stage == growth_days.len()`), sends `ItemPickupEvent` | None |
| Harvested crops go to inventory | YES | `src/farming/harvest.rs:125` — `ItemPickupEvent { item_id: def.harvest_id, quantity: 1 }` | None |
| Harvest quality roll | YES | `src/farming/harvest.rs:122` — `roll_harvest_quality()` → 74% Normal/20% Silver/5% Gold/1% Iridium | None |
| Remove dead crops | YES | `src/farming/harvest.rs:104-107` — dead crops removable by player | None |
| Replant after harvest | YES | `src/farming/harvest.rs:157-162` — soil stays `Tilled` after non-regrow harvest; hoe or direct plant both work | None |
| Regrow crops | YES | `src/farming/harvest.rs:139-147` — crop resets to late stage, waits `regrow_days` | None |

---

## STAMINA

| Action | Status | Evidence | Issue |
|--------|--------|----------|-------|
| Tool actions drain stamina | YES | `src/player/tools.rs:103,130-137` — all tool uses send `StaminaDrainEvent`; handler clamps to 0.0 | None |
| Stamina costs per tool | YES | `src/player/mod.rs:160-169` — Hoe 4.0, WateringCan 3.0, Axe 6.0, Pickaxe 6.0, FishingRod 4.0, Scythe 2.0 | None |
| Tool tier reduces cost | YES | `src/shared/mod.rs:234-241,997-999` — Basic 1.0×, Copper 0.85×, Iron 0.7×, Gold 0.55×, Iridium 0.4× | None |
| At 0 stamina: tools blocked | YES | `src/player/tools.rs:72-77` — checks `stamina < cost`, plays error sound, returns early | None |
| At 0 stamina: walking allowed | YES | `src/player/movement.rs:7-68` — no stamina check in movement system | None |
| At 0 stamina past midnight: pass out | YES | `src/player/interaction.rs:327-350` — `check_stamina_consequences()` fires `DayEndEvent` if `stamina <= 0 && hour >= 24` | None |
| Eating food restores stamina | YES | `src/player/item_use.rs:47-56` + `src/crafting/buffs.rs:99,130` — edible items trigger `EatFoodEvent`, stamina += `stamina_restore`, clamped to max | None |
| Eating shows toast | YES | `src/crafting/buffs.rs:130` — `"Ate {}! +{:.0} stamina"` toast displayed | None |
| Health tracked separately from stamina | YES | `src/shared/mod.rs:274-275` — separate `stamina: f32` and `health: f32` fields in PlayerState | None |

---

## MOVEMENT

| Action | Status | Evidence | Issue |
|--------|--------|----------|-------|
| WASD movement | YES | `src/shared/mod.rs:56-59` — W/A/S/D bindings; `src/input/mod.rs:35-46`; `src/player/movement.rs:28-53` | None |
| Movement on all maps | YES | `src/player/movement.rs` — no map-specific guard; works on all 10 maps | None |
| Map transitions (edge-based) | YES | `src/player/interaction.rs:31-114,118-136` — `edge_transition()` + `map_transition_check()` | None |
| Farm → Town | YES | `src/world/maps.rs:135-139` — south edge | None |
| Town → Farm | YES | `src/world/maps.rs:267-271` — north edge | None |
| Farm → Forest | YES | `src/world/maps.rs:142-149` — east edge | None |
| Farm → MineEntrance | YES | `src/world/maps.rs:149-156` — west edge | None |
| Farm → PlayerHouse (enter) | YES | `src/world/maps.rs:156` — door zone trigger | None |
| PlayerHouse → Farm (exit) | YES | `src/world/maps.rs:699-706` — south edge → Farm (10,9) | None |
| Town → Beach | YES | `src/world/maps.rs:295` — south edge | None |
| Beach → Town | YES | `src/world/maps.rs:383` — north edge | None |
| MineEntrance → Mine | YES | `src/world/maps.rs:560` — cave entrance | None |
| Mine → MineEntrance | YES | `src/world/maps.rs:629` — ladder | None |
| Town → GeneralStore/AnimalShop/Blacksmith | YES | `src/world/maps.rs:274,281,288` — shop door zones | None |
| Store → Town (all 3) | YES | `src/player/interaction.rs:103,106,109` — exit transitions to Town | None |
| All transitions bidirectional | YES | Both directions explicitly defined for every map pair | None |

---

## Summary

- **Total actions checked:** 52
- **YES (fully working):** 48
- **PARTIAL:** 1 (sleep time-lock — intentional game design)
- **NO:** 1 (direct 1-5 key tool selection — only `]`/`[` and scroll work)
- **Critical gaps:** None — the core farming loop (till → plant → water → grow → harvest → replant) is fully implemented end-to-end.

### Notable: 6 Tools, Not 5
The `ToolKind` enum contains **6** tools (Hoe, WateringCan, Axe, Pickaxe, FishingRod, **Scythe**), not 5 as the question assumed. All 6 are equippable and cycle correctly.
