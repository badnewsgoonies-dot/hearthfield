# Worker: FIX-MISSING-ANIMAL-ITEMS

## Scope (mechanically enforced — your edits outside this path will be reverted)
You may only modify files under: src/data/
Out-of-scope edits will be silently reverted after you finish.

## Required reading (read these files from disk before writing any code)
1. src/data/items.rs (search for "animal_chicken" to find existing animal ItemDefs at ~line 1567-1602)
2. src/data/shops.rs (search for "animal_" to see shop listings — all 10 animals are listed)
3. src/shared/mod.rs — search for ItemDef, ItemCategory

## Bug: 7 of 10 Animal Types Missing from ItemRegistry

### Root Cause
`src/data/shops.rs` lists all 10 animal types in the Animal Shop, but `src/data/items.rs` only defines ItemDefs for chicken, cow, and sheep. When the player tries to buy goat, duck, rabbit, pig, horse, cat, or dog, the shop UI can't find the ItemDef and silently skips the listing.

### Fix Required
In `src/data/items.rs`, after the existing animal_sheep ItemDef (after ~line 1602), add 7 new ItemDefs. Use the EXACT same pattern as existing animals:

```rust
ItemDef {
    id: "animal_goat".into(),
    name: "Goat".into(),
    description: "A nimble goat. Produces goat milk when happy.".into(),
    category: ItemCategory::Special,
    sell_price: 0,
    buy_price: Some(2000),
    stack_size: 1,
    edible: false,
    energy_restore: 0.0,
    sprite_index: 81,
},
ItemDef {
    id: "animal_duck".into(),
    name: "Duck".into(),
    description: "A waterfowl. Produces eggs and feathers.".into(),
    category: ItemCategory::Special,
    sell_price: 0,
    buy_price: Some(1200),
    stack_size: 1,
    edible: false,
    energy_restore: 0.0,
    sprite_index: 82,
},
ItemDef {
    id: "animal_rabbit".into(),
    name: "Rabbit".into(),
    description: "A fluffy rabbit. Produces fur when happy.".into(),
    category: ItemCategory::Special,
    sell_price: 0,
    buy_price: Some(4000),
    stack_size: 1,
    edible: false,
    energy_restore: 0.0,
    sprite_index: 83,
},
ItemDef {
    id: "animal_pig".into(),
    name: "Pig".into(),
    description: "A truffle-hunting pig. Finds truffles outdoors.".into(),
    category: ItemCategory::Special,
    sell_price: 0,
    buy_price: Some(8000),
    stack_size: 1,
    edible: false,
    energy_restore: 0.0,
    sprite_index: 84,
},
ItemDef {
    id: "animal_horse".into(),
    name: "Horse".into(),
    description: "A loyal horse. Allows faster travel around the farm.".into(),
    category: ItemCategory::Special,
    sell_price: 0,
    buy_price: Some(10000),
    stack_size: 1,
    edible: false,
    energy_restore: 0.0,
    sprite_index: 85,
},
ItemDef {
    id: "animal_cat".into(),
    name: "Cat".into(),
    description: "A pet cat. Keeps rodents away from crops.".into(),
    category: ItemCategory::Special,
    sell_price: 0,
    buy_price: Some(500),
    stack_size: 1,
    edible: false,
    energy_restore: 0.0,
    sprite_index: 86,
},
ItemDef {
    id: "animal_dog".into(),
    name: "Dog".into(),
    description: "A loyal dog. Guards the farm at night.".into(),
    category: ItemCategory::Special,
    sell_price: 0,
    buy_price: Some(500),
    stack_size: 1,
    edible: false,
    energy_restore: 0.0,
    sprite_index: 87,
},
```

Prices MUST match the shop listings in shops.rs: goat=2000, duck=1200, rabbit=4000, pig=8000, horse=10000, cat=500, dog=500.

### Validation
```
cargo check
cargo test --test headless
cargo clippy -- -D warnings
```

## When done
Write completion report to status/workers/fix-missing-animal-items.md
