# Animation Systems - Code Reference Guide

## Quick File Lookup

### Player Animation Files
- **Main module:** `src/player/mod.rs` (lines 1-85)
- **Spawn system:** `src/player/spawn.rs` (lines 1-96)
- **Movement/Walk anim:** `src/player/movement.rs` (lines 1-212)
  - `animate_player_sprite()` — lines 91-145
  - `player_movement()` — lines 7-88
- **Tool animation:** `src/player/tool_anim.rs` (lines 1-149)
  - `animate_tool_use()` — lines 64-149 ⚠️ BUG at line 128
  - `action_atlas_index()` — lines 36-45 ❌ Missing direction logic
- **Tool use trigger:** `src/player/tools.rs` (lines 1-180)
  - `tool_use()` — lines 37-146 (sets ToolUse state at line 138)

### Shared Types
- **Animation enums:** `src/shared/mod.rs` (lines 150-1060)
  - `Facing` enum — lines 158-164 (Up, Down, Left, Right)
  - `PlayerAnimState` enum — lines 1025-1036 (Idle, Walk, ToolUse)
  - `PlayerMovement` struct — lines 259-277 (facing, is_moving, anim_state)
  - `DistanceAnimator` component — lines 110-133 (walk animation config)

### NPC Animation Files
- **Main module:** `src/npcs/mod.rs` (lines 1-135)
- **Animation system:** `src/npcs/animation.rs` (lines 1-70)
  - `animate_npc_sprites()` — lines 27-69
  - `NpcAnimationTimer` component — lines 14-19
- **NPC spawning:** `src/npcs/spawning.rs` (lines 1-223)
  - `spawn_initial_npcs()` — lines 86-107
  - `spawn_npcs_for_map()` — lines 111-222
  - Animation timer setup — lines 193-197 (0.15s/frame)

### Animal Animation Files
- **Main module:** `src/animals/mod.rs` (lines 1-170)
  - `AnimalSpriteData` resource — lines 69-82 ❌ Loaded but unused
  - `load_animal_sprites()` — lines 85-132
- **Animal spawning:** `src/animals/spawning.rs` (lines 1-460)
  - `handle_animal_purchase()` — lines 152-403
  - Animal sprite setup — lines 278-349 ❌ NO ANIMATION TIMER
- **Animal movement:** `src/animals/movement.rs` (lines 1-59)
  - `handle_animal_wander()` — lines 12-59 (movement, NOT animation)
  - Sprite flip logic — lines 45-47 (only horizontal flip, no walk frames)

### Farming/Crops Animation Files
- **Crop rendering:** `src/farming/render.rs` (lines 1-150+)
  - `crop_atlas_index()` — lines 37-40 ❌ Static frame selection, NO animation

### Fishing Animation Files
- **Bobber animation:** `src/fishing/render.rs` (lines 1-217)
  - `animate_bobber()` — lines 171-192 ✅ Sine wave bobbing
  - `Bobber` component — defined in fishing/mod.rs
  - `FishingMinigameState` resource — used for fish bite detection

### UI Animation Files
- **HUD system:** `src/ui/hud.rs` (lines 1-1700+)
  - `MapNameFadeTimer` resource — lines 19-28
  - `update_map_name()` — lines 1023-1068 ✅ Two-stage fade
  - `ControlsHintTimer` resource — lines 129-130
  - `update_controls_hint()` — lines 1115-1147 ✅ 55s hold + 5s fade
  - `update_floating_gold_text()` — lines 1348-1378 ✅ Drift + fade animation
  - `spawn_floating_gold_text()` — lines 1303-1344
  - `FloatingGoldText` component — defined above spawn function
  - `FloatingGoldCooldown` resource — defined above spawn function
- **Crafting UI:** `src/ui/crafting_screen.rs`
  - `crafting_status_timer()` — lines 359-372 ✅ Status message timer

### Emote System (NOT ANIMATED)
- **Emote system:** `src/npcs/emotes.rs`
  - `spawn_emote_bubbles()` — spawns but ❌ NO animation system
  - `animate_emote_bubbles()` — listed in systems but may not exist/do nothing

---

## Animation Component Locations

### DistanceAnimator
**File:** `src/player/mod.rs` (lines 109-133)
**Used by:** Player walk animation
**Fields:**
- `last_pos: Vec2` — position at last frame advance
- `distance_budget: f32` — accumulated distance since last frame
- `pixels_per_frame: f32` — default 6.0 (world pixels per frame)
- `frames_per_row: usize` — default 4
- `current_frame: usize` — 0-3 for current walk frame

### NpcAnimationTimer
**File:** `src/npcs/animation.rs` (lines 14-19)
**Used by:** NPC walk animation
**Fields:**
- `timer: Timer` — 0.15s repeating timer
- `frame_count: usize` — 4 frames per direction
- `current_frame: usize` — 0-3 current frame

### WanderAi
**File:** `src/animals/mod.rs` (lines 34-45)
**Used by:** Animal movement (NOT animation)
**Note:** Controls position updates, not sprite frames

### PlayerMovement
**File:** `src/shared/mod.rs` (lines 259-277)
**Fields:**
- `facing: Facing` — Up/Down/Left/Right
- `is_moving: bool` — triggers Walk state
- `anim_state: PlayerAnimState` — Idle/Walk/ToolUse
- `speed: f32` — movement speed (80.0 default)
- `move_cooldown: Timer` — prevents rapid movement

---

## Timer Configuration Reference

### Walk/Movement Animations

**Player Walk:**
- Method: Distance-based
- Config: 6 pixels per frame, 4 frames per direction
- Cycle: ~24 pixels per full walk cycle
- File: `src/player/mod.rs:123-132` + `src/player/movement.rs:112-127`

**NPC Walk:**
- Method: Time-based
- Config: 0.15 seconds per frame, Repeating timer
- Cycle: 0.60 seconds per 4-frame cycle
- File: `src/npcs/spawning.rs:193-197`

### Tool Animations

**Config by tool (per-tool durations):**
```
Axe:         0.15s/frame = 0.60s total (heavy)
Pickaxe:     0.14s/frame = 0.56s total (heavy)
Hoe:         0.12s/frame = 0.48s total (deliberate)
FishingRod:  0.11s/frame = 0.44s total (quick)
WateringCan: 0.10s/frame = 0.40s total (smooth)
Scythe:      0.08s/frame = 0.32s total (fast)
```
- File: `src/player/tool_anim.rs:50-58`
- System: `src/player/tool_anim.rs:64-149`
- Impact event: Frame 2 (once per animation) — lines 103-114

### UI Animations

**Map Name:**
- Display: 2.0 seconds (opaque)
- Fade: 0.8 seconds (linear)
- File: `src/ui/hud.rs:1026-1048`

**Controls Hint:**
- Hold: 55 seconds (alpha=0.45)
- Fade: 5 seconds (0.45 → 0.0)
- File: `src/ui/hud.rs:1135-1139`

**Floating Gold:**
- Lifetime: 1.5 seconds
- Velocity: 13.0 px/sec upward
- Fade start: 40% (0.6s)
- Fade end: 100% (1.5s)
- File: `src/ui/hud.rs:1354-1377`

**Fishing Bobber:**
- Normal bob: speed=1.5, amplitude=2.0
- Fish bite: speed=4.0, amplitude=6.0
- Method: Sine wave (no timer, uses global elapsed time)
- File: `src/fishing/render.rs:171-192`

---

## State Machine States

### Player Animation States
```
PlayerAnimState::Idle
  ↓ (on input) ↑ (no input)
PlayerAnimState::Walk
  ↓ (tool button)
PlayerAnimState::ToolUse { tool, frame, total_frames }
  ↓ (animation complete)
PlayerAnimState::Idle
```
File: `src/shared/mod.rs:1025-1036`

### Direction Transitions
Determined by input `move_axis` (Vec2):
- Y-axis prioritized when |y| ≥ |x|
- Otherwise X-axis prioritized
- Up: y > 0, Down: y < 0, Right: x > 0, Left: x < 0
- File: `src/player/movement.rs:33-44`

---

## Sprite Atlas Layouts

### Player Character
**File:** `character_spritesheet.png` (192×192)
**Layout:** 4 cols × 4 rows of 48×48 frames
```
Row 0 (indices 0-3):  Walk down
Row 1 (indices 4-7):  Walk up
Row 2 (indices 8-11): Walk left
Row 3 (indices 12-15): Walk right
```

### Player Tools
**File:** `character_actions.png` (2×12 layout = 96×576)
**Layout:** 2 cols × 12 rows of 48×48 frames
```
Hoe:         indices 0-3
Water:       indices 4-7
Axe:         indices 8-11
Pickaxe:     indices 12-15
FishingRod:  indices 16-19
Scythe:      indices 20-23
```
 **Note:** Comment says "2 rows per tool direction" but code uses linear indices

### NPCs
**File:** Per-NPC spritesheet (192×192 each)
**Layout:** 4 cols × 4 rows of 48×48 frames (same as player)
**File locations in assets:** `assets/sprites/{npc_id}.png`

### Animals
**Chicken:** `chicken.png` (64×32)
- Layout: 4 cols × 2 rows of 16×16 frames
- Status: ❌ Loaded but never used

**Cow:** `cow.png` (96×64)
- Layout: 3 cols × 2 rows of 32×32 frames
- Status: ❌ Loaded but never used

**Sheep/Cat/Dog:** Reuse `character_spritesheet.png` with color tint
- Status: ❌ Loaded but frame index never changes

### Crops
**File:** `plants.png` (192×32)
**Layout:** 6 frames in row 0 (indices 0-5, 32×32 each)
- Frame selection: Based on `crop_stage` value
- Status: ❌ NO animation; instant frame change

### Fishing Bobber
**Status:** ✅ No texture atlas needed; sprite animates via transform.translation.y

---

## Key Variables & Resources

### PlayerState (in shared)
- `equipped_tool: ToolKind`
- `stamina: f32` / `max_stamina: f32`
- `current_map: MapId`

### Calendar (in shared)
- `day: u32`
- `year: u32`

### AnimalState (in shared)
- `animals: Vec<Animal>`
- `has_coop: bool` / `coop_level: u8`
- `has_barn: bool` / `barn_level: u8`

---

## System Execution Order

### Update Phase Scheduling

**UpdatePhase::Intent** (runs first):
- `dispatch_world_interaction` — player interaction input
- `dispatch_item_use` — item usage

**UpdatePhase::Simulation** (runs second):
- `tool_use` — sets ToolUse state (line 138 in tools.rs)
- `player_movement` — updates is_moving, facing (line 7-88)
- `animate_player_sprite` — reads anim_state, updates frame (line 91-145)
- `animate_tool_use` — handles ToolUse animation (line 64-149)
- `move_npcs_toward_targets` — NPC position updates
- `animate_npc_sprites` — NPC frame updates
- `handle_animal_wander` — animal position updates
- `spawn_emote_bubbles` — emote spawning

**UpdatePhase::Presentation** (runs last):
- `camera_follow_player` — camera positioning
- HUD updates (map name, floating gold, etc.)

---

