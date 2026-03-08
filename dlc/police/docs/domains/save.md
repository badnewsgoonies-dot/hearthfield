# Save Domain Spec — Precinct

## Purpose
Full state serialization. 3 slots, auto-save on shift end, round-trip fidelity.

## Scope
`src/domains/save/` — owns save/load logic, file I/O, state restoration.

## What This Domain Does
- Listen to SaveRequestEvent → serialize all game resources to JSON file
- Listen to LoadRequestEvent → deserialize and restore all game resources
- Auto-save on ShiftEndEvent (save to current slot)
- 3 save slots (files: save_0.json, save_1.json, save_2.json)
- Round-trip guarantee: save at any point, reload, state is identical

## Resources to Serialize (ALL of these — missing any is a bug)
1. ShiftClock
2. PlayerState
3. Inventory
4. CaseBoard
5. EvidenceLocker
6. NpcRegistry (relationships + schedules, NOT definitions)
7. PartnerArc
8. Economy
9. Skills
10. PatrolState

## What This Domain Does NOT Do
- UI for save/load screen (ui domain, future)
- Static data (NPC definitions, case definitions — reconstructed on load)
- Runtime entities (player entity, map tiles, NPC entities — respawned on load)

## Key Types (import from crate::shared)
- `SaveRequestEvent`, `LoadRequestEvent`
- `ShiftEndEvent`
- All 10 Resource types listed above
- `GameState`

## Systems to Implement
1. `handle_save` — read SaveRequestEvent, serialize all resources to JSON
   - Create FullSaveData struct containing all 10 resources
   - Write to save_{slot}.json in a saves/ directory
2. `handle_load` — read LoadRequestEvent, deserialize and replace all resources
   - Read save_{slot}.json
   - Replace each resource with deserialized version
   - Transition to GameState::Playing (respawns player/map via OnEnter)
3. `auto_save` — read ShiftEndEvent, trigger save to current slot
4. `ensure_save_dir` — Startup system, create saves/ directory if missing

## FullSaveData Struct
```rust
#[derive(Serialize, Deserialize)]
struct FullSaveData {
    shift_clock: ShiftClock,
    player_state: PlayerState,
    inventory: Inventory,
    case_board: CaseBoard,
    evidence_locker: EvidenceLocker,
    npc_relationships: HashMap<NpcId, NpcRelationship>,
    partner_arc: PartnerArc,
    economy: Economy,
    skills: Skills,
    patrol_state: PatrolState,
}
```

## Quantitative Targets
- 3 save slots
- 10 serialized resources
- Auto-save every shift end
- Round-trip: save → load → all 10 resources identical

## Decision Fields
- **Preferred**: serialize everything via serde JSON, one file per slot
- **Tempting alternative**: serialize only "important" state, reconstruct rest
- **Consequence**: reconstructed state diverges from saved state; OnEnter bugs
- **Drift cue**: any resource that has Default but not Serialize

- **Preferred**: replace resources on load, let OnEnter systems respawn entities
- **Tempting alternative**: try to patch live entities during load
- **Consequence**: entity patching is fragile, ordering bugs, stale references
- **Drift cue**: worker queries entities during load instead of just replacing resources

## Plugin Export
```rust
pub struct SavePlugin;
```

## Tests (minimum 5)
1. Save creates file on disk
2. Load restores ShiftClock correctly
3. Load restores PlayerState correctly
4. Load restores CaseBoard correctly
5. Round-trip: save all resources → load → compare → identical
6. Auto-save triggers on ShiftEndEvent
7. All 10 resources present in FullSaveData
