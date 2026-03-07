You are implementing the Blacksmith interior for a Harvest Moon-style farming game in Rust/Bevy.

## Read these files first:
- src/world/maps.rs — find existing generate_blacksmith() (around line 826)
- src/world/objects.rs — find existing blacksmith_furniture() (around line 1451)
- docs/domains/blacksmith.md — your design spec
- docs/interior_contract.md — furniture atlas index map

## Task  
Write TWO functions to status/workers/blacksmith_impl.rs:

1. `generate_blacksmith() -> MapDef` — 12×12 map
   - Void perimeter. Door at x=5,6 y=11 (WoodFloor).
   - Base floor: Stone (not WoodFloor - it's a smith!)
   - Forge area (bottom-right y=1-3, x=7-10): Dirt tiles (heated)
   - Anvil workspace (center y=4-6, x=4-6): WoodFloor island
   - Counter line (y=3, x=2-6): WoodFloor
   - Storage (bottom-left y=1-3, x=1-3): crates/barrels area
   - Transition: from_rect (5, 11, 2, 1) → Town at (22, 14)

2. `blacksmith_furniture() -> Vec<FurniturePlacement>` — 12-15 pieces
   - Anvil: idx 19 center
   - Furnace/forge: idx 18 along back wall
   - Crates: idx 36,37,38
   - Barrels: idx 27 (water barrel)
   - Counter: idx 32,33
   - Lamp: idx 24
   - Shelf: idx 9,10

struct FurniturePlacement { x: i32, y: i32, index: usize, wide: bool }
Use TileKind::{Void, Stone, WoodFloor, Dirt}; MapId::{Blacksmith, Town}
tiles[y * width + x]. Do NOT modify existing files. Write completion report to status/workers/blacksmith.md.
