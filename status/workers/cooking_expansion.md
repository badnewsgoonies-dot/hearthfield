# Cooking Recipe Expansion Report

## Scope
Expanded cooking recipes in `src/data/recipes.rs` from 15 to 25 (`is_cooking: true`).

## Files Modified
- `src/data/recipes.rs`
- `src/data/items.rs`

## Recipes Added (10)
1. `recipe_grilled_fish` -> `grilled_fish` (ingredient: `trout` x1)
2. `recipe_fish_stew_catfish` -> `fish_stew` (ingredients: `catfish` x1, `potato` x1)
3. `recipe_sashimi` -> `sashimi` (ingredient: `salmon` x1)
4. `recipe_roasted_pumpkin` -> `roasted_pumpkin` (ingredient: `pumpkin` x1)
5. `recipe_corn_chowder` -> `corn_chowder` (ingredients: `corn` x1, `milk` x1)
6. `recipe_melon_smoothie` -> `melon_smoothie` (ingredient: `melon` x1)
7. `recipe_cheese_omelet` -> `cheese_omelet` (ingredients: `egg` x1, `cheese` x1)
8. `recipe_truffle_risotto` -> `truffle_risotto` (ingredients: `truffle` x1, `rice` x1)
9. `recipe_blueberry_pie` -> `blueberry_pie` (ingredients: `blueberry` x1, `wheat_flour` x1)
10. `recipe_cranberry_sauce` -> `cranberry_sauce` (ingredients: `cranberry` x1, `sugar` x1)

All new recipes are marked `is_cooking: true` and `unlocked_by_default: false`.

## Item Definitions Added
Added missing result items required by the new recipes:
- `grilled_fish`
- `sashimi`
- `roasted_pumpkin`
- `corn_chowder`
- `melon_smoothie`
- `cheese_omelet`
- `truffle_risotto`
- `blueberry_pie`
- `cranberry_sauce`

Added missing ingredient items needed so recipe ingredient IDs are valid:
- `truffle`
- `rice`
- `wheat_flour`
- `sugar`

## Validation
- Cooking recipe count (`is_cooking: true`) is now **25**.
- All requested result IDs now exist in `items.rs`.
- All ingredient IDs used by the 10 new recipes now exist in `items.rs`.
