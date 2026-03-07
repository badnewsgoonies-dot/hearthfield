You are implementing the Animal Shop interior for a Harvest Moon-style farming game in Rust/Bevy.

## Read these files first:
- src/world/maps.rs — find existing generate_animal_shop() (around line 772)
- src/world/objects.rs — find existing animal_shop_furniture() (around line 1463)
- docs/domains/animal_shop.md — your design spec
- docs/interior_contract.md — furniture atlas index map

## Task
Write TWO functions to status/workers/animal_shop_impl.rs:

1. `generate_animal_shop() -> MapDef` — 12×12 map
   - Void perimeter. Door at x=5,6 y=11 (WoodFloor).
   - Feed/hay storage (bottom-left y=1-3, x=1-4): Dirt floor tiles
   - Sales counter (Stone) at y=3, x=4-8
   - Customer area (y=4-10): WoodFloor
   - Transition: from_rect (5, 11, 2, 1) → Town at (22, 3)

2. `animal_shop_furniture() -> Vec<FurniturePlacement>` — 12-15 pieces
   - Hay bales: idx 45,46,47 in storage area
   - Counter: idx 32,33
   - Barrels: idx 27,28 
   - Shelves: idx 9,10,11
   - Crates: idx 36
   - Plants: idx 22

struct FurniturePlacement { x: i32, y: i32, index: usize, wide: bool }
Use TileKind::{Void, WoodFloor, Stone, Dirt}; MapId::{AnimalShop, Town}
tiles[y * width + x]. Do NOT modify existing files. Write completion report to status/workers/animal_shop.md.
