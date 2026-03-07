# Performance Retrospective (Worker 5)

Scope: static analysis of `src/` for query breadth, iteration mutability, frame scheduling, map generation/memory behavior, and scaling risks.

## 1) Query Patterns and Broad Scans

### High-frequency broad scans (runs in `Update`/`PostUpdate`)

1. HUD prompt scans all nearby interactables + NPCs + chests every frame:
   - `src/ui/hud.rs:1099-1173` (`update_interaction_prompt`)
   - Complexity: `O(I + N + C)` per frame.

2. Interactable highlight scans all interactables every frame:
   - `src/world/mod.rs:895-914` (`highlight_nearby_interactables`)
   - Complexity: `O(I)` per frame.

3. Global Y-sort sync performs 3 full query passes each `PostUpdate`:
   - `src/world/ysort.rs:10-33` (`sync_position_and_ysort`)
   - Complexity: `O(E_ysorted + E_not_ysorted + E_static_ysorted)` per frame.

4. Farming visual sync performs full-map/state reconciliation every `PostUpdate`:
   - `src/farming/mod.rs:202-210` schedules the systems.
   - `src/farming/render.rs:53-173` (`sync_soil_sprites`) scans existing + missing + stale.
   - `src/farming/render.rs:185-349` (`sync_crop_sprites`) scans existing + missing + stale.
   - `src/farming/render.rs:398-538` (`sync_farm_objects_sprites`) scans missing + stale.
   - Complexity: multiple full passes over farm state/entity maps each frame.

5. Minimap writes full 64x64 texture every frame (even when static map unchanged):
   - `src/ui/minimap.rs:196-207` full pixel copy loop (`MAX_MAP * MAX_MAP = 4096` pixels/frame).
   - Plus NPC overlay each frame: `src/ui/minimap.rs:212-224`.

6. Weather system counts all weather entities every frame before spawning:
   - `src/world/weather_fx.rs:95-99` uses `rain_query.iter().count() + snow_query.iter().count()`.
   - Also updates each particle per frame: `src/world/weather_fx.rs:199-237`.

### Event-driven nested scans (not always per frame, but broad when triggered)

1. Tool-hit object lookup does linear search over world objects for each tool event:
   - `src/world/objects.rs:540-552`.

2. Mine combat/rock systems do linear search over enemies/rocks per tool event:
   - `src/mining/combat.rs:44-65`.
   - `src/mining/rock_breaking.rs:47-80`.

3. NPC interaction/gift systems linearly scan NPCs when input is pressed:
   - `src/npcs/dialogue.rs:65-77`.
   - `src/npcs/gifts.rs:183-193`.

## 2) `.iter()` vs `.iter_mut()` and Unnecessary Mutability

- Raw usage count in `src/` from grep:
  - `.iter()`: **145**
  - `.iter_mut()`: **55**

Observations:
1. Mutability use is mostly appropriate in hot loops (components/resources are actually mutated).
2. No clear widespread pattern of unnecessary `iter_mut()` in the primary frame loops.
3. Main mutability issue is not `iter_mut()` misuse, but avoidable allocations/linear membership checks in loops (see O(n^2) section).

## 3) Systems Running Per Frame vs `FixedUpdate` Candidates

Current state:
- No `FixedUpdate` scheduling found in `src/` (`rg "FixedUpdate" src` returned none).
- Core simulation work is concentrated in `Update`.

Candidate systems for moving simulation cadence to fixed-step:

1. Player/NPC/animal movement and AI:
   - Player movement: `src/player/movement.rs:7-88`.
   - NPC movement: `src/npcs/schedule.rs:103-135`.
   - Animal wander: `src/animals/movement.rs:12-59`.
   - Mine enemy AI: `src/mining/combat.rs:144-211`.

2. Mine grid movement/cooldowns:
   - `src/mining/movement.rs:29-96` (currently manual timer + `Update`).

3. Weather/particle simulation could optionally move to fixed-step when counts are high:
   - `src/world/weather_fx.rs:199-237`.

Keep in `Update`:
- UI systems, input edge detection, event dispatch, menu/UI state transitions.

## 4) Map Generation Approach and Memory Impact

### Approach

- Main-world maps are generated on demand by hardcoded constructors (not precomputed assets):
  - `src/world/maps.rs:82-95` dispatches `generate_*` functions.
- `load_map` regenerates full `MapDef` on each map load/transition:
  - `src/world/mod.rs:521-609`.
- Per load, tiles are spawned as ECS entities (`1 entity per tile`):
  - `src/world/mod.rs:612-669`.

### Memory/CPU impact notes

1. Tile-data vectors are small (largest map in code is Farm 32x24 = 768 tiles):
   - `src/world/maps.rs:103-104` etc.
   - MapDef tile memory itself is modest.

2. ECS entity count is the bigger cost than `Vec<TileKind>`:
   - Farm load: 768 `MapTile` entities.
   - Mine map load: 576 `MapTile` entities.

3. Mine state duplicates tile layers:
   - World map for `MapId::Mine` loads tiles (`src/world/mod.rs:521-669`).
   - Mining system also spawns its own 24x24 `MineTile` layer (`src/mining/spawning.rs:116-163`).
   - This adds another 576 entities before rocks/enemies/ladder/exit.

4. Mine floor generation allocates transient blueprint vectors each spawn:
   - `FloorBlueprint` with `Vec<RockBlueprint>` + `Vec<EnemyBlueprint>` (`src/mining/floor_gen.rs:18-30`).
   - Rock coverage targets ~40-60% of 24x24 floor (`src/mining/floor_gen.rs:61-65`), i.e. roughly 230-345 rock blueprints/entities.

## 5) Scaling Behavior Estimates

Scenario requested: **100 NPCs, 500 crops, 200 mine entities**.

### 100 NPCs

Per-frame costs rise mostly linearly:
- NPC schedule update: `O(N)` (`src/npcs/schedule.rs:67-99`).
- NPC move step: `O(N)` (`src/npcs/schedule.rs:103-135`).
- NPC animation: `O(N)` (`src/npcs/animation.rs:27-68`).
- HUD prompt adds another `O(N)` scan while in Playing (`src/ui/hud.rs:1135-1163`).

Expected behavior: still linear; likely CPU-visible once combined with UI/world broad scans.

### 500 crops

Dominant hotspot is farming render sync each frame:
- Crop sync does 3 passes (`existing`, `missing`, `stale`): `src/farming/render.rs:194-266`, `269-334`, `337-348`.
- With 500 crops, crop-only work is roughly ~1500 map/query operations per frame before soil/object passes.

Expected behavior: frame time pressure in `PostUpdate`, even without player interaction.

### 200 mine entities

For mine simulation, cost is linear but duplicated across multiple systems:
- Enemy AI builds rock hash each frame + iterates enemies (`src/mining/combat.rs:163-167`, `167-210`).
- Enemy attack iterates enemies each frame (`src/mining/combat.rs:240-268`).
- Player mine movement checks rocks linearly on movement attempts (`src/mining/movement.rs:80-84`).

Expected behavior: manageable at 200 dynamic entities, but combined with duplicated tile layers + y-sort + weather can produce noticeable spikes.

## 6) O(n^2) / Nested-Loop Risks

### Confirmed O(n^2)

1. NPC map transition cleanup retains by calling `Vec::contains` inside `retain`:
   - `src/npcs/map_events.rs:31-40`.
   - `retain(|_, e| !despawned_entities.contains(e))` is `O(M*N)` worst-case.
   - Convert `despawned_entities` to `HashSet<Entity>` to reduce to near `O(N)`.

### Repeated nested loops (event_count * entity_count)

1. Forage pickup:
   - `src/world/objects.rs:820-839` loops tool events x forageables.

2. Weed scythe:
   - `src/world/objects.rs:971-993` loops tool events x weeds.

3. Rock breaking / mine combat hits:
   - `src/mining/rock_breaking.rs:47-80`, `src/mining/combat.rs:44-65`.

These are usually fine with low event volume, but degrade under batched events or large entity pools.

## 7) Priority Hotspots

1. Farming render full-state reconciliation each frame (`src/farming/render.rs`).
2. Per-frame UI/world proximity scans (`src/ui/hud.rs`, `src/world/mod.rs`).
3. Minimap full texture write every frame (`src/ui/minimap.rs`).
4. O(n^2) retain/contains in NPC transition cleanup (`src/npcs/map_events.rs`).
5. Mine dual tile-layer entity cost (`src/world/mod.rs` + `src/mining/spawning.rs`).
