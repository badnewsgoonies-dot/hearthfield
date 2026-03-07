# Save / Load System Audit
**File audited:** `src/save/mod.rs`  
**`SAVE_VERSION`:** 2  
**Date:** 2026-03-02

---

## 1. Resources Saved (FullSaveFile struct fields)

The `FullSaveFile` struct (line 284) serializes the following game state:

| Field | Source Resource | Notes |
|-------|----------------|-------|
| `version` | constant `SAVE_VERSION` | metadata, not a resource |
| `slot` | `ActiveSaveSlot` | metadata |
| `save_timestamp` | wall-clock | metadata |
| `play_time_seconds` | `GameStatistics` | |
| `farm_name` | `GameStatistics` | |
| `calendar` | `Calendar` | full struct incl. `time_scale`, `time_paused`, `elapsed_real_seconds` |
| `player_state` | `PlayerState` | stamina, health, gold, equipped tool, tool tiers, current_map — **no grid position** |
| `inventory` | `Inventory` | all slots + selected_slot |
| `farm_state` | `FarmState` | soil, crops, farm objects (trees/rocks/sprinklers etc.) |
| `animal_state` | `AnimalState` | all owned animals |
| `relationships` | `Relationships` | NPC heart points |
| `mine_state` | `MineState` | current_floor, deepest_floor_reached, elevator_floors |
| `unlocked_recipes` | `UnlockedRecipes` | |
| `shipping_bin` | `ShippingBin` | items pending sale |
| `total_gold_earned` | `GameStatistics` | |
| `total_items_shipped` | `GameStatistics` | |
| `house_state` | `HouseState` | tier, has_kitchen, has_nursery |
| `marriage_state` | `MarriageState` | spouse, wedding date, days married, happiness |
| `quest_log` | `QuestLog` | |
| `sprinkler_state` | `SprinklerState` | placed sprinkler positions |
| `active_buffs` | `ActiveBuffs` | food buff durations |
| `evaluation_score` | `EvaluationScore` | |
| `relationship_stages` | `RelationshipStages` | per-NPC romance stage |
| `achievements` | `Achievements` | |
| `tutorial_state` | `TutorialState` | |
| `play_stats` | `PlayStats` | various counters (days played, crops harvested, fish caught…) |
| `building_levels` | `BuildingLevels` (economy) | barn/coop/greenhouse tiers |
| `shipping_log` | `ShippingLog` | day-by-day shipping history |
| `fish_encyclopedia` | `FishEncyclopedia` (fishing) | discovered fish entries |
| `fishing_skill` | `FishingSkill` (fishing) | XP and level |

**Total: 30 saved fields** (26 distinct game-state resources + 4 metadata fields).

---

## 2. Resources Loaded (handle_load_request)

`handle_load_request` (line 641) restores every field listed in section 1 directly from the deserialized `FullSaveFile`. The mapping is **1-to-1 and complete** — no saved field is skipped on load.

Special post-load steps:
- `inventory.selected_slot` is clamped to `[0, slots.len())` to guard against malformed saves.
- `GameStatistics` fields are assigned individually (not cloned wholesale).
- `current_map_id` is deliberately set to `MapId::Mine` as a dummy sentinel to force a mismatch so `handle_map_transition` does not skip the reload.
- A `MapTransitionEvent` is sent to re-load the correct map; player is teleported to **hardcoded coordinates** (see Bug §5.3).

---

## 3. Resources Reset in New Game (handle_new_game)

`handle_new_game` (line 738) resets all 26 game-state resources to `Default`, then seeds the starter inventory:

| Reset call | Result |
|-----------|--------|
| `*calendar = Calendar::default()` | Year 1, Spring 1, 6:00 AM |
| `*player_state = PlayerState::default()` | 500 gold, basic tools, house map |
| `*inventory = Inventory::default()` | empty + 15 turnip seeds + 5 potato seeds + 3 bread (added after) |
| `*farm_state = FarmState::default()` | empty soil/crop/object maps |
| `*animal_state = AnimalState::default()` | no animals |
| `*relationships = Relationships::default()` | all hearts 0 |
| `*mine_state = MineState::default()` | floor 0 |
| `*unlocked_recipes = UnlockedRecipes::default()` | |
| `*shipping_bin = ShippingBin::default()` | empty |
| `*statistics = GameStatistics::new(farm_name)` | preserves farm name |
| `*ext.house_state = HouseState::default()` | Basic tier |
| `*ext.marriage_state = MarriageState::default()` | |
| `*ext.quest_log = QuestLog::default()` | |
| `*ext.sprinkler_state = SprinklerState::default()` | |
| `*ext.active_buffs = ActiveBuffs::default()` | |
| `*ext.evaluation_score = EvaluationScore::default()` | |
| `*ext.relationship_stages = RelationshipStages::default()` | |
| `*ext.achievements = Achievements::default()` | |
| `*ext.tutorial_state = TutorialState::default()` | |
| `*ext.play_stats = PlayStats::default()` | |
| `*ext.building_levels = BuildingLevels::default()` | |
| `*ext.shipping_log = ShippingLog::default()` | |
| `*ext.fish_encyclopedia = FishEncyclopedia::default()` | |
| `*ext.fishing_skill = FishingSkill::default()` | |

No `MapTransitionEvent` is fired from `handle_new_game`. The game presumably transitions to `GameState::Playing` separately (in the main menu flow), which calls `spawn_player` — placing the player at the hardcoded house spawn `(8, 8)`. This is correct for a new game.

---

## 4. MISSING: Serializable Resources NOT in FullSaveFile

The following resources have `#[derive(..., Serialize, Deserialize)]` and are registered with `.init_resource` in active plugins, but are **absent from `FullSaveFile`**:

### 4.1 `HarvestStats` — `src/economy/stats.rs:13`
```rust
#[derive(Resource, Debug, Clone, Default, Serialize, Deserialize)]
pub struct HarvestStats {
    pub crops: HashMap<String, (u32, u32)>, // crop_id → (count, revenue)
}
```
Registered in `EconomyPlugin` (`economy/mod.rs:54`). Tracks per-crop harvest totals and estimated revenue. Lost on save/load and new-game reset. Not critical for gameplay but impacts statistics displays.

### 4.2 `AnimalProductStats` — `src/economy/stats.rs:23`
```rust
#[derive(Resource, Debug, Clone, Default, Serialize, Deserialize)]
pub struct AnimalProductStats {
    pub total_eggs: u32,
    pub total_milk: u32,
    pub total_wool: u32,
    pub total_other: u32,
    pub total_revenue: u32,
}
```
Registered in `EconomyPlugin` (`economy/mod.rs:55`). Same issue as `HarvestStats`. Both counters silently reset to zero on every load.

---

## 5. BUGS

### 5.1 Chest Contents NOT Saved (Data Loss Bug — Severity: HIGH)
`StorageChest` is a Bevy **component** (not a resource). Placed chest entities exist only at runtime. The `FarmObject` enum in `FarmState` has **no `Chest` variant**:
```rust
pub enum FarmObject {
    Tree { health: u8 }, Rock { health: u8 }, Stump { health: u8 },
    Bush, Sprinkler, Scarecrow, Fence, Path, ShippingBin,
    // ← no Chest!
}
```
`world/chests.rs:place_chest` spawns a `ChestMarker + StorageChest` entity but does **not** record it in `FarmState.objects`. As a result:
- Chest **positions** are not saved → chests disappear after save/load.
- Chest **contents** are not saved → all stored items are permanently lost on every save/load cycle.

### 5.2 HarvestStats / AnimalProductStats Not Reset on New Game (Severity: MEDIUM)
`handle_new_game` resets 24 resources but does **not** reset `HarvestStats` or `AnimalProductStats`. A new game started in the same session will inherit statistics from the previous game.

### 5.3 Player Grid Position Not Saved — Hardcoded Respawn (Severity: MEDIUM)
`PlayerState` has no `grid_pos` field. On load, the player is sent to **hardcoded tile coordinates** based only on the saved `current_map`:

```rust
// src/save/mod.rs:707-712
let (spawn_x, spawn_y) = match player_state.current_map {
    MapId::PlayerHouse => (8, 8),
    MapId::Farm        => (16, 4),
    MapId::Town        => (14, 10),
    _                  => (8, 8),  // ← catch-all
};
```

This means:
- The player's exact position at save time is always discarded.
- Maps `Beach`, `Forest`, `MineEntrance`, `GeneralStore`, `AnimalShop`, and `Blacksmith` all fall through to `(8, 8)`, which may not be a valid or sensible spawn point on those maps.
- `MapId::Mine` also falls to `(8, 8)` — which is the mine entrance area, not the floor the player was on. Combined with `MineState.current_floor` being preserved, the visual state and the mine-floor state will be inconsistent (see §5.4).

### 5.4 Mine Floor State vs. Spawn Position Mismatch on Load (Severity: MEDIUM)
`MineState.current_floor` is saved and restored correctly. However, on load the `MapTransitionEvent` will spawn the mine entry level (`floor 0` tiles), not the floor indicated by `current_floor`. The mining transition system (`src/mining/transitions.rs`) reads the `MapTransitionEvent` and sets up the correct floor — but the load-time event sends `to_map: MapId::Mine` with coordinates `(8, 8)`. Whether `transitions.rs` correctly interprets this as "resume at `current_floor`" needs to be verified; if it does not, the player lands at the mine entrance despite the save saying they were on floor 15.

### 5.5 `Calendar.time_paused` and `Calendar.elapsed_real_seconds` Saved (Minor — Severity: LOW)
Saving `time_paused = true` is harmless (it gets set `false` by normal gameplay), but saving `elapsed_real_seconds` (a sub-second accumulator) is unnecessary noise in the save file. If saved mid-tick, the restored accumulator will cause a spurious extra game-minute tick on the first frame after load. Not game-breaking but subtly wrong.

### 5.6 `active_buffs` Durations Are Wall-Clock Agnostic (Severity: LOW)
`ActiveBuffs` stores remaining duration in game-seconds but there is no save-timestamp comparison. If a player saves, quits, and reloads a week later, all food buffs are still active with exactly the remaining duration from the save. This is probably intentional (offline time doesn't progress) but worth noting.

---

## 6. Save Version Migration

`SAVE_VERSION = 2` (line 21). The version strategy is:

| Scenario | Behaviour |
|---------|-----------|
| `file.version > SAVE_VERSION` | **Hard error** — rejected, game prints message to update the game. |
| `file.version == SAVE_VERSION` | Normal load. |
| `file.version < SAVE_VERSION` | **Soft warning** — loads with `serde(default)` filling missing fields. |

All fields added after version 1 (the `#[serde(default)]` block: `house_state` through `fishing_skill`) will silently use their `Default` implementation when loading a v1 file. There is **no explicit migration function** — no v1→v2 data transformation is applied. This is acceptable as long as defaults represent a valid starting state, which they do.

**Potential issue:** if a field was *renamed* or *restructured* (not merely added) between versions, `serde(default)` would silently fill the new field with its default while discarding the old value. The current code has no protection against this. A future version bump should document all schema changes.

---

## 7. Player Position: Saved and Restored Correctly?

**Short answer: No — exact position is lost, map is preserved.**

`PlayerState` (line 273) stores:
```rust
pub stamina: f32, pub max_stamina: f32, pub health: f32, pub max_health: f32,
pub equipped_tool: ToolKind, pub tools: HashMap<ToolKind, ToolTier>,
pub gold: u32, pub current_map: MapId,
// ← no grid_x / grid_y
```

The player's actual position at runtime lives in two ECS components on the player entity:
- `GridPosition { x, y }` (component)
- `LogicalPosition(Vec2)` (component)

Neither of these is serialized. On load, the save system only knows **which map** the player was on, and sends a `MapTransitionEvent` with hardcoded spawn coordinates. `src/player/interaction.rs:142-165` handles the event and updates the player entity's `GridPosition` and `LogicalPosition` to `(to_x, to_y)`.

**Effect:** The player always wakes up at a fixed map entry point after loading, regardless of where they actually saved. For most maps this is acceptable (the player wakes in their house bed, or at the farm gate), but for sub-maps (`Beach`, `Forest`, `MineEntrance`, `GeneralStore`, `AnimalShop`, `Blacksmith`) the hardcoded `(8, 8)` fallback may be an invalid or out-of-bounds tile.

**Recommendation:** Add `pub grid_pos: (i32, i32)` to `PlayerState` and set it from the `GridPosition` component in the save system. On load, pass the saved coordinates directly in the `MapTransitionEvent` instead of hardcoding per-map defaults.

---

## Summary Table

| Category | Count | Items |
|----------|-------|-------|
| Fields saved | 30 | see §1 |
| Fields loaded | 30 | symmetric with save, see §2 |
| Fields in new-game reset | 24 | see §3 |
| Missing from save | **2** | `HarvestStats`, `AnimalProductStats` |
| Bugs found | **6** | Chests (HIGH), Stats reset (MED), Player pos (MED), Mine mismatch (MED), Calendar noise (LOW), Buff time (LOW) |
| Version migration | Partial | `serde(default)` only, no explicit migration |
