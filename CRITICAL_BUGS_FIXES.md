# Critical Bugs - Quick Fix Guide

This document provides ready-to-use fixes for the 2 critical bugs found in the animation audit.

---

## Bug #1: Tool Animation Flicker While Moving

### Problem
When a tool animation completes while the player is moving, the animation briefly shows the Idle frame before resuming the Walk animation.

### Location
`src/player/tool_anim.rs` — lines 121-141

### Root Cause
Line 128 forces the animation state to `Idle` unconditionally:
```rust
movement.anim_state = PlayerAnimState::Idle;
```

This happens even if `movement.is_moving` is true. The next frame, `player_movement.rs` will see `is_moving=true` and set it back to `Walk`, but there's a 1-frame gap where the Idle frame displays.

### Current Code (BUGGY)
```rust
// src/player/tool_anim.rs:121-141
if new_frame >= total_frames {
    // Animation complete — swap back to walk atlas
    sprite.image = walk_sprites.image.clone();
    if let Some(atlas) = &mut sprite.texture_atlas {
        atlas.layout = walk_sprites.layout.clone();
        atlas.index = 0;
    }
    movement.anim_state = PlayerAnimState::Idle;  // ❌ FORCES IDLE
    *frame_timer = 0.0;
    *impact_fired = false;
}
```

### Fixed Code
```rust
// src/player/tool_anim.rs:121-141 — FIXED
if new_frame >= total_frames {
    // Animation complete — swap back to walk atlas
    sprite.image = walk_sprites.image.clone();
    if let Some(atlas) = &mut sprite.texture_atlas {
        atlas.layout = walk_sprites.layout.clone();
        atlas.index = 0;
    }
    // ✅ FIX: Check if player is moving before setting Idle
    movement.anim_state = if movement.is_moving {
        PlayerAnimState::Walk
    } else {
        PlayerAnimState::Idle
    };
    *frame_timer = 0.0;
    *impact_fired = false;
}
```

### Changes Required
- **File:** `src/player/tool_anim.rs`
- **Lines:** 128 (change 1 line to 4 lines)
- **Time:** 5 minutes
- **Testing:** Use tool while moving — should not flicker to Idle frame

### Verification
After applying the fix:
1. Spawn player
2. Start moving (hold movement key)
3. Use tool (press action key) while moving
4. Tool animation should complete seamlessly without flicker
5. Walk animation should resume smoothly

---

## Bug #2: Tool Animation Missing Direction Variants

### Problem
Tool swings don't vary by facing direction. The player always uses the same animation frames regardless of facing Up/Down/Left/Right.

### Location
`src/player/tool_anim.rs` — lines 36-45

### Root Cause
The `action_atlas_index()` function only adds the tool offset and frame number, but doesn't account for facing direction. The atlas comment suggests there are "2 rows per tool direction" but the code doesn't use them.

### Current Code (INCOMPLETE)
```rust
// src/player/tool_anim.rs:36-45
fn action_atlas_index(tool: ToolKind, frame: usize) -> usize {
    let tool_offset = match tool {
        ToolKind::Hoe => ACTION_HOE_BASE,        // 0
        ToolKind::WateringCan => ACTION_WATER_BASE,  // 4
        ToolKind::Axe => ACTION_AXE_BASE,        // 8
        ToolKind::Pickaxe => ACTION_PICK_BASE,   // 12
        ToolKind::FishingRod => ACTION_FISH_BASE, // 16
        ToolKind::Scythe => ACTION_SCYTHE_BASE,  // 20
    };
    tool_offset + frame  // ❌ NO DIRECTION MULTIPLIER
}
```

### Analysis of Atlas Layout
Looking at `character_actions.png`:
- **Dimensions:** 2 cols × 12 rows of 48×48 frames
- **Total:** 24 frames (2 cols × 12 rows)
- **Layout comment:** "2 frames × 2 rows per tool direction"

This suggests:
- Each tool should have 4 frames total
- 2 rows = 2 directions (probably up/down and left/right)
- OR: 2 frames in each of 2 directions

**Current implementation assumes:** All 4 frames per tool in a single linear sequence.

### Proposed Fix (Option A - Simple)
Only add direction offset for certain tools, or assume current layout is correct and atlas needs no fix:

```rust
// src/player/tool_anim.rs:36-45
fn action_atlas_index(tool: ToolKind, frame: usize) -> usize {
    let tool_offset = match tool {
        ToolKind::Hoe => ACTION_HOE_BASE,        // 0
        ToolKind::WateringCan => ACTION_WATER_BASE,  // 4
        ToolKind::Axe => ACTION_AXE_BASE,        // 8
        ToolKind::Pickaxe => ACTION_PICK_BASE,   // 12
        ToolKind::FishingRod => ACTION_FISH_BASE, // 16
        ToolKind::Scythe => ACTION_SCYTHE_BASE,  // 20
    };
    // If atlas has direction variants, add direction offset here
    tool_offset + frame
}
```

### Proposed Fix (Option B - With Directions)
Modify function to accept facing and add direction offset:

```rust
// src/player/tool_anim.rs:36-50 — FIXED VERSION
fn action_atlas_index(tool: ToolKind, frame: usize, facing: Facing) -> usize {
    let tool_offset = match tool {
        ToolKind::Hoe => ACTION_HOE_BASE,        // 0
        ToolKind::WateringCan => ACTION_WATER_BASE,  // 4
        ToolKind::Axe => ACTION_AXE_BASE,        // 8
        ToolKind::Pickaxe => ACTION_PICK_BASE,   // 12
        ToolKind::FishingRod => ACTION_FISH_BASE, // 16
        ToolKind::Scythe => ACTION_SCYTHE_BASE,  // 20
    };
    
    // Add direction offset if atlas supports it
    // Adjust these offsets based on actual character_actions.png layout
    let direction_offset = match facing {
        Facing::Down => 0,    // First row variant (default)
        Facing::Up => 2,      // Second row variant (if exists)
        Facing::Left => 0,    // Reuse down or add different offset
        Facing::Right => 0,   // Reuse down or add different offset
    };
    
    tool_offset + direction_offset + frame
}
```

### Changes Required for Option B
**File:** `src/player/tool_anim.rs`

**Change 1 - Function signature (line 36):**
```rust
// BEFORE:
fn action_atlas_index(tool: ToolKind, frame: usize) -> usize {

// AFTER:
fn action_atlas_index(tool: ToolKind, frame: usize, facing: Facing) -> usize {
```

**Change 2 - Function body (add direction logic):**
Add direction_offset calculation (see Option B above)

**Change 3 - Call site 1 (line 94):**
```rust
// BEFORE:
atlas.index = action_atlas_index(tool, 0);

// AFTER:
atlas.index = action_atlas_index(tool, 0, movement.facing);
```

**Change 4 - Call site 2 (line 134):**
```rust
// BEFORE:
atlas.index = action_atlas_index(tool, new_frame as usize);

// AFTER:
atlas.index = action_atlas_index(tool, new_frame as usize, movement.facing);
```

**Additional import needed:**
Add `Facing` to imports (likely already imported from crate::shared)

**Time:** 15 minutes
**Testing:** Use different tools while facing different directions — should see different swing animations

### Current Behavior
All tools swing the same way regardless of facing direction.

### Expected Behavior (After Fix)
Tools should swing differently based on facing direction (e.g., vertical swing when facing up/down, horizontal swing when facing left/right).

### Risk Assessment
- **Low Risk:** Changes only affect tool animation frame selection
- **Backwards Compatible:** If atlas doesn't have direction variants, reusing same frame is harmless
- **Testing Surface:** Single player action, easy to verify

### Verification
After applying the fix:
1. Spawn player with tool equipped
2. Face different directions (up/down/left/right)
3. Use tool in each direction
4. Verify: Tool animation should vary by direction (if atlas supports it)
5. If no difference appears: Atlas might not have direction variants

---

## Implementation Priority

### Quick Wins (Do First)
1. **Bug #1 (Tool Flicker)** — 5 minutes, high impact, low risk ✅ HIGHEST PRIORITY
2. **Bug #2 (Tool Direction)** — 15 minutes, medium impact, low risk

### Total Time
- **20 minutes** to fix both bugs
- **Recommend doing both in one commit**

---

## Testing Checklist

- [ ] Tool animation completes smoothly while moving (Bug #1)
- [ ] No 1-frame flicker to Idle (Bug #1)
- [ ] Tool animations vary by facing direction (Bug #2, if atlas supports)
- [ ] All 6 tools animate correctly (Hoe, Water, Axe, Pickaxe, Fish, Scythe)
- [ ] Walk animation resumes seamlessly after tool animation (Bug #1)
- [ ] No compile errors
- [ ] No runtime warnings

