# MANIFEST — Hearthfield Robustness Campaign

## Current Phase: 2 (spec writing)

## Map Bounds (truth — all coordinates must respect these)
- Farm: 32×24 (x: 0-31, y: 0-23)
- Town: 28×22 (x: 0-27, y: 0-21)
- Beach: 20×14 (x: 0-19, y: 0-13)
- Forest: 22×18 (x: 0-21, y: 0-17)
- MineEntrance: 14×12 (x: 0-13, y: 0-11)
- PlayerHouse: 16×16 (x: 0-15, y: 0-15)
- GeneralStore: 12×12 (x: 0-11, y: 0-11)
- AnimalShop: 12×12 (x: 0-11, y: 0-11)
- Blacksmith: 12×12 (x: 0-11, y: 0-11)

## Coordinate System
- tiles[y * width + x] row-major
- y=0 is back wall (north), y=h-1 is front/door (south)
- grid_to_world_center(x, y) converts to pixel coords

## Domains
1. npc_schedules — Fix all OOB coordinates in schedules.rs
2. transition_coords — Validate all map transition zones
3. unwrap_safety — Replace panicking unwraps with graceful handling
4. save_integrity — Ensure save/load round-trips all current state
5. farming_validation — Crop/soil bounds checking
6. fishing_robustness — Fishing state machine edge cases

## Key Decisions
- Tile size: 16px
- NPC spawn uses grid coords, converted via grid_to_world_center()
- All schedule entries must be within map bounds
- Farm plots exist at specific tile ranges (check generate_farm)

## Open Blockers
- None yet
