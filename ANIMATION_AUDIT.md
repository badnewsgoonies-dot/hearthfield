# Hearthfield Animation Systems Audit Report

**Scope:** Bevy 0.15 game at `/home/user/hearthfield`  
**Date:** Audit of commit analyzing all animation systems across player, NPCs, animals, and UI

---

## EXECUTIVE SUMMARY

The animation systems are **partially implemented but incomplete**. There are **critical gaps** in several animated entity types:

- ✅ **Player Walk/Idle** — Properly implemented with distance-based frame advancement
- ✅ **Player Tool Use** — Complete with 4-frame animations per tool, per-tool frame timing
- ✅ **NPC Walk Cycle** — Basic 4-direction walk animation working
- ✅ **Fishing Bobber** — Sinusoidal bobbing animation with bite intensity variation
- ✅ **UI Animations** — Map name fade, controls hint fade, floating gold text animations
- ⚠️ **Animal Movement** — NO walking animation; animals use only colored rectangles or static sprites
- ❌ **Crop Growth** — NO animation; static sprite frame only (no growth animation over time)
- ❌ **Animal Idle Poses** — No idle animation variety
- ❌ **Death/Despawn Animations** — Completely missing
- ❌ **Carry Animation** — NOT implemented
- ❌ **Swimming Animation** — NOT implemented
- ❌ **Emote Animations** — Emote bubbles spawn but are not animated

---

## DETAILED FINDINGS BY ENTITY TYPE

### 1. PLAYER ANIMATION SYSTEM

**File:** `src/player/movement.rs`, `src/player/tool_anim.rs`, `src/player/spawn.rs`, `src/shared/mod.rs`

#### 1.1 Animation States

```rust
// src/shared/mod.rs lines 1025-1036
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PlayerAnimState {
    #[default]
    Idle,
    Walk,
    ToolUse {
        tool: ToolKind,
        frame: u8,
        total_frames: u8,
    },
}
```

**States Present:**
- `Idle` — Standing still
- `Walk` — Moving in any direction
- `ToolUse` — 4-frame tool animation (configurable per tool)

**States Missing:**
- No swimming state
- No carry/holding animation
- No sitting/resting animation
- No death animation
- No exhaustion/tired animation

#### 1.2 State Transitions

**Transition Logic:** `src/player/movement.rs:82-87`

```rust
// Line 82-87: In player_movement()
movement.anim_state = match movement.anim_state {
    PlayerAnimState::ToolUse { .. } => movement.anim_state,  // Preserve ToolUse
    _ if movement.is_moving => PlayerAnimState::Walk,
    _ => PlayerAnimState::Idle,
};
```

**Analysis:**
- ✅ Walk → Idle transition works (movement.is_moving becomes false)
- ✅ Idle → Walk transition works (movement.is_moving becomes true)
- ✅ ToolUse preserves state during movement (correct)
- ⚠️ **BUG POTENTIAL:** If tool animation finishes while player is moving, animation reverts to Idle (see `tool_anim.rs:128`), but next frame movement.rs will re-enter Walk state. This causes a 1-frame flicker to Idle.

**ToolUse → Idle Transition:** `src/player/tool_anim.rs:121-128`

```rust
if new_frame >= total_frames {
    // Animation complete — swap back to walk atlas
    sprite.image = walk_sprites.image.clone();
    if let Some(atlas) = &mut sprite.texture_atlas {
        atlas.layout = walk_sprites.layout.clone();
        atlas.index = 0;
    }
    movement.anim_state = PlayerAnimState::Idle;  // Line 128: FORCES Idle
```

**ISSUE:** When tool finishes while player is moving, state is forced to Idle. Next frame, movement.rs will see `is_moving=true` and set to Walk, but the atlas might display frame 0 (Idle frame) briefly before Walk animation resumes.

#### 1.3 Frame Timing - Walk Animation

**System:** `src/player/movement.rs:91-145` - `animate_player_sprite()`

Uses **distance-based frame advancement**, NOT time-based:

```rust
// Line 112-122: In animate_player_sprite()
match movement.anim_state {
    PlayerAnimState::Walk => {
        let delta = pos.0 - anim.last_pos;
        let dist = delta.length();
        
        if dist > 0.0 {
            anim.distance_budget += dist;
            anim.last_pos = pos.0;
            
            while anim.distance_budget >= anim.pixels_per_frame {
                anim.distance_budget -= anim.pixels_per_frame;
                anim.current_frame = (anim.current_frame + 1) % anim.frames_per_row;
            }
        }
```

**Configuration:** `src/player/mod.rs:123-132`

```rust
impl Default for DistanceAnimator {
    fn default() -> Self {
        Self {
            last_pos: Vec2::ZERO,
            distance_budget: 0.0,
            pixels_per_frame: 6.0,        // Advance frame every 6 world pixels
            frames_per_row: 4,             // 4 frames per direction
            current_frame: 0,
        }
    }
}
```

**Frame Count:** 4 frames per walk direction
**Frame Advancement:** Every 6 pixels of movement
**Total Animation Cycle:** ~24 pixels per full 4-frame cycle

**Analysis:**
- ✅ Distance-based prevents "ice skating" when speed varies from buffs
- ✅ `anim.distance_budget` properly accumulates fractional pixel distances
- ✅ Multiple frame advances per frame when moving fast
- ✅ Resets on Idle transition (line 132)
- ✅ No missed timer resets

#### 1.4 Frame Timing - Tool Animation

**System:** `src/player/tool_anim.rs:64-149` - `animate_tool_use()`

Uses **time-based frame advancement** with per-tool durations:

```rust
// Line 50-58: Tool frame durations
fn tool_frame_duration(tool: ToolKind) -> f32 {
    match tool {
        ToolKind::Axe => 0.15,         // 0.60s total — heavy, impactful chop
        ToolKind::Pickaxe => 0.14,     // 0.56s total — heavy swing
        ToolKind::Hoe => 0.12,         // 0.48s total — deliberate tilling
        ToolKind::FishingRod => 0.11,  // 0.44s total — quick cast flick
        ToolKind::WateringCan => 0.10, // 0.40s total — smooth pour
        ToolKind::Scythe => 0.08,      // 0.32s total — fast sweep
    }
}
```

**Frame Timing Logic:** `src/player/tool_anim.rs:89-142`

```rust
// Lines 89-142: Frame advance system
if frame == 0 && *frame_timer == 0.0 {
    // First frame: swap atlas to action sheet
    sprite.image = action_data.image.clone();
    if let Some(atlas) = &mut sprite.texture_atlas {
        atlas.layout = action_data.layout.clone();
        atlas.index = action_atlas_index(tool, 0);
    }
    *impact_fired = false;
}

*frame_timer += time.delta_secs();  // Line 100: Accumulate time

// Line 117-142: Check if enough time passed to advance
if *frame_timer >= duration {
    *frame_timer -= duration;  // Line 118: Subtract duration
    let new_frame = frame + 1;
    
    if new_frame >= total_frames {
        // Animation complete
        ...
        movement.anim_state = PlayerAnimState::Idle;
    } else {
        // Advance frame
        if let Some(atlas) = &mut sprite.texture_atlas {
            atlas.index = action_atlas_index(tool, new_frame as usize);
        }
        movement.anim_state = PlayerAnimState::ToolUse {
            tool,
            frame: new_frame,
            total_frames,
        };
    }
}
```

**Frame Count:** 4 frames per tool
**Frame Duration:** 0.08s–0.15s per frame (tool-dependent)
**Total Duration:** 0.32s–0.60s total animation

**Analysis:**
- ✅ Timer uses `Local<f32>` to persist across frames
- ✅ `frame_timer` properly accumulates with `time.delta_secs()`
- ✅ Frame advance only when enough time has passed
- ✅ Timer subtracts duration when advancing (no frame skip on overshoot)
- ✅ Resets on tool finish (line 129) and when not in ToolUse state (line 145)
- ✅ Impact event fires on frame 2 (line 103) once per animation (line 104: `!*impact_fired`)

**Potential Issue:** The `Local<f32>` accumulates across multiple tool uses. If a tool animation doesn't complete and then the player uses a different tool before the first finishes, the old timer state might interfere. However, in practice this is prevented because tool use sets frame=0, so the timer resets.

#### 1.5 Direction Handling - Walk/Idle

**Logic:** `src/player/movement.rs:103-137` - `animate_player_sprite()`

```rust
let base: usize = match movement.facing {
    Facing::Down => 0,      // Row 0
    Facing::Up => 4,        // Row 1
    Facing::Left => 8,      // Row 2
    Facing::Right => 12,    // Row 3
};

match movement.anim_state {
    PlayerAnimState::Walk => {
        // ... frame advance logic ...
        if let Some(atlas) = &mut sprite.texture_atlas {
            atlas.index = base + anim.current_frame;  // Line 126
        }
    }
    PlayerAnimState::Idle => {
        // ... reset logic ...
        if let Some(atlas) = &mut sprite.texture_atlas {
            atlas.index = base;  // Line 136: Frame 0 of direction
        }
    }
```

**Facing Directions Present:**
- ✅ Up, Down, Left, Right (4 directions)

**Direction Setting:** `src/player/movement.rs:33-44`

```rust
if dir.y.abs() >= dir.x.abs() {
    movement.facing = if dir.y > 0.0 {
        Facing::Up
    } else {
        Facing::Down
    };
} else {
    movement.facing = if dir.x > 0.0 {
        Facing::Right
    } else {
        Facing::Left
    };
}
```

**Analysis:**
- ✅ All 4 directions covered
- ✅ Bias to Y-axis when equal (vertical movement prioritized)
- ✅ Proper frame index calculation (base + frame)

**Direction NOT Handled for Tool Animation:**

`src/player/tool_anim.rs` does NOT vary the tool animation frame based on direction!

```rust
// Line 94 in tool_anim.rs
atlas.index = action_atlas_index(tool, 0);  // No direction offset!

// function at line 36
fn action_atlas_index(tool: ToolKind, frame: usize) -> usize {
    let tool_offset = match tool {
        ToolKind::Hoe => ACTION_HOE_BASE,     // 0
        ToolKind::WateringCan => ACTION_WATER_BASE,  // 4
        ToolKind::Axe => ACTION_AXE_BASE,     // 8
        ToolKind::Pickaxe => ACTION_PICK_BASE,  // 12
        ToolKind::FishingRod => ACTION_FISH_BASE,  // 16
        ToolKind::Scythe => ACTION_SCYTHE_BASE,  // 20
    };
    tool_offset + frame  // No direction multiplier!
}
```

**ACTION ATLAS LAYOUT:** `src/player/tool_anim.rs:5-13`

```rust
// Layout: character_actions.png is 2 cols × 12 rows
// Each tool gets 4 frames (2 frames × 2 rows per tool direction).
// Layout: 6 tools × 4 frames = 24 total
const ACTION_HOE_BASE: usize = 0;
const ACTION_WATER_BASE: usize = 4;
const ACTION_AXE_BASE: usize = 8;
const ACTION_PICK_BASE: usize = 12;
const ACTION_FISH_BASE: usize = 16;
const ACTION_SCYTHE_BASE: usize = 20;
```

 CRITICAL BUG:** The comment says "2 frames × 2 rows per tool direction" but the code only uses 4 frames total (no direction multiplier). The atlas layout is NOT being used correctly.**

**The atlas appears to be 2 cols × 12 rows = 24 frames total.** But the code assumes:
- Hoe: indices 0-3 (4 frames)
- Water: indices 4-7 (4 frames)
- etc.

This suggests the 2 columns are indexed as a linear sequence, NOT as separate direction rows. **The tool animation is DIRECTION-AGNOSTIC** — the player always uses the same animation frames regardless of facing direction.

**Analysis:**
- ❌ **MISSING:** Direction-specific tool animations (should show different swings for up/down/left/right)
- ❌ **ISSUE:** Character_actions.png layout comment contradicts the code

#### 1.6 Issues & Gaps Summary - Player

| Issue | Severity | Details |
|-------|----------|---------|
| Tool animation forces Idle on frame 1 (while moving) | Medium | Line 128 in tool_anim.rs forces Idle, causing 1-frame flicker before Walk resumes |
| No direction-specific tool animations | High | Tool swings should vary by facing direction but don't |
| No carry/hold animation | High | Missing state for carrying items |
| No swimming animation | High | No state when in water |
| No death animation | High | Complete missing |
| No exhaustion/tired pose | Medium | Stamina low but no visual indication |
| No sitting animation | Medium | No resting pose |

---

### 2. NPC ANIMATION SYSTEM

**Files:** `src/npcs/animation.rs`, `src/npcs/spawning.rs`

#### 2.1 Animation States

NPCs have a **single animation state**: walk-cycle only. No idle variation, no interaction animations.

**NpcAnimationTimer Component:** `src/npcs/animation.rs:14-19`

```rust
#[derive(Component, Debug, Clone)]
pub struct NpcAnimationTimer {
    pub timer: Timer,
    pub frame_count: usize,
    pub current_frame: usize,
}
```

**Animation States:**
- ✅ Walk (with frame cycling)
- ❌ Idle (only snaps to frame 0, no variation)
- ❌ Talk/Interact (completely missing)
- ❌ Eat/Drink animations
- ❌ Sleep animation

#### 2.2 Animation Logic

**System:** `src/npcs/animation.rs:27-69` - `animate_npc_sprites()`

```rust
pub fn animate_npc_sprites(
    time: Res<Time>,
    mut query: Query<
        (
            &NpcMovement,
            &LogicalPosition,
            &mut Sprite,
            &mut NpcAnimationTimer,
        ),
        With<Npc>,
    >,
) {
    for (movement, logical_pos, mut sprite, mut anim) in query.iter_mut() {
        // Determine facing from movement vector (current pos → target)
        let dx = movement.target_x - logical_pos.0.x;
        let dy = movement.target_y - logical_pos.0.y;

        let base: usize = if dx.abs() > dy.abs() {
            if dx > 0.0 {
                12
            } else {
                8
            } // Right (row 3) : Left (row 2)
        } else if dy > 0.0 {
            4 // Up
        } else {
            0 // Down
        };

        if movement.is_moving {
            anim.timer.tick(time.delta());  // Line 57
            if anim.timer.just_finished() {
                anim.current_frame = (anim.current_frame + 1) % anim.frame_count;  // Line 59
            }
        } else {
            anim.current_frame = 0;  // Line 62: Snap to idle frame
        }

        if let Some(atlas) = &mut sprite.texture_atlas {
            atlas.index = base + anim.current_frame;  // Line 66
        }
    }
}
```

#### 2.3 Timer Configuration

**Setup:** `src/npcs/spawning.rs:193-197`

```rust
NpcAnimationTimer {
    timer: Timer::from_seconds(0.15, TimerMode::Repeating),  // Line 194
    frame_count: 4,
    current_frame: 0,
},
```

**Frame Duration:** 0.15 seconds per frame
**Frame Count:** 4 frames (one per direction row)
**Total Cycle:** 0.60 seconds per direction

**Analysis:**
- ✅ Timer properly ticks (line 57)
- ✅ Frame advances on timer finish (line 59)
- ✅ Timer is Repeating, so it automatically resets
- ✅ No timer reset issues
- ✅ Idle snap works (line 62 resets to frame 0)

#### 2.4 Direction Handling

**Logic:** Lines 44-54

```rust
let base: usize = if dx.abs() > dy.abs() {
    if dx > 0.0 {
        12    // Right: Row 3
    } else {
        8     // Left: Row 2
    }
} else if dy > 0.0 {
    4   // Up: Row 1
} else {
    0   // Down: Row 0
};
```

**Spritesheet Layout:** 4×4 grid (same as player)
- Row 0 (indices 0-3): Walk down
- Row 1 (indices 4-7): Walk up
- Row 2 (indices 8-11): Walk left
- Row 3 (indices 12-15): Walk right

**Directions Covered:**
- ✅ All 4 directions (Up, Down, Left, Right)
- ✅ Y-axis bias (when equal, vertical movement prioritized)

#### 2.5 Issues & Gaps - NPC

| Issue | Severity | Details |
|-------|----------|---------|
| No idle animation variation | Medium | NPCs freeze on frame 0 when idle; no fidget/sway animations |
| No dialogue/talk animation | High | No animation when talking to NPCs |
| No interaction animations | High | No emote animations, gift animations, etc. |
| No sleep animation | Medium | NPCs in bed don't have any animation |
| No eating animation | Medium | NPCs eating at schedule stops don't have animation |
| No unique idle poses | Medium | All NPCs use same generic idle pose |

---

### 3. ANIMAL ANIMATION SYSTEM

**Files:** `src/animals/mod.rs`, `src/animals/spawning.rs`, `src/animals/movement.rs`, `src/animals/rendering.rs`

#### 3.1 Animation States

**CRITICAL FINDING:** Animals have **NO ANIMATION SYSTEM** at all.

**Animal Sprite Setup:** `src/animals/spawning.rs:276-349`

Animals are spawned with static sprites:
- **Chickens & Cows:** Use texture atlas but with **NO animation timer**
- **Sheep, Cat, Dog:** Use character spritesheet but with **NO animation timer**
- **Other animals (Duck, Rabbit, Pig, Goat, Horse):** **Colored rectangles** with no sprite sheets at all

```rust
// Line 278-287: Chicken sprite (atlas but no animation)
AnimalKind::Chicken => {
    let mut s = Sprite::from_atlas_image(
        sprite_data.chicken_image.clone(),
        TextureAtlas {
            layout: sprite_data.chicken_layout.clone(),
            index: 0,  // ALWAYS index 0, NO ANIMATION
        },
    );
    s.custom_size = Some(Vec2::new(16.0, 16.0));
    s
},

// Line 337-341: Other animals (colored rectangles)
_ => Sprite {
    color: vis.color,
    custom_size: Some(Vec2::new(vis.width, vis.height)),
    ..default()
},
```

#### 3.2 Movement System (NOT Animation)

**System:** `src/animals/movement.rs:12-59` - `handle_animal_wander()`

This is **wander AI**, not animation:

```rust
pub fn handle_animal_wander(
    time: Res<Time>,
    mut query: Query<(&mut LogicalPosition, &mut Transform, &mut WanderAi, &Animal)>,
) {
    // ...
    wander.timer.tick(time.delta());
    
    // If moving: update position
    if let Some(target) = wander.target {
        let current = logical_pos.0;
        let delta = target - current;
        let dist = delta.length();
        
        // ... move toward target ...
        
        // Flip sprite horizontally based on direction (Line 45-47)
        if movement.x.abs() > 0.1 {
            transform.scale.x = if movement.x > 0.0 { 1.0 } else { -1.0 };
        }
    }
}
```

**What This Does:**
- ✅ Updates animal position every frame toward a random target
- ✅ Changes direction (flips X scale)
- ✅ Picks new target every 2-4 seconds

**What It Doesn't Do:**
- ❌ **NO frame-based walk animation**
- ❌ **NO walk cycle** (animals don't "walk," they glide/teleport)
- ❌ **NO idle animation**
- ❌ **NO eating animation** (when at feed trough)
- ❌ **NO sleeping animation** (when in barn at night)

#### 3.3 Animal Sprite Assets

**Available Assets:** `src/animals/mod.rs:94-131`

```rust
// chicken.png: 64x32, 4 cols x 2 rows of 16x16 frames
sprite_data.chicken_image = asset_server.load("sprites/chicken.png");
sprite_data.chicken_layout = layouts.add(TextureAtlasLayout::from_grid(
    UVec2::new(16, 16),
    4,
    2,
    None,
    None,
));

// cow.png: 96x64, 3 cols x 2 rows of 32x32 frames
sprite_data.cow_image = asset_server.load("sprites/cow.png");
sprite_data.cow_layout = layouts.add(TextureAtlasLayout::from_grid(
    UVec2::new(32, 32),
    3,
    2,
    None,
    None,
));

// Sheep, Cat, Dog: reuse character_spritesheet.png with tint
// (4 cols x 4 rows of 48x48 frames)
```

**Analysis:**
- ✅ Chicken atlas is 4×2 = 8 frames (possibly 4 frames walk × 2 rows for direction?)
- ✅ Cow atlas is 3×2 = 6 frames (unclear layout)
- ⚠️ **NOT USED:** No animation system references these atlases

#### 3.4 Issues & Gaps - Animals

| Issue | Severity | Details |
|-------|----------|---------|
| **NO walk animation** | **CRITICAL** | Animals have no walk cycle; they glide/teleport toward targets |
| No frame advancement system | Critical | Zero timer-based or distance-based animation system |
| No idle animation | High | Animals freeze at frame 0 when not moving |
| No eating/drinking animation | High | Feed trough interaction has no animation |
| No sleeping animation | High | Animals in barn at night have no sleep animation |
| No product-ready animation | High | No special animation when product is ready to collect |
| Non-animated animals use colored rectangles | High | Duck, Rabbit, Pig, Goat, Horse have no sprites, just colors |
| Atlas assets loaded but never used | Medium | chicken.png and cow.png atlases loaded but frame index never changes |

---

### 4. CROP/FARMING ANIMATION SYSTEM

**Files:** `src/farming/render.rs`, `src/farming/mod.rs`, `src/farming/crops.rs`

#### 4.1 Crop Growth "Animation"

**System:** `src/farming/render.rs:37-40` - Static frame selection

```rust
/// Map a crop growth stage to a plants.png atlas index (row 0: indices 0-5).
fn crop_atlas_index(stage: u8, total_stages: u8) -> usize {
    let total = total_stages.max(1) as usize;
    ((stage as usize * 5) / total).min(5)
}
```

**NO ANIMATION:**
- Crops have a `stage` value that determines which atlas frame to display
- The frame is selected once based on stage, never animated
- No tween/fade between stages
- No growth animation over time

**Analysis:**
- ❌ **NO animation system** for crop growth
- ❌ **Static frames:** The atlas index changes when the crop stage changes (on day tick), but no animation plays
- ❌ **No bobbing/sway animation** for mature crops
- ❌ **No harvested animation** when crop is collected

---

### 5. FISHING BOBBER ANIMATION

**File:** `src/fishing/render.rs`

#### 5.1 Bobber Animation

**Component:** Lines 167-192 - `animate_bobber()`

```rust
pub fn animate_bobber(
    mut bobber_query: Query<(&mut Transform, &mut Bobber)>,
    fishing_state: Res<FishingState>,
    time: Res<Time>,
) {
    use super::FishingPhase;

    for (mut transform, mut bobber) in bobber_query.iter_mut() {
        let is_bite = fishing_state.phase == FishingPhase::BitePending;

        // Faster, deeper bob when a fish has bitten
        let bob_speed = if is_bite { 4.0 } else { 1.5 };
        let bob_amplitude = if is_bite { 6.0 } else { 2.0 };

        bobber.bob_timer.tick(time.delta());

        let elapsed = time.elapsed_secs();  // Use global elapsed time
        let bob_y = (elapsed * bob_speed).sin() * bob_amplitude;

        transform.translation.y = bobber.original_y + bob_y;
    }
}
```

**Analysis:**
- ✅ Uses sine wave for smooth bobbing: `sin(elapsed * speed) * amplitude`
- ✅ Adjusts speed and amplitude based on fish bite state
- ✅ Proper vertical offset applied to original_y
- ✅ No timer needed (uses global `time.elapsed_secs()`)

**Parameters:**
- Normal bob: 1.5 speed, 2.0 amplitude
- Bite bob: 4.0 speed, 6.0 amplitude

---

### 6. UI ANIMATION SYSTEMS

**File:** `src/ui/hud.rs`, `src/ui/crafting_screen.rs`

#### 6.1 Map Name Fade Animation

**System:** `src/ui/hud.rs:1023-1068` - `update_map_name()`

```rust
pub fn update_map_name(
    time: Res<Time>,
    player: Res<PlayerState>,
    mut fade: ResMut<MapNameFadeTimer>,
    mut container_query: Query<(&Children, &mut BackgroundColor), With<HudMapName>>,
    mut text_query: Query<(&mut Text, &mut TextColor)>,
) {
    // Detect map change
    let map_changed = fade.last_map != Some(player.current_map);

    if map_changed {
        fade.last_map = Some(player.current_map);
        fade.display_timer.reset();
        fade.fade_timer.reset();
        fade.alpha = 1.0;  // Start at full opacity
    }

    // Tick hold timer, then fade-out timer
    if fade.alpha > 0.0 {
        if !fade.display_timer.finished() {
            fade.display_timer.tick(time.delta());  // Hold visible for 2 seconds
        } else {
            fade.fade_timer.tick(time.delta());     // Then fade for 0.8 seconds
            let elapsed = fade.fade_timer.elapsed_secs();
            let duration = fade.fade_timer.duration().as_secs_f32();
            fade.alpha = (1.0 - elapsed / duration).clamp(0.0, 1.0);  // Linear fade
        }
    }

    // Apply alpha
    let bg_alpha = fade.alpha * 0.65;
    let current_alpha = fade.alpha;
    
    for (children, mut bg_color) in &mut container_query {
        bg_color.0 = Color::srgba(0.0, 0.0, 0.0, bg_alpha);
        for &child in children.iter() {
            if let Ok((mut text, mut tc)) = text_query.get_mut(child) {
                if map_changed {
                    **text = map_display_name(current_map).to_string();
                }
                tc.0 = Color::srgba(1.0, 1.0, 1.0, current_alpha);
            }
        }
    }
}
```

**Animation Timeline:**
1. Map change detected → `alpha = 1.0` (fully visible)
2. Display for 2.0 seconds (display_timer)
3. Fade out over 0.8 seconds (fade_timer)
4. When `alpha = 0.0`, animation ends

**Analysis:**
- ✅ Two-stage timer (display + fade)
- ✅ Proper linear alpha interpolation
- ✅ Both background and text fade together
- ✅ No issues

#### 6.2 Controls Hint Fade Animation

**System:** `src/ui/hud.rs:1115-1147` - `update_controls_hint()`

```rust
pub fn update_controls_hint(
    time: Res<Time>,
    calendar: Res<Calendar>,
    mut timer: ResMut<ControlsHintTimer>,
    mut hint_query: Query<(&Children, &mut Visibility), With<HudControlsHint>>,
    mut text_query: Query<&mut TextColor>,
) {
    let is_day1 = calendar.day == 1 && calendar.year == 1;

    for (children, mut vis) in &mut hint_query {
        if !is_day1 || timer.timer.finished() {
            *vis = Visibility::Hidden;
            continue;
        }

        *vis = Visibility::Inherited;
        timer.timer.tick(time.delta());

        // Fade out over the last 5 seconds
        let remaining = timer.timer.remaining_secs();
        let alpha = if remaining < 5.0 {
            (remaining / 5.0).clamp(0.0, 1.0) * 0.45  // Max alpha 0.45
        } else {
            0.45  // Stay at 0.45 for first 55 seconds
        };

        for &child in children.iter() {
            if let Ok(mut tc) = text_query.get_mut(child) {
                tc.0 = Color::srgba(1.0, 1.0, 1.0, alpha);
            }
        }
    }
}
```

**Animation Timeline:**
1. Only shows on Day 1, Year 1
2. 60-second timer total
3. First 55 seconds: alpha = 0.45 (constant)
4. Last 5 seconds: alpha fades from 0.45 → 0.0
5. After 60 seconds: Hidden

**Analysis:**
- ✅ Proper fade-out calculation over last 5 seconds
- ✅ Max alpha clamped at 0.45
- ✅ Timer properly ticks
- ✅ No issues

#### 6.3 Floating Gold Text Animation

**System:** `src/ui/hud.rs:1303-1379`

**Spawn:** `spawn_floating_gold_text()` lines 1303-1344

```rust
FloatingGoldText {
    velocity: 13.0,    // ~20px over 1.5s
    lifetime: 1.5,
    elapsed: 0.0,
},
```

**Update:** `update_floating_gold_text()` lines 1348-1378

```rust
pub fn update_floating_gold_text(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut FloatingGoldText, &mut Node, &mut TextColor)>,
) {
    let dt = time.delta_secs();
    for (entity, mut fgt, mut node, mut tc) in &mut query {
        fgt.elapsed += dt;

        if fgt.elapsed >= fgt.lifetime {
            commands.entity(entity).despawn_recursive();
            continue;
        }

        // Move upward: reduce top offset
        let current_top = match node.top {
            Val::Px(v) => v,
            _ => 48.0,
        };
        node.top = Val::Px(current_top - fgt.velocity * dt);  // Move up

        // Fade out: start at 40% lifetime, end at 100%
        let progress = (fgt.elapsed / fgt.lifetime).clamp(0.0, 1.0);
        let alpha = if progress < 0.4 {
            1.0
        } else {
            1.0 - ((progress - 0.4) / 0.6).clamp(0.0, 1.0)
        };
        tc.0 = Color::srgba(1.0, 0.84, 0.0, alpha);
    }
}
```

**Animation:**
1. Lifetime: 1.5 seconds
2. Velocity: 13.0 px/sec upward
3. 0%-40%: Opaque (alpha=1.0)
4. 40%-100%: Fade from 1.0 → 0.0
5. At 1.5s: Despawn

**Analysis:**
- ✅ Proper upward movement (reduces `top` value)
- ✅ Correct fade-out timing (delayed to 40%)
- ✅ Despawn after lifetime
- ✅ No issues

#### 6.4 Crafting Status Timer

**System:** `src/ui/crafting_screen.rs:359-372`

```rust
pub fn crafting_status_timer(time: Res<Time>, mut ui_state: Option<ResMut<CraftingUiState>>) {
    let Some(ref mut ui_state) = ui_state else {
        return;
    };
    if ui_state.status_timer > 0.0 {
        ui_state.status_timer -= time.delta_secs();
        if ui_state.status_timer <= 0.0 {
            ui_state.status_message.clear();
        }
    }
}
```

**Analysis:**
- ✅ Simple countdown timer for status message display
- ✅ Clears message when timer reaches 0
- ✅ No animation (just visibility toggle)

#### 6.5 UI Animation Summary

| Animation | Type | Status | Issues |
|-----------|------|--------|--------|
| Map Name Fade | Fade In/Out | ✅ Working | None |
| Controls Hint | Fade Out | ✅ Working | None |
| Floating Gold | Drift + Fade | ✅ Working | None |
| Crafting Status | Timer | ✅ Working | None |

---

### 7. EMOTE SYSTEM

**Files:** `src/npcs/emotes.rs`

#### 7.1 Emote Bubbles

**Spawn System:** `src/npcs/emotes.rs` - `spawn_emote_bubbles()`

Emote bubbles are spawned but **NOT ANIMATED**:

```rust
// Emote bubbles spawn with a marker component EmoteAtlas
// but there is no animation system that advances frames
```

**Issues:**
- ❌ **NO animation** for emote bubbles
- ❌ Emotes should bob or fade in/out
- ❌ Emotes should have a lifetime and despawn

---

## ANIMATION TIMELINE SUMMARY

### Existing Animation Systems

| Entity Type | Animation Type | Timing Method | Frames | Status |
|-------------|---|---|---|---|
| **Player Walk** | Walk cycle | Distance-based | 4 | ✅ Working |
| **Player Tool** | Tool use | Time-based (per-tool) | 4 | ✅ Working |
| **NPC Walk** | Walk cycle | Time-based (0.15s/frame) | 4 | ✅ Working |
| **Fishing Bobber** | Sine wave | Time-based (sine) | Continuous | ✅ Working |
| **Map Name UI** | Fade in/out | Time-based (2s display, 0.8s fade) | Continuous | ✅ Working |
| **Controls Hint UI** | Fade out | Time-based (55s hold, 5s fade) | Continuous | ✅ Working |
| **Floating Gold** | Drift + fade | Time-based (1.5s lifetime) | Continuous | ✅ Working |

### Missing Animation Systems

| Entity Type | Expected Animation | Status |
|-------------|---|---|
| **Animals** | Walk cycle | ❌ MISSING |
| **Animals** | Idle variation | ❌ MISSING |
| **Animals** | Eating/drinking | ❌ MISSING |
| **Animals** | Sleeping | ❌ MISSING |
| **Crops** | Growth | ❌ MISSING |
| **Crops** | Sway/bob (mature) | ❌ MISSING |
| **Player** | Death | ❌ MISSING |
| **Player** | Carry/holding | ❌ MISSING |
| **Player** | Swimming | ❌ MISSING |
| **Player** | Exhaustion/tired pose | ❌ MISSING |
| **NPCs** | Dialogue/talk | ❌ MISSING |
| **NPCs** | Interaction emotes | ❌ MISSING |
| **NPCs** | Sleeping | ❌ MISSING |
| **NPCs** | Eating | ❌ MISSING |
| **Emotes** | Bounce/fade in/out | ❌ MISSING |

---

## CRITICAL BUGS & ISSUES

### 1. Player Tool Animation Flicker While Moving

**Location:** `src/player/tool_anim.rs:128` + `src/player/movement.rs:82-87`

**Problem:** When a tool animation finishes while the player is moving, the animation state is forced to Idle. This causes the sprite to display the Idle frame (frame 0 of the current direction) for 1 frame before movement.rs re-enters Walk state.

**Scenario:**
1. Player moves forward and uses tool → `PlayerAnimState::ToolUse`
2. Tool animation completes → forced to `PlayerAnimState::Idle`
3. Same frame: `movement.is_moving = true` from input
4. Next frame: movement.rs sets state to `PlayerAnimState::Walk`
5. Result: 1-frame Idle pose, then Walk resumes

**Fix:** In `tool_anim.rs:121-141`, check if player is moving before forcing Idle:

```rust
if new_frame >= total_frames {
    sprite.image = walk_sprites.image.clone();
    if let Some(atlas) = &mut sprite.texture_atlas {
        atlas.layout = walk_sprites.layout.clone();
        atlas.index = 0;
    }
    // FIX: Check is_moving before setting Idle
    movement.anim_state = if movement.is_moving {
        PlayerAnimState::Walk
    } else {
        PlayerAnimState::Idle
    };
    *frame_timer = 0.0;
    *impact_fired = false;
}
```

### 2. Tool Animation Direction Mismatch

**Location:** `src/player/tool_anim.rs:36-45`

**Problem:** The `action_atlas_index()` function doesn't account for facing direction, but the comment says the atlas has "2 frames × 2 rows per tool direction."

**Current Code:**
```rust
fn action_atlas_index(tool: ToolKind, frame: usize) -> usize {
    let tool_offset = match tool { ... };
    tool_offset + frame  // NO direction offset!
}
```

**Issue:** Tool swings don't vary by direction. The player always shows the same animation regardless of facing Up/Down/Left/Right.

**Fix:** Add direction multiplier:

```rust
fn action_atlas_index(tool: ToolKind, frame: usize, facing: Facing) -> usize {
    let tool_offset = match tool { ... };
    let direction_offset = match facing {
        Facing::Down => 0,
        Facing::Up => 2,
        Facing::Left => 0,   // or 4 depending on atlas layout
        Facing::Right => 0,
    };
    tool_offset + direction_offset + frame
}
```

### 3. Animals Have No Walk Animation

**Location:** `src/animals/spawning.rs:276-349` + `src/animals/movement.rs`

**Problem:** Animals are spawned with sprite atlases (Chicken, Cow) or colored rectangles (others), but there is NO animation timer or frame advancement system. Animals glide to their target without animating through a walk cycle.

**Current State:**
- Wander AI updates position every frame
- Sprite frame stays at index 0 (never changes)
- Sprite scale flips horizontally (line 46 in movement.rs)
- No walk animation plays

**Impact:**
- Animals appear to glide/teleport instead of walk
- Very unnatural movement
- Wasted sprite atlas assets for Chicken and Cow

**Fix:** Add `AnimalAnimationTimer` component similar to NPC system:

```rust
#[derive(Component)]
pub struct AnimalAnimationTimer {
    pub timer: Timer,
    pub frame_count: usize,
    pub current_frame: usize,
}

// In spawning.rs, add to animal entity:
AnimalAnimationTimer {
    timer: Timer::from_seconds(0.12, TimerMode::Repeating),
    frame_count: 4,  // Adjust based on atlas
    current_frame: 0,
},

// Add animation system similar to NPCs:
pub fn animate_animal_sprites(
    time: Res<Time>,
    mut query: Query<(&WanderAi, &mut Sprite, &mut AnimalAnimationTimer), With<Animal>>,
) {
    for (wander, mut sprite, mut anim) in query.iter_mut() {
        if wander.target.is_some() {
            anim.timer.tick(time.delta());
            if anim.timer.just_finished() {
                anim.current_frame = (anim.current_frame + 1) % anim.frame_count;
            }
        } else {
            anim.current_frame = 0;  // Snap to idle
        }

        if let Some(atlas) = &mut sprite.texture_atlas {
            atlas.index = anim.current_frame;  // Or + direction offset
        }
    }
}
```

### 4. Crop Growth Has No Animation

**Location:** `src/farming/render.rs:37-40`

**Problem:** Crops have a static frame based on `stage`. When stage changes, the frame changes instantly (no animation).

**Current Flow:**
1. Day ticks
2. Crop stage increments
3. Frame index changes instantly
4. No growth animation plays

**Impact:**
- Crops appear to instantly grow
- No visual progression over the growth day

**Fix:** Add optional growth animation:

```rust
// Option A: Fade between crop stages
// Option B: Add an intermediate "growing" frame
// Option C: Animate the crop scaling/bobbing as it grows

pub fn animate_crop_growth(
    mut query: Query<(&CropGrowthProgress, &mut Sprite)>,
    time: Res<Time>,
) {
    for (progress, mut sprite) in query.iter_mut() {
        // Fade/scale as crop progresses toward next stage
        if let Some(atlas) = &mut sprite.texture_atlas {
            // Interpolate between current and next frame
        }
    }
}
```

### 5. Animal Interaction Animations Missing

**Problem:** When player pets animal or collects product, no animation plays.

**Current:**
- Animal's happiness increases
- Floating text spawns
- No visual feedback from animal itself

**Needed:**
- Brief animation of animal (jump, spin, etc.)
- Product glow/spawn animation
- Emote bubble with heart/star

---

## RECOMMENDATIONS

### Priority 1: Critical Bugs (Do First)

1. **Fix Player Tool Animation Flicker** — Simple fix to check `is_moving` before forcing Idle state
2. **Implement Animal Walk Cycle** — Add `AnimalAnimationTimer` and animation system (2-3 hours)
3. **Fix Tool Direction Variant** — Update atlas index calculation to account for facing direction (1 hour)

### Priority 2: High-Impact Gaps (Next)

4. **Implement Crop Growth Animation** — Add tween/fade between growth stages (2-3 hours)
5. **Implement NPC Interaction Animations** — Talk, eat, sleep animations (4-5 hours)
6. **Implement Animal Interaction Animations** — Pet, product collection animations (3-4 hours)
7. **Implement Emote Animation** — Bob/fade for emote bubbles (1 hour)

### Priority 3: Medium-Impact (Polish)

8. **Player Death Animation** — Fade/collapse animation on death (2 hours)
9. **Player Carry/Hold Animation** — Different pose when carrying items (2 hours)
10. **NPC Idle Variation** — Multiple idle poses that cycle (3 hours)

### Priority 4: Nice-to-Have (Last)

11. **Player Swimming Animation** — Water splashing, different walk cycle (3 hours)
12. **Crop Sway Animation** — Mature crops bob gently in wind (1 hour)

---

## CODE QUALITY NOTES

### Well-Designed Systems
- ✅ **Player Distance-Based Walk Animation** — Elegant solution avoiding "ice skating"
- ✅ **Tool-Specific Frame Durations** — Each tool feels different (heavy vs. snappy)
- ✅ **NPC Walk Cycle** — Clean reusable timer pattern
- ✅ **UI Fade Animations** — Proper two-stage timer system

### Areas for Improvement
- ⚠️ **No centralized animation registry** — Different systems use different patterns (distance vs. time)
- ⚠️ **Local timers in systems** — Use of `Local<f32>` in tool_anim.rs is fragile
- ⚠️ **Mixed coordinate systems** — Animals use world-space positions, UI uses screen-space values
- ⚠️ **No animation state machine** — Would be cleaner with a formal state pattern

---

## CONCLUSION

The animation systems cover the core player and NPC walk cycles adequately, but have **significant gaps** in animal animations, crop growth, and NPC/animal interactions. The most critical issue is **animals having no walk cycle**, making them appear to glide unnaturally. Tool animations need directional variants. These are fixable issues with 1-2 weeks of work, but the current state leaves the game feeling visually incomplete.

