# Hearthfield Sprite Overhaul — Modern Farm Integration Spec

Source: LimeZu Modern Farm v1.2 (`16x16/` directory)
Target: `assets/sprites/` in the Hearthfield repo root
Tile size: 16×16, rendered at PIXEL_SCALE = 3.0 (nearest-neighbor)

## Orchestrator Instructions

This is an art integration pass. Workers extract sprites from Modern Farm, build texture atlases compatible with Bevy's atlas system, and wire them into the existing rendering code. The base game currently loads sprites as texture atlases — workers must match the existing atlas layout or update the atlas indices in the rendering code.

**After each step, audit from the player's perspective: boot the game, walk around, verify the sprite renders at the correct position, size, and animation frame. If it's not visible, it's not done.**

IMPORTANT: Do NOT modify src/shared/mod.rs (frozen contract).
Each worker edits ONLY files listed in their scope.

---

## Priority 1 — Player Character

**Source files:**
- `Characters_16x16/Farmer_1_16x16.png` (6144×832 — main spritesheet, all directions + idle/walk)
- `Characters_16x16/Farmer_1_Watering_56_frames_16x16.png`
- `Characters_16x16/Farmer_1_Chopping_40_frames_16x16.png`
- `Characters_16x16/Farmer_1_Dig_36_frames_16x16.png`
- `Characters_16x16/Farmer_1_Fishing_128_frames_16x16.png`
- `Characters_16x16/Farmer_1_Harvesting_36_frames_16x16.png`

**Target:** `assets/sprites/character_spritesheet.png` (replace existing)
**Code scope:** `src/player/spawn.rs`, `src/player/tool_anim.rs`, `src/player/movement.rs`

**Worker task:**
1. Extract idle (4 dir × 4 frames) and walk (4 dir × 4 frames) from Farmer_1_16x16.png
2. Build a combined atlas matching the existing layout the code expects, OR update atlas indices
3. Extract tool animations from the separate sheets
4. Verify: player spawns, walks 4 directions, tool animations play

---

## Priority 2 — Crops (15 types with growth stages)

**Source files (19 available, need 15):**
- `Crops_Growth_16x16/Turnip_Growth_Stages_16x16.png` → Spring
- `Crops_Growth_16x16/Carrot_Growth_Stages_16x16.png` → Spring (potato substitute)
- `Crops_Growth_16x16/Cauliflower_Growth_Stages_16x16.png` → Spring
- `Crops_Growth_16x16/Strawberry_Growth_Stages_16x16.png` → Spring
- `Crops_Growth_16x16/Watermelon_Growth_Stages_16x16.png` → Summer (melon)
- `Crops_Growth_16x16/Tomato_Growth_Stages_16x16.png` → Summer
- `Crops_Growth_16x16/Grape_Growth_Stages_16x16.png` → Summer (blueberry substitute)
- `Crops_Growth_16x16/Corn_Growth_Stages_16x16.png` → Summer
- `Crops_Growth_16x16/Pepper_Growth_Stages_16x16.png` → Fall (eggplant substitute)
- `Crops_Growth_16x16/Pumpkin_Growth_Stages_16x16.png` → Fall
- `Crops_Growth_16x16/Radish_Growth_Stages_16x16.png` → Fall (cranberry substitute)
- `Crops_Growth_16x16/Onion_Growth_Stages_16x16.png` → Fall (yam substitute)
- `Crops_Growth_16x16/Wheat_Growth_Stages_16x16.png` → Any season
- `Crops_Growth_16x16/Coffee_Growth_Stages_16x16.png` → Any season
- `Crops_Growth_16x16/Pineapple_Growth_Stages_16x16.png` → Any (ancient fruit substitute)

Each sheet is 112×64 = 7 stages wide × 4 rows (7 growth stages per crop)

**Target:** `assets/sprites/plants.png` (replace existing crop atlas)
**Code scope:** `src/farming/render.rs`, `src/data/crops.rs`

**Worker task:**
1. Combine 15 crop growth sheets into a single atlas
2. Map atlas indices to match CropDef data in `src/data/crops.rs`
3. Verify: plant seed, see stage 0 sprite, water daily, see growth stages advance visually

**Pickup item sprites for harvested crops:**
- `Single_Files_16x16/Pickup_Items_16x16/Pickup_Crop_Tomato_16x16.png` (etc, one per crop)
- Map these into the items atlas for inventory display

---

## Priority 3 — Animals

**Source files:**
- Chicken: `Animals_16x16/Chickens_and_Roosters/Chicken_White_16x16.png` (main)
  - Baby: `Animals_16x16/Chickens_and_Roosters/Chick_16x16.png`
- Cow: `Animals_16x16/Cows/Cow_16x16.png`
  - Baby: `Animals_16x16/Cows/Cow_Baby_16x16.png`
- Sheep: `Animals_16x16/Sheeps/Sheep_White_16x16.png`
  - Baby: `Animals_16x16/Sheeps/Sheep_Baby_White_16x16.png`
- Dog: `Animals_16x16/Dogs/Dog_Labrador_Brown_16x16.png`

**Target:** `assets/sprites/chicken.png`, `assets/sprites/cow.png` (replace existing)
**Code scope:** `src/animals/rendering.rs`, `src/animals/spawning.rs`

**Worker task:**
1. Extract walk/idle animations from each animal sheet
2. Build per-animal atlases matching existing code expectations
3. Verify: animals spawn, walk, and display correct baby/adult variants

---

## Priority 4 — Terrain & Tilesets

**Source files:**
- `Autotiles_16x16/Autotiles_Godot_16x16.png` — terrain autotile (grass, dirt, water edges)
- `Single_Files_16x16/Fences_16x16/Wooden_Fence_Type_3_Brown_*.png` — fences
- `Single_Files_16x16/Trees_16x16/Tree_Oak_Green_*.png` — trees (spring/summer)
- `Single_Files_16x16/Trees_16x16/Tree_Oak_Brown_*.png` — trees (fall)
- `Single_Files_16x16/Trees_16x16/Tree_Pine_Blue_*.png` — trees (winter)
- `Single_Files_16x16/Props_and_Buildings_16x16/Rock_*.png` — rocks for mining
- `Single_Files_16x16/Props_and_Buildings_16x16/Farmer_House_1_16x16.png`
- `Single_Files_16x16/Props_and_Buildings_16x16/Barn_Small_16x16.png`

**Target:** `assets/tilesets/grass.png`, `assets/sprites/tree_sprites.png`, etc
**Code scope:** `src/world/maps.rs`, `src/world/objects.rs`, `src/world/seasonal.rs`

**Worker task:**
1. Build terrain tileset from autotiles
2. Build seasonal tree variants (green/brown/blue for spring-summer/fall/winter)
3. Replace rock sprites for mining
4. Replace building sprites
5. Verify: walk between maps, see correct terrain, seasonal tree swaps work

---

## Priority 5 — Buildings & Animated Objects

**Source files:**
- `Animated_16x16/Animated_sheets_16x16/Farm_House_Door_16x16.png` — door open/close
- `Animated_16x16/Animated_sheets_16x16/Barn_Door_16x16.png`
- `Animated_16x16/Animated_sheets_16x16/Cheese_Machine_16x16.png` — processing machine
- `Animated_16x16/Animated_sheets_16x16/Sprinkler_16x16.png` — sprinkler animation
- `Animated_16x16/Animated_sheets_16x16/Well_16x16.png`
- `Animated_16x16/Animated_sheets_16x16/Stone_Oven_Smoke_16x16.png`

**Target:** Various sprite files in `assets/sprites/`
**Code scope:** `src/world/objects.rs`, `src/crafting/machines.rs`, `src/farming/sprinklers.rs`

---

## Priority 6 — Items Atlas

**Source files (pickup items for inventory display):**
All files in `Single_Files_16x16/Pickup_Items_16x16/`:
- 38 crop pickup sprites (19 normal + 19 rare variants)
- `Pickup_Fishing_Blue_Fish_16x16.png` + other fishing items
- Egg variants from `Props_and_Buildings_16x16/Egg_*.png`
- Milk cans from `Props_and_Buildings_16x16/Milk_Can_*.png`

**Target:** `assets/sprites/items_atlas.png`
**Code scope:** `src/data/items.rs`, `src/ui/inventory_screen.rs`

---

## Quantitative Targets (non-negotiable)

- [ ] 1 player character with idle + walk (4 dir) + 5 tool animations
- [ ] 15 crops with 7 growth stages each = 105 crop sprites
- [ ] 15 harvested crop item sprites
- [ ] 4 animal types with walk + idle + baby variant = 8 animal sheets minimum
- [ ] Terrain autotiles for grass/dirt/water
- [ ] 4 seasonal tree variants
- [ ] 6 building/prop sprites
- [ ] 6 animated object sheets (door, machine, sprinkler, well, oven)
- [ ] Items atlas with all harvestable/craftable items

## Validation

After each worker:
1. `cargo check` passes
2. `cargo test` passes
3. VISUAL CHECK: boot the game, walk around, verify sprites render correctly
4. No white rectangles, no missing textures, no wrong-size sprites
