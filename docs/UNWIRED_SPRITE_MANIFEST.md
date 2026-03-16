# UNWIRED SPRITE MANIFEST
## Every unused asset, its exact game data connection, and wiring plan
### Hearthfield — 62,800 LOC, 172 PNG assets, 117 unwired

---

## SECTION 1: CROP FAMILY — 53 unwired sprites

### 1A. Crop growth sprites on disk but NOT in dynamic loader (6 files)

These PNGs have proper growth-stage frames ready to render, but `src/farming/mod.rs` `crop_sheets` doesn't load them. Adding one line each would give these crops per-stage visual progression instead of generic plants.png fallback.

| File | Dimensions | Frames | Matching CropDef? | Wiring |
|------|-----------|--------|-------------------|--------|
| crop_carrot.png | 112×32 | 7×2=14 | **NO** — "carrot" not in crops.rs | Add CropDef + add to crop_sheets |
| crop_grape.png | 112×96 | 7×6=42 | **NO** — "grape" not in crops.rs | Add CropDef + add to crop_sheets |
| crop_onion.png | 112×64 | 7×4=28 | **NO** — "onion" not in crops.rs | Add CropDef + add to crop_sheets |
| crop_pepper.png | 112×32 | 7×2=14 | **NO** — "pepper" not in crops.rs | Add CropDef + add to crop_sheets |
| crop_pineapple.png | 112×64 | 7×4=28 | **NO** — "pineapple" not in crops.rs | Add CropDef + add to crop_sheets |
| crop_radish.png | 112×32 | 7×2=14 | **NO** — "radish" not in crops.rs | Add CropDef + add to crop_sheets |

**Key insight:** None of these 6 crops exist in game data at all. They have full growth-stage art but no CropDef, no ItemDef, no shop listing. Each needs: CropDef in crops.rs, ItemDef for seeds + harvest in items.rs, shop listing in shops.rs, one line in crop_sheets loader.

### 1B. Crops that EXIST in game data but use generic plants.png fallback (9 crops)

These crops have CropDef entries and are plantable/harvestable, but their growth stages render with the 12-sprite generic plants.png atlas instead of per-crop art.

| Crop ID | Has crop_*.png? | Has Pickup icon? | Fix |
|---------|----------------|-----------------|-----|
| potato | **NO** | ✓ Pickup_Crop_Pumpkin? (no exact match) | Need crop_potato.png generated |
| blueberry | **NO** | **NO** | Need crop_blueberry.png generated |
| melon | **NO** | **NO** (watermelon exists) | Need crop_melon.png generated |
| eggplant | **NO** | **NO** | Need crop_eggplant.png generated |
| cranberry | **NO** | **NO** | Need crop_cranberry.png generated |
| yam | **NO** | **NO** | Need crop_yam.png generated |
| hops | **NO** | **NO** | Need crop_hops.png generated |
| ancient_fruit | **NO** | **NO** | Need crop_ancient_fruit.png generated |
| coffee | ✓ crop_coffee.png | ✓ Pickup_Crop_Coffee | **Already loaded** — this one works correctly |

Wait — coffee IS in the loader. Rechecking... the grep matched "coffee_beans" (the seed ID) not "coffee" (the crop ID). Actual missing from loader: potato, blueberry, melon, eggplant, cranberry, yam, hops, ancient_fruit. **8 crops render generically.**

### 1C. Pickup crop icons — 38 files, ZERO loaded (items/Pickup_Crop_*.png)

Every harvested crop displays via `items_atlas.png` sprite index. These 38 individual 16×16 PNGs with proper per-crop art (including 19 rare/quality variants) are never used anywhere.

**Full mapping to game crops:**

| Pickup sprite | Game crop match | Normal | Rare |
|--------------|----------------|--------|------|
| Pickup_Crop_Turnip | turnip ✓ | ✓ | ✓ |
| Pickup_Crop_Cauliflower | cauliflower ✓ | ✓ | ✓ (typo: "Cauliflowert") |
| Pickup_Crop_Strawberry | strawberry ✓ | ✓ | ✓ |
| Pickup_Crop_Tomato | tomato ✓ | ✓ | ✓ |
| Pickup_Crop_Corn | corn ✓ | ✓ | ✓ |
| Pickup_Crop_Pumpkin | pumpkin ✓ | ✓ | ✓ |
| Pickup_Crop_Watermelon | watermelon ✓ | ✓ | ✓ |
| Pickup_Crop_Coffee | coffee ✓ | ✓ | ✓ |
| Pickup_Crop_Radish | **NO crop** | ✓ | ✓ |
| Pickup_Crop_Onion | **NO crop** | ✓ | ✓ |
| Pickup_Crop_Grape | **NO crop** | ✓ | ✓ |
| Pickup_Crop_Carrot | **NO crop** | ✓ | ✓ |
| Pickup_Crop_Pineapple | **NO crop** | ✓ | ✓ |
| Pickup_Crop_Chili_Pepper | **NO crop** | ✓ | ✓ |
| Pickup_Crop_Cabbage | **NO crop** | ✓ | ✓ |
| Pickup_Crop_Cotton | **NO crop** | ✓ | ✓ |
| Pickup_Crop_Grain | wheat? (close) | ✓ | ✓ |
| Pickup_Crop_Prickly_Pear | **NO crop** | ✓ | ✓ |
| Pickup_Crop_Zucchini | **NO crop** | ✓ | ✓ |

**Wiring plan:** For matched crops (8 pairs), load individual Pickup PNGs as item icons instead of items_atlas indices. Requires: per-crop image handle in ItemDef or a lookup table in UI rendering. For rare variants, display when crop quality ≥ gold.

### 1D. pickup_items.png atlas — 140 tiles, UNUSED

`assets/sprites/pickup_items.png` (224×160, 14×10 grid) contains 140 pickup item sprites — appears to be a consolidated atlas of all item pickups. Completely unreferenced in code. Could replace items_atlas.png for ground-drop rendering or supplement it.

---

## SECTION 2: NPC FAMILY — 2 unwired sprites + 1 duplicate

### 2A. Sprite assignments

| NPC | Role | Sprite | Match quality | Issue |
|-----|------|--------|--------------|-------|
| margaret | Baker | npc_miner.png | ✗ **Wrong** | Baker using miner sprite |
| bjorn | Carpenter | npc_miner.png | ✗ **DUPLICATE** | Same sprite as Margaret |
| marco | Chef | npc_traveler.png | ~ Acceptable | Worldly traveler fits Italian chef |
| lily | Florist | npc_child.png | ~ Acceptable | Young/youthful fits if she's young |
| old_tom | Fisherman | npc_pirate.png | ✓ Good | Seafaring pirate fits old fisherman |
| elena | Blacksmith | npc_blacksmith.png | ✓ Perfect | Exact match |
| mira | Merchant | npc_merchant.png | ✓ Perfect | Exact match |
| doc | Doctor | npc_healer.png | ✓ Good | Healer ≈ doctor |
| mayor_rex | Mayor | npc_noble.png | ✓ Good | Noble fits mayor |
| sam | Musician | npc_scholar.png | ~ Weak | Scholar for musician is a stretch |
| nora | Botanist | npc_farmer.png | ✓ Good | Farmer fits botanist |

### 2B. Unused NPC sprites

| Sprite | Best reassignment |
|--------|------------------|
| **npc_guard.png** | → bjorn (Carpenter — sturdy/bulky fits) |
| **npc_mage.png** | → sam (Musician — artistic/expressive fits) OR margaret (Baker — if mage reads as "magical with food") |

### 2C. Wiring fix

**File:** `src/npcs/definitions.rs` lines 18-29
```rust
// CHANGE:
"margaret" => "sprites/npcs/npc_mage.png",      // was npc_miner
"bjorn" => "sprites/npcs/npc_guard.png",         // was npc_miner (DUPLICATE)
"sam" => "sprites/npcs/npc_mage.png",            // was npc_scholar — OR swap with margaret
```

**Exact code location:** `src/npcs/definitions.rs::npc_sprite_file()` — 3 line changes.

---

## SECTION 3: TREE FAMILY — 4 unwired sprites

### 3A. Unused tree PNGs

| File | Size | Style | Best biome mapping |
|------|------|-------|--------------------|
| tree_birch_green.png | 48×80 | Slender birch, bright green | Forest, Farm (spring) |
| tree_oak_brown.png | 80×96 | Broad oak, autumn brown | Farm/Town (fall), Forest |
| tree_oak_green.png | 80×96 | Broad oak, summer green | Farm/Town (spring/summer) |
| tree_pine_blue.png | 64×96 | Tall pine, blue-tinted | Snow Mountain, Deep Forest |

### 3B. Current rendering

All trees use `tree_sprites.png` (128×96 = 8×6 tiles). The atlas has:
- Row 0 (indices 0-3): Deciduous tree × 4 seasons (Sp/Su/Fa/Wi)
- Row 4 (indices 4-7): Pine tree × 4 seasons

Only `WorldObjectKind::Tree` (index 0+season) and `WorldObjectKind::Pine` (index 4+season) exist.

### 3C. Wiring plan

**Option A (per-map composite sprites):** Load individual tree PNGs as alternate rendering for specific maps. SnowMountain trees use tree_pine_blue.png. Forest uses tree_birch_green.png for variety. Town uses tree_oak_green.png.

**Option B (new WorldObjectKind variants):** Add `Birch`, `Oak`, `BluePine` to WorldObjectKind enum. Each loads its individual PNG. Place on appropriate maps.

**Option A is simpler** — keep existing WorldObjectKind, but in `spawn_world_object_sprite`, check `player_state.current_map` and swap the image handle for map-appropriate variants. 

**Wiring location:** `src/world/objects.rs::spawn_world_object_sprite()` around line 700.

---

## SECTION 4: FENCE FAMILY — 2 unwired tilesets

### 4A. fences.png (64×64 = 4×4 = 16 tiles)

**STATUS: Loaded into GPU memory, has autotile code, renders for PLAYER-PLACED fences on the farm.** But NO fences exist as map decoration on Town, TownWest, or any other exterior map. The tileset + autotile system is fully wired for the farming system but not for world decoration.

**Wiring plan:** In `town_buildings()` or `town_decorations()`, add fence segments as placed farm objects around NPC house yards. Requires either extending WorldObjectKind with Fence, or placing them through the FarmState system with fixed positions.

### 4B. modern_farm_fences.png (512×272 = 32×17 = 544 tiles)

**STATUS: Completely unwired.** This is a massive, high-quality fence tileset with stone walls, wooden fences, gates, corners, T-junctions — the full autotile set for multiple fence styles. Never loaded, never referenced.

**Wiring plan:** This is a significant upgrade over fences.png (16 tiles → 544 tiles). Could replace fences.png entirely, or be used alongside it for different fence materials. Requires: loading the atlas, updating the autotile lookup to select from different material rows, adding fence style to FarmObject::Fence or creating world-decoration fence entities.

---

## SECTION 5: UI FAMILY — 29 unwired sprites

### 5A. Custom cursors (3 files, zero used)

| File | Purpose | Wiring target |
|------|---------|--------------|
| cursor_default.png | Default pointer | `CursorIcon` or Bevy window settings |
| cursor_holding.png | When holding/dragging item | Inventory drag state |
| cursor_pointing.png | When hovering interactable | NPC/object hover state |

**Wiring:** Bevy supports custom cursors via `Window.cursor.icon` or by hiding system cursor and spawning a sprite entity that follows mouse position. The tool tile cursor system in `src/player/tool_anim.rs` already tracks cursor position.

### 5B. Dialog boxes (3 unused variants)

| File | Size | Currently used? |
|------|------|----------------|
| dialog_box_big.png | 176×48 | ✓ YES — the only one used |
| dialog_box_medium.png | 128×48 | ✗ unused |
| dialog_box_small.png | 112×48 | ✗ unused |
| dialog_box.png | 48×48 | ✗ unused |

**Wiring:** Use medium for toast messages, small for item pickup names, base for tooltips. Target: `src/ui/toast.rs`, `src/ui/dialogue_box.rs`.

### 5C. Premade dialogs (3 files, zero used)

| File | Size |
|------|------|
| premade_dialog_big.png | 304×64 |
| premade_dialog_medium.png | 240×64 |
| premade_dialog_small.png | 176×64 |

These appear to be pre-rendered dialog boxes with built-in styling. Could replace the current programmatic dialog rendering for a more polished look.

### 5D. Emote/emoji sheets (2 files, zero used)

| File | Size | Tiles |
|------|------|-------|
| emoji_spritesheet.png | 160×608 | 10×38 = 380 emoji icons |
| emotes.png | 160×480 | 10×30 = 300 emote icons |

**Current system:** `src/npcs/emotes.rs` generates procedural 16×16 emotes from code (Heart, Happy, Neutral, Sad, etc.). These two sprite sheets contain 680 hand-drawn emote/emoji icons that could replace or supplement the procedural ones.

**Wiring:** Replace `make_emote_image()` procedural generation with atlas lookups into these sheets. Map `EmoteKind::Heart` → specific atlas index, etc.

### 5E. Speech bubble (1 file, unused)

`speech_bubble.png` — 448×192 (28×12 tiles). A large, detailed speech bubble sprite sheet with multiple styles, tails, and sizes.

**Wiring:** Could be used for NPC thought bubbles above their heads when they have something to say, or as an alternative to the dialog box for short exclamations.

### 5F. Icon sheets (4 files, zero used)

| File | Size | Content |
|------|------|---------|
| icons.png | 288×48 | 18×3 = 54 general icons |
| icons_happiness.png | 96×32 | 6×2 = 12 mood icons |
| icons_special.png | 112×64 | 7×4 = 28 special icons |
| icons_white.png | 96×48 | 6×3 = 18 white outline icons |

**Wiring targets:** HUD status indicators, relationship screen mood icons, buff/debuff icons, minimap legend, settings menu icons.

### 5G. Inventory UI (4 files, zero used)

| File | Size | Content |
|------|------|---------|
| inventory_blocks.png | 144×144 | 9×9 grid/slot backgrounds |
| inventory_hearts.png | 112×336 | 7×21 heart fill states |
| inventory_hearts_light.png | 112×336 | Light-colored variant |
| inventory_spritesheet.png | 368×336 | 23×21 full inventory UI kit |

**Current:** Inventory and relationship screens are built with Bevy UI nodes + colored rectangles. These sprites would give them a hand-drawn pixel art look.

### 5H. Settings/buttons (4 files, zero used)

| File | Content |
|------|---------|
| buttons_19x26.png | Small vertical buttons |
| buttons_26x19.png | Small horizontal buttons |
| buttons_26x26.png | Square buttons |
| buttons_small.png | Compact buttons |
| settings_buttons.png | Settings-specific buttons (toggles, sliders) |
| settings_menu.png | Pre-rendered settings panel |

### 5I. Remaining UI

| File | Content | Wiring target |
|------|---------|--------------|
| dialog_continue_indicator.png | "Press to continue" arrow/icon | Dialog system next-page indicator |
| ui_spritesheet.png | 896×240 (56×15 = 840 tiles!) massive UI kit | Everything |
| weather_ui.png | Weather display frame | HUD weather panel background |

The **ui_spritesheet.png** alone has 840 UI tiles — borders, panels, buttons, tabs, scrollbars, health bars, inventory frames, everything a full game UI needs. This is the single largest unused asset.

---

## SECTION 6: TERRAIN FAMILY — 5 unwired tilesets

| File | Size | Tiles | Status |
|------|------|-------|--------|
| tilled_dirt_v2.png | 176×112 | 77 | Alternative dirt — unused |
| tilled_dirt_wide.png | 176×112 | 77 | Wide dirt tiles — unused |
| tilled_dirt_wide_v2.png | 176×112 | 77 | Wide dirt v2 — unused |
| modern_farm_autotiles.png | 192×896 | 672 | Complete autotile system — unused |
| modern_farm_fences.png | 512×272 | 544 | Full fence library — unused |
| bitmask_ref_1.png | 480×256 | — | Reference image, not game art |
| bitmask_ref_2.png | 480×256 | — | Reference image, not game art |

**modern_farm_autotiles.png** is the most significant: 672 tiles organized as a complete autotile ruleset for terrain transitions (grass↔dirt, grass↔water, etc.). This would replace the current manual transition index lookups in `dirt_grass_transition_index()` and `water_grass_transition_index()` with proper autotile rendering.

---

## SECTION 7: REMAINING GAME SPRITES — 3 unwired

| File | Size | Content | Wiring |
|------|------|---------|--------|
| palette.png | 16×7 | Color reference palette | Not game art — reference only |
| pickup_items.png | 224×160 | 140-tile item pickup atlas | Alternative to items_atlas for ground drops |
| tools_and_materials.png | 48×32 | 3×2 small tool/material icons | Could supplement tools.png for crafting UI |

---

## SECTION 8: SOURCE ORIGINALS — 12 files (reference copies)

The `_source_limezu/` folder contains original unmodified copies of sprites that were later edited and placed in the main sprites/ folder. These are backups, not meant to be loaded. **No action needed.**

---

## SUMMARY STATISTICS

| Category | Total on disk | Loaded in code | Unwired | Wirable without new art |
|----------|--------------|---------------|---------|------------------------|
| Crop growth PNGs | 15 | 9 | 6 | 0 (no matching CropDef) |
| Crop pickup icons | 38 | 0 | 38 | 16 (8 matched crops × 2) |
| NPC sprites | 12 | 10 | 2 | 2 (reassign to NPCs) |
| Tree individuals | 4 | 0 | 4 | 4 (biome mapping) |
| Fence tilesets | 2 | 1 (partial) | 1.5 | 1.5 (place on maps) |
| UI sprites | 32 | 3 | 29 | 29 (all usable) |
| Terrain extras | 7 | 0 | 5+2ref | 3 (dirt variants, autotile) |
| Game sprites | 3 | 0 | 2+1ref | 2 |
| Source originals | 12 | 0 | 0 | N/A (backups) |
| pickup_items atlas | 1 | 0 | 1 | 1 |
| **TOTAL** | **~126** | **~23** | **~88** | **~58 immediately wirable** |

---

## IMPLEMENTATION PRIORITY (for parallel dispatch)

### P0: Fix broken things (< 30 min total)
1. **NPC duplicate fix** — Margaret + Bjorn sharing npc_miner.png. 3 lines in definitions.rs.
2. **NPC unused sprite assignment** — Wire npc_guard.png and npc_mage.png to appropriate NPCs.

### P1: Wire existing matched assets (2-4 hours)
3. **Crop pickup icons for 8 matched crops** — Replace items_atlas indices with individual Pickup_Crop PNGs for turnip, cauliflower, strawberry, tomato, corn, pumpkin, watermelon, coffee.
4. **Tree biome variety** — Load 4 individual tree PNGs, render per-map.
5. **Place fences on town/farm exterior maps** — Use loaded fences.png tileset.
6. **Custom cursors** — Wire 3 cursor PNGs to game states.

### P2: Add new game content from existing art (4-8 hours)
7. **6 new crops** — carrot, grape, onion, pepper, pineapple, radish. Art exists (growth stages + pickup icons). Need CropDef, ItemDef, shop entries.
8. **Dialog box variants** — Use medium/small for toasts and tooltips.
9. **Emote sheet integration** — Replace procedural emotes with hand-drawn atlas.

### P3: System upgrades using unused tilesets (8-16 hours)
10. **modern_farm_fences.png** — Multi-material fence system.
11. **Tilled dirt variants** — Seasonal/quality soil visuals.
12. **UI spritesheet integration** — Replace programmatic UI with pixel art panels.

### P4: Major features (days)
13. **modern_farm_autotiles.png** — Full autotile terrain rendering.
14. **Complete UI overhaul** — Wire all 29 unused UI sprites.
15. **6 more potential crops** — Cabbage, Chili Pepper, Cotton, Grain, Prickly Pear, Zucchini.
