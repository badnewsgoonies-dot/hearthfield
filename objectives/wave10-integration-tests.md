# Worker: Integration Test Expansion

## Scope (mechanically enforced)
You may only modify files under: tests/
Out-of-scope edits will be silently reverted.

## Required reading
1. tests/headless.rs — current test suite (88 tests passing)
2. src/shared/mod.rs — all types (import only)

## Task
Add integration tests for under-tested gameplay loops. Target: 100+ total tests.

Priority test areas:
1. **Festival system**: Test that festivals activate on correct days, have correct states, clean up properly
2. **Crafting machines**: Test furnace/preserves jar/cheese press/loom processing timers and outputs
3. **Animal lifecycle**: Test buy → feed → produce → sell loop, baby → adult aging
4. **Quest completion**: Test quest acceptance, progress tracking, completion rewards
5. **Tool upgrades**: Test upgrade request → wait → completion flow
6. **Mining elevator**: Test elevator unlock every 5 floors, floor selection
7. **Fishing skill**: Test skill XP gain and level-up bonuses
8. **Weather effects**: Test weather changes, crop watering interaction

Each test should:
- Be self-contained (no test ordering dependencies)
- Use the existing `App::new()` + plugin setup pattern from headless.rs
- Have a descriptive name: `test_{system}_{scenario}`
- Assert specific outcomes, not just "doesn't crash"

## Validation
```
cargo check
cargo test --test headless
cargo clippy -- -D warnings
```
Done = all three pass, 100+ tests total.
