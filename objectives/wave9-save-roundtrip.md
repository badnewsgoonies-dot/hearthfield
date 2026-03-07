# Worker: Save System Round-Trip Test Coverage

## Scope (mechanically enforced)
You may only modify files under: tests/ and src/save/
Out-of-scope edits will be silently reverted after you finish.

## Required reading
1. src/save/mod.rs — FullSaveFile struct, save/load systems
2. tests/headless.rs — existing integration tests
3. src/shared/mod.rs — all resource types (import only)

## Task
Add comprehensive save/load round-trip tests to `tests/headless.rs`:

1. **Full round-trip test**: Create a test that:
   - Sets up a world with non-default values in ALL serializable resources
   - Triggers a save (send SaveRequestEvent)
   - Verifies the save file exists and is valid JSON
   - Triggers a load (send LoadRequestEvent)
   - Verifies all resource values match what was saved

2. **Extended resources test**: Verify these extended resources survive round-trip:
   - QuestLog (with active quests)
   - Achievements (with some unlocked)
   - FishEncyclopedia (with caught fish)
   - ShippingLog (with shipped items)
   - BuildingLevels (with upgrades)
   - RelationshipStages (with dating/married NPCs)
   - ToolUpgradeQueue (with pending upgrade)
   - HouseState, MarriageState, PlayStats

3. **Edge cases**:
   - Save slot 0, 1, 2 all work
   - Loading a non-existent slot doesn't crash
   - SaveSlotInfoCache updates after save

4. **Chest and machine round-trip**: Verify placed chests with items and processing machines survive save/load.

## Important constraints
- Tests run headless (no GPU, no window)
- Use the existing test setup patterns from headless.rs
- Each test function should be self-contained
- Clean up save files after tests (delete test saves)

## Validation
```
cargo check
cargo test --test headless
cargo clippy -- -D warnings
```
Done = all three pass, including your new tests.
