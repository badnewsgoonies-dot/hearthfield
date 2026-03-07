# Mining Combat Audit Report
**File audited:** `src/mining/combat.rs`
**Cross-referenced:** `src/data/items.rs`, `src/mining/components.rs`, `src/shared/mod.rs`, `src/mining/spawning.rs`
**Date:** 2026-03-02

---

## 1. Function Inventory

### `player_attack_damage(tier: ToolTier) -> f32` (private)
Returns damage dealt by the player's pickaxe swing based on tool tier.

| Tier     | Damage |
|----------|--------|
| Basic    | 10.0   |
| Copper   | 15.0   |
| Iron     | 20.0   |
| Gold     | 30.0   |
| Iridium  | 50.0   |

---

### `handle_player_attack(...)` (public system)
**System signature:**
```rust
pub fn handle_player_attack(
    mut commands: Commands,
    mut tool_events: EventReader<ToolUseEvent>,
    mut enemies: Query<(Entity, &MineGridPos, &mut MineMonster)>,
    mut pickup_events: EventWriter<ItemPickupEvent>,
    mut sfx_events: EventWriter<PlaySfxEvent>,
    mut monster_slain_events: EventWriter<MonsterSlainEvent>,
    in_mine: Res<InMine>,
)
```
**Purpose:** Reads `ToolUseEvent` filtered to `ToolKind::Pickaxe`. For each pickaxe event, iterates all mine enemies; if an enemy's `MineGridPos` matches the event's target tile, applies damage. On kill: despawns the entity, plays a sound, sends `ItemPickupEvent` (loot), and sends `MonsterSlainEvent` (for quest tracking).

---

### `enemy_loot(kind: MineEnemy) -> (String, u8)` (private)
**Purpose:** Rolls a random loot drop for a killed enemy. Returns `(item_id, quantity)`.

| Enemy       | Roll      | Item         | Qty     | Probability |
|-------------|-----------|--------------|---------|-------------|
| GreenSlime  | < 0.25    | slime        | 1–3     | 25%         |
| GreenSlime  | 0.25–0.45 | slime_jelly  | 1–2     | 20%         |
| GreenSlime  | 0.45–0.60 | copper_ore   | 1       | 15%         |
| GreenSlime  | 0.60–0.75 | sap          | 1–2     | 15%         |
| GreenSlime  | 0.75–1.0  | stone        | 1–3     | 25%         |
| Bat         | < 0.25    | bat_wing     | 1       | 25%         |
| Bat         | 0.25–0.50 | iron_ore     | 1       | 25%         |
| Bat         | 0.50–0.70 | copper_ore   | 1–2     | 20%         |
| Bat         | 0.70–1.0  | stone        | 1–2     | 30%         |
| RockCrab    | < 0.20    | crab_shell   | 1       | 20%         |
| RockCrab    | 0.20–0.45 | gold_ore     | 1       | 25%         |
| RockCrab    | 0.45–0.65 | iron_ore     | 1–2     | 20%         |
| RockCrab    | 0.65–1.0  | stone        | 2–4     | 35%         |

---

### `enemy_ai_movement(...)` (public system)
**System signature:**
```rust
pub fn enemy_ai_movement(
    time: Res<Time>,
    mut enemies: Query<(&mut MineGridPos, &mut Transform, &MineMonster, &mut EnemyMoveTick)>,
    rocks: Query<&MineGridPos, (With<MineRock>, Without<MineMonster>)>,
    active_floor: Res<ActiveFloor>,
    in_mine: Res<InMine>,
)
```
**Purpose:** Each frame, ticks every enemy's `EnemyMoveTick` timer. When the timer fires, moves the enemy one tile toward the player using greedy Manhattan-distance pathfinding. Rocks are treated as obstacles; enemies stop one tile short of the player (they attack from adjacency, not by occupying the player's tile).

---

### `enemy_attack_player(...)` (public system)
**System signature:**
```rust
pub fn enemy_attack_player(
    time: Res<Time>,
    mut enemies: Query<(&MineGridPos, &MineMonster, &mut EnemyAttackCooldown)>,
    active_floor: Res<ActiveFloor>,
    mut player_state: ResMut<PlayerState>,
    mut iframes: ResMut<PlayerIFrames>,
    mut sfx_events: EventWriter<PlaySfxEvent>,
    in_mine: Res<InMine>,
)
```
**Purpose:** Ticks each enemy's `EnemyAttackCooldown` timer and ticks the player's invincibility-frames (`PlayerIFrames`) timer. When an enemy's attack cooldown fires and it is adjacent to the player (Manhattan distance ≤ 1) and the player has no active iframes, the player takes `monster.damage` (clamped to 0), receives a 0.5-second iframe window, and a sound plays. Only one enemy can deal damage per frame (the first qualifying enemy in query iteration order breaks the loop).

---

### `check_player_knockout(...)` (public system)
**System signature:**
```rust
pub fn check_player_knockout(
    mut player_state: ResMut<PlayerState>,
    mut mine_state: ResMut<MineState>,
    mut active_floor: ResMut<ActiveFloor>,
    mut in_mine: ResMut<InMine>,
    mut map_events: EventWriter<MapTransitionEvent>,
    mut sfx_events: EventWriter<PlaySfxEvent>,
    mut gold_events: EventWriter<GoldChangeEvent>,
)
```
**Purpose:** Runs every frame while in the mine; if `player_state.health <= 0.0`, triggers a knockout: plays knockout SFX, deducts 10% of current gold, restores health to 50% of max, resets `mine_state.current_floor` to 0, sets `in_mine.0 = false`, sets `active_floor.spawned = false`, and sends `MapTransitionEvent { to_map: MapId::MineEntrance, to_x: 7, to_y: 4 }`.

---

## 2. Combat Flow: Player Attack → Damage → Kill → Loot

```
Player presses attack with pickaxe equipped
    └─► PlayerPlugin emits ToolUseEvent { tool: Pickaxe, tier: <current tier>, target_x, target_y }
            └─► handle_player_attack reads ToolUseEvent
                    ├─► Skips if tool != Pickaxe or !in_mine
                    ├─► player_attack_damage(tier) → damage value (10–50)
                    ├─► Iterates enemies; if enemy.grid_pos == (target_x, target_y):
                    │       monster.health -= damage
                    │       PlaySfxEvent("mine_enemy_hit")
                    │       If health <= 0: record killed = Some((entity, kind))
                    │       break (only one enemy hit per event)
                    └─► If killed:
                            commands.entity(entity).despawn()
                            PlaySfxEvent("mine_enemy_die")
                            enemy_loot(kind) → (item_id, qty)
                            ItemPickupEvent { item_id, quantity: qty }
                            MonsterSlainEvent { monster_kind: "<string>" }
```

`ItemPickupEvent` is received by the inventory domain (adds item to player inventory) and several other systems (crafting unlock milestones, fishing skill, play stats).

`MonsterSlainEvent` is received by `src/npcs/quests.rs` to advance kill-count quest objectives.

---

## 3. Enemy AI: Movement and Attack

### Movement (`enemy_ai_movement`)
- Each enemy has an `EnemyMoveTick` timer (Bat: 0.5s, GreenSlime: 1.0s, RockCrab: 1.5s — set at spawn).
- On timer fire: compute `(dx, dy)` to player. Prioritize the axis with greater absolute difference (primary move); fall back to the other axis (secondary move) if primary is blocked.
- Candidate tiles are checked for: map bounds, rock occupancy, player tile (stop adjacent).
- **No enemy–enemy collision**: the occupancy check only considers rocks. Multiple enemies can stack on the same tile.
- Enemies never leave a 1-tile border (`nx < 1`, `nx >= MINE_WIDTH - 1`).

### Attack (`enemy_attack_player`)
- Each enemy has an `EnemyAttackCooldown` repeating timer of **1.0 seconds** (hardcoded at spawn in `spawning.rs`).
- On cooldown fire: if enemy is within Manhattan distance ≤ 1 of player, and player has no active iframes, deal `monster.damage` to player.
- One-enemy-per-frame limit (breaks after first hit).
- After hit, player gets 0.5-second iframes.

---

## 4. Knockout System

When `player_state.health <= 0.0`:

1. `PlaySfxEvent("player_knockout")`
2. Gold penalty: `floor(gold * 0.10)` deducted via `GoldChangeEvent` (only if > 0)
3. Health restored to `max_health * 0.5`
4. `mine_state.current_floor = 0`
5. `in_mine.0 = false`
6. `active_floor.spawned = false`
7. `MapTransitionEvent { to_map: MineEntrance, to_x: 7, to_y: 4 }`

Progress (floor depth) is **fully lost** on knockout — `current_floor` resets to 0 and `deepest_floor_reached` / `elevator_floors` are **not** reset, so elevator progress is preserved.

---

## 5. Bugs and Gaps

### 🐛 BUG: Y-axis bounds check is off-by-one in `enemy_ai_movement`
**Line 183:**
```rust
if nx < 1 || nx >= MINE_WIDTH - 1 || ny < 0 || ny >= MINE_HEIGHT - 1 {
```
The x-axis lower bound excludes tile 0 (`nx < 1`), treating it as a wall border. The y-axis lower bound uses `ny < 0`, which **allows enemies to walk onto y=0** — the wall row. Floor generation places the player spawn at `(MINE_WIDTH/2, 1)`, confirming y=0 is a wall row. The check should be `ny < 1`.

---

### 🐛 BUG: `PlayerIFrames` not reset on knockout
`check_player_knockout` resets health, mine state, and triggers a map transition, but does **not** reset `iframes.timer`. If the player had an active iframe window at the moment of knockout, the timer carries over into the next mine visit. Because `enemy_attack_player` ticks the iframes timer unconditionally (even when `!in_mine.0`), this is minor but could grant brief unintended immunity on mine re-entry.

---

### 🐛 BUG: `entity.despawn()` instead of `entity.despawn_recursive()` on enemy kill
In `handle_player_attack` (line 66):
```rust
commands.entity(entity).despawn();
```
Enemies are spawned as flat entities in `spawning.rs` (no child entities currently), so this is safe **today**. However, if an enemy ever gains a child entity (health bar, animation, shadow), this will leave orphaned entities. `despawn_recursive()` would be safer.

---

### ⚠️ GAP: No `despawn` for enemies when leaving the mine or changing floors
`check_player_knockout` sets `active_floor.spawned = false` and `in_mine.0 = false`, but does not despawn remaining enemy entities. Bulk despawn of `MineFloorEntity`-tagged entities likely happens elsewhere (transitions/floor_gen), but this system has no direct cleanup — a dependency that is not immediately obvious.

---

### ⚠️ GAP: All enemies always drop exactly one loot item
`enemy_loot` always returns a single `(item_id, qty)` pair. There is no chance of dropping nothing and no chance of dropping multiple different items. Many similar games allow a small "no drop" probability. This is a design gap rather than a crash bug.

---

### ⚠️ HARDCODED VALUES that should be configurable

| Location | Value | What it is |
|---|---|---|
| `player_attack_damage`, lines 20–25 | 10/15/20/30/50 | Damage per pickaxe tier |
| `enemy_attack_player`, line 247 | `0.5` seconds | Post-hit invincibility duration |
| `check_player_knockout`, line 277 | `0.10` (10%) | Gold penalty on knockout |
| `check_player_knockout`, line 286 | `0.5` | Fraction of max_health restored |
| `check_player_knockout`, lines 296–298 | `to_x: 7, to_y: 4` | Mine entrance respawn coordinates |
| `spawning.rs`, lines 211–213 | 0.5/1.0/1.5 seconds | Enemy move intervals per type |
| `spawning.rs`, line 238 | `1.0` second | Enemy attack cooldown |

None of these are exposed in a data file or constant block — they are magic numbers scattered across two files.

---

### ⚠️ GAP: No stamina consumption on player attack in the mine
`handle_player_attack` does not check or deduct `PlayerState::stamina` when the player swings the pickaxe at an enemy. Rock-breaking likely deducts stamina via `tool_stamina_cost()` in another system, but combat swings are free. This is inconsistent with the stamina system design.

---

### ⚠️ GAP: Enemy–enemy collision ignored
`enemy_ai_movement` only checks rocks and map bounds. Multiple enemies can stack on the same tile. If two enemies occupy the same tile, only one will be hit per pickaxe event (the first match that `break`s). The second becomes unkillable via a normal swing — the `break` on line 61 prevents processing a second enemy at identical coordinates.

> **Secondary bug from this**: if enemies stack, the second enemy is effectively immortal until the stack separates.

---

### ⚠️ GAP: No `TODO`/`FIXME`/`HACK` comments
No annotation comments exist in `combat.rs`. The comment on line 249 ("If player dies, we'll handle that in a separate system") is an informal note that the coupling to `check_player_knockout` is intentional but undocumented in terms of ordering requirements.

---

### ⚠️ GAP: `MonsterSlainEvent` string conversion is not exhaustive-safe
Lines 80–84:
```rust
let kind_str = match kind {
    MineEnemy::GreenSlime => "green_slime",
    MineEnemy::Bat => "bat",
    MineEnemy::RockCrab => "rock_crab",
};
```
This `match` covers all three current variants and is exhaustive. However, if a new `MineEnemy` variant is ever added to `shared/mod.rs`, the compiler will require this match to be updated — so this is actually safe at compile time. ✓

---

## 6. Loot Table vs. `src/data/items.rs` Cross-Reference

All nine item IDs used in `enemy_loot` were verified against `src/data/items.rs`:

| Item ID      | Found in items.rs | Notes |
|--------------|:-----------------:|-------|
| `slime`      | ✅ line 1462      | |
| `slime_jelly`| ✅ line 1751      | |
| `bat_wing`   | ✅ line 1474      | |
| `crab_shell` | ✅ line 1752      | |
| `copper_ore` | ✅ line 748       | |
| `iron_ore`   | ✅ line 760       | |
| `gold_ore`   | ✅ line 772       | |
| `sap`        | ✅ line 1402      | |
| `stone`      | ✅ line 736       | |

**All loot item IDs are valid.** No broken references.

### Notable loot table observations:
- **GreenSlime** never drops iron or gold ore — consistent with early-floor placement.
- **Bat** drops iron ore (25%) but no gold ore — somewhat surprising for a mid-floor enemy.
- **RockCrab** drops gold ore (25%) — the highest-value ore — making it the most lucrative enemy to hunt. No `iridium_ore` drop exists for any enemy despite Iridium tier tools being implemented.
- No enemy drops a **gem** (`topaz`, `aquamarine`, `ruby`, `diamond` likely exist in items.rs given the full progression) — this may be an intentional gap since gems are rock-only drops, but is worth confirming with the game spec.
- No enemy drops **food** or **health-restoring** items, so there is no in-combat recovery loop from monster drops.

---

## Summary Table

| Severity | Issue |
|----------|-------|
| 🐛 Bug   | Y-axis bounds off-by-one allows enemies on wall row (y=0) |
| 🐛 Bug   | `PlayerIFrames` not reset on knockout |
| 🐛 Bug   | `despawn()` instead of `despawn_recursive()` (fragile) |
| ⚠️ Gap   | Stacked enemies: second enemy at same tile is unkillable per swing |
| ⚠️ Gap   | No stamina cost for combat attacks |
| ⚠️ Gap   | No "no drop" probability in loot table |
| ⚠️ Gap   | 7+ hardcoded values that should live in a config/data structure |
| ⚠️ Gap   | Enemy floor cleanup on knockout is implicit / handled elsewhere |
| ℹ️ Info  | All 9 loot item IDs verified valid in items.rs |
| ℹ️ Info  | MonsterSlainEvent is received by npcs/quests.rs ✓ |
| ℹ️ Info  | ItemPickupEvent is received by inventory + multiple downstream systems ✓ |
