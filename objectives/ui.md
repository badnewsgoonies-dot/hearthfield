# Worker: UI

## Scope (mechanically enforced — your edits outside this path will be reverted)
You may only modify files under: src/ui/
Out-of-scope edits will be silently reverted after you finish.

## Required reading (read these files from disk before writing any code)
1. GAME_SPEC.md
2. docs/domains/ui.md
3. src/shared/mod.rs (the type contract — import from here, do not redefine)

## Required imports (use exactly, do not redefine locally)
- `GameState`, `Calendar`, `Season`, `Weather`
- `PlayerState`, `Inventory`, `InventorySlot`, `ItemRegistry`, `ItemDef`, `ItemCategory`
- `ShopData`, `ShopListing`, `ShopId`
- `RecipeRegistry`, `Recipe`, `UnlockedRecipes`
- `Relationships`, `NpcRegistry`, `NpcDef`
- `QuestLog`, `Quest`, `QuestObjective`
- `Achievements`, `PlayStats`
- `StorageChest`, `QualityStack`
- `MineState`
- `TutorialState`, `CutsceneQueue`, `CutsceneStep`
- `MenuTheme`, `MenuItem`, `MenuAction`
- `PlayerInput`, `InputContext`, `InputBlocks`, `InteractionClaimed`
- `DebugOverlayState`
- Events: `DialogueStartEvent`, `DialogueEndEvent`, `ToastEvent`, `PlaySfxEvent`, `PlayMusicEvent`, `ShopTransactionEvent`, `GoldChangeEvent`, `EatFoodEvent`, `BuildingUpgradeEvent`, `AchievementUnlockedEvent`, `HintEvent`, `MapTransitionEvent`
- Constants: `SCREEN_WIDTH`, `SCREEN_HEIGHT`, `HOTBAR_SLOTS`, `TOTAL_INVENTORY_SLOTS`

## Deliverables
- `src/ui/mod.rs` — `UiPlugin`
- All 20 source files under src/ui/ functional
- HUD: time, gold, stamina, health, hotbar (12 slots)
- Inventory: 36 slots with item tooltips
- Shop: buy/sell with price display
- Crafting: recipe list with ingredient checking
- Dialogue: portrait + typewriter text + advance
- Pause: resume, save, settings, quit
- Main menu: new game, continue, settings
- Toast: slide-in notifications (stack 3)
- Minimap, debug overlay, transitions

## Quantitative targets (non-negotiable)
- 12-slot hotbar displayed
- 36-slot inventory screen
- Typewriter text speed: ~30 chars/sec
- Toast duration: 3-5 seconds
- Fade transition: 0.3 seconds each direction
- Menu theme: consistent across all screens (from MenuTheme resource)

## Validation (run before reporting done)
```
cargo check
cargo test --test headless
cargo clippy -- -D warnings
```
Done = all three commands pass with zero errors and zero warnings.

## When done
Write completion report to status/workers/ui.md containing:
- Files created/modified (with line counts)
- What was implemented
- Quantitative targets hit (with actual counts)
- Shared type imports used
- Validation results (pass/fail)
- Known risks for integration
