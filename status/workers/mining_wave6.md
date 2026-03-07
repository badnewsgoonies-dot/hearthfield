# Mining Domain — Wave 6 Completion Report

## Files Modified (with line counts)
- `src/mining/components.rs` (106 LOC) — Added `rocks_broken_this_floor` field to `ActiveFloor`
- `src/mining/floor_gen.rs` (453 LOC) — Rewrote rock drop tables, enemy counts, enemy kind picker, monster stats, gem distribution, rock count formula
- `src/mining/rock_breaking.rs` (138 LOC) — Fixed pickaxe damage values, added probability-based ladder reveal, track rocks broken
- `src/mining/spawning.rs` (359 LOC) — Added iridium ore color/atlas mapping, set `rocks_broken_this_floor` on floor spawn

## What Was Implemented

### Pickaxe Damage (spec alignment)
- Basic: 1, Copper: 2, Iron: 3, Gold: 4, Iridium: 5 (was 1/2/2/3/4)

### Rock Count Per Floor
- Now 8-15 rocks per floor (increasing with depth), was 40-60% coverage of 24x24 grid

### Rock Drop Tables (spec alignment)
- Floors 1-5: Stone (70%), Copper ore (30%)
- Floors 6-10: Stone (40%), Copper (40%), Iron ore (20%)
- Floors 11-15: Stone (35%), Iron (40%), Gold ore (20%), gems (5%)
- Floors 16-20: Stone (20%), Gold (30%), Iridium ore (10%), gems (10%), Iron (30%)

### Gem Distribution (spec alignment)
- Quartz 40%, Amethyst 25%, Emerald 15%, Ruby 12%, Diamond 8%

### Monster Spawn Counts (spec alignment)
- Floors 1-5: 1-2 (GreenSlime only)
- Floors 6-10: 3-4 (70% GreenSlime, 30% Bat)
- Floors 11-15: 3-5 (60% Bat, 40% RockCrab)
- Floors 16-20: 2-4 (mixed all three)

### Monster Base Stats (spec alignment)
- GreenSlime: HP 20, DMG 5, Speed 30 (was speed 24)
- Bat: HP 15, DMG 8, Speed 50 (was speed 48)
- RockCrab: HP 40, DMG 12, Speed 15 (was speed 16)

### Ladder Probability Formula
- 5% base + 2% per rock broken this floor, max 30%
- Added `rocks_broken_this_floor` tracking to `ActiveFloor`
- Ladder also reveals if its containing rock is broken or all rocks destroyed

### Iridium Ore Support
- Added iridium ore to rock color palette (purple tint) and atlas index mapping
- Floors 16-20 now drop iridium ore at 10% rate

## Quantitative Targets Hit
- 20 mine floors: YES (unchanged, floor cap at 20)
- Elevator every 5 floors (5, 10, 15, 20): YES (unchanged)
- Rocks per floor 8-15: YES (was ~230+, now 8-15)
- Monster stats match spec: YES (GreenSlime HP 20/DMG 5/SPD 30, Bat HP 15/DMG 8/SPD 50, RockCrab HP 40/DMG 12/SPD 15)
- Ore distribution matches spec: YES (Copper 1-10, Iron 6-15, Gold 11-20, Iridium 16-20)
- Gem distribution matches spec: YES (quartz 40%, amethyst 25%, emerald 15%, ruby 12%, diamond 8%)
- Ladder probability: YES (5% base + 2% per rock, max 30%)
- Pickaxe damage: YES (Basic 1, Copper 2, Iron 3, Gold 4, Iridium 5)

## Shared Type Imports Used
- `MineState`, `MineRock`, `MineMonster`, `MineEnemy`
- `PlayerState`, `PlayerInput`, `Player`, `PlayerMovement`, `Facing`
- `ToolUseEvent`, `ToolKind`, `ToolTier`
- `MapTransitionEvent`, `MapId`
- `StaminaDrainEvent`, `ItemPickupEvent`, `MonsterSlainEvent`
- `PlaySfxEvent`, `PlayMusicEvent`, `ToastEvent`
- `GoldChangeEvent`, `DayEndEvent`
- `LogicalPosition`, `GridPosition`, `InputBlocks`
- `TILE_SIZE`, `GameState`
- `grid_to_world_center`

## Validation Results
- `cargo check`: PASS
- `cargo clippy -- -D warnings`: PASS
- `cargo test --lib -- mining::floor_gen`: PASS (3/3 tests)
- `cargo test --test headless`: 84 passed, 4 failed (all failures in animals domain, not mining)

## Known Risks for Integration
- The 4 failing headless tests are all in the animals domain (test_animal_fed_and_petted_bonus, test_animal_happiness_*), pre-existing before this work
- Pre-existing UI dialogue_box.rs borrow error (committed version lacks `.cloned()`) — working tree has the fix but it's uncommitted; another worker should commit it
- Rock health values (3-6) are reasonable but not explicitly documented in spec as exact values
