# Adventure Systems Audit — Fishing & Mining
*Audited: 2026-03-03 | Auditor: gameplay-systems-agent*

Each action traced: **input → event → system → state change → feedback**

---

## Results Table

| # | Action | Status | Chain Traced | Issues Found |
|---|--------|--------|--------------|--------------|
| 1 | EQUIP ROD | PARTIAL | `PlayerState.tools[FishingRod]` → HUD reads current tool → displays name | No explicit equip action exists. Rod is always implicitly available via `ToolKind::FishingRod` in PlayerState. No UI for selecting rod tier or swapping rods. HUD displays tool name correctly once selected via hotbar, but the equip gesture (switching to rod) has no dedicated input path. |
| 2 | CAST LINE | YES | `PlayerInput.tool_use` → `ToolUseEvent(FishingRod)` → `cast.rs::handle_tool_use_for_fishing` → validates water tile → consumes bait → spawns `Bobber` entity → `FishingPhase::WaitingForBite` | Full chain intact. Water tile validation, bobber spawn, and bait consumption all wired. |
| 3 | BITE | YES | `update_bite_timer` countdown → `fish_select::select_fish()` (filters by location/season/time/weather) → `FishingPhase::BitePending` → `PlaySfxEvent("fish_bite")` → `render.rs::animate_bobber` switches to aggressive dip | `select_fish()` correctly prioritizes legendary check first, then filters normal pool by `FishLocation`, `Calendar.season`, time-of-day, and weather. |
| 4 | HOOK | YES | `handle_bite_reaction_window` detects `PlayerInput.fishing_reel` (Space) within 1.0s window → looks up fish difficulty → `minigame_state.setup_with_skill(rod_tier, tackle, skill)` → `GameState::Fishing` → `render.rs::spawn_minigame_ui` creates bar+zones | 4-system minigame chain (`update_fish_zone` → `update_catch_bar` → `update_progress` → `check_minigame_result`) all runs in `Fishing` state. |
| 5 | CATCH | PARTIAL | `check_minigame_result`: progress ≥ 100% → `resolve.rs::catch_fish()` → `ItemPickupEvent(fish)` → `FishEncyclopedia.record_catch` → `skill.rs::update_fishing_skill` increments on `ItemPickupEvent` | **Bug:** Perfect-catch quality upgrade fires a toast ("Quality upgraded!") but does **not** populate any `ItemQuality` field on the caught fish item — normal-quality fish lands in inventory. `FoodBuff::Fishing` active-buff check is absent from minigame progress math. |
| 6 | ESCAPE | YES | `check_minigame_result`: progress ≤ 0% (or Escape key) → `end_fishing_escape()` → 2 stamina drain → `FishingState` reset → `GameState::Playing` | Bobber despawned, stamina correctly cheaper than catch penalty (2 vs 4). Player can immediately re-cast. |
| 7 | LEGENDARY | PARTIAL | `fish_select::try_roll_legendary()` (1–2% per cast, gated by location+season) → returns legendary fish ID → difficulty 0.80–0.99 → minigame harder → `ToastEvent("Legendary catch!")` on resolve | **Gap:** No distinct legendary-specific reward path beyond the fish item itself landing in inventory. The `is_legendary()` flag in `resolve.rs` triggers a special toast but awards no unique bonus (no gold bonus, no special quality marker, no encyclopedia "legendary" badge). Special reward described in game spec is not implemented. |
| 8 | BAIT | YES | Bait detected at cast time (`wild_bait`, `worm_bait`, `magnet_bait`) → multiplier applied to bite timer (0.70–1.00×) → `ItemRemovedEvent` consumes bait → bait type stored in `FishingState.bait_id` → treasure odds modified in `resolve.rs` (+5% wild_bait, +15% magnet_bait) | Wild bait also rolls 15% double-catch on success. Full chain intact. |
| 9 | ENTER MINE | YES | Player walks to mine entrance tile → `MapTransitionEvent(to_map=Mine)` → `transitions.rs::handle_mine_entry` → `InMine = true` → if `elevator_floors` non-empty: `ElevatorUiOpen = true`; else: `FloorSpawnRequest.pending = true` → `spawning.rs::spawn_mine_floor` materialises floor 1 | Music transition (`PlayMusicEvent("mine_ambient")`) also fires. Full chain intact. |
| 10 | BREAK ROCKS | YES | `movement.rs::mine_player_action` → `ToolUseEvent(Pickaxe, target_tile)` → `rock_breaking.rs::handle_rock_breaking` → `MineRock.health -= pickaxe_damage` → on zero: `ItemPickupEvent(drops)` + despawn + `ActiveFloor.rocks_remaining--` + `StaminaDrainEvent` | Damage tiers correctly scaled (Basic=1 … Iridium=4). Stamina cost also scales inversely with tier. |
| 11 | FIND LADDER | YES | `rock_breaking.rs::check_ladder_reveal()` called after each rock break → sets `MineLadder.revealed = true` if rock contained ladder OR `rocks_remaining == 0` → `ladder.rs::handle_ladder_interaction` detects player on ladder tile + Space → `MineState.current_floor++` → `FloorSpawnRequest` triggers next floor | Ladder can be hidden (60% of floors) or exposed (40%); both paths correctly feed into the same traversal system. |
| 12 | FIGHT ENEMY | YES | `movement.rs::mine_player_action` → `ToolUseEvent(Pickaxe, adjacent_tile)` → `combat.rs::handle_player_attack` → `MineMonster.health -= player_damage` → on zero: `ItemPickupEvent(drops)` + `MonsterSlainEvent` + entity despawn | Player damage tiers: Basic=10 … Iridium=50. Enemy drops and quest event correctly fire on kill. |
| 13 | TAKE DAMAGE | YES | `combat.rs::enemy_ai_movement` pathfinds toward player each tick → `enemy_attack_player` checks adjacency + cooldown + iframes expired → `PlayerState.health -= monster.damage` → `PlayerIFrames` timer set (0.5s) → main HUD reads `PlayerState.health` and updates health bar | Three enemy types with distinct speeds (Slime=24, Bat=48, RockCrab=16) for varied combat feel. |
| 14 | KNOCKOUT | YES | `combat.rs::check_player_knockout`: `PlayerState.health ≤ 0` → `GoldChangeEvent(-10%)` → `PlayerState.health = 50%` → `MapTransitionEvent(MineEntrance)` → `InMine = false` → `cleanup_mine_on_exit` despawns all `MineFloorEntity` | Day-end-in-mine path in `transitions.rs` applies same penalty independently (correct). |
| 15 | ELEVATOR | YES | Every 5 floors: `MineState.elevator_floors` gains entry → on next mine entry (or after `handle_ladder_interaction` flag): `ElevatorUiOpen = true` → `ladder.rs::handle_elevator_selection` maps keys 1–8 → selected floor → `FloorSpawnRequest` | Elevator floors correctly accumulate across sessions via save system (MineState is persisted). |
| 16 | EXIT MINE | YES | Player on `MineExit` tile + Space → `ladder.rs::handle_mine_exit` → `InMine = false` → `MineState.current_floor = 0` → `MapTransitionEvent(MineEntrance)` → `cleanup_mine_on_exit` despawns all floor entities | DayEnd can also force exit through `transitions.rs::handle_day_end_in_mine` (same penalty path). |

---

## Summary

| System | YES | PARTIAL | NO |
|--------|-----|---------|-----|
| Fishing (8 actions) | 5 | 3 | 0 |
| Mining (8 actions) | 8 | 0 | 0 |
| **Total** | **13** | **3** | **0** |

---

## PARTIAL — Detailed Issue Breakdown

### Action 1 — EQUIP ROD (PARTIAL)
- **Root cause:** `ToolKind::FishingRod` is not an equippable item in inventory; it is a permanent field on `PlayerState`. There is no hotbar slot for it nor a key binding to "switch to rod."
- **Effect:** Players may be confused about how to activate fishing. The HUD does display the tool name but only when rod is already the "active tool," which has no input path to set.
- **Fix candidate:** Add rod equip to the tool-cycling input (`PlayerInput.tool_prev/next`) or add a dedicated hotbar key in `item_use.rs`.

### Action 5 — CATCH quality (PARTIAL)
- **Root cause:** `catch_fish()` in `resolve.rs` checks `FishingMinigameState.is_perfect_catch()` and fires a toast but calls `ItemPickupEvent` with a normal fish item regardless.
- **Effect:** Perfect-catch visual/audio feedback is misleading — no quality difference in inventory.
- **Fix candidate:** Add an `ItemQuality` field to `FishDef` (or `ItemPickupEvent`) and populate it from `is_perfect_catch()` before dispatching the event.

### Action 7 — LEGENDARY reward (PARTIAL)
- **Root cause:** `resolve.rs::catch_fish()` calls `is_legendary()` for a toast and SFX but takes no additional reward action.
- **Effect:** Legendary fish feel identical to rare fish at the inventory/reward layer, contradicting the game spec's implied special status.
- **Fix candidate:** On `is_legendary()` match in resolve: award a gold bonus, mark item as legendary-quality, and add an encyclopedia "legendary" badge flag.
