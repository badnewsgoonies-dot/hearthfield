# Hearthfield Visual Systems Audit — Complete Report
**Generated:** 2026-03-06 | **Method:** 5× Sonnet 4.6 parallel audit workers via Copilot CLI

---

## EXECUTIVE SUMMARY

The codebase is **structurally sound** — all atlas configurations match PNG dimensions exactly, z-ordering is well-architected, and the plugin system is clean. However, the **visual layer is incomplete**: most entities lack animation, 8/10 animal types have no sprites, the portrait system is broken, and water/grass rendering is static. These are the issues that make the game look "not professional."

---

## 🔴 CRITICAL ISSUES (Game-Breaking Visual Bugs)

### C1. Fish sprite indices out of bounds
- **File:** `src/data/fish.rs`
- **Bug:** 5 fish (koi, ghostfish, stonefish, ice_pip, lava_eel) have `sprite_index` values 48–52, but `fishing_atlas.png` only has 48 frames (0–47)
- **Impact:** Would panic at runtime if fish rendering is ever implemented

### C2. No portrait system — dialogue shows walk frames as "portraits"
- **File:** `src/ui/dialogue_box.rs:138–163`, `src/data/npcs.rs`
- **Bug:** `portrait_index` 0–9 indexes into the 4×4 walk-cycle spritesheet. NPCs 0–3 show walk-down frames, 4–7 show walk-up frames, 8–9 show walk-left frames
- **Impact:** Every NPC's "portrait" is a tiny walking sprite, not a face

### C3. Animals are colored rectangles or tinted humans
- **File:** `src/animals/spawning.rs:276–349`, `src/animals/mod.rs`
- **Bug:** Only chicken and cow have dedicated sprites (static, no animation). Sheep/cat/dog reuse the human character spritesheet with color tints. Duck/rabbit/pig/goat/horse are plain colored rectangles
- **Impact:** 8 of 10 animal types have no proper visual representation

### C4. Tool animations ignore facing direction
- **File:** `src/player/tool_anim.rs:36–42`
- **Bug:** `action_atlas_index()` uses `tool_offset + frame` with no direction multiplier. Player always shows the same swing regardless of facing up/down/left/right
- **Impact:** Breaks visual coherence during all tool use

---

## 🟠 HIGH ISSUES (Visually Broken but Not Crashing)

### H1. Water is completely static (4 animation frames wasted)
- **File:** `src/world/mod.rs` (tile rendering)
- **Bug:** `water.png` has 4 frames but only index 0 is ever rendered. No animation timer exists
- **Impact:** Water looks like a flat blue texture, not liquid

### H2. Grass is monotone (73 of 77 tileset frames unused)
- **File:** `src/world/mod.rs` (tile rendering)
- **Bug:** Every grass tile on the map uses the exact same atlas frame per season. No per-tile variation, no noise
- **Impact:** Visible grid pattern across all grass areas

### H3. NPCs don't face the player during conversation
- **File:** `src/npcs/dialogue.rs`, `src/npcs/animation.rs`
- **Bug:** When player presses F to talk, NPC keeps facing whatever direction it was walking. Can speak with back turned
- **Impact:** Breaks immersion during all NPC interactions

### H4. No animal animation system at all
- **File:** `src/animals/` (entire domain)
- **Bug:** Chickens/cows are static at atlas index 0. No walk, idle, eat, sleep, or produce animations
- **Impact:** Animals look like frozen sprites

### H5. Bed and Stove always render on top of player
- **File:** `src/world/objects.rs:~2204, ~2217`
- **Bug:** Missing `YSorted` + `LogicalPosition` components. Fixed at z=100.1, always above all y-sorted entities
- **Impact:** Player always appears behind furniture regardless of position

### H6. Emote atlas indices are "educated guesses"
- **File:** `src/npcs/emotes.rs:46–55`
- **Bug:** Code comments explicitly say indices are guesses for the Sprout Lands emote sheet. Unverified against actual asset
- **Impact:** All emotional reactions may display wrong emotes

---

## 🟡 MEDIUM ISSUES (Incorrect but Not Immediately Obvious)

### M1. Stone and WoodFloor tiles reuse dirt tileset indices
- **File:** `src/world/mod.rs` (tile_atlas_info)
- **Bug:** `TileKind::Stone` uses index 22 and `WoodFloor` uses index 33 from `tilled_dirt.png` — neither is an actual stone/wood texture
- **Impact:** Interiors have wrong floor textures

### M2. Path/fence autotile assumes bitmask == atlas index
- **File:** `src/world/mod.rs` (path_autotile_index, fence_autotile_index)
- **Bug:** Code maps `bitmask_value` directly to atlas index. Only correct if Sprout Lands tileset is ordered in ascending bitmask order (unverified)
- **Impact:** All path/fence connections may display wrong tile variants

### M3. Duplicate forageable sprite indices
- **File:** `src/world/objects.rs` (forageable_atlas_index)
- **Bug:** Multiple forageables share indices: wild_horseradish=snow_yam=3, dandelion=crocus=7, sweet_pea=crystal_fruit=13, hazelnut=winter_root=16
- **Impact:** Different items look identical

### M4. Crop growth has no animation
- **File:** `src/farming/render.rs`
- **Bug:** Crop sprite jumps instantly between growth stages. No transition animation
- **Impact:** Growth feels mechanical rather than organic

### M5. Season change strips custom_size from all map tiles
- **File:** `src/world/mod.rs` (handle_season_change)
- **Bug:** Replacing sprites with `Sprite::from_atlas_image()` loses `custom_size: Some(Vec2::new(16.0, 16.0))`
- **Impact:** Harmless at current TILE_SIZE=16 but latent bug

### M6. No water edge transitions
- **File:** `src/world/mod.rs`
- **Bug:** Water bodies have hard tile borders. No autotile for water↔grass/sand edges
- **Impact:** Water looks like a blue grid rather than a natural body

### M7. 1-frame flicker on tool animation end while moving
- **File:** `src/player/tool_anim.rs:128`
- **Bug:** Forces `PlayerAnimState::Idle` on completion. If player is moving, next frame sets Walk, causing 1-frame idle flicker
- **Impact:** Brief visual glitch after every tool use while moving

---

## 🟢 MINOR / INFO

### I1. Dead code fields
- `NpcDef.sprite_index` — never read by spawning system
- `FishDef.sprite_index` — no rendering consumer exists

### I2. Orphaned assets (10 files never loaded)
- `sprites/tools_and_materials.png`, `sprites/milk_and_grass.png`, `sprites/egg_and_nest.png`
- `sprites/palette.png`, `tilesets/tilled_dirt_v2.png`, `tilesets/tilled_dirt_wide.png`
- `tilesets/tilled_dirt_wide_v2.png`, `sprites/npcs/npc_mage.png`
- `tilesets/bitmask_ref_1.png`, `tilesets/bitmask_ref_2.png`

### I3. Map borders use single repeated hill tile (no cliff autotile)

### I4. NPC idle is frame 0 of last walk direction (may be mid-stride)

### I5. No seasonal NPC outfit variants

### I6. Fishing UI bar at z=50 renders under nearby entities

---

## ✅ WHAT'S WORKING CORRECTLY

- **All 28 atlas configurations match their PNG dimensions exactly** — zero slicing errors
- **Y-sort system is properly implemented** — player/NPC/animal/object ordering correct
- **Z-layer architecture is clean** — ground(0) → overlay(10) → entities(84–100) → effects(200) → seasonal(300) → weather(400)
- **Player walk animation** — distance-based frame advancement, 4 directions, proper idle reset
- **Player tool animation** — per-tool frame timing (0.08–0.15s), impact events on frame 2
- **NPC walk cycles** — 4-direction, 4-frame, 0.15s/frame, proper timer handling
- **NPC schedule movement** — velocity-based interpolation, not teleportation
- **Building tile layering** — walls → doors → roof properly stacked
- **Tree seasonal sprites** — 4 seasonal columns working
- **Mining uses intentional flat z-stacking** (separate system, correct by design)

---

## PRIORITY FIX ORDER (for maximum visual impact)

1. **C3 + H4**: Animal sprites + animation (biggest visual embarrassment)
2. **C2**: Portrait system (every NPC conversation looks broken)
3. **C4**: Direction-aware tool animations
4. **H1 + H2**: Water animation + grass variation (world looks dead)
5. **H3**: NPC face-player on interaction
6. **H5**: Bed/stove y-sort fix (quick 2-line fix)
7. **M1 + M2**: Stone/wood tiles + autotile verification
8. **M6**: Water edge transitions
