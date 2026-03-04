# Domain Spec: Crafting & Cooking

## Scope
`src/crafting/` — `mod.rs`, `bench.rs`, `recipes.rs`, `cooking.rs`, `machines.rs`, `buffs.rs`, `unlock.rs`

## Responsibility
Crafting bench interaction, recipe management, cooking at kitchen, processing machines (furnace, preserves jar, cheese press, loom), food buff application, and recipe unlocking.

## Shared Contract Types (import from `crate::shared`)
- `Recipe` (id, name, ingredients, result, result_quantity, is_cooking, unlocked_by_default)
- `RecipeRegistry` (Resource — recipes HashMap)
- `UnlockedRecipes` (Resource — ids Vec)
- `Inventory`, `InventorySlot`, `ItemId`, `ItemCategory`
- `ItemRegistry`, `ItemDef`
- `FoodBuff`, `BuffType` (Speed, Mining, Fishing, Farming, Defense, Attack, Luck, MaxStamina)
- `ActiveBuffs` (Resource — buffs Vec)
- `HouseState`, `HouseTier` (kitchen requires Big+ house)
- `PlayerState`, `PlayerInput`
- `GameState`, `InputBlocks`
- `Interactable`, `InteractionKind` (CraftingBench, Machine, KitchenStove)
- Events: `EatFoodEvent`, `ItemPickupEvent`, `ItemRemovedEvent`, `ToastEvent`
- Constants: `TILE_SIZE` (16.0)

## Quantitative Targets
- 20 crafting recipes:
  - Basic: chest, fence (x10), path (x10), torch, gate
  - Sprinklers: basic sprinkler, quality sprinkler, iridium sprinkler
  - Machines: furnace, preserves jar, cheese press, loom, bee house, keg, oil maker
  - Tools: scarecrow, seed maker, recycling machine
  - Decorative: bench, lamppost
- 15 cooking recipes:
  - Fried egg (egg → +30 stamina)
  - Salad (mixed greens → +20 stamina, Speed buff 2min)
  - Fish soup (any fish + potato → +50 stamina, Fishing buff 5min)
  - Pumpkin pie (pumpkin + wheat + egg → +75 stamina, Farming buff 7min)
  - Cranberry sauce (cranberry → +25 stamina)
  - Corn chowder (corn + milk → +40 stamina)
  - Blueberry tart (blueberry + wheat → +35 stamina)
  - (8 more to reach 15 total)
- Processing machine times:
  - Furnace: 5 ore → 1 bar (30 game-minutes per batch)
  - Preserves jar: 1 fruit → 1 jam (240 game-minutes)
  - Cheese press: 1 milk → 1 cheese (180 game-minutes)
  - Loom: 1 wool → 1 cloth (240 game-minutes)
- Buff durations: 2-10 game minutes based on recipe

## Constants & Formulas
- Crafting: check `inventory.has(ingredient_id, quantity)` for all ingredients, remove all, add result
- Cooking: requires `HouseState.has_kitchen == true`
- Machine processing: timer in game-minutes, output produced when timer reaches 0
- Buff magnitude varies by recipe (e.g., Speed +20%, Mining +15%, etc.)

## Key Systems
1. `crafting_bench_system` — open crafting menu when player interacts with bench
2. `craft_item` — validate ingredients, deduct from inventory, produce result
3. `cooking_system` — validate kitchen access, deduct ingredients, produce cooked food
4. `machine_processing` — tick machine timers, produce output when ready
5. `buff_application` — on `EatFoodEvent`, apply `FoodBuff` to `ActiveBuffs`
6. `buff_decay` — each game tick, decrement buff minutes, remove expired
7. `recipe_unlock` — unlock recipes based on events (friendship level, items found, etc.)

## Does NOT Handle
- Crafting screen UI rendering (ui domain)
- Recipe data loading from tables (data domain)
- Item definitions for ingredients/results (data domain)
- Kitchen stove placement in house map (world domain)
