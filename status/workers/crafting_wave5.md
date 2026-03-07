# Worker Report: CRAFTING (Wave 5)

## Files Modified (with line counts)
- `src/crafting/recipes.rs` (523 lines) — added 4 new crafting recipes
- `src/crafting/machines.rs` (642 lines) — fixed processing times to match spec

## What Was Implemented

### Machine Processing Times (Fixed)
Updated `MachineType::processing_hours()` to match spec targets:
- Furnace: 0.5 hours (30 game-minutes) — was 1.0 hours
- Preserves Jar: 4.0 hours (240 game-minutes) — was 72.0 hours
- Cheese Press: 3.0 hours (180 game-minutes) — was 24.0 hours
- Loom: 4.0 hours (240 game-minutes) — was 24.0 hours

### New Crafting Recipes Added (4)
- `iridium_sprinkler` — gold_bar + iridium_bar (completes sprinkler tier)
- `gate` — 10 wood (companion to fence/path)
- `bench` (decorative) — 20 wood + 1 iron_bar
- `lamppost` (decorative) — 3 iron_bar + 1 coal

## Quantitative Targets
- Crafting recipes: 25 (target: 20+) -- EXCEEDED
  - Sprinklers: 3 (basic, quality, iridium)
  - Fences/Paths: fence, path, gate
  - Storage: chest
  - Machines: furnace, preserves_jar, cheese_press, loom, bee_house, keg, oil_maker
  - Tools: scarecrow, seed_maker, recycler
  - Combat: bomb, mega_bomb
  - Lighting: torch, campfire, lightning_rod
  - Other: worm_bin
  - Decorative: bench, lamppost
- Cooking recipes: 15 (target: 15) -- MET
- Machine times match spec: Furnace 30min, Preserves 240min, Cheese 180min, Loom 240min -- MET
- Buff durations: 2-10 game minutes (60-240 in-game minutes mapped) -- MET
- Kitchen cooking requires HouseState.has_kitchen -- MET

## Shared Type Imports Used
- Recipe, RecipeRegistry, UnlockedRecipes
- Inventory, InventorySlot, ItemId, ItemCategory
- ItemRegistry, ItemDef
- FoodBuff, BuffType, ActiveBuffs
- HouseState
- PlayerState, PlayerInput, PlayerMovement, Player
- GameState, InputBlocks
- Interactable, InteractionKind
- EatFoodEvent, ItemPickupEvent, StaminaDrainEvent, ToastEvent, PlaySfxEvent, DayEndEvent
- Calendar, Season, Achievements, Relationships
- TILE_SIZE, MAX_STAMINA, GridPosition, LogicalPosition, YSorted, Z_ENTITY_BASE, MapId

## Validation Results
- `cargo check`: PASS (zero errors in crafting domain)
- `cargo clippy -- -D warnings`: PASS for crafting (errors exist in animals/world domains, outside scope)
- `cargo test --test headless`: PASS (88 passed, 0 failed, 2 ignored)

## Existing Systems (already complete from prior waves)
- bench.rs: CraftingUiState, OpenCraftingEvent, CraftItemEvent, handle_open_crafting, handle_craft_item, trigger_crafting_key, ingredient validation/consumption/refund helpers
- cooking.rs: Kitchen cooking with HouseState.has_kitchen check, "any_fish" wildcard resolution, ingredient management
- machines.rs: 11 MachineTypes, ProcessingMachine component, tick system, insert/collect/place handlers, machine output resolution tables, ProcessingMachineRegistry, SavedMachine for save/load
- buffs.rs: 24 food buff mappings, handle_eat_food, tick_buff_durations, apply_buff_effects (Speed + MaxStamina), get_buff_magnitude helper
- unlock.rs: 17 friendship unlocks across 9 NPCs, 10 milestone unlocks, shop-unlockable recipe list, initialize_unlocked_recipes, handle_unlock_recipe, check_milestone_recipe_unlocks, check_friendship_recipe_unlocks

## Known Risks for Integration
- Clippy errors in animals/day_end.rs and world/ysort.rs (type_complexity) will block the full `cargo clippy -- -D warnings` gate but are outside crafting scope
- New items (iridium_sprinkler, gate, bench, lamppost) need ItemDef entries in data domain
- Decorative items (bench, lamppost) need world-domain placement support
