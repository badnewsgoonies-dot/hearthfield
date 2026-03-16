# MASTER SPRITE AUDIT — Every Family, Every Gap, Every Improvement

## INVENTORY SUMMARY

- **172 PNG assets** on disk
- **~55 loaded in code** (32%)
- **~117 UNUSED** (68%) — massive untapped asset library

---

## FAMILY 1: WATER (ponds, rivers, ocean, fishing spots)

### Current state
- `tilesets/water.png` — 64×16 (4 tiles × 1 row). Only 4 water frames.
- Animation: `animate_water_tiles` cycles WaterBaseIndex through 4 frames at 0.4s intervals
- Water edge overlays: WaterEdgeOverlay with alpha pulse (4 phases)
- Fishing has: bobber sprite, landing splash, ambient ripple, bite splash (in `fishing/render.rs`)

### What's correct
- Water tile animation exists and cycles 4 frames ✓
- Edge blending with grass exists via WaterEdgeMask ✓
- Bobber has splash/ripple VFX ✓

### What's wrong or missing
- **Only 4 animation frames** for all water everywhere. Ocean, pond, river all look identical.
- **No depth variation** — shallow pond edges vs deep ocean center vs flowing river all use same tiles
- **No reflection/shimmer overlay** on water surface
- **No fish jumping** — rare ambient event of a fish sprite briefly arcing above water surface
- **No lily pads/reeds** for pond edges (differentiate pond from river from ocean)
- **No wave/surf animation** on beach ocean tiles
- **No splash when player walks near water** or steps on edge tiles
- **Fishing cast has no arc animation** — bobber appears instantly at target

### Implementation plan
1. **Differentiated water tinting** — color-multiply water tiles by map: blue-green for ocean, brown-green for pond, clear blue for mountain lake, dark for mine pools. Pure code change, no new assets.
2. **Ambient fish jump** — rare event (~1/200 frames) spawns a small arc particle above a random water tile. Uses fishing_atlas.png sprites already in game. Code only.
3. **Lily pad/reed objects** — place Bush WorldObjects at pond edges with green tint. Code only (object placement).
4. **Wave overlay for beach** — add a second animated overlay on Beach map water tiles with higher amplitude alpha pulse. Code change to `animate_water_tiles`.
5. **Fishing cast arc** — animate bobber position from player to target over 0.3s parabolic arc instead of instant placement. Modify `fishing/cast.rs`.
6. **Water edge reeds** — spawn thin green-tinted sprites at water/grass boundary tiles. Similar to existing grass_biome decorations.

### Assets needed: **NONE** — all achievable with existing sprites + code tinting/particles

---

## FAMILY 2: TREES (farm, forest, town, snow mountain)

### Current state
- `sprites/tree_sprites.png` — 128×96 (8×6 = 48 tiles). The ONLY tree atlas used.
- Two object types: `WorldObjectKind::Tree` and `WorldObjectKind::Pine`
- Both render at 2×3 tile size, health=10, same chop behavior
- Seasonal tint applied via `update_tree_sprites_on_season_change`
- Tree VFX: shake, flash, leaf burst, destruction poof (all in `tree_fx.rs`)

### UNUSED ASSETS ON DISK
- **tree_birch_green.png** (48×80) — completely unused
- **tree_oak_brown.png** (80×96) — completely unused
- **tree_oak_green.png** (80×96) — completely unused
- **tree_pine_blue.png** (64×96) — completely unused

These are 4 distinct, high-quality tree sprites sitting in assets/ doing nothing.

### What's wrong or missing
- **Only 2 visual tree types** (Tree, Pine) across the entire game
- **4 individual tree PNGs unused** — birch, oak brown, oak green, pine blue
- **No fruit tree visuals** — fruit trees exist in game data but use same generic tree sprite
- **No stump variety** — all stumps look the same
- **No seasonal tree variety** — tint changes but no leaf shape/density changes
- **No tree size variation** — all trees are exactly the same dimensions
- **Forest lacks understory** — no bushes at tree bases, no fallen logs, no mushroom clusters
- **Snow mountain has no snow-capped variants** — just tinted regular trees

### Implementation plan
1. **Load and use the 4 individual tree PNGs** — map them to biomes: birch→Forest, oak_brown→Farm/Town in Fall, oak_green→Farm/Town in Spring/Summer, pine_blue→SnowMountain. Requires new WorldObjectKind variants (Birch, Oak) or a per-map tree sprite selector.
2. **Fruit tree visual stages** — use tree_sprites atlas with a colored dot overlay when fruit is ready to harvest. Code-only addition to the seasonal render.
3. **Tree size variation** — randomize custom_size by ±15% per tree entity at spawn. One-line change in spawn code.
4. **Forest understory** — spawn Bush objects at 50% of tree base positions with dark green tint. Object placement code.
5. **Snow cap overlay** — in Winter on SnowMountain, add a white-tinted triangle sprite at tree tops. Seasonal system addition.

### Assets available but unused: **4 tree PNGs ready to integrate**

---

## FAMILY 3: CROPS (growth stages, harvest, items)

### Current state
- `sprites/plants.png` — 96×32 (6×2 = 12 generic crop tiles). Fallback.
- **15 individual crop_*.png files** — loaded dynamically per crop ID, 7 columns × 2-6 rows each
- Growth stages rendered via `sync_crop_sprites` using atlas index per stage
- Harvest particles exist (HarvestParticle)
- Crop growth shimmer just added this session ✓

### UNUSED ASSETS ON DISK
- **38 Pickup_Crop_*.png files** — individual 16×16 harvested crop icons
- **19 have RARE variants** — gold-starred/special quality versions
- These include crops NOT in the game: Cabbage, Chili Pepper, Cotton, Grain, Prickly Pear, Zucchini

### What's correct
- Per-crop growth stage sprites work ✓
- Harvest particles ✓
- Growth shimmer ✓

### What's wrong or missing
- **38 beautiful crop item icons completely unused** — currently items display via items_atlas.png sprite indices (generic). The Pickup_Crop PNGs are higher quality, per-crop, and include rare/quality variants.
- **No quality tiers for harvested crops visually** — rare crop variants exist on disk but quality system only uses star count in UI
- **No withered/dead crop visual** — crops that die (unwatered too long) should look brown/wilted
- **No wind sway on tall crops** (corn, wheat) — static sprites
- **6 crop types exist as assets but not in game** — Cabbage, Chili Pepper, Cotton, Grain, Prickly Pear, Zucchini

### Implementation plan
1. **Replace items_atlas crop indices with Pickup_Crop PNGs** — load individual crop pickup sprites for inventory/shop display. Higher visual fidelity, per-crop unique art.
2. **Quality crop visuals using Rare variants** — when crop quality is gold/iridium, use the _Rare_ PNG variant for the item icon.
3. **Tall crop wind sway** — apply the existing wind_sway animation (from tree/bush) to corn and wheat crop entities. Code-only, pattern already exists in objects.rs.
4. **Withered crop tint** — when a crop dies, multiply its sprite by brown Color. No new asset.
5. **Add missing crops to game data** — Cabbage, Chili Pepper, Cotton, Grain (Prickly Pear and Zucchini optional). Assets already on disk, just needs CropDef entries + recipe/shop integration.

### Assets available but unused: **38 crop pickup sprites + 19 rare variants**

---

## FAMILY 4: NPCs (character sprites, mapping, variety)

### Current state
- **12 NPC sprite sheets** in sprites/npcs/ (all 192×192 = 12×12 @ 16px)
- **11 NPCs** mapped to sprites via `npc_sprite_file()`

### WRONG SPRITE ASSIGNMENTS
| NPC | Role | Assigned Sprite | Problem |
|-----|------|----------------|---------|
| Margaret | Baker | npc_miner.png | Baker using miner sprite |
| Bjorn | Carpenter | npc_miner.png | **DUPLICATE** — same sprite as Margaret |
| Sam | Musician | npc_scholar.png | Musician using scholar sprite |
| Lily | Florist | npc_child.png | Adult florist using child sprite |

### UNUSED NPC SPRITES
- **npc_guard.png** — only used as fallback for unknown NPC IDs
- **npc_mage.png** — completely unmapped to any NPC

### What's wrong or missing
- **Margaret and Bjorn share the same sprite** — two different NPCs look identical
- **No sprite matches the character's role** for 4 of 11 NPCs
- **2 NPC sprites go unused** (guard, mage)
- **No NPC seasonal clothing variation** — same outfit year-round
- **No NPC expression changes** — same face in all contexts (happy gift, angry, festival)
- **No unique NPC idle animations** — all NPCs use identical animation frames

### Implementation plan
1. **Fix sprite assignments** — immediate code-only fix:
   - Margaret (baker): npc_farmer.png or generate a baker sprite
   - Bjorn (carpenter): npc_guard.png (bulky, fits carpenter)
   - Sam (musician): npc_mage.png (artistic/expressive)
   - Lily (florist): reassess — npc_child could work if she's young
2. **Verify no duplicates remain** after reassignment
3. **NPC color tinting per season** — multiply NPC sprite by subtle seasonal tint (warmer in summer, cooler in winter). Code-only, similar to tree seasonal tint.

### Assets available: **12 sprite sheets, only 9 effectively used, 2 wasted**

---

## FAMILY 5: BUILDINGS (already audited — see docs/BUILDING_AUDIT.md)

### Key unused assets
- `tilesets/house_roof.png` — 112×80, partially used (legacy tile path)
- `tilesets/house_walls.png` — 80×48, partially used (legacy tile path)
- `tilesets/fences.png` — 64×64, loaded but **never placed on any exterior map**
- `tilesets/modern_farm_fences.png` — 512×272, **completely unused** — massive fence tileset
- `tilesets/modern_farm_autotiles.png` — 192×896, **completely unused** — autotile system

### Fences gap is critical
The game has a fence tileset, a modern_farm_fences tileset, craftable fence items, fence dialogue references, but **zero fences actually rendered on any map**. This is the biggest building-adjacent gap.

---

## FAMILY 6: TERRAIN & PATHS

### Current state
- `tilesets/grass.png` — 176×112, loaded for base terrain
- `tilesets/tilled_dirt.png` — 176×112, loaded for farm soil
- `tilesets/modern_farm_terrain.png` — 512×368, loaded for terrain atlas
- `sprites/paths.png` — 64×64 (4×4 = 16 path tiles)
- `tilesets/hills.png` — 176×144, loaded for elevation tiles

### UNUSED TERRAIN ASSETS
- **tilled_dirt_v2.png** — alternative dirt tileset, unused
- **tilled_dirt_wide.png** — wider dirt tileset, unused
- **tilled_dirt_wide_v2.png** — wider v2, unused
- **modern_farm_autotiles.png** — 192×896 massive autotile system, unused

### What's wrong or missing
- **Path autotiling not implemented** — paths are hardcoded to single index (crossroads). The paths.png has 16 tiles designed for directional autotiling.
- **No terrain variety per biome** — farm, forest, beach, mountain all use same grass base
- **3 tilled dirt alternatives unused** — could provide seasonal/quality variation
- **Autotile system on disk but not implemented** — modern_farm_autotiles has a complete autotile ruleset

### Implementation plan
1. **Path autotiling** — implement bitmask neighbor lookup for TileKind::Path tiles, selecting from the 16-tile paths.png atlas based on adjacent path neighbors. Pattern exists in water edge rendering.
2. **Seasonal soil tinting** — multiply tilled dirt by season color (darker in winter, richer in spring). Code-only.
3. **Biome grass variation** — tint grass tiles per map (yellow-green for beach, dark green for forest, blue-green for snow mountain). Code-only.

---

## FAMILY 7: UI (massive unused library)

### UNUSED UI ASSETS (30+ files!)
- buttons_19x26.png, buttons_26x19.png, buttons_26x26.png, buttons_small.png
- cursor_default.png, cursor_holding.png, cursor_pointing.png
- dialog_box.png, dialog_box_medium.png, dialog_box_small.png
- dialog_continue_indicator.png
- emoji_spritesheet.png (160×608 — huge!)
- emotes.png (160×480)
- icons.png, icons_happiness.png, icons_special.png, icons_white.png
- inventory_blocks.png, inventory_hearts.png, inventory_hearts_light.png, inventory_spritesheet.png
- premade_dialog_big/medium/small.png
- settings_buttons.png, settings_menu.png
- speech_bubble.png (448×192)
- ui_spritesheet.png (896×240 — massive!)
- weather_ui.png

### Currently used
- Only: play_button.png, dialog_box_big.png, weather_icons.png

### Critical UI gaps
1. **Custom cursors unused** — 3 cursor PNGs on disk, game uses system cursor
2. **Speech bubble unused** — NPC dialogue uses text box, not the speech bubble sprite
3. **Emoji/emote sheets unused** — 2 large emote/emoji atlases with hundreds of icons
4. **Icon sheets unused** — happiness icons, special icons, white icons — all on disk, none in HUD
5. **Inventory UI sprites unused** — hearts, blocks, full inventory spritesheet
6. **Settings menu sprites unused** — settings_menu.png + settings_buttons.png

---

## FAMILY 8: ANIMALS

### Current state
All animal sprites ARE loaded and used (chicken, cow, duck, goat, pig, rabbit, sheep, dog).

### What's missing
- **No baby animal variants** — all animals render at adult size from birth
- **No animal color variation** — all cows are the same cow, all chickens same chicken
- **No sleeping animal pose** — animals stay in walking animation at night
- **No animal pregnancy/egg visual** — no nest with eggs near chickens, no belly change

### Implementation plan (code-only)
1. **Baby animal scale** — multiply animal sprite by 0.6 when age < threshold. One-line change.
2. **Color variation** — multiply animal sprite by slight random tint at spawn. One-line change.
3. **Night rest pose** — stop animation cycling when Calendar.hour > 20. Check in animation system.

---

## FAMILY 9: MINING & CAVE

### Current state
- `tilesets/fungus_cave.png` — 128×560 (8×35 = 280 tiles). Well-stocked cave tileset.
- `sprites/mine_enemies.png` — 48×16 (3 enemies at 16×16)
- `sprites/mining_atlas.png` — 128×96 (8×6 = 48 tiles for rocks/ores)

### What's wrong or missing
- **Only 3 enemy sprites** for all mine floors
- **No floor-based enemy variety** — should get darker/different enemies deeper
- **No ore glow** for rare ores (gold, iridium should have subtle sparkle)
- **No cave ambient particles** — dripping water, dust motes, glowing mushrooms
- **ShimmerParticle exists** but may be underused

### Implementation plan
1. **Ore glow** — for gold/iridium ore tiles, add the existing ShimmerParticle spawner at those positions
2. **Cave dust motes** — adapt the firefly system for mine maps with grey/white color
3. **Enemy tinting by floor** — multiply mine_enemies sprite by floor-based color (green→red as you go deeper)

---

## FAMILY 10: FISHING

### Current state
- `sprites/fishing_atlas.png` — 128×96 (8×6 = 48 fish sprites)
- Bobber, splash, ripple, bite VFX all exist
- `sprites/fishing_atlas.png` used for fish encyclopedia display

### What's missing
- **No fish shadow in water before bite** — a dark oval could drift under the bobber
- **No catch celebration** — fish is caught, no sparkle/popup
- **No rare fish visual distinction** — legendary fish look same as common in water
- **Fishing encyclopedia shows sprites** but no size/rarity visual indicator
- **No "one that got away" animation** — line snap visual when fish escapes

### Implementation plan
1. **Fish shadow** — spawn a small dark oval sprite near bobber that moves randomly, appears 1-2s before bite. Code-only.
2. **Catch celebration** — reuse PickupSparkle pattern when fish is successfully caught. Already in game.
3. **Rare fish sparkle** — when a legendary/rare fish bites, add gold sparkle particles around bobber. Code-only.

---

## PRIORITY RANKING FOR PARALLEL DISPATCH

### TIER 1: Wrong/broken (fix immediately)
1. **NPC sprite duplicates** — Margaret=Bjorn both use npc_miner.png. 5 min fix.
2. **Fence tileset loaded but never placed** — fences.png is in memory doing nothing.

### TIER 2: Massive unused asset integration (high visual impact)
3. **38 Pickup_Crop sprites** — replace generic item icons with per-crop art + rare variants
4. **4 individual tree PNGs** — add tree variety per biome
5. **Custom cursors** — 3 cursor sprites ready to use
6. **Path autotiling** — 16 directional path tiles ready, just needs bitmask logic

### TIER 3: Code-only visual polish (no new assets needed)
7. **Water differentiation by map** — tint water tiles per biome
8. **Tree size variation** — ±15% random scale
9. **Baby animal scale** — 0.6× for young animals
10. **Ambient fish jump** — rare particle event on water tiles
11. **Tall crop wind sway** — apply existing wind system to corn/wheat
12. **Enemy tinting by mine floor** — progressive color shift
13. **Ore glow in mines** — existing shimmer particle on gold/iridium

### TIER 4: New systems (medium effort)
14. **Exterior fences around NPC houses** — use the loaded fences tileset
15. **Cave ambient particles** — adapt firefly system for mines
16. **Fish shadow before bite** — dark oval near bobber
17. **UI icon integration** — use icons.png in HUD for status effects
18. **Speech bubble system** — use speech_bubble.png for NPC thought bubbles

### TIER 5: Large features (significant effort)
19. **Seasonal tree models** — swap between individual tree PNGs per season
20. **Autotile terrain system** — implement modern_farm_autotiles ruleset
21. **Full UI overhaul** — integrate the massive unused UI sprite library
22. **New crop types** — add Cabbage, Chili Pepper, Cotton, Grain using existing Pickup sprites
