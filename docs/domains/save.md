# Domain Spec: Save & Load

## Scope
`src/save/` — `mod.rs`

## Responsibility
Full game state serialization and deserialization. Save to file on player sleep (auto-save) or manual save. Load from file on continue/load. 3 save slots.

## Shared Contract Types (import from `crate::shared`)
All serializable resources:
- `Calendar`
- `PlayerState`
- `Inventory`
- `FarmState`
- `AnimalState`
- `Relationships`
- `MineState`
- `UnlockedRecipes`
- `ShippingBin`
- `HouseState`
- `MarriageState`
- `QuestLog`
- `SprinklerState`
- `ActiveBuffs`
- `EvaluationScore`
- `RelationshipStages`
- `Achievements`
- `ShippingLog`
- `TutorialState`
- `PlayStats`
- `GameState`

## Quantitative Targets
- 3 save slots (slot 0, 1, 2)
- 20+ serializable resources covered
- Round-trip fidelity: save → load → save produces identical JSON
- Save format: JSON (via serde_json)
- File path: platform-appropriate (e.g., `hearthfield_save_{slot}.json`)
- Auto-save: triggered on `DayEndEvent` (player sleeps)

## Key Systems
1. `save_game` — serialize all resources to JSON, write to file
2. `load_game` — read file, deserialize, insert/overwrite all resources
3. `auto_save` — listen to `DayEndEvent`, trigger save to current slot
4. `save_slot_management` — track active slot, list available saves

## Does NOT Handle
- Save slot UI (ui domain handles save/load menu)
- Game state transition after load (main.rs / state system)
- Settings persistence (separate config file, not game save)
