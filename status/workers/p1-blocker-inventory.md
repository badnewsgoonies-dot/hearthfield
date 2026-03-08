# P1 Blocker Inventory — Source Assets Needed for Atlas Gaps

Generated: 2026-03-08

The LimeZu Modern Farm v1.2 source pack is NOT currently present on disk.
All source filenames below come from `docs/sprite-mapping-base-game.md`.

---

## P1 Gap 1: character_actions.png

- **Atlas:** 96x576px, 2 columns x 12 rows @ 48x48 = 24 slots
- **Current:** All 24 slots exist (file is 96x576), but content quality/completeness unknown
- **Layout (from `src/player/tool_anim.rs`):**

| Index Range | Tool        | Constant            |
|-------------|-------------|---------------------|
| 0-3         | Hoe         | ACTION_HOE_BASE     |
| 4-7         | WateringCan | ACTION_WATER_BASE   |
| 8-11        | Axe         | ACTION_AXE_BASE     |
| 12-15       | Pickaxe     | ACTION_PICK_BASE    |
| 16-19       | FishingRod  | ACTION_FISH_BASE    |
| 20-23       | Scythe      | ACTION_SCYTHE_BASE  |

Each tool uses 4 consecutive frames (indices base+0 through base+3).
Sprite flip_x handles left-facing; only one direction set needed.

- **Source files needed from LimeZu Modern Farm v1.2:**
  - `Characters_16x16/Farmer_1_Watering_56_frames_16x16.png` -> indices 4-7 (watering animation, extract 4 key frames)
  - `Characters_16x16/Farmer_1_Chopping_40_frames_16x16.png` -> indices 8-11 (axe chop animation, extract 4 key frames)
  - `Characters_16x16/Farmer_1_Dig_36_frames_16x16.png` -> indices 0-3 (hoe dig), indices 12-15 (pickaxe swing — share dig motion)
  - `Characters_16x16/Farmer_1_Fishing_128_frames_16x16.png` -> indices 16-19 (fishing cast, extract 4 key frames)
  - `Characters_16x16/Farmer_1_Harvesting_36_frames_16x16.png` -> indices 20-23 (scythe harvest, extract 4 key frames)

- **Notes:** LimeZu frames are 16x16 but the atlas uses 48x48 frames. Each source animation has many more frames than needed (36-128); the worker must select 4 representative keyframes per tool and upscale/composite them into 48x48 cells. The atlas already exists at the correct dimensions, so the question is whether the current content is placeholder or properly extracted.

- **Alternative sources if LimeZu unavailable:** Create simple 4-frame tool swing animations programmatically (rotate a tool sprite overlay on the character base). Or use any farming game sprite pack with tool animations at 16x16 or 48x48.

---

## P1 Gap 2: fishing_atlas.png

- **Atlas:** 128x96px, 8 columns x 6 rows @ 16x16 = 48 slots
- **Current:** File exists at correct dimensions (128x96). The comment in `src/fishing/mod.rs` claims all 48 slots are assigned. Actual pixel fill unknown — likely many are transparent placeholders.
- **Slot map (from `src/fishing/mod.rs` lines 39-44):**

| Row | Col 0 | Col 1 | Col 2 | Col 3 | Col 4 | Col 5 | Col 6 | Col 7 |
|-----|-------|-------|-------|-------|-------|-------|-------|-------|
| 0   | rod (0) | old rod (1) | bobber (2) | hook (3) | worm bait (4) | spinner lure (5) | tackle box (6) | bucket (7) |
| 1   | net (8) | crab trap (9) | cooler (10) | treasure chest (11) | wooden crate (12) | splash (13) | ripple (14) | fish shadow (15) |
| 2   | carp (16) | bluegill/herring (17) | perch (18) | trout (19) | catfish (20) | bass (21) | salmon (22) | sardine (23) |
| 3   | pike (24) | tuna (25) | swordfish (26) | eel (27) | anglerfish (28) | pufferfish (29) | koi (30) | legend (31) |
| 4   | seaweed (32) | coral (33) | shell (34) | pearl (35) | starfish (36) | driftwood (37) | message bottle (38) | old boot (39) |
| 5   | ancient coin (40) | sunken key (41) | fish bone (42) | crab (43) | lobster/crimsonfish (44) | octopus/goldfish (45) | squid/sunfish (46) | jellyfish/glacier+bullhead (47) |

- **Fish species using this atlas (from `src/data/fish.rs` — 28 species total):**
  All 28 fish reference sprite_index values that map into this atlas. The indices used are: 16-36, 44-47.

  NOTE: There are INDEX COLLISIONS in fish.rs:
  - Index 45: both `octopus` and `goldfish`
  - Index 46: both `squid` and `sunfish`
  - Index 47: both `glacier_fish` and `bullhead`
  - Index 32: `koi` (fish.rs says 32, but atlas comment says koi=30)

  This is a DATA BUG — multiple fish share sprites. However, the fishing_atlas is only used for the fishing minigame rendering, not for inventory icons (items_atlas handles that).

- **Source files needed from LimeZu Modern Farm v1.2:**
  - `Single_Files_16x16/Pickup_Items_16x16/Pickup_Fishing_Blue_Fish_16x16.png` -> base fish sprite template
  - Other fish pickup sprites from `Pickup_Items_16x16/` directory (any fish-themed sprites)
  - Equipment sprites (rods, bobbers, tackle) are not in standard LimeZu pack — need custom or placeholder

- **What needs filling:**
  - Row 0 (indices 0-7): Fishing equipment icons (rod, bobber, bait, tackle, bucket) — likely need custom sprites, LimeZu has no fishing equipment
  - Row 1 (indices 8-15): More equipment + FX (net, crab trap, splash effects) — mostly custom
  - Rows 2-3 (indices 16-31): Fish species sprites — need 16 distinct fish icons. LimeZu only provides ~1-2 fish pickup sprites
  - Row 4 (indices 32-39): Treasure/junk items (seaweed, coral, shells) — some may come from props
  - Row 5 (indices 40-47): More catches (crabs, squid, jellyfish) — mostly custom

- **Alternative sources:** OpenGameArt fishing sprite packs, or procedurally generate simple colored fish silhouettes (the minigame display is small enough that basic shapes work).

---

## P1 Gap 3: items_atlas.png

- **Atlas:** 208x160px, 13 columns x 10 rows @ 16x16 = 130 slots
- **Current:** File exists at correct dimensions. Claimed 57/130 filled — but based on the code, far more indices are USED.
- **ALL items and their sprite_index values (from `src/data/items.rs`):**

### Indices actually referenced by items in code:

| Index | Items Using It | Category |
|-------|---------------|----------|
| 0 | turnip_seeds, koi | Seed, Fish |
| 1 | potato_seeds, ghostfish | Seed, Fish |
| 2 | cauliflower_seeds, stonefish | Seed, Fish |
| 3 | strawberry_seeds, ice_pip | Seed, Fish |
| 4 | melon_seeds, lava_eel | Seed, Fish |
| 5 | tomato_seeds, bouquet | Seed, Special |
| 6 | blueberry_seeds, mermaid_pendant | Seed, Special |
| 7 | corn_seeds, toast | Seed, Food |
| 8 | eggplant_seeds, porridge | Seed, Food |
| 9 | pumpkin_seeds | Seed |
| 10 | cranberry_seeds | Seed |
| 11 | yam_seeds | Seed |
| 12 | wheat_seeds | Seed |
| 13 | coffee_beans | Seed |
| 14 | ancient_seeds | Seed |
| 20 | turnip | Crop |
| 21 | potato | Crop |
| 22 | cauliflower | Crop |
| 23 | strawberry | Crop |
| 24 | melon | Crop |
| 25 | tomato | Crop |
| 26 | blueberry | Crop |
| 27 | corn | Crop |
| 28 | eggplant | Crop |
| 29 | pumpkin | Crop |
| 30 | cranberry | Crop |
| 31 | yam | Crop |
| 32 | wheat | Crop |
| 33 | coffee | Crop |
| 34 | ancient_fruit | Crop |
| 35 | bat_wing | CraftingMaterial |
| 36 | oak_resin, furnace | CraftingMaterial, Furniture |
| 37 | preserves_jar | Furniture |
| 38 | cheese_press | Furniture |
| 39 | loom | Furniture |
| 40 | egg, keg | AnimalProduct, Furniture |
| 41 | large_egg, lightning_rod | AnimalProduct, Furniture |
| 42 | milk, mayonnaise_machine | AnimalProduct, Furniture |
| 43 | large_milk, crab_pot | AnimalProduct, Furniture |
| 44 | wool, torch | AnimalProduct, Furniture |
| 45 | cheese | ArtisanGood |
| 46 | cloth, wooden_sign | ArtisanGood, Furniture |
| 47 | mayonnaise, tapper | ArtisanGood, Furniture |
| 48 | bee_house | Furniture |
| 49 | recycling_machine | Furniture |
| 50 | sardine (item) | Fish |
| 51 | herring (item) | Fish |
| 52 | bass (item) | Fish |
| 53 | trout (item) | Fish |
| 54 | salmon (item) | Fish |
| 55 | catfish (item) | Fish |
| 56 | carp (item) | Fish |
| 57 | pike (item) | Fish |
| 58 | perch (item) | Fish |
| 59 | eel (item) | Fish |
| 60 | tuna (item) | Fish |
| 61 | swordfish (item) | Fish |
| 62 | sturgeon (item) | Fish |
| 63 | pufferfish (item) | Fish |
| 64 | octopus (item) | Fish |
| 65 | squid (item) | Fish |
| 66 | anglerfish (item) | Fish |
| 67 | legend_fish (item) | Fish |
| 68 | glacier_fish (item) | Fish |
| 69 | crimson_fish (item) | Fish |
| 70 | stone | Mineral |
| 71 | copper_ore | Mineral |
| 72 | iron_ore | Mineral |
| 73 | gold_ore | Mineral |
| 74 | coal | Mineral |
| 75 | quartz | Gem |
| 76 | amethyst, bomb | Gem, CraftingMaterial |
| 77 | emerald, cherry_bomb | Gem, CraftingMaterial |
| 78 | ruby, animal_chicken | Gem, Special |
| 79 | diamond, animal_cow | Gem, Special |
| 80 | copper_bar, animal_sheep | CraftingMaterial, Special |
| 81 | iron_bar, animal_goat, building_coop/big_coop/deluxe_coop | CraftingMaterial, Special |
| 82 | gold_bar, animal_duck, building_barn/big_barn/deluxe_barn | CraftingMaterial, Special |
| 83 | iridium_bar, animal_rabbit, recipe_books | CraftingMaterial, Special |
| 84 | animal_pig, chest | Special, Furniture |
| 85 | animal_horse, fence | Special, Furniture |
| 86 | animal_cat, wood_path | Special, Furniture |
| 87 | animal_dog, stone_path | Special, Furniture |
| 88 | scarecrow | Furniture |
| 89 | basic_sprinkler, quality_sprinkler | Furniture |
| 90 | fried_egg | Food |
| 91 | baked_potato | Food |
| 92 | salad | Food |
| 93 | cheese_omelette | Food |
| 94 | pancakes | Food |
| 95 | fish_stew, corn_chowder | Food |
| 96 | pumpkin_soup | Food |
| 97 | cranberry_sauce, fruit_salad | Food |
| 98 | cooked_fish | Food |
| 99 | bread | Food |
| 100 | pizza, goat_milk | Food, AnimalProduct |
| 101 | truffle_risotto, spaghetti, duck_egg | Food, AnimalProduct |
| 102 | melon_smoothie, ice_cream, rabbit_foot | Food, AnimalProduct |
| 103 | blueberry_pie, cake, large_cheese | Food, ArtisanGood |
| 104 | cookie, slime_jelly | Food, CraftingMaterial |
| 105 | turnip_soup, crab_shell | Food, CraftingMaterial |
| 106 | strawberry_jam, geode | Food, Mineral |
| 107 | cauliflower_gratin, refined_quartz | Food, Mineral |
| 108 | sardine_toast, tree_seed | Food, Seed |
| 109 | corn_tortilla, wild_berry | Food, Crop |
| 110 | blueberry_muffin, wood, oil_maker | Food, CraftingMaterial, Furniture |
| 111 | pumpkin_bread, hardwood, blueberry_jelly | Food, CraftingMaterial, ArtisanGood |
| 112 | cranberry_tart, fiber, strawberry_jelly | Food, CraftingMaterial, ArtisanGood |
| 113 | yam_pudding, sap, melon_jelly | Food, CraftingMaterial, ArtisanGood |
| 114 | stuffed_eggplant, pine_tar, apple_jelly | Food, CraftingMaterial, ArtisanGood |
| 115 | coffee_pudding, maple_syrup, ancient_jelly | Food, ArtisanGood |
| 116 | smoked_eel, slime, blueberry_wine | Food, CraftingMaterial, ArtisanGood |
| 117 | truffle_pasta, strawberry_wine | Food, ArtisanGood |
| 118 | tuna_tartare, melon_wine | Food, ArtisanGood |
| 119 | eggplant_stir_fry, ancient_fruit_wine | Food, ArtisanGood |
| 120 | hay, truffle_oil | Special, ArtisanGood |
| 121 | bait, pickled_* (8 items) | Special, ArtisanGood |
| 122 | tackle, beer, pale_ale, mead | Special, ArtisanGood |
| 123 | rice, pumpkin_juice, apple_cider | Crop, ArtisanGood |
| 124 | truffle, oil | Crop, ArtisanGood |
| 125 | wheat_flour, honey | ArtisanGood |
| 126 | sugar, crab | ArtisanGood, Fish |
| 127 | grilled_fish, goldfish (item) | Food, Fish |
| 128 | sashimi, sunfish (item) | Food, Fish |
| 129 | roasted_pumpkin, bullhead (item) | Food, Fish |

**NOT USED (empty indices):** 15, 16, 17, 18, 19

That's only 5 truly unused indices out of 130. The indices 0-14 and 15-19 are the gap:
- **Indices 15-19:** Completely empty — no item references these 5 slots.
- All other indices (0-14, 20-129) are used by at least one item.

**CRITICAL FINDING:** The "57/130 filled" claim is outdated or wrong. The code references **125 out of 130 indices** (indices 0-14, 20-129). However, many indices are SHARED by multiple items that need different sprites (e.g., index 0 = turnip_seeds AND koi). There are **~60 index collisions** where multiple items share the same sprite_index. This means either:
1. Many items display the wrong sprite (a koi fish shows as turnip seeds), or
2. The atlas needs to be enlarged to give each item a unique slot.

**Indices with most severe collisions (different categories sharing one sprite):**
- 0-8: Seeds collide with pond/mine fish and special items
- 35-49: CraftingMaterial/Furniture collisions
- 76-89: Gems/bombs, animals/bars, buildings/equipment all tangled
- 100-129: Foods/ArtisanGoods/CraftingMaterials heavily overlapping

- **Source files needed from LimeZu Modern Farm v1.2:**
  - `Single_Files_16x16/Pickup_Items_16x16/Pickup_Crop_Turnip_16x16.png` (and one per crop — 15 crop pickup sprites)
  - `Single_Files_16x16/Pickup_Items_16x16/Pickup_Fishing_Blue_Fish_16x16.png` (fish item icon template)
  - `Props_and_Buildings_16x16/Egg_*.png` (egg variants for animal products)
  - `Props_and_Buildings_16x16/Milk_Can_*.png` (milk items)
  - Seed packet sprites (may need to be custom — LimeZu may not have seed bag icons)
  - Mining ore/gem sprites from `Props_and_Buildings_16x16/Rock_*.png` or similar
  - Food/cooking sprites (LimeZu may not have cooked food icons — likely need custom)

- **Alternative sources:** OpenGameArt item icon packs, or generate simple colored squares with text labels as placeholders. The most critical sprites to get right are the ones players see most: crops (20-34), fish (50-69), minerals (70-74), and food (90-99).

---

## P1 Gap 4: tools.png

- **Atlas:** 96x96px, 6 columns x 6 rows @ 16x16 = 36 slots
- **Current:** File exists at correct dimensions.
- **Usage:** Loaded in `src/world/objects.rs` (line 207-215) into `ObjectAtlases.tools_image`/`tools_layout`, but **NOT REFERENCED ANYWHERE ELSE IN THE CODEBASE**. The tools atlas is a dead resource — no rendering code reads from it.
- **The HUD tool display is TEXT-ONLY** (see `src/ui/hud.rs` line 845-866) — it just prints the tool name as a string, not a sprite icon.
- **Tool animations** use `character_actions.png` (48x48 frames), NOT `tools.png`.

**CONCLUSION:** tools.png is currently unused infrastructure. It was loaded in anticipation of a future HUD icon feature or inventory tool display, but no code path renders from it.

- **Expected layout (inferred from 6 tools x 5 tiers + 6 spare):**

| Row | Col 0 (Basic) | Col 1 (Copper) | Col 2 (Iron) | Col 3 (Gold) | Col 4 (Iridium) | Col 5 (spare) |
|-----|---------------|----------------|--------------|--------------|-----------------|---------------|
| 0   | Hoe           | Hoe            | Hoe          | Hoe          | Hoe             | —             |
| 1   | WateringCan   | WateringCan    | WateringCan  | WateringCan  | WateringCan     | —             |
| 2   | Axe           | Axe            | Axe          | Axe          | Axe             | —             |
| 3   | Pickaxe       | Pickaxe        | Pickaxe      | Pickaxe      | Pickaxe         | —             |
| 4   | FishingRod    | FishingRod     | FishingRod   | FishingRod   | FishingRod      | —             |
| 5   | Scythe        | Scythe         | Scythe       | Scythe       | Scythe          | —             |

That would be 30 tool icons + 6 spare slots = 36 total. But this is speculation — no code maps tool+tier to a specific index.

- **Source files needed from LimeZu Modern Farm v1.2:**
  - LimeZu Modern Farm v1.2 does NOT include tool icon sprites. The pack focuses on farm environments, crops, animals, and buildings.
  - No exact source files available in the LimeZu pack for tool icons.

- **Alternative sources:**
  - Create simple 16x16 tool silhouettes (hoe, watering can, axe, pickaxe, rod, scythe) with color tinting per tier (grey=basic, copper, silver, gold, purple=iridium)
  - OpenGameArt RPG item icon packs often include farming tools
  - Port from the character_actions.png frames — extract the tool portion of each 48x48 tool animation frame and downscale to 16x16

---

## Summary — Priority Order for Art Work

| Priority | Atlas | Blocking? | Effort | Notes |
|----------|-------|-----------|--------|-------|
| **HIGH** | items_atlas.png | YES — 60+ index collisions cause wrong sprites | Large | Need atlas expansion or collision resolution; ~125/130 indices used but many shared |
| **MEDIUM** | fishing_atlas.png | PARTIAL — minigame works but sprites may be placeholder | Medium | 48 slots all assigned in code; need actual fish/equipment pixel art |
| **MEDIUM** | character_actions.png | PARTIAL — animations exist but may be placeholder quality | Medium | 24 slots, need to extract/composite from LimeZu animation sheets |
| **LOW** | tools.png | NO — completely unused, dead code | Low | No rendering code references it; can defer indefinitely |

## LimeZu Modern Farm v1.2 Source Files Summary

**Files the spec names that are needed:**

### For character_actions.png:
1. `Characters_16x16/Farmer_1_Watering_56_frames_16x16.png`
2. `Characters_16x16/Farmer_1_Chopping_40_frames_16x16.png`
3. `Characters_16x16/Farmer_1_Dig_36_frames_16x16.png`
4. `Characters_16x16/Farmer_1_Fishing_128_frames_16x16.png`
5. `Characters_16x16/Farmer_1_Harvesting_36_frames_16x16.png`

### For items_atlas.png (crop pickups):
6. `Single_Files_16x16/Pickup_Items_16x16/Pickup_Crop_Turnip_16x16.png`
7. `Single_Files_16x16/Pickup_Items_16x16/Pickup_Crop_Tomato_16x16.png`
8. (one per each of 15 crop types + fish + eggs + milk)

### For fishing_atlas.png:
9. `Single_Files_16x16/Pickup_Items_16x16/Pickup_Fishing_Blue_Fish_16x16.png`

### For tools.png:
10. **No LimeZu source available** — need custom or alternative pack

**IMPORTANT:** The LimeZu Modern Farm v1.2 pack is NOT currently on disk in this environment. All source filenames above are from the integration spec (`docs/sprite-mapping-base-game.md`). The pack must be downloaded/copied into the workspace before any art extraction work can proceed.

### Data Bugs Found During Research

1. **items_atlas.png index collisions:** ~60 pairs of items share the same sprite_index, causing wrong sprites to display. Most severe: seeds sharing indices with fish (0-8), animal products sharing with furniture (40-44), gems sharing with bombs/animals (76-87).
2. **fish.rs index collisions:** goldfish/octopus share index 45, sunfish/squid share 46, bullhead/glacier_fish share 47. These are fishing_atlas indices used during the minigame.
3. **tools.png dead code:** Atlas loaded but never rendered from. The `ObjectAtlases.tools_image`/`tools_layout` fields are written but never read.
