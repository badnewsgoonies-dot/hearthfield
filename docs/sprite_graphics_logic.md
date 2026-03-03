# Domain: Sprite & Graphics Logic

## Constants

| Constant | Value | Location | Purpose |
|----------|-------|----------|---------|
| `PIXEL_SCALE` | `3.0` | `src/shared/mod.rs:949` | 16px art → 48px on screen (3× upscaling) |
| `TILE_SIZE` | `16.0` | `src/shared/mod.rs` | Base world unit for tiles |

`ImagePlugin::default_nearest()` enables nearest-neighbor GPU filtering for crisp pixel art (no bilinear blurring).

## Sprite Loading & Atlas Resources

### Atlas Resources

| Resource | File | Assets |
|----------|------|--------|
| `FarmingAtlases` | `src/farming/mod.rs:62` | `plants.png`, `tilled_dirt.png` |
| `TerrainAtlases` | `src/world/mod.rs` | `grass.png`, `dirt.png`, `water.png`, `paths.png`, `bridge.png`, `hills.png` |
| `AnimalSpriteData` | `src/animals/mod.rs:70` | `chicken.png`, `cow.png`, `sheep.png`, `cat.png`, `dog.png` |
| `ItemAtlasData` | `src/ui/` | Item icons for hotbar/inventory |

### Load Sites

All sprites load via `AssetServer::load()` wrapped in resource structs with a `loaded: bool` flag. Loading triggers on `OnEnter(GameState::Playing)`.

| File | Asset Path |
|------|------------|
| `src/player/spawn.rs` | `sprites/character_spritesheet.png` |
| `src/animals/mod.rs` | `sprites/{animal}.png` |
| `src/farming/mod.rs` | `sprites/plants.png`, `sprites/tilled_dirt.png` |
| `src/world/mod.rs` | `tilesets/{terrain}.png` |
| `src/npcs/spawning.rs` | Dynamic path per NPC via `npc_sprite_file(npc_id)` |
| `src/fishing/mod.rs` | `sprites/fishing_atlas.png` |

### Sprite Index Fields in Data Definitions (`src/shared/mod.rs`)

| Field | Line | Description |
|-------|------|-------------|
| `ItemDef.sprite_index: u32` | 339 | Item atlas index |
| `CropDef.sprite_stages: Vec<u32>` | 474 | One atlas index per growth stage |
| `NpcDef.sprite_index: u32` | 646 | NPC atlas index |
| `NpcDef.portrait_index: u32` | 646 | NPC portrait atlas index |
| `FishDef.sprite_index: u32` | 783 | Fish atlas index |

## Y-Sort / Draw Ordering

**File:** `src/world/ysort.rs`

**System:** `sync_position_and_ysort` — runs in `PostUpdate` after all movement.

### Z-Order Formula

```
Z = Z_ENTITY_BASE - logical_y × Z_Y_SORT_SCALE
```

- `Z_ENTITY_BASE = 100.0`
- `Z_Y_SORT_SCALE = 0.01`

Higher Y-position entities render behind lower Y-position ones. At Y=0 → Z=100; at Y=1000 → Z=90.

### Three Query Groups

| Group | Components | Behavior |
|-------|------------|----------|
| Dynamic entities (player, NPCs, animals) | `YSorted` + `LogicalPosition` | Rounds XY to pixel grid, computes Z from logical Y |
| Fixed-depth entities | `LogicalPosition` without `YSorted` | Rounds XY only, Z unchanged |
| Static objects (terrain props) | `YSorted` without `LogicalPosition` | Computes Z from `Transform.y`, XY untouched |

### Marker Components (`src/shared/mod.rs`)

| Component | Line | Purpose |
|-----------|------|---------|
| `YSorted` | 1001 | Unit struct marker for y-sort participation |
| `LogicalPosition(Vec2)` | 1007 | Lossless gameplay coordinates |

## Animation Approach (Per Domain)

### Player Tool Animations

**File:** `src/player/tool_anim.rs`

- Frame-based (not timer-based): advances once per tick (~60 FPS → ~67ms per frame)
- **Sprite sheet:** `character_actions.png` — 2×12 layout, 6 tools × 4 frames = 24 sprites
- **State machine:** `PlayerAnimState::ToolUse { tool, frame, total_frames }`
- Swaps between walk atlas and action atlas; emits `ToolImpactEvent` on frame 2 (mid-swing)

| Tool | Base Index |
|------|-----------|
| HOE | 0 |
| WATER | 4 |
| AXE | 8 |
| PICKAXE | 12 |
| FISHING | 16 |
| SCYTHE | 20 |

### NPC Animations

**File:** `src/npcs/animation.rs`

- Timer-based with `NpcAnimationTimer` (timer + frame counter)
- **Sprite sheet:** 4×4 layout — rows = direction (down/up/left/right), cols = walk frames (0–3)
- Cycles 4 frames when moving; snaps to frame 0 when idle

| Direction | Base Index |
|-----------|-----------|
| Down | 0 |
| Up | 4 |
| Left | 8 |
| Right | 12 |

### Animal Floating Feedback

**File:** `src/animals/rendering.rs`

- `FloatingFeedback` component with lifetime timer (1.2s)
- Drifts upward at 18 units/sec, alpha fades linearly with remaining time fraction
- Used for hearts, "Yum!" text via `Text2d` + `TextFont` + `TextColor`
- Despawns when lifetime expires

### Fishing Visuals

**File:** `src/fishing/render.rs`

**Bobber:** Continuous sinusoidal animation — `sin(elapsed_secs())` at speed 1.5 / amplitude 2.0 (normal) or speed 4.0 / amplitude 6.0 (bite pending).

**Minigame UI:** World-space colored sprites (not Bevy UI nodes):
- Dark background bar, red/orange fish zone, green catch bar, progress bar fill
- Progress bar color lerps blue → yellow at 75%+ progress
- Uses `Anchor::CenterLeft` for left-anchored fill scaling

## Camera & Pixel Scaling

**File:** `src/player/camera.rs`

### Setup (`setup_camera`)

```rust
Camera2d with:
  Msaa::Off                           // no anti-aliasing
  Tonemapping::None                   // preserves color accuracy
  Transform::from_scale(Vec3::splat(1.0 / PIXEL_SCALE))  // 1/3 = 0.333
```

The inverse scale makes 1 world unit = 3 screen pixels, enabling pixel-perfect 16px → 48px rendering.

### Camera Follow (`camera_follow_player`)

| Parameter | Value | Notes |
|-----------|-------|-------|
| Lerp speed | 5.0 | `t = (5.0 × dt).min(1.0)` — exponential smoothing |
| Teleport threshold | `TILE_SIZE × 4.0` (64 units) | Triggers instant snap |
| Snap frames | 2 | Set by map transitions to survive system ordering |

### Viewport Clamping (lines 45–72)

- Calculates bounds from `WorldMap.width × TILE_SIZE` and `WorldMap.height × TILE_SIZE`
- **Small maps:** Centers camera if map dimension ≤ viewport
- **Large maps:** Clamps to `[half_viewport, map_size - half_viewport]`
- **Safety:** Skips clamping if map dimensions are 0 (async load in progress)

### Pixel Rounding

Positions are `.round()`-ed in the y-sort system to prevent sub-pixel jitter.

## Seasonal Visuals

**File:** `src/world/seasonal.rs`

### Terrain and Tree Tinting

| Season | Terrain Tint | Tree Tint |
|--------|-------------|-----------|
| Spring | White (none) | Pinks / greens |
| Summer | Warm golden | Deep greens |
| Fall | Orange | Golds / oranges |
| Winter | Cool blue-white | Brown-grey |

**Resources:** `SeasonalTintApplied` (tracks last applied season), `LeafSpawnAccumulator`.

Applied by batch-tinting `MapTile` + `WorldObject` entities with `Sprite` component.

### Falling Leaves (Fall Only)

- 4px colored squares (orange/deep red)
- Spawn ~1/60 frames above camera
- Sine-wave drift: 8–20px amplitude, 1.5–3.5 rad/s
- Fall speed: 20–40 px/s
- Despawn after 10s

## Weather Effects

**File:** `src/world/weather_fx.rs`

| Weather | Particle | Color | Size | Speed | Rate |
|---------|----------|-------|------|-------|------|
| Rain | `RainDrop` | `rgba(0.5, 0.6, 1.0, 0.6)` blue | 1×6px | 200–400 px/s | 3/frame |
| Storm | `RainDrop` | `rgba(0.4, 0.5, 0.9, 0.7)` darker | 1×6px | 250–450 px/s | 5/frame |
| Snow | `SnowFlake` | `rgba(1.0, 1.0, 1.0, 0.7)` white | 3×3px | 30–60 px/s | 2/frame |
| Sunny | None | — | — | — | — |

- **Particle cap:** `MAX_WEATHER_PARTICLES = 600`
- **Snow drift:** Lateral sine-wave (frequency 1–3 rad/s, amplitude 5–15px)
- **Resource:** `PreviousWeather` detects weather changes to despawn old particles
- **Spawn area:** Screen width ±20px, above camera ±20px

## Day/Night Lighting

**File:** `src/world/lighting.rs`

### `DayNightOverlay`

Full-screen UI node at Z-index 900 with tint color + alpha.

| Hour | Tint Color | Alpha |
|------|-----------|-------|
| 0–5 | `(0.3, 0.3, 0.5)` dark blue | 0.50 |
| 6 | `(1.0, 0.9, 0.7)` warm dawn | 0.15 |
| 8 | `(1.0, 1.0, 0.95)` pale | 0.05 |
| 10–16 | `(1.0, 1.0, 1.0)` white | 0.00 |
| 18 | `(1.0, 0.85, 0.6)` sunset orange | 0.15 |
| 20 | `(0.6, 0.6, 0.9)` twilight blue | 0.30 |
| 22 | `(0.3, 0.3, 0.5)` night blue | 0.50 |

Overlay formula: `RGB = tint × 0.15`, `alpha = intensity`. Values lerp-interpolated between keyframes.

**Lightning** (storms): 0.3s white flash `rgba(1.0, 1.0, 1.0, 0.7)` every 8–15s, thunder sound 0.5–2s after.

**Indoor maps** (no tint): PlayerHouse, GeneralStore, AnimalShop, Blacksmith.

## Placeholder Sprite Patterns

The codebase uses a consistent **"atlas-or-fallback"** pattern: check if atlas `loaded` flag is true, use textured sprite if yes, otherwise spawn a colored rectangle.

### `Sprite::from_color()` Usage

| File | Line | Color | Size | Purpose |
|------|------|-------|------|---------|
| `src/crafting/machines.rs` | 606 | `srgb(0.6, 0.4, 0.2)` brown | TILE_SIZE² | Processing machine fallback |

### `Sprite { color, custom_size }` Fallback Pattern

| File | Line(s) | Color | Purpose |
|------|---------|-------|---------|
| `src/world/objects.rs` | 423 | `kind.color()` | World object fallback (trees, rocks, etc.) |
| `src/world/objects.rs` | 567 | `WorldObjectKind::Stump.color()` | Tree stump fallback |
| `src/world/objects.rs` | 710, 717 | Per-object color | Forageable items |
| `src/world/objects.rs` | 859 | `srgb(0.25, 0.55, 0.2)` green | Bush fallback |
| `src/world/objects.rs` | 1027 | `kind.color()` | Spawned resource objects |
| `src/world/objects.rs` | 1111 | `srgb(0.55, 0.35, 0.15)` brown | Wooden object |
| `src/world/objects.rs` | 1154 | `srgb(0.6, 0.5, 0.3)` tan | Wood variant |
| `src/world/objects.rs` | 1197 | `srgb(0.65, 0.55, 0.35)` light brown | Wood variant |
| `src/world/objects.rs` | 1744 | `srgb(0.6, 0.4, 0.2)` brown | Generic object fallback |
| `src/world/chests.rs` | 170 | `srgb(0.55, 0.35, 0.15)` brown | Chest fallback |
| `src/world/mod.rs` | 590 | `tile_color(tile, season)` | Void/untextured terrain tile |
| `src/farming/render.rs` | 125 | `soil_color(state)` | Soil tile fallback |
| `src/farming/render.rs` | 398 | `farm_object_color(&obj)` | Farm object fallback |
| `src/animals/spawning.rs` | 317 | `vis.color` | New animal kinds without sheets |
| `src/animals/spawning.rs` | 324 | `vis.color` | Animal fallback when atlas unloaded |
| `src/animals/spawning.rs` | 414 | `srgb(0.55, 0.38, 0.18)` brown | Feed trough fallback |
| `src/mining/spawning.rs` | 296 | `EXIT_COLOR` | Mine exit marker fallback |

### Fishing Minigame (Intentional Colored Sprites)

These are permanent colored-rectangle UI, not placeholders:
- `src/fishing/render.rs` lines 92, 103, 114, 125, 136, 148 — background bar, fish zone, catch bar, progress bar

## File Reference Summary

| Topic | Key Files |
|-------|-----------|
| Shared types & constants | `src/shared/mod.rs` |
| Y-sort system | `src/world/ysort.rs` |
| Camera & scaling | `src/player/camera.rs` |
| Player sprites & tool anim | `src/player/spawn.rs`, `src/player/tool_anim.rs` |
| NPC animation | `src/npcs/animation.rs`, `src/npcs/spawning.rs` |
| Animal rendering | `src/animals/rendering.rs`, `src/animals/spawning.rs`, `src/animals/mod.rs` |
| Farming sprites | `src/farming/render.rs`, `src/farming/mod.rs` |
| Fishing visuals | `src/fishing/render.rs`, `src/fishing/mod.rs` |
| World objects & terrain | `src/world/objects.rs`, `src/world/mod.rs` |
| Seasonal tinting | `src/world/seasonal.rs` |
| Weather particles | `src/world/weather_fx.rs` |
| Day/night lighting | `src/world/lighting.rs` |
| UI rendering | `src/ui/hud.rs`, `src/ui/toast.rs` |
| Chests | `src/world/chests.rs` |
| Crafting machines | `src/crafting/machines.rs` |
| Mining spawning | `src/mining/spawning.rs` |
