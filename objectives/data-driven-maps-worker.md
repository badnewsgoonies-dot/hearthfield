# Worker: Data-Driven Maps

## Scope (mechanically enforced)
You may modify files under: src/world/, src/player/, assets/maps/
Out-of-scope edits will be reverted.

## Required reading (read these files before writing any code)
1. docs/domains/data-driven-maps.md — the full design spec
2. src/world/maps.rs — all 10 hardcoded map generators
3. src/world/mod.rs — WorldMap resource, map loading systems
4. src/player/interaction.rs — edge_transition(), map_bounds(), door triggers
5. src/shared/mod.rs — MapId, TileKind, MapTransition (DO NOT MODIFY this file)

## Deliverables

### Phase 1: Types + Serialization
1. Create `src/world/map_data.rs` with:
   - `MapData` struct (RON-serializable version of map definitions)
   - `ObjectDef` struct with `(x, y, kind)` — WorldObjectKind needs Serialize/Deserialize added in maps.rs
   - `TransitionDef` struct for zone-based transitions
   - `DoorDef` struct for door entry points
   - `EdgeTarget` enum (ClampX, ClampY, Fixed(i32, i32))
   - `EdgeDefs` struct with north/south/east/west options
   - All types derive `Debug, Clone, Serialize, Deserialize`

2. Add `Serialize, Deserialize` derives to `WorldObjectKind` in `src/world/maps.rs`

### Phase 2: Export Existing Maps to RON
1. Write an `export_all_maps()` function (can be a test or a binary) that:
   - Calls each `generate_*()` function
   - Extracts transition data from the hardcoded `edge_transition()` logic
   - Converts to `MapData`
   - Serializes to RON with `ron::ser::to_string_pretty()`
   - Writes to `assets/maps/{map_id_lowercase}.ron`
2. Run the export to produce all 10 RON files
3. The RON files must be committed

### Phase 3: Loader
1. Write `load_map_data(map_id: MapId) -> MapData` that:
   - Tries to read `assets/maps/{map_id}.ron` (lowercase map id name)
   - Falls back to the hardcoded `generate_*()` if file not found
   - Parses with `ron::from_str()`
2. Write `map_data_to_map_def(data: &MapData) -> MapDef` converter
3. Create `MapRegistry` resource: `HashMap<MapId, MapData>`
4. Add a startup system that loads all 10 maps into `MapRegistry`
5. Modify `generate_map()` to check `MapRegistry` first, fall back to hardcoded

### Phase 4: Data-Drive Transitions
1. In `src/player/interaction.rs`:
   - Modify `map_bounds()` to read width/height from `MapRegistry` resource
   - Modify `edge_transition()` to read doors + edges from `MapRegistry`
   - Remove all hardcoded door/edge if-chains (replace with data lookups)
   - Keep the same public API: `map_transition_check()` and `handle_map_transition()`
2. Make the system function signatures accept `Res<MapRegistry>` parameter
3. Ensure existing unit tests still pass (update if needed)

### Phase 5: Wire Up
1. In `src/world/mod.rs`:
   - `pub mod map_data;`
   - Register `MapRegistry` as a resource
   - Add startup system to populate it
2. Verify `generate_map()` uses the loaded data

## Quantitative targets
- 10 RON files in assets/maps/ (one per MapId variant)
- 0 hardcoded tile coordinates in edge_transition() — all from data
- 0 hardcoded tile coordinates in door triggers — all from data
- map_bounds() reads from MapRegistry, no hardcoded match arms
- Fallback to hardcoded generators preserved (if RON file missing)

## Constraints
- DO NOT modify src/shared/mod.rs (the contract is frozen + checksummed)
- DO NOT change MapId or TileKind enum variants
- DO NOT change the public API of map_transition_check or handle_map_transition
- The RON files must produce IDENTICAL game behavior to the current hardcoded maps
- Keep generate_*() functions as fallbacks — do not delete them

## Validation (run before reporting done)
```bash
cargo check
cargo test --test headless
cargo clippy -- -D warnings
shasum -a 256 -c .contract.sha256
```
Done = all four commands pass with zero errors and zero warnings.

## When done
Write completion report to status/workers/data_driven_maps.md containing:
- Files created/modified (with line counts)
- All 10 RON file paths
- Validation results
