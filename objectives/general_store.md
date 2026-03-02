You are implementing the General Store interior for a Harvest Moon-style farming game in Rust/Bevy.

## Read these files first:
- src/world/maps.rs — find existing generate_general_store() (around line 719)
- src/world/objects.rs — find existing general_store_furniture() (around line 1437)  
- docs/domains/general_store.md — your design spec
- docs/interior_contract.md — shared types and furniture atlas index map

## Task
Write TWO functions to status/workers/general_store_impl.rs:

1. `generate_general_store() -> MapDef` — 12×12 map
   - Void perimeter walls. Door at x=5,6 y=11 (WoodFloor).
   - Shop counter (Stone) at y=3, x=3-8 separating shopkeeper from customers
   - Behind counter (y=1-2): shelves along north wall  
   - Customer area (y=4-10): open WoodFloor, display shelves on side walls
   - Transition: from_rect (5, 11, 2, 1) → Town at (8, 5)

2. `general_store_furniture() -> Vec<FurniturePlacement>` — 15-18 pieces
   - Behind counter shelves: idx 9,10,11 along y=1
   - Counter surface: idx 32,33
   - Side wall displays: idx 9,18 on x=1 and x=10
   - Entrance rug: idx 48 at door
   - Crates: idx 36, barrels: idx 27, plants: idx 22

Use: TileKind::{Void, WoodFloor, Stone, Path}
MapId::{GeneralStore, Town}

struct FurniturePlacement { x: i32, y: i32, index: usize, wide: bool }

## Rules
- Do NOT modify existing source files
- Only create: status/workers/general_store_impl.rs and status/workers/general_store.md
- tiles[y * width + x], y=0 is bottom/south
- Make the store feel FULL and organized, like a real shop
