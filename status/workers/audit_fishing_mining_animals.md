# Gameplay Action Audit — Fishing / Mining / Animals

Audited: 2026-03-03  
Scope: `src/fishing/`, `src/mining/`, `src/animals/`, `src/data/`, `src/shared/mod.rs`

---

## Fishing

| # | Action | Status | Evidence | Issue |
|---|--------|--------|----------|-------|
| 1 | Equip fishing rod | **YES** | `src/player/tools.rs:6-31` — `tool_cycle()` cycles through `TOOL_ORDER`; `FishingRod` inserted into `player_state.tools` at startup (`shared/mod.rs:295`). | — |
| 2 | Cast into water tile (and ONLY water) | **YES** | `src/fishing/cast.rs:79-89` — `handle_tool_use_for_fishing` checks `world_map.map_def.get_tile(x,y) == TileKind::Water`; non-water sends `ToastEvent("You need to cast into water!")` and `continue`. | — |
| 3 | Wait for bite (bobber animation) | **YES** | `cast.rs:154-165` — bobber `Sprite` spawned at water tile; `update_bite_timer` (`cast.rs:170-207`) counts down 2–8 s (modulated by bait/skill). Bobber entity carries a `bob_timer` for visual dip. | — |
| 4 | React to bite within time window (Space) | **YES** | `cast.rs:213-246` — `handle_bite_reaction_window` checks `player_input.tool_use` while `FishingPhase::BitePending`; `REACTION_WINDOW = 1.0 s`. Missed window calls `end_fishing_escape`. | — |
| 5 | Minigame plays (progress bar, fish zone, catch bar) | **YES** | `src/fishing/minigame.rs` — `update_fish_zone`, `update_catch_bar`, `update_progress` run each frame in `GameState::Fishing`; full vertical bar with `CATCH_RISE_SPEED=60`, `CATCH_FALL_SPEED=45`, `PROGRESS_FILL_RATE=20`, `PROGRESS_DRAIN_RATE=15`. | — |
| 6 | Catch fish (goes to inventory + encyclopedia) | **YES** | `src/fishing/resolve.rs:catch_fish` — sends `ItemPickupEvent` for the fish, then calls `encyclopedia.record_catch()` (`fishing/mod.rs:133`). New species triggers `"New fish: X!"` toast. | — |
| 7 | Legendary fish can appear (conditions) | **YES** | `src/fishing/legendaries.rs:LEGENDARY_FISH` — 5 legendaries each gated by `(MapId, Season, spawn_chance)`. `try_roll_legendary` checked first in `fish_select.rs:55-59` before the normal pool. Hint toast `"Something legendary is biting..."` fires (`cast.rs:191-199`). | — |
| 8 | Treasure roll on catch | **YES** | `src/fishing/resolve.rs:91-103` calls `check_and_grant_treasure(treasure_chance, ...)`. Base 5 %; magnet_bait → 20 %; wild_bait → 10 %. Loot table in `treasure.rs`: ore (60 %), gem (20 %), artifact (15 %), rare (5 %) + 50–200 g. | — |
| 9 | Fishing skill levels up (every 10 catches) | **YES** | `src/fishing/skill.rs:FishingSkill::CATCHES_PER_LEVEL = 10`. `update_fishing_skill` watches `ItemPickupEvent`, cross-checks against `FishRegistry`, increments `total_catches`, calls `recalculate()`, fires level-up toast. | — |
| 10 | Fish encyclopedia tracks catches | **YES** | `src/fishing/mod.rs:125-154` — `FishEncyclopedia.record_catch()` stores `CaughtFishEntry { fish_id, times_caught, first_caught_day, first_caught_season }`. Called inside `catch_fish` (`resolve.rs:56`). | — |

---

## Mining

| # | Action | Status | Evidence | Issue |
|---|--------|--------|----------|-------|
| 11 | Enter mine from mine entrance map | **YES** | `src/mining/transitions.rs:handle_mine_entry` — listens for `MapTransitionEvent { to_map: MapId::Mine }`; sets `InMine(true)`, plays music, either shows elevator UI or starts floor 1. | — |
| 12 | Break rocks with pickaxe (drops ores) | **YES** | `src/mining/rock_breaking.rs:handle_rock_breaking` — listens for `ToolUseEvent { tool: Pickaxe }`, matches tile position, decrements `rock.health`, on 0 despawns rock and sends `ItemPickupEvent`. Stamina drained per tier. | — |
| 13 | Ore types match floor depth bands | **YES** | `src/mining/floor_gen.rs:184-227` `rock_drop(floor)`: F1-5 → stone/copper; F6-10 → stone/copper/iron; F11-15 → stone/iron/gold/quartz/amethyst; F16-20 → stone/gold/diamond/ruby/emerald. | — |
| 14 | Enemies spawn and move toward player | **YES** | `floor_gen.rs:241-258` `pick_enemy_kind`: F<5 slimes only; F5-10 slimes+bats; F10+ slimes/bats/rock-crabs. `combat.rs:enemy_ai_movement` runs greedy pathfinding toward `active_floor.player_grid_x/y` on a per-enemy tick timer. | — |
| 15 | Player attacks enemy with pickaxe (deals damage) | **YES** | `src/mining/combat.rs:handle_player_attack` — reads `ToolUseEvent { tool: Pickaxe }`, finds enemy at target tile, subtracts `player_attack_damage(tier)` (10→50 by tier). | — |
| 16 | Enemy dies and drops loot | **YES** | `combat.rs:enemy_loot()` — slime drops slime/jelly/copper/sap/stone; bat drops bat_wing/iron/copper/stone; rock_crab drops crab_shell/gold/iron/stone. `MonsterSlainEvent` also fired for quests. | — |
| 17 | Find and reveal hidden ladder | **YES** | `floor_gen.rs:102-118` — 60 % chance ladder is hidden inside a random rock (same grid pos). `rock_breaking.rs:check_ladder_reveal` reveals ladder when `rocks_remaining == 0` **or** when the broken tile matches the ladder's grid position. | — |
| 18 | Descend to next floor via ladder | **YES** | `src/mining/ladder.rs:handle_ladder_interaction` — player stands on revealed ladder, presses Space, `mine_state.current_floor += 1` (capped at 20), `FloorSpawnRequest` triggers `spawn_mine_floor`. | — |
| 19 | Elevator unlocks every 5 floors | **YES** | `ladder.rs:54-57` — `if next_floor % 5 == 0 && !mine_state.elevator_floors.contains(&next_floor)` pushes floor to `elevator_floors`. | — |
| 20 | Use elevator on re-entry | **YES** | `transitions.rs:handle_mine_entry:40-46` — if `elevator_floors` non-empty, sets `ElevatorUiOpen(true)`. `ladder.rs:handle_elevator_selection` maps number keys to floor choices. | — |
| 21 | Player knockout at 0 HP (gold penalty, return to entrance) | **YES** | `combat.rs:check_player_knockout` — `health <= 0` → 10 % gold loss, health restored to 50 %, `in_mine=false`, `MapTransitionEvent { to_map: MapId::MineEntrance, to_x:7, to_y:4 }`. | — |
| 22 | Day end while in mine (pass-out penalty) | **YES** | `transitions.rs:handle_day_end_in_mine` — `DayEndEvent` while `in_mine.0` → 10 % gold loss, health 50 %, transition to `MapId::PlayerHouse` (not MineEntrance). | Knock-out sends to `MineEntrance`; day-end pass-out sends to `PlayerHouse`. Behaviour differs by ~1 tile start point — likely intentional but worth a QA note. |

---

## Animals

| # | Action | Status | Evidence | Issue |
|---|--------|--------|----------|-------|
| 23 | Buy an animal from animal shop (all 10 kinds) | **YES** | `src/data/shops.rs:165-263` — all 10 SKUs listed: chicken/cow/sheep/goat/duck/rabbit/pig/horse/cat/dog. `src/animals/spawning.rs:handle_animal_purchase` maps `ShopTransactionEvent` item IDs via `item_to_animal()` (supports both `"chicken"` and `"animal_chicken"` forms). | — |
| 24 | Animal spawns in correct pen (coop/barn/roamer) | **YES** | `spawning.rs:pen_bounds_for()` — Chicken/Duck/Rabbit → coop yard `(-96,-192)→(96,-96)`; Cow/Sheep/Goat/Pig → barn pen `(-192,-192)→(-32,-64)`; Horse/Cat/Dog → wide farm `(-256,-256)→(256,256)`. | — |
| 25 | Feed animals via feed trough (with proximity check) | **YES** | `src/animals/feeding.rs:handle_feed_trough_interact` — checks Manhattan distance ≤ 2 from trough (`grid_x/y`), then listens for `ItemRemovedEvent { item_id: "hay" }`. Only Chicken/Cow/Sheep/Goat/Duck/Rabbit/Pig fed; Cat/Dog/Horse skipped. | — |
| 26 | Pet an animal (happiness increase) | **YES** | `src/animals/interaction.rs:handle_animal_interact` — 32 px range, `player_input.tool_use`, sets `petted_today=true`, `happiness.saturating_add(5)`. Unique per-animal sound text ("Bawk!", "Moo~", etc.). | — |
| 27 | Collect animal products (eggs, milk, wool) | **YES** | `src/animals/products.rs:handle_product_collection` — 7 product types: Chicken→egg, Cow→milk, Sheep→wool, Goat→goat_milk, Duck→duck_egg, Rabbit→rabbit_foot, Pig→truffle. Sends `ItemPickupEvent` + `AnimalProductEvent`. | — |
| 28 | Animal happiness affects product quality | **YES** | `src/animals/day_end.rs:quality_from_happiness()` — ≥230 → Iridium; ≥200 → Gold; ≥128 → Silver; <128 → Normal. Quality written as `PendingProductQuality` component, read on collection. | — |
| 29 | Unfed animals lose happiness over time | **YES** | `day_end.rs:handle_day_end_for_animals` — unfed → `happiness.saturating_sub(12)`. After 3 consecutive unfed days production blocked and toast fires. Block lifts when fed again (`new_unfed_count==0`). | — |
| 30 | Housing cap enforced per building level | **YES** | `spawning.rs:handle_animal_purchase` — coop cap = `coop_level * 4`; barn cap = `barn_level * 4`. Companions (Horse/Cat/Dog) limited to one each. Exceeding cap shows toast and skips spawn (double-checked against `AnimalState` flags). | — |

---

## Summary

| Domain | Total | YES | PARTIAL | NO |
|--------|-------|-----|---------|-----|
| Fishing | 10 | **10** | 0 | 0 |
| Mining | 12 | **12** | 0 | 0 |
| Animals | 8 | **8** | 0 | 0 |
| **Total** | **30** | **30** | **0** | **0** |

### Notable observations

1. **Day-end vs. knockout exit destination** (Action 22): Knockout sends player to `MapId::MineEntrance (7,4)`; day-end pass-out sends to `MapId::PlayerHouse (5,5)`. Both are functional but inconsistent — a player who forgets the time wakes at home rather than the mine entrance. May be intentional (sleeping ≈ waking at home) but should be verified against the spec.

2. **Ladder reveal edge case**: The `check_ladder_reveal` function reveals the ladder when any rock at the ladder's exact tile is broken, OR when all rocks are cleared. Since `has_ladder` is set on the rock at `ladder_pos`, both conditions point to the same tile — correct. However, `has_ladder` field on `RockBlueprint` is stored but never read in `handle_rock_breaking` (the reveal uses position matching only). The field is dead data; harmless but could be cleaned up.

3. **Fishing skill counts quantity, not events**: `update_fishing_skill` increments `total_catches += event.quantity`. Wild bait double-catch sends a second `ItemPickupEvent` with `quantity:1`, so it correctly awards two catch credits. Intentional and consistent.
