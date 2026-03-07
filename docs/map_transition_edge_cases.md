# Map Transition Edge Cases

**Files audited:** `src/player/interaction.rs`, `src/world/mod.rs`, `src/world/maps.rs`, `src/mining/transitions.rs`, `src/mining/ladder.rs`, `src/mining/combat.rs`, `src/npcs/map_events.rs`, `src/ui/cutscene_runner.rs`, `src/ui/transitions.rs`, `src/ui/audio.rs`, `src/save/mod.rs`, `src/world/lighting.rs`, `src/world/weather_fx.rs`, `src/npcs/schedule.rs`

---

## How Transitions Work

Map transitions flow through parallel handlers that all independently read the same `MapTransitionEvent`. There is no orchestrator — each domain reacts independently.

| Step | System | File | Purpose |
|------|--------|------|---------|
| 1. Trigger | `map_transition_check` | `src/player/interaction.rs:118` | Player walks to map edge → fires `MapTransitionEvent` |
| 2. Player handler | `handle_map_transition` | `src/player/interaction.rs:141` | Repositions player, updates `PlayerState.current_map`, invalidates `CollisionMap`, snaps camera |
| 3. World handler | `handle_map_transition` | `src/world/mod.rs:665` | Despawns `MapTile`/`WorldObject` entities, loads new map |
| 4. NPC handler | `handle_map_transition` | `src/npcs/map_events.rs:17` | Despawns all NPC entities, spawns new ones for target map |
| 5. UI fade | `trigger_fade_on_transition` | `src/ui/transitions.rs:54` | Fade-to-black-and-back animation |
| 6. Audio | `switch_music_on_map_change` | `src/ui/audio.rs:186` | Switches background music |
| 7. Mining | `handle_mine_entry` | `src/mining/transitions.rs:15` | Sets up mine state when entering `MapId::Mine` |

**Additional trigger sources:** day-end sleep (`src/player/interaction.rs:287`), mine exit (`src/mining/ladder.rs:91`), mine death (`src/mining/combat.rs:266`), cutscene teleport (`src/ui/cutscene_runner.rs:196`), save-load (`src/save/mod.rs:734`).

---

## Edge Cases

### 1. Dual `current_map` Update — Race Condition

**Files:** `src/player/interaction.rs:158`, `src/world/mod.rs:690`

Both `player::interaction::handle_map_transition` and `world::handle_map_transition` independently set `player_state.current_map = ev.to_map`. The player handler also sets it *again* in `handle_day_end` at line 313 before the event is even sent. `PlayerState.current_map` is written 2–3 times per transition by different systems. While not a bug (all write the same value), it creates fragile coupling.

### 2. Rapid/Repeated Transitions — Multiple Events Per Frame

**File:** `src/player/interaction.rs:118-136`

`map_transition_check` runs every frame. If the player is on a map edge, it fires `MapTransitionEvent` every frame the player remains there.

| Handler | Dedup strategy | Risk |
|---------|---------------|------|
| Player handler (`interaction.rs:149`) | `events.read().last()` — processes only last event | Low |
| World handler (`world/mod.rs:680`) | Loops over **all** events with `for event in events.read()` | Despawn/reload N times per frame |
| World handler same-map guard (`world/mod.rs:682`) | `if event.to_map == current_map_id.map_id { continue; }` | Mitigates after first event updates ID |

Two transitions to *different* maps in the same frame (theoretically impossible) would cause double load/despawn.

### 3. No State Guard on World Transition Handler

**Files:** `src/world/mod.rs:80-104`, `src/player/mod.rs:65-69`, `src/npcs/mod.rs:103`, `src/mining/mod.rs:51-58`

| System | State guard |
|--------|-------------|
| `world::handle_map_transition` | `run_if(in_state(GameState::Playing))` |
| `npcs::handle_map_transition` | `run_if(in_state(GameState::Playing))` |
| `mining::handle_mine_entry` | `run_if(in_state(GameState::Playing))` |
| `player::handle_day_end` | **No state guard** — runs "regardless of sub-state" |

If `DayEndEvent` fires while in a non-Playing state (Dialogue, Shop), `handle_day_end` sends a `MapTransitionEvent` but the world/NPC/mining handlers won't process it. Bevy events persist for 2 frames — if the state doesn't return to `Playing` in time, the event is lost.

### 4. `TransitionZone` Entities Are Spawned But Never Queried

**Files:** `src/world/mod.rs:434-444`, `src/world/mod.rs:525-539`

`TransitionZone` components are spawned into the ECS for every map's transition zones, but **no system ever queries for them**. Actual transition detection uses the hardcoded `edge_transition()` function in `src/player/interaction.rs:31-114`. The rect-based trigger areas from `maps.rs` are completely unused at runtime.

**Implications:**
- Rect-based zones from `maps.rs` (e.g., Farm south exit at `(13, 23, 5, 1)`) are ignored.
- Interior door transitions and mid-map zones (Farm house entrance at `(15, 0, 2, 1)`, Mine entrance cave at `(6, 1, 2, 2)`) are **not functional** via `TransitionZone`.

### 5. Inconsistent Map Connectivity Between `edge_transition()` and `maps.rs`

**Files:** `src/player/interaction.rs:31-114`, `src/world/maps.rs`

Since `edge_transition()` is what actually runs, the game's map graph differs from what `maps.rs` defines:

| Transition | `maps.rs` defines | `edge_transition()` defines (active) |
|---|---|---|
| Farm west | → MineEntrance (pos 12,6) | → Beach (pos 18, gy) |
| Beach east | Not defined | → Farm (pos 1, gy) |
| Town east | → Beach (pos 1,4) | → Forest (pos 1, gy) |
| Town south | Not defined | → Beach (pos gx, 12) |
| Forest north | Not defined | → MineEntrance (pos 7, 1) |
| MineEntrance south | Not defined | → Forest (pos 11, 16) |

### 6. Cutscene Teleport Uses Hardcoded Position (10, 10)

**File:** `src/ui/cutscene_runner.rs:196-204`

`CutsceneStep::Teleport(map_id)` always sends `to_x: 10, to_y: 10` regardless of target map. For 12×12 interiors, (10, 10) is near the edge. No validation that (10, 10) is a walkable tile.

### 7. No Validation of Target Position Walkability

**Files:** `src/player/interaction.rs:160-165`, `src/world/mod.rs:706`

When a transition fires, the player is repositioned to `(to_x, to_y)` without checking if that tile is walkable, solid, water, or void. `edge_transition()` uses `.clamp()` to keep coordinates in range, but doesn't verify walkability.

### 8. `despawn_map` Uses `despawn()` Not `despawn_recursive()`

**File:** `src/world/mod.rs:609-620`

`despawn_map` calls `commands.entity(entity).despawn()` for `MapTile` and `WorldObject` entities. If any have children (child sprites, labels), the children become orphaned. The NPC handler correctly uses `despawn_recursive()` (`src/npcs/map_events.rs:32`).

### 9. Collision Map Race — Brief Window of No Collision

**Files:** `src/player/interaction.rs:170-178`, `src/world/mod.rs:750-762`

During a transition, the player handler immediately clears and re-initializes `CollisionMap`. However, `sync_collision_map` (`src/world/mod.rs:750`) that copies `WorldMap.solid_tiles` only runs when `WorldMap` changes — 1-frame window where `CollisionMap.solid_tiles` is empty but `initialised` is true.

The player handler sets `collision_map.initialised = false` then immediately `= true` (lines 172, 178) — the `false` state is never visible to other systems, making the invalidation a no-op.

### 10. Save/Load Force-Transition Hack

**File:** `src/save/mod.rs:728-738`

When loading a save, the code sets `current_map_id.map_id = MapId::Mine` as a "dummy value" to force a mismatch, then sends a `MapTransitionEvent` to the saved map. If the player was on `MapId::Mine` when saving, this sets the dummy to `Mine` then transitions to `Mine` — the world handler skips it (same-map check). The player handler still repositions (no same-map guard), but the world map won't reload.

### 11. Mine Entry Doesn't Use Standard Transition for Floor Loading

**Files:** `src/mining/transitions.rs:15-59`, `src/mining/spawning.rs:55`

Mine floor entities use `MineFloorEntity` marker, spawned in addition to `MapTile` entities from `generate_map(MapId::Mine)`. On exit, `cleanup_mine_on_exit` despawns `MineFloorEntity`; the world handler despawns `MapTile`. This dual-cleanup approach works but creates complexity.

### 12. Mine Floor 20 Cap — Silent No-Op

**File:** `src/mining/ladder.rs:51-53`

When the player reaches floor 20 and tries to descend, the code returns without feedback (`if next_floor > 20 { return; }`). No toast message, no sound effect.

### 13. Day-End in Mine Sends Conflicting Transitions

**Files:** `src/mining/transitions.rs:61-98`, `src/player/interaction.rs:287-323`

When `DayEndEvent` fires while in the mine, both handlers send `MapTransitionEvent` to `PlayerHouse` with **different positions**:

| Handler | Target position | Additional effects |
|---------|----------------|-------------------|
| `mining::handle_day_end_in_mine` (line 91–95) | `to_x: 5, to_y: 5` | Gold penalty (10%), health → 50% |
| `player::handle_day_end` (line 305–309) | `to_x: 12, to_y: 4` | None |

The player handler takes `.last()`, so whichever system runs second determines actual position.

### 14. Weather/Lighting State Not Explicitly Reset on Transition

**Files:** `src/world/lighting.rs:114`, `src/world/weather_fx.rs`

Weather particles and day/night tint are not explicitly reset during map transitions. The lighting system checks `is_indoor_map()` using `player_state.current_map` which updates during transition. `cleanup_weather_on_change` reacts to weather *changes*, not map changes — if weather hasn't changed but the player moved indoors, particles may linger.

### 15. NPC Schedule Position vs. Current Map

**File:** `src/npcs/schedule.rs:70-82`

NPCs on other maps have schedules effectively paused. When the player transitions to their map, NPCs are freshly spawned at their schedule position, not interpolated. This is by design (Harvest Moon style) but means NPCs "teleport" to their schedule position on map entry.

### 16. `spawn_initial_map` Re-entry Guard May Cause Stale Map

**File:** `src/world/mod.rs:640-643`

The `OnEnter(GameState::Playing)` system `spawn_initial_map` has guard: `if !existing_tiles.is_empty() { return; }`. If a cutscene teleports the player to a different map (via `CutsceneStep::Teleport`), tiles from the old map still exist. On return to `Playing`, the guard sees existing tiles and skips the spawn — the player could be on a different map than what's rendered.

If the cutscene fires the teleport *before* returning to `Playing`, the `MapTransitionEvent` could be missed (world handler is gated on `Playing`).

### 17. Music Not Switched for All Map Types

**File:** `src/ui/audio.rs:191-204`

`switch_music_on_map_change` only handles Farm, Town, Mine, Forest, and Beach. Interior maps hit the `_ => continue` branch:

| Map | Music switch | Status |
|-----|-------------|--------|
| Farm, Town, Mine, Forest, Beach | Handled | ✅ |
| PlayerHouse, GeneralStore, AnimalShop, Blacksmith | `_ => continue` | ❌ No change — previous outdoor music continues |
| MineEntrance | `_ => continue` | ❌ No change |

---

## Potential Bugs

### Confirmed Issues

| # | Issue | File | Severity |
|---|-------|------|----------|
| 1 | `TransitionZone` is dead code — spawned but never queried; rect-based zones from `maps.rs` are non-functional | `src/world/mod.rs:434-444` | Medium |
| 2 | Day-end in mine sends two conflicting `MapTransitionEvent`s with different target positions (5,5 vs 12,4) | `src/mining/transitions.rs:91`, `src/player/interaction.rs:305` | Medium |
| 3 | `despawn_map` doesn't use `despawn_recursive()` — could leak child entities | `src/world/mod.rs:609-620` | Low |
| 4 | Collision map invalidation is a no-op — `initialised` set false then immediately true | `src/player/interaction.rs:172-178` | Low |

### Potential Issues

| # | Issue | File | Severity |
|---|-------|------|----------|
| 5 | Events fired from non-Playing states may be lost — `handle_day_end` runs without state guard but world/NPC handlers require `Playing` | `src/player/mod.rs:65-69`, `src/world/mod.rs:80-104` | Medium |
| 6 | Cutscene teleport always uses (10,10) — no walkability validation | `src/ui/cutscene_runner.rs:196-204` | Medium |
| 7 | Save/load hack with `MapId::Mine` dummy fails if the player was actually on the Mine map when saving | `src/save/mod.rs:728-738` | Medium |
| 8 | Weather particles can briefly appear indoors — no explicit cleanup on map transition to indoor maps | `src/world/weather_fx.rs` | Low |
| 9 | `edge_transition()` map connectivity differs from `maps.rs` — Farm west goes to Beach not MineEntrance, Town east goes to Forest not Beach | `src/player/interaction.rs:31-114`, `src/world/maps.rs` | High |
