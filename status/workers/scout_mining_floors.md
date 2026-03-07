# Mining Floor Audit — `floor_gen.rs`, `rock_breaking.rs`, `spawning.rs`

**Auditor:** Automated code scout  
**Date:** 2026-03-02  
**Files read:** `src/mining/floor_gen.rs`, `src/mining/rock_breaking.rs`, `src/mining/spawning.rs`, `src/mining/components.rs`, `src/data/items.rs`, `src/shared/mod.rs`

---

## 1. Floor Generation

### Grid Size
Every floor is a **24×24 tile grid** (`MINE_WIDTH = MINE_HEIGHT = 24`). This is fixed — there is no variance by depth.

### Player Spawn
Always at **(12, 1)** — bottom-center. A 3×3 clear zone around it is reserved, plus the entire bottom row (y=0) is kept clear for the entrance/exit feel.

### What Spawns

| Category | Placement Range | Count |
|---|---|---|
| **Floor tiles** | Full 24×24 | 576 tiles |
| **Walls** | x=0, x=23 (columns), y=23 (top row) — **bottom row y=0 is NOT walled** | 71 tiles |
| **Rocks** | x∈[1,22], y∈[2,22] | 40–60% of 576 tiles (~230–346) |
| **Enemies** | x∈[2,21], y∈[3,22] | 2–6 (see below) |
| **Ladder** | Upper half (y≥12) or hidden in a rock | 1 per floor |
| **Exit** | Fixed at (12, 0) | 1 per floor |

### Difficulty Scaling

**Rock health & drops** (see §2 for detail):

| Floor Band | Rock health range | Ore types |
|---|---|---|
| 1–5  | 2–3 HP | Stone (80%), Copper (20%) |
| 6–10 | 2–4 HP | Stone (55%), Copper (30%), Iron (15%) |
| 11–15 | 2–4 HP | Stone (55%), Iron (30%), Gold (10%), Quartz (5%) |
| 16+  | 2–4 HP | Stone (68%), Gold (25%), Diamond (3%), Ruby (2%), Emerald (2%) |

**Enemy count** — `base=2 + floor/4` (integer) + random 0–1, capped at 6:

| Floor | Base | Extra | Range |
|---|---|---|---|
| 1 | 2 | 0 | 2–3 |
| 8 | 2 | 2 | 4–5 |
| 16 | 2 | 4 | 6 (capped) |
| 20 | 2 | 5 | 6 (capped) |

**Enemy types by floor:**

| Floor | Possible enemies |
|---|---|
| 1–4 | GreenSlime only |
| 5–9 | GreenSlime (60%), Bat (40%) |
| 10+ | GreenSlime (35%), Bat (30%), RockCrab (35%) |

**Enemy stat scaling** — all stats scale linearly with floor number `f`:

| Enemy | HP | Damage | Speed (tile/s) |
|---|---|---|---|
| GreenSlime | 20+f | 5+f/2 | 24 |
| Bat | 15+f | 8+f/2 | 48 |
| RockCrab | 40+f | 12+f/2 | 16 |

**Determinism:** Floor `n` always generates identically via `StdRng::seed_from_u64(n * 7919 + 42)`. Re-entering without progressing gives the same layout.

---

## 2. Rock Breaking

### Pickaxe Damage per Tier

| Tier | Damage | Stamina Cost |
|---|---|---|
| Basic | **1** | 4.0 |
| Copper | **1** | 3.5 |
| Iron | 2 | 3.0 |
| Gold | 3 | 2.5 |
| Iridium | 4 | 2.0 |

> ⚠️ **Basic and Copper tiers deal identical damage (1).** The upgrade from Basic → Copper provides no mining throughput benefit, only reduced stamina cost. This may confuse players who expect a damage increase. See §5.

### Rock Health vs. Pickaxe Damage

| Rock HP | Hits to break (Basic/Copper) | Hits (Iron) | Hits (Gold) | Hits (Iridium) |
|---|---|---|---|---|
| 2 | 2 | 1 | 1 | 1 |
| 3 | 3 | 2 | 1 | 1 |
| 4 | 4 | 2 | 2 | 1 |

With a Basic tool, the deepest rocks (HP=4) require 4 hits. Iron and above always break any rock in ≤2 hits.

### What Rocks Drop
Drops are delivered via `ItemPickupEvent { item_id, quantity }`. All item IDs are confirmed valid in `src/data/items.rs`:

| Item ID | Valid? | Floor band |
|---|---|---|
| `stone` | ✓ | All floors |
| `copper_ore` | ✓ | Floors 1–10 |
| `iron_ore` | ✓ | Floors 6–15 |
| `gold_ore` | ✓ | Floors 11+ |
| `quartz` | ✓ | Floors 11–15 |
| `diamond` | ✓ | Floors 16+ |
| `ruby` | ✓ | Floors 16+ |
| `emerald` | ✓ | Floors 16+ |

> ⚠️ `amethyst` is a valid item in `items.rs` and has a visual match arm in `rock_color()` / `rock_atlas_index()` in `spawning.rs`, but **`rock_drop()` in `floor_gen.rs` never generates it**. It is dead code in the mining system. See §5.

### Ore Depth Tie-In
Yes — ore types are strictly gated by floor band. No copper spawns below floor 10. No iron above floor 5 or below floor 15. The bands are hard-coded if/else chains, not a table, making future extension prone to ordering errors.

---

## 3. Spawning (ECS)

`spawn_mine_floor` is a Bevy system triggered by `FloorSpawnRequest.pending = true`. It:

1. **Despawns** all existing entities with `MineFloorEntity` marker.
2. Calls `floor_gen::generate_floor(floor_num)` for a fresh `FloorBlueprint`.
3. Calls five sub-functions in sequence:

| Sub-function | What it spawns | Z-layer |
|---|---|---|
| `spawn_tiles` | 576 `MineTile` + `MineGridPos` sprites | 0.0 |
| `spawn_rocks` | `MineRock` + `MineGridPos` sprites | 1.0 |
| `spawn_enemies` | `MineMonster` + `EnemyMoveTick` + `EnemyAttackCooldown` | 2.0 |
| `spawn_ladder` | `MineLadder` + `MineGridPos` sprite (hidden or visible) | 0.5 |
| `spawn_exit` | `MineExit` + `MineGridPos` at (12, 0) | 0.5 |

4. Updates `ActiveFloor` resource: floor number, rock counts, `ladder_revealed`, player spawn coords.

**Atlas vs. color fallback:** All spawners check `atlas.loaded` and use `Sprite::from_atlas_image` if available, otherwise `Sprite::from_color`. The atlas is `sprites/mining_atlas.png` (8×6 = 48 tiles). The enemy atlas is `sprites/mine_enemies.png` (3×1: GreenSlime=0, Bat=1, RockCrab=2).

---

## 4. Ladder Reveal

**60% chance** the ladder is hidden inside a rock (selected from rocks in the upper half, y≥12).  
**40% chance** the ladder is placed visibly in an unoccupied upper-half tile.

### Hidden Ladder Reveal Logic (`check_ladder_reveal`)
Reveal is triggered when either condition is met:
- `active_floor.rocks_remaining == 0` (all rocks cleared), OR
- The broken rock's grid position matches the ladder entity's grid position.

When revealed, `MineLadder.revealed = true`, `sprite.color` is set to `LADDER_COLOR` (brown), and `ActiveFloor.ladder_revealed = true`.

Hidden ladders are spawned with `LADDER_HIDDEN_COLOR = Color::srgb(0.15, 0.12, 0.18)`, which matches `FLOOR_COLOR` exactly — the ladder is visually indistinguishable from the floor until revealed.

**The ladder is always present in the ECS from floor spawn.** It is never absent; only its visual/revealed state differs. The ladder entity is placed at the exact position of the rock that hides it.

---

## 5. BUGS and GAPS

### 🐛 BUG-1: Enemy can spawn on top of visible ladder (position collision)

**File:** `floor_gen.rs`, lines 128–156  
When the ladder is placed *openly* (40% case), its position is found via loop but **never added to `occupied`**:
```rust
loop {
    lx = ...; ly = ...;
    if !occupied.contains(&(lx, ly)) || ladder_attempts >= 100 { break; }
}
((lx, ly), false, None)  // lx/ly NOT inserted into occupied
```
Enemies are placed afterward using `occupied` to avoid collisions. An enemy can therefore legally spawn on the same tile as a visible ladder. In the hidden-ladder case this does not occur because the rock's position was already in `occupied`.

---

### 🐛 BUG-2: Open ladder placement can force-place on occupied tile

**File:** `floor_gen.rs`, line 135  
After 100 attempts the loop breaks regardless:
```rust
if !occupied.contains(&(lx, ly)) || ladder_attempts >= 100 { break; }
```
If all upper-half tiles are full (extreme coverage), the ladder lands on a rock, creating two overlapping entities at the same grid position. A player breaking the rock would destroy the rock but not the ladder (which has no `MineRock` component) — the ladder would permanently block the tile visually.

---

### 🐛 BUG-3: `_effective_dmg` computed but silently discarded

**File:** `rock_breaking.rs`, line 59  
```rust
let _effective_dmg = damage.min(rock.health);
rock.health = rock.health.saturating_sub(damage);
```
The effective damage was clearly intended for stamina drain calculation (don't cost full stamina if the rock had only 1 HP remaining and you dealt 3). Currently stamina is always drained by the full pickaxe cost regardless of overkill. The `_` prefix suppresses the warning but the variable is never used.

---

### 🐛 BUG-4: Basic and Copper pickaxe deal identical damage (1)

**File:** `rock_breaking.rs`, lines 10–16  
`ToolTier::Basic => 1` and `ToolTier::Copper => 1` — the Copper upgrade provides only a stamina reduction (4.0 → 3.5), zero mining speed improvement. This is likely a design oversight given that Iron → 2, Gold → 3, Iridium → 4 follow a clear progression. Copper should probably deal 1–2 damage to match the tier ramp.

---

### ⚠️ GAP-1: `amethyst` is dead code in rock drops

**Files:** `floor_gen.rs`, `spawning.rs`  
`rock_color()` and `rock_atlas_index()` in `spawning.rs` both handle `"amethyst"` (atlas index 17, purple gem color). However `rock_drop()` in `floor_gen.rs` never generates it — `amethyst` was likely planned for a floor band that was not implemented. Item is valid in `items.rs`. Either add a floor band for amethyst or remove the match arms.

---

### ⚠️ GAP-2: Floor depth band boundary inconsistency between enemies and rocks

**File:** `floor_gen.rs`, lines 173–244  
Rock drops use `floor <= 5` for the first band; enemy kinds use `floor < 5`. This creates a one-floor seam at **floor 5**:
- Rocks: still in the "Floors 1–5" band (copper max)
- Enemies: already in the "Floors 5–9" band (bats can appear)

This means the player encounters their first bats one floor earlier than the ore tier escalates. Minor but inconsistent.

---

### ⚠️ GAP-3: No progression past floor 20

**File:** `floor_gen.rs`, line 200  
The `rock_drop` function's final `else` catches floors 16–255. Floor cap is u8 (max 255), but enemy/rock stats keep scaling linearly with no new content. Floors 21–255 are mechanically identical to floor 20. There is no `max_floor` guard — a player could descend indefinitely to mathematically extreme enemy stats (e.g. floor 100: RockCrab has 140 HP, 62 damage).

---

### ⚠️ GAP-4: Atlas index 45 shared by both ladder and exit sprites

**File:** `spawning.rs`, lines 249, 289  
Both `spawn_ladder` (visible case) and `spawn_exit` use `TextureAtlas { index: 45 }`. These should presumably look different. Exit has an `EXIT_COLOR` tint applied; ladder does not (tint is white/default). Visually distinguishable only by color, not sprite shape. Likely intentional placeholder but worth flagging.

---

### ⚠️ GAP-5: Hidden ladder revealed via color change only — no atlas sprite on reveal

**File:** `rock_breaking.rs`, lines 117–126  
Hidden ladders are spawned as color-only sprites (the `atlas.loaded && !blueprint.ladder_hidden` condition routes to the `else` branch). On reveal, `sprite.color` is set to `LADDER_COLOR`. The revealed ladder never uses the atlas sprite (index 45). Visible ladders that started openly DO use the atlas sprite. Inconsistent visual: hidden-then-revealed ladders look different from ladders that were always visible.

---

### ✅ Item ID Cross-Reference: All `rock_drop` IDs Are Valid

All 8 item strings generated by `rock_drop()` (`"stone"`, `"copper_ore"`, `"iron_ore"`, `"gold_ore"`, `"quartz"`, `"diamond"`, `"ruby"`, `"emerald"`) are confirmed present in `src/data/items.rs`. No invalid item IDs are used. `"amethyst"` is in `items.rs` but not generated (see GAP-1).

---

## Summary Table

| Severity | ID | Description |
|---|---|---|
| 🐛 Bug | BUG-1 | Enemy can spawn on top of visible ladder |
| 🐛 Bug | BUG-2 | Open ladder placement can force onto occupied tile after 100 attempts |
| 🐛 Bug | BUG-3 | `_effective_dmg` computed but unused; full stamina always drained on overkill |
| 🐛 Bug | BUG-4 | Copper pickaxe deals same damage as Basic (1); no mining speed benefit |
| ⚠️ Gap | GAP-1 | `amethyst` handled in spawning color/atlas tables but never generated in drops |
| ⚠️ Gap | GAP-2 | Enemy kind band boundary (`< 5`) inconsistent with rock drop band (`<= 5`) |
| ⚠️ Gap | GAP-3 | No new content or floor cap past floor 20; stats scale to absurd levels at floor 100+ |
| ⚠️ Gap | GAP-4 | Atlas index 45 shared by ladder and exit sprites |
| ⚠️ Gap | GAP-5 | Revealed hidden ladders never switch to atlas sprite; visual inconsistency |
