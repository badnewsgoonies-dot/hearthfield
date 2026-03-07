# Mining Navigation Audit Report

**Files audited:** `src/mining/transitions.rs`, `src/mining/movement.rs`, `src/mining/ladder.rs`, `src/mining/hud.rs`, `src/mining/components.rs`, `src/mining/mod.rs`
**Date:** 2026-03-02

---

## 1. Mine Entry

Entry is triggered by a `MapTransitionEvent { to_map: MapId::Mine, .. }` received in `transitions::handle_mine_entry`.

**State changes on entry:**
1. `InMine.0 = true`
2. `PlaySfxEvent("mine_enter")` and `PlayMusicEvent("mine_ambient", fade_in=true)` fired
3. **Branch A — elevator unlocked** (`mine_state.elevator_floors` is non-empty):
   - `ElevatorUiOpen.0 = true`
   - Floor spawn is deferred; the player stays frozen until they pick a floor via `handle_elevator_selection`
4. **Branch B — first visit** (no elevator stops yet):
   - `mine_state.current_floor = 1`
   - `FloorSpawnRequest { pending: true, floor: 1 }` set
   - `active_floor.spawned = false`

**What is NOT reset on entry:** `mine_state.deepest_floor_reached` and `mine_state.elevator_floors` are intentionally persisted across sessions (save-compatible). `ActiveFloor` fields other than `spawned` are not explicitly reset — they are overwritten when the floor spawns.

---

## 2. Floor Transitions

### Ladders (descend only)

Handled by `ladder::handle_ladder_interaction`. Conditions to trigger:
- `InMine.0 && active_floor.spawned && !input_blocks.is_blocked()`
- Player presses `tool_use` or `ui_confirm`
- Player grid position exactly matches a `MineLadder { revealed: true }` entity's `MineGridPos`

On trigger:
1. `next_floor = mine_state.current_floor + 1`, hard-capped at **floor 20** (silently returns if already 20)
2. `mine_state.current_floor = next_floor`
3. `mine_state.deepest_floor_reached` updated if a new record
4. If `next_floor % 5 == 0` and not already in `elevator_floors`, the floor is pushed and sorted
5. `FloorSpawnRequest { pending: true, floor: next_floor }` set, `active_floor.spawned = false`

**There is no ascending ladder.** Once the player descends, they cannot go up a floor — the only ways out are: reach the `MineExit` entity, press the elevator on next entry, or pass out.

### Elevator

Selected via `ladder::handle_elevator_selection`, active while `ElevatorUiOpen.0 = true`.

- Supports at most **4 slots** (key slots 0–3 → floors 1, elevator[0], elevator[1], elevator[2])
- Pressing `ui_cancel` (Esc) defaults to floor 1
- On selection: `mine_state.current_floor = floor`, `ElevatorUiOpen.0 = false`, `FloorSpawnRequest` set

**The elevator is only available at mine entry, not mid-run.** There is no in-mine elevator terminal entity. If the player has 4+ elevator stops (floors 5, 10, 15, 20), slot index 3 (floor 15, elevator[2]) is the maximum reachable via elevator; **floor 20 is unreachable via elevator** even when unlocked as the 4th stop (elevator[3] has no key binding).

---

## 3. Movement

**Mine movement is strictly grid-based**, not continuous.

- Implemented in `movement::mine_player_movement`
- Uses a `MineMoveCooldown` timer (0.15 s, `TimerMode::Once`) to throttle step rate
- Only one axis moves per step: vertical (Y) takes priority over horizontal (X)
- Grid bounds: `x ∈ [1, MINE_WIDTH-2]`, `y ∈ [0, MINE_HEIGHT-2]` (24×24 grid)
- Collision: iterates all `MineRock`-tagged entities and checks `MineGridPos` equality — O(n) per step

**Collision does not include enemies.** Players can overlap enemy grid cells freely; damage is handled separately by `combat::enemy_attack_player`. This is likely intentional (allows passing through enemies) but may feel odd.

**Facing direction** for tool use is read from the `PlayerMovement` component. If `player_movement.get_single()` fails (no `Player` entity), it silently defaults to facing Up.

**Position synchronisation:** `LogicalPosition` is updated immediately in `mine_player_movement`; the comment notes a sync system writes `Transform` in `PostUpdate`. That sync system is in the player domain, not this module — the coupling is implicit.

---

## 4. Mine Exit

Handled by `ladder::handle_mine_exit`. Conditions:
- `InMine.0 && active_floor.spawned && !input_blocks.is_blocked()`
- Player presses `tool_use` or `ui_confirm`
- Manhattan distance from player to any `MineExit` entity ≤ 1

On trigger:
1. `mine_state.current_floor = 0`
2. `InMine.0 = false`
3. `active_floor.spawned = false`
4. `MapTransitionEvent { to_map: MapId::MineEntrance, to_x: 7, to_y: 4 }` sent

`cleanup_mine_on_exit` in `transitions.rs` then despawns all `MineFloorEntity`-tagged entities in the same `Update` tick (it fires every frame when `InMine` is false, including after day-end knockout).

`despawn_mine_hud` in `hud.rs` despawns all `MineHudEntity`-tagged entities when `InMine` is false.

**Notice:** `handle_mine_entry` also handles exit transitions — it sets `InMine.0 = false` when `event.to_map != MapId::Mine && in_mine.0`. This means there are **two code paths** that flip `InMine` to false: the `handle_mine_exit` system and the entry handler's else-branch. Both are correct but redundant. The entry handler's else-branch does not send a `MapTransitionEvent`; it relies on one already being in flight.

---

## 5. Day End in Mine

Handled by `transitions::handle_day_end_in_mine`.

On `DayEndEvent` while `InMine.0`:
1. Gold penalty: 10% of current gold (skipped if `gold_loss == 0`)
2. Health restored to 50% of max
3. `mine_state.current_floor = 0`, `InMine.0 = false`, `active_floor.spawned = false`
4. `MapTransitionEvent { to_map: MapId::PlayerHouse, to_x: 5, to_y: 5 }` sent

This is mechanically identical to the Stardew Valley "pass out" penalty. Inventory items collected during the run are **not** penalised or lost — only gold.

`cleanup_mine_on_exit` and `despawn_mine_hud` will clean up floor entities and HUD in the same frame since they key on `InMine.0`.

---

## 6. HUD Elements

All mine HUD entities carry the `MineHudEntity` marker for bulk cleanup.

| Element | Component | Description |
|---|---|---|
| Floor label | `FloorLabel` | "Floor N" text, top-left (10px, 10px), 18px white font |
| Elevator prompt | `ElevatorPrompt` | Multi-line text listing key bindings for floor selection, top 100px / left 50px, 16px warm-yellow font |

- `spawn_mine_hud`: spawns the floor label when `InMine.0 && existing.is_empty()`
- `update_floor_label`: reads `active_floor.floor` each frame to keep the label current
- `show_elevator_prompt`: spawns the prompt when `ElevatorUiOpen.0` flips true; despawns when it flips false
- `despawn_mine_hud`: bulk despawns all `MineHudEntity` when `InMine.0` is false

**No health bar, stamina, or enemy indicators are rendered in this module.** Those are assumed to live in the main HUD (ui domain).

---

## 7. Bugs and Gaps

### BUG — `cleanup_mine_on_exit` runs every frame when not in mine
`cleanup_mine_on_exit` has no change-detection guard. It runs unconditionally every frame when `InMine.0 == false` — i.e., during all normal gameplay outside the mine. While harmless when the query returns nothing, this is wasteful. It should use `Changed<InMine>` or be scheduled `OnExit(GameState::Mining)`.

### BUG — `despawn_mine_hud` has the same always-running issue
Same problem as above: fires every frame outside the mine. Should use change detection.

### BUG — Elevator hard-capped at 3 unlocked stops (floor 20 unreachable via elevator)
`handle_elevator_selection` maps `tool_slot Some(0..=3)` → 4 total options (floor 1 + 3 elevator stops). `elevator_floors` can grow to 4 entries (floors 5, 10, 15, 20). `elevator_floors[3]` (floor 20) is never reachable via the elevator because there is no `Some(4)` arm. **Floor 20 is silently ignored** in the elevator UI but is still displayed in the `show_elevator_prompt` text (the loop iterates all entries), which will confuse players.

### BUG — Elevator prompt shows more floors than are selectable
`show_elevator_prompt` iterates the entire `mine_state.elevator_floors` vec and lists all of them with sequential key numbers (2, 3, 4, 5…). But `handle_elevator_selection` only handles slots 0–3. If there are 4 elevator stops, the prompt shows `[5] Floor 20` with no corresponding key handling — the label is wrong and the option is inert.

### GAP — No "go up" ladder
There is no mechanism to ascend floors mid-run. If the player descends to floor 10 but does not find the exit, their only way out is to reach the exit tile or wait for day-end. This may be intentional (Stardew Valley also has no backtracking) but is worth confirming.

### GAP — `ActiveFloor.player_grid_x/y` not reset when starting a new floor
When `floor_req.pending = true` is set (ladder or elevator selection), `active_floor.player_grid_x/y` are **not** reset. They are only set when `spawning::spawn_mine_floor` runs and (presumably) places the player at the spawn position. Between the ladder press and the spawn completing (`active_floor.spawned = false`), the stale grid coordinates remain. If `spawn_mine_floor` does not explicitly reset these, the player could start a floor at the previous floor's position.

### GAP — Elevator state not persisted across save/load
`elevator_floors` lives in `MineState` (defined in `shared`). Whether `MineState` is included in `FullSaveFile` in `src/save/mod.rs` was not audited here, but if it is omitted, all elevator progress is lost on save/load.

### GAP — `ElevatorUiOpen` not reset on non-Mine map transitions
`handle_mine_entry` only checks `event.to_map == MapId::Mine` for the elevator-open branch. If the game somehow transitions away from the mine while `ElevatorUiOpen.0 == true` (e.g., a crash recovery or edge case), the flag stays true. `InMine.0` is correctly cleared, but `elevator_ui.0` would remain true and could interfere on the next mine entry.

### GAP — Bounds check asymmetry (Y lower bound is 0, not 1)
`mine_player_movement` allows `new_y >= 0` but `new_x >= 1`. The south wall is at `y = 0` and walkable (no fence). Whether this is intentional (the exit tile is at the south wall) or a missing wall constraint is unclear.

### GAP — Tool action and ladder share the same key (`tool_use` / `ui_confirm`)
Both `handle_ladder_interaction` and `mine_player_action` fire on `player_input.tool_use`. Standing on a revealed ladder and swinging the pickaxe will simultaneously descend the floor. There is no priority guard between them — both events fire in the same frame.

### MINOR — `MonsterSlainEvent` registered locally but also likely in `main.rs`
`MiningPlugin::build` calls `app.add_event::<MonsterSlainEvent>()`. If `main.rs` also registers this event, it is a duplicate registration (benign in Bevy 0.15 but indicates a bookkeeping inconsistency).
