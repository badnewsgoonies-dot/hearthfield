# Worker: Add ShippingLog to Save/Load System

## Scope (hard allowlist — enforced mechanically)
You may ONLY modify: src/save/mod.rs
All other file edits will be reverted.

## Required Reading
1. This objective file
2. src/save/mod.rs — the save system
3. src/shared/mod.rs — for ShippingLog definition (DO NOT EDIT)

## Problem
ShippingLog is a Resource with Serialize+Deserialize (defined in shared/mod.rs) that tracks which items the player has shipped. It is initialized via init_resource in main.rs but is MISSING from the save/load system. This means shipping progress is lost on save/load.

## ShippingLog Definition (in shared/mod.rs — read only)
```rust
#[derive(Resource, Debug, Clone, Default, Serialize, Deserialize)]
pub struct ShippingLog {
    pub shipped_items: HashMap<ItemId, u32>,
}
```

## Exact Changes Required (6 locations in src/save/mod.rs)

### 1. Add to FullSaveFile struct (after building_levels field)
```rust
    #[serde(default)]
    pub shipping_log: ShippingLog,
```
Must use #[serde(default)] for backward compatibility with existing saves.

### 2. Add to ExtendedResources struct (after building_levels)
```rust
    pub shipping_log: Res<'w, ShippingLog>,
```

### 3. Add to ExtendedResourcesMut struct (after building_levels)
```rust
    pub shipping_log: ResMut<'w, ShippingLog>,
```

### 4. Add to write_save function — FullSaveFile construction
In the `let file = FullSaveFile { ... }` block, add:
```rust
        shipping_log: ext_shipping_log.clone(),
```
Also add the parameter to write_save's signature and the call site in handle_save_request.

IMPORTANT: The write_save function takes individual &Resource params for the core resources, and then ExtendedResources fields are passed individually too. Look at how building_levels is handled — follow the exact same pattern for shipping_log.

### 5. Add to load restore section
After `*ext.building_levels = file.building_levels;` add:
```rust
                *ext.shipping_log = file.shipping_log;
```

### 6. Add to handle_new_game reset section
After `*ext.building_levels = BuildingLevels::default();` add:
```rust
        *ext.shipping_log = ShippingLog::default();
```

### 7. Add use import
Make sure `ShippingLog` is imported from `crate::shared::ShippingLog` at the top (check if it's already in the existing use statement).

## Quantitative Target
- 6 insertion points, all in src/save/mod.rs
- ShippingLog must appear in: FullSaveFile, ExtendedResources, ExtendedResourcesMut, write_save, load restore, new_game reset
- #[serde(default)] on FullSaveFile field for backward compat

## Validation
After changes, search for ShippingLog in save/mod.rs — should find it in all 6 locations:
```bash
grep -n "ShippingLog\|shipping_log" src/save/mod.rs
```
Expected: at least 8 lines (struct field, 2 param structs, write_save param+construction, load restore, new_game reset, use import)

## When Done
Write completion report to status/workers/save_shipping_log.md
