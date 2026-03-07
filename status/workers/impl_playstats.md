# PlayStats Tracking Fix

## Bug 1: track_animals_petted

**Finding:** No `PetAnimalEvent`, `AnimalInteractEvent`, or similar event exists in the codebase. The petting logic in `src/animals/interaction.rs` (lines 65–94) sets `animal.petted_today = true` and fires a `PlaySfxEvent` directly — no dedicated event.

**Action:** Left `AnimalProductEvent` as the trigger (no correct event to switch to). Updated the section header and doc comment to clearly state this tracks **product collection**, not petting, and explains why.

## Bug 2: track_recipes_cooked

**Finding:** No `CookEvent`, `RecipeCraftedEvent`, or `FoodCookedEvent` exists in `src/shared/mod.rs` or `src/crafting/`.

**Action:** Left `EatFoodEvent` as the trigger. Updated the section header and doc comment to clearly state this tracks **food eaten**, not recipes cooked, and explains why.

## Files Changed

- `src/economy/play_stats.rs` — updated comments for both trackers only; no logic changed.
