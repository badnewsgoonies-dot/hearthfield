# Worker: FIX-ECONOMY-SAVE (EconomyStats Serialization)

## Scope (mechanically enforced — your edits outside this path will be reverted)
You may only modify files under: src/economy/ AND src/save/
Out-of-scope edits will be silently reverted after you finish.

## Required reading (read these files from disk before writing any code)
1. src/economy/gold.rs (read the FULL file — EconomyStats struct is here)
2. src/save/mod.rs (read the FULL file — FullSaveFile, write_save, handle_load_request, handle_new_game)
3. src/shared/mod.rs — search for "Serialize" to see how other resources derive serde traits

## Bug: EconomyStats Lost on Save/Load

### Root Cause
`EconomyStats` in `src/economy/gold.rs` only derives `Resource, Debug, Clone, Default`. It is missing `Serialize, Deserialize`. Additionally, `FullSaveFile` in `src/save/mod.rs` does not include an `economy_stats` field, so the data is never written to or read from save files.

### Fix Required

#### 1. src/economy/gold.rs — Add serde derives
Change the derive macro on `EconomyStats` from:
```rust
#[derive(Resource, Debug, Clone, Default)]
```
to:
```rust
#[derive(Resource, Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
```

#### 2. src/save/mod.rs — Add to FullSaveFile struct
Add a new field (with `#[serde(default)]` for backwards compatibility with old saves):
```rust
#[serde(default)]
pub economy_stats: crate::economy::gold::EconomyStats,
```

#### 3. src/save/mod.rs — Add to ExtendedResources (read-only SystemParam)
Add:
```rust
pub economy_stats: Res<'w, crate::economy::gold::EconomyStats>,
```

#### 4. src/save/mod.rs — Add to ExtendedResourcesMut (mutable SystemParam)
Add:
```rust
pub economy_stats: ResMut<'w, crate::economy::gold::EconomyStats>,
```

#### 5. src/save/mod.rs — Add to write_save
In the FullSaveFile construction, add:
```rust
economy_stats: ext.economy_stats.clone(),
```

#### 6. src/save/mod.rs — Add to handle_load_request
In the resource restoration section, add:
```rust
*ext.economy_stats = file.economy_stats;
```

#### 7. src/save/mod.rs — Add to handle_new_game
In the resource reset section, add:
```rust
*ext.economy_stats = crate::economy::gold::EconomyStats::default();
```

### Validation
```
cargo check
cargo test --test headless
cargo clippy -- -D warnings
```

## When done
Write completion report to status/workers/fix-economy-save.md
