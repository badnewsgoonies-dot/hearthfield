# Wave 8 — Furniture Placement & Fence Autotiling

## Problem
Players can craft furniture items (fence, wood_path, stone_path, scarecrow, torch, wooden_sign, bee_house, etc.) but pressing tool_secondary does nothing — no placement handler exists for ItemCategory::Furniture items. Only sprinklers and machines have placement pipelines.

## Plan

### Wave 8b: Contract Amendment
Add PlaceFurnitureEvent to src/shared/mod.rs:
```rust
#[derive(Event, Debug, Clone)]
pub struct PlaceFurnitureEvent {
    pub item_id: String,
    pub grid_x: i32,
    pub grid_y: i32,
}
```
Re-checksum.

### Wave 8c-1: Player Worker (src/player/)
Add furniture placement handler in item_use.rs:
- After the machines block, add a catch-all for remaining Furniture items
- Check current_map == Farm
- Send PlaceFurnitureEvent
- Exclude items already handled (sprinklers, machines, chest, hay, bouquet, mermaid_pendant)

### Wave 8c-2: World Worker (src/world/)
Add handler for PlaceFurnitureEvent in objects.rs:
- Validate tile is walkable and not occupied
- Remove item from inventory
- Spawn entity with sprite at grid position
- Send ToastEvent confirmation
- For fence items: use fences atlas with autotiling (same bitmask pattern as paths)
- For path items: convert tile to TileKind::Path (modify map_def)
- For other furniture: spawn with furniture atlas sprite

### Wave 8d: Regression audit
Verify placement works, autotiling correct, no scope violations.

## Current Phase: Wave 8b
