# Gameplay Action Audit — Fishing & Mining
_Audited 2026-03-03_

---

## FISHING

| Action | Status | Evidence | Issue |
|--------|--------|----------|-------|
| **Equip fishing rod** — cycle via E/Q, FishingRod in TOOL_ORDER | ✅ YES | `src/player/mod.rs:172-177` (TOOL_ORDER), `src/player/tools.rs:14-30` (cycle logic) | — |
| **Walk to water tile → cast (F/Space key)** — ToolUseEvent(FishingRod) emitted | ✅ YES | `src/player/tools.rs:69` reads `equipped_tool`; emits `ToolUseEvent` | — |
| **Cast validates water tile** — `TileKind::Water` check on target tile | ✅ YES | `src/fishing/cast.rs:82-90` — shows toast "You need to cast into water!" if not water | Water validated via `world_map.map_def.get_tile()` |
| **Bite timer fires → player presses Space → minigame starts** | ✅ YES | `src/fishing/cast.rs:165-238` — `update_bite_timer` sets `BitePending`, `handle_bite_reaction_window` starts minigame on `player_input.tool_use` | Reaction window is 1.0 s |
| **Minigame: catch bar moves (Space = rise, release = fall)** | ✅ YES | `src/fishing/minigame.rs:102-146` — rise 60 u/s, fall 45 u/s; `player_input.fishing_reel` | — |
| **Progress fills when overlapping, drains when not** | ✅ YES | `src/fishing/minigame.rs:148-189` — fill 20%/s, drain 15%/s; TrapBobber slows drain | — |
| **On catch: fish added to inventory** | ✅ YES | `src/fishing/resolve.rs:54-55` — `ItemPickupEvent { item_id: valid_id, quantity: 1 }` | — |
| **On catch: encyclopedia updated** | ✅ YES | `src/fishing/resolve.rs:58-73` — `encyclopedia.record_catch()` called; "New fish!" toast on first catch | `src/fishing/mod.rs:133-152` |
| **On escape (reaction expired)** — resets properly, player can cast again | ✅ YES | `src/fishing/resolve.rs:120-148` — `end_fishing_escape` resets state to `Idle`, despawns bobber, transitions back to `Playing` | — |
| **On escape (Escape key in minigame)** — handled separately | ✅ YES | `src/fishing/minigame.rs:220-230` — `player_input.ui_cancel` calls `end_fishing_escape` | — |
| **Bait items detected from inventory** — priority order | ✅ YES | `src/fishing/cast.rs:43-55` — priority: `wild_bait > magnet_bait > worm_bait > bait` | — |
| **Bait affects bite speed** | ✅ YES | `src/fishing/cast.rs:27-39` — multipliers: `wild_bait`=0.70, `worm_bait`=0.75, `bait`=0.85, `magnet_bait`=1.00 (no speed bonus) | `magnet_bait` bonus is treasure chance, not speed — documented |
| **Bait consumed on cast** | ✅ YES | `src/fishing/cast.rs:143-153` — `inventory.try_remove(bait_id, 1)` + `ItemRemovedEvent` | — |
| **Legendary fish catchable** | ✅ YES | `src/fishing/legendaries.rs:18-50` — 5 legendaries; `try_roll_legendary` embedded in `select_fish` | — |
| **Legendary fish: location/season gated** | ✅ YES | `src/fishing/legendaries.rs:32-49` — each requires exact `MapId` + `Season` match before spawn roll | Spawn chances 1–2%; no time-of-day gate (all day, intentional) |
| **Legendary catch: special toast + SFX** | ✅ YES | `src/fishing/resolve.rs:77-89` — "LEGENDARY CATCH: X! Incredible!" toast (5 s) + `legendary_catch` SFX | — |
| **Fishing skill levels up every 10 catches** | ✅ YES | `src/fishing/skill.rs:83-120` — `update_fishing_skill` listens for `ItemPickupEvent` of fish; increments `total_catches`, fires `FishingLevelUpEvent` + toast | — |
| **Skill affects bite speed** | ✅ YES | `src/fishing/skill.rs:63-65` — `apply_bite_speed`: `base_wait * (1 - bite_speed_bonus)`; max -50% at level 10 | Applied in `cast.rs:130` |
| **Skill affects catch bar size** | ✅ YES | `src/fishing/skill.rs:68-70` — `apply_catch_zone`: `base_half * (1 + catch_zone_bonus)`; max +30% at level 10 | Applied in `FishingMinigameState::setup_with_skill` |
| **Wild bait double-catch (15% chance)** | ✅ YES | `src/fishing/minigame.rs:196-207` — `wild_bait_double_catch_roll()` rolls 15%, sends second `ItemPickupEvent` | — |
| **Perfect catch (90% overlap → quality upgrade toast)** | ✅ YES | `src/fishing/minigame.rs:176-189` — `is_perfect_catch()` checks `overlap_time_total / minigame_total_time ≥ 0.90` | Toast only; quality upgrade not written to item data |

---

## MINING

| Action | Status | Evidence | Issue |
|--------|--------|----------|-------|
| **Enter mine from mine entrance map** | ✅ YES | `src/mining/transitions.rs:17-53` — `handle_mine_entry` fires on `MapTransitionEvent { to_map: MapId::Mine }` | Plays music, sets `InMine(true)` |
| **Elevator UI on entry if floors unlocked** | ✅ YES | `src/mining/transitions.rs:38-45` — if `elevator_floors` non-empty, sets `ElevatorUiOpen(true)` and delays floor spawn | — |
| **Mine floor procedurally generated** | ✅ YES | `src/mining/floor_gen.rs:54-180` — 24×24 grid, seeded by floor number (deterministic), 40–60% rock coverage | — |
| **Break rocks with pickaxe** | ✅ YES | `src/mining/rock_breaking.rs:35-100` — `handle_rock_breaking` listens for `ToolUseEvent { tool: Pickaxe }`, damages rock at target tile | — |
| **Rocks drop ores (floor-tiered)** | ✅ YES | `src/mining/floor_gen.rs:183-230` — F1–5: stone/copper; F6–10: +iron; F11–15: +gold/quartz/amethyst; F16–20: +diamond/ruby/emerald | Loot set at generation time, not at break time |
| **Stamina consumed by pickaxe swings** | ✅ YES | `src/mining/rock_breaking.rs:19-29, 67-69` — 4.0 (Basic) → 2.0 (Iridium); `StaminaDrainEvent` sent per swing | — |
| **Enemies spawn on each floor** | ✅ YES | `src/mining/floor_gen.rs:157-172` — count scales with floor (`2 + floor/4`); slimes F1–4, bats F5–9, rock crabs F10+ | — |
| **Enemies move toward player** | ✅ YES | `src/mining/combat.rs:98-155` — `enemy_ai_movement`: greedy pathfinding, blocked by rocks, respects bounds | One tile per move-tick; not full pathfinding |
| **Enemies attack when adjacent** | ✅ YES | `src/mining/combat.rs:177-218` — `enemy_attack_player`: checks Manhattan distance ≤ 1, applies `monster.damage`, 0.5 s iframes | One attacker per frame (break after first hit) |
| **Player attacks enemy with pickaxe** | ✅ YES | `src/mining/combat.rs:32-95` — `handle_player_attack`: `ToolUseEvent { Pickaxe }` at enemy tile, damage 10–50 by tier | Stamina drained 2.0 per swing regardless of hit |
| **Enemy dies → drops loot** | ✅ YES | `src/mining/combat.rs:74-90` — despawns entity, `ItemPickupEvent` + `MonsterSlainEvent`; loot tables per enemy type | `enemy_loot()` uses random tables with typed drops |
| **Ladder hidden in rock 60% of time** | ✅ YES | `src/mining/floor_gen.rs:100-120` — `hide_ladder=true` (60% roll) buries ladder in a rock in upper half | — |
| **Ladder reveals when its rock is broken** | ✅ YES | `src/mining/rock_breaking.rs:105-124` — `check_ladder_reveal` matches broken tile position to ladder entity position | — |
| **Ladder reveals when all rocks cleared** | ✅ YES | `src/mining/rock_breaking.rs:113` — `rocks_remaining == 0` triggers reveal regardless of position | Fallback if rock was skipped |
| **Player steps on ladder → descends** | ✅ YES | `src/mining/ladder.rs:22-70` — requires `player_input.tool_use` AND `ladder.revealed` AND player tile == ladder tile | Floor cap: 20 |
| **Descending unlocks elevator every 5 floors** | ✅ YES | `src/mining/ladder.rs:57-62` — `if next_floor % 5 == 0`, pushes to `elevator_floors` | — |
| **Elevator: select floor on mine re-entry** | ✅ YES | `src/mining/ladder.rs:115-167` — `handle_elevator_selection`: number keys 1–8 map to floor 1 and elevator stops | Escape/cancel defaults to floor 1 |
| **Knockout at 0 HP → 10% gold penalty → respawn** | ✅ YES | `src/mining/combat.rs:221-265` — `check_player_knockout`: 10% gold loss, health=50%, transitions to `MapId::MineEntrance` | — |
| **Day end in mine → same penalty** | ✅ YES | `src/mining/transitions.rs:58-95` — `handle_day_end_in_mine`: 10% gold, health=50% | ⚠️ **Inconsistency**: knockout → `MineEntrance`; day-end → `PlayerHouse`. Day-end victim respawns at home, not mine entrance. |
| **Voluntary exit from mine** | ✅ YES | `src/mining/ladder.rs:72-105` — `handle_mine_exit` detects `MineExit` tile within distance ≤ 1; Space/Enter transitions to `MineEntrance` | — |
| **Mine entities cleaned up on exit** | ✅ YES | `src/mining/transitions.rs:97-109` — `cleanup_mine_on_exit` despawns all `MineFloorEntity` when `InMine` becomes false | — |

---

## Summary

**All 21 fishing actions and 19 mining actions are implemented.** No blocking gaps were found.

### Notable Issues

| Severity | Location | Description |
|----------|----------|-------------|
| ⚠️ Minor | `transitions.rs:90` vs `combat.rs:258` | Day-end knockout sends player to `PlayerHouse`; enemy knockout sends to `MineEntrance`. Inconsistent respawn location — day-end player wakes at home without feedback that they were in the mine. |
| ⚠️ Minor | `minigame.rs:176-189` | "Perfect catch" fires a "Quality upgraded!" toast but no item quality field exists on `ItemPickupEvent`. The quality upgrade is not applied to the item — toast is misleading. |
| 🔍 Note | `ladder.rs:30-34` | Ladder descent guard reads `tool_use_events.read().next()` to prevent dual-action. This is safe (per-reader cursors in Bevy), but consumes the event from the ladder system's own reader without processing it. Code has a comment explaining this. |
| 🔍 Note | `floor_gen.rs:183` | Rock loot is baked into the `RockBlueprint` at generation time (seeded RNG). Same floor always drops same ores in same positions — feature or limitation depending on intent. |
