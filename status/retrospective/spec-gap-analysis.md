# Spec Gap Analysis: `GAME_SPEC.md` vs Implementation

## Scope and method
- Compared the 12 domains in [`GAME_SPEC.md`](../../GAME_SPEC.md) (lines 40-162) against runtime code in `src/`.
- Percentages are implementation estimates (feature-weighted, not LOC-weighted).
- Evidence references use `path:line`.

## Coverage summary
| Domain | Estimated spec implemented |
|---|---:|
| Calendar & Time | 80% |
| Player | 82% |
| Farm & Crops | 90% |
| Animals | 78% |
| World & Maps | 68% |
| NPCs & Dialogue | 76% |
| Shops & Economy | 90% |
| Crafting & Cooking | 92% |
| Fishing | 90% |
| Mining | 93% |
| UI | 74% |
| Save & Settings | 72% |

## Domain-by-domain gaps

### 1) Calendar & Time (`calendar`) — 80%
**Implemented evidence**
- 4x28 season/year model: `src/shared/mod.rs:962-963`.
- Calendar starts 6:00 and rolls at 2:00 (hour 26): `src/shared/mod.rs:103-105`, `src/calendar/mod.rs:305-307`.
- Pause outside Playing state (menus/dialogue/cutscene/shop/etc. pause simulation): `src/calendar/mod.rs:54-63`.
- Festival day markers (1 per season): `src/shared/mod.rs:136-140`.

**Specced but not built / incomplete**
- Named seasonal festivals from spec are not fully represented as such (spec names at `GAME_SPEC.md:47`); implementation has festival systems but not a full named parity contract in spec terms.

**Built but not specced**
- Manual sleep trigger proximity logic and cutscene queue transitions: `src/calendar/mod.rs:114-223`.

**Built incorrectly vs spec**
- Time speed mismatch: spec says ~1 real minute = 10 game minutes (`GAME_SPEC.md:43`), but runtime uses `secs_per_game_minute = 1.0 / time_scale` with default `time_scale=10.0`, i.e. ~1 real second = 10 game minutes (`src/shared/mod.rs:108`, `src/calendar/mod.rs:247-251`).

---

### 2) Player (`player`) — 82%
**Implemented evidence**
- 4-direction movement and facing animations: `src/player/movement.rs:30-45`, `src/player/movement.rs:103-138`.
- Tool set + stamina costs in 2-8 range: `src/player/mod.rs:153-160`.
- Base stamina 100 and 36-slot inventory (12+24): `src/shared/mod.rs:965`, `src/shared/mod.rs:968-970`.
- Q/E tool cycling: `src/player/tools.rs:5-31`.
- Collision with terrain/objects: `src/player/movement.rs:176-210`.

**Specced but not built / incomplete**
- Number-key direct equipped-tool cycling is not wired as described; number keys primarily set hotbar selection (`src/ui/menu_input.rs:114-116`) while equipped tool is changed by Q/E or inventory activation (`src/player/tools.rs:5-31`, `src/ui/inventory_screen.rs:371-378`).

**Built but not specced**
- Touch/gamepad unified input paths: `src/input/mod.rs:2-4`, `src/input/mod.rs:47-80`, `src/input/mod.rs:86-205`.

**Built incorrectly vs spec**
- None major beyond number-key behavior mismatch above.

---

### 3) Farm & Crops (`farming`) — 90%
**Implemented evidence**
- Till/water/plant/harvest loops present: `src/farming/mod.rs:151-173`, `src/farming/crops.rs:112-188`.
- Wither logic follows spec behavior (2 days wilt, 3rd day dead): `src/farming/crops.rs:291-299`.
- 15 crops populated: `src/data/crops.rs` (15 `CropDef` entries).
- Sprinklers auto-water and scarecrow crow protection implemented: `src/farming/sprinklers.rs:23-44`, `src/farming/events_handler.rs:221-253`.

**Specced but not built / incomplete**
- None major identified.

**Built but not specced**
- Multiple sprinkler tiers including iridium behavior: `src/farming/sprinklers.rs:25-28`, `src/farming/sprinklers.rs:50-59`.

**Built incorrectly vs spec**
- Minor crop-season interpretation drift for “Any” label in spec; implementation constrains several “Any” examples to specific seasons (`src/data/crops.rs:193-228`).

---

### 4) Animals (`animals`) — 78%
**Implemented evidence**
- Shop purchases + trough feeding + happiness 0-255 + product collection: `src/data/shops.rs:158-209`, `src/animals/feeding.rs` (domain), `src/shared/mod.rs:565`, `src/animals/products.rs:67-99`.
- Chicken/cow daily products and sheep 3-day wool cadence: `src/animals/day_end.rs:191-223`.
- Companion no-product behavior for cat/dog: `src/animals/day_end.rs:285-287`.

**Specced but not built / incomplete**
- “Being outside” affecting happiness (spec `GAME_SPEC.md:81`) is not explicit in current happiness update model (`src/animals/day_end.rs:112-123`).

**Built but not specced**
- Many extra animal types (goat/duck/rabbit/pig/horse) and quality tiers: `src/animals/day_end.rs:225-284`, `src/animals/products.rs:71-77`.

**Built incorrectly vs spec**
- Baby->adult timing is 5 days in code, spec says 7 days (`GAME_SPEC.md:84`, `src/animals/day_end.rs:147-150`).

---

### 5) World & Maps (`world`) — 68%
**Implemented evidence**
- Tile-based maps and map transitions (edge + door logic): `src/world/maps.rs:79-91`, `src/player/interaction.rs:34-56`, `src/player/interaction.rs:60-145`.
- Collision layer sync and world objects with breakable tools: `src/world/mod.rs:106-116`, `src/world/objects.rs:297-337`.
- Seasonal visuals + forageable seasonal spawning: `src/world/mod.rs:82-97`, `src/world/objects.rs:675-705`.

**Specced but not built / incomplete**
- Spec map sizes are much larger than implementation maps (`GAME_SPEC.md:89` vs `src/player/interaction.rs:16-20`, and `src/world/maps.rs:96-99`, `src/world/maps.rs:259-262`).

**Built but not specced**
- Interior maps beyond overworld set (PlayerHouse/GeneralStore/AnimalShop/Blacksmith): `src/shared/mod.rs:592-595`, `src/world/maps.rs:88-91`.

**Built incorrectly vs spec**
- Transition inconsistency: farm west edge currently routes to Beach (`src/player/interaction.rs:72-74`) while world map defs/spec intent route farm west toward MineEntrance pathing (`GAME_SPEC.md:89`, `src/world/maps.rs:176-181`).

---

### 6) NPCs & Dialogue (`npcs`) — 76%
**Implemented evidence**
- 10 named NPCs with schedules: `src/data/npcs.rs` (10 `NpcDef` entries).
- Gift preferences and +8x birthday gift multiplier: `src/data/npcs.rs`, `src/npcs/gifts.rs:60-66`.
- Friendship 0-1000 / hearts: `src/shared/mod.rs:683-693`.
- Two marriage candidates supported by data/events: `src/data/npcs.rs` (`is_marriageable`), `src/main.rs:118-121`.

**Specced but not built / incomplete**
- Dialogue “tree” shape (greeting -> topics -> goodbye) is mostly linear/contextual line bundles, not explicit branch-tree state: `src/npcs/dialogue.rs:107-199`.

**Built but not specced**
- Quests/romance/wedding/spouse systems: `src/npcs/mod.rs:91-112`, `src/main.rs:122-124`.

**Built incorrectly vs spec**
- NPC movement is greedy step movement, not full pathfinding around obstacles (`src/npcs/schedule.rs:173-188`).

---

### 7) Shops & Economy (`economy`) — 90%
**Implemented evidence**
- 3 shops + fixed listings + start gold 500: `src/data/shops.rs:18-264`, `src/shared/mod.rs:308`.
- Shipping bin end-of-day settlement: `src/economy/shipping.rs:120-226`.
- Blacksmith upgrade tiers/costs and bar requirements: `src/shared/mod.rs:204-210`, `src/economy/blacksmith.rs:126-143`.

**Specced but not built / incomplete**
- None major.

**Built but not specced**
- Extra economy systems: achievements, building upgrades, year-end evaluation, detailed play stats: `src/economy/mod.rs:9-19`, `src/economy/mod.rs:74-103`.

**Built incorrectly vs spec**
- Shipping supports quality multipliers (not base-price-only simplification from spec): `src/economy/shipping.rs:122-155` vs `GAME_SPEC.md:112`.

---

### 8) Crafting & Cooking (`crafting`) — 92%
**Implemented evidence**
- Crafting bench + kitchen-gated cooking: `src/crafting/bench.rs:73-114`, `src/crafting/cooking.rs:62-66`.
- Recipe unlock and buff systems: `src/crafting/mod.rs:43-68`.
- Processing machines include furnace/preserves jar/cheese press/loom: `src/crafting/machines.rs:11-14`, `src/crafting/machines.rs:126-153`.
- Recipe counts meet/exceed target (21 crafting, 15 cooking): `src/crafting/recipes.rs:423-446`.

**Specced but not built / incomplete**
- None major.

**Built but not specced**
- Many additional machines/recipes (keg, oil maker, tapper, bee house, recycler, crab pot): `src/crafting/machines.rs:15-22`, `src/crafting/recipes.rs:423-446`.

**Built incorrectly vs spec**
- None major.

---

### 9) Fishing (`fishing`) — 90%
**Implemented evidence**
- Rod cast + timing minigame loop: `src/fishing/mod.rs:69-95`, `src/fishing/mod.rs:103-110`.
- Fish selection varies by season/location/time/weather: `src/data/fish.rs:9-14` and entries.
- Bait/tackle states and modifiers present: `src/fishing/mod.rs:203-217`, `src/fishing/mod.rs:165-177`.

**Specced but not built / incomplete**
- None major.

**Built but not specced**
- 28 fish species instead of 20 plus encyclopedia/skill systems: `src/data/fish.rs:3`, `src/fishing/mod.rs:121-157`.

**Built incorrectly vs spec**
- Scope expansion (28 vs 20) is intentional extension, not harmful mismatch.

---

### 10) Mining (`mining`) — 93%
**Implemented evidence**
- 20-floor mine, ladder progression, elevator unlock every 5 floors: `src/mining/ladder.rs:51-55`, `src/mining/ladder.rs:81-85`.
- Depth-based ore/gem tables and 3 enemy types: `src/mining/floor_gen.rs:188-241`, `src/mining/floor_gen.rs:254-270`.
- Health separated from stamina and combat loop present: `src/shared/mod.rs:279-280`, `src/mining/combat.rs:246-252`.

**Specced but not built / incomplete**
- None major.

**Built but not specced**
- Knockout penalties and mine HUD/elevator UI flow details: `src/mining/combat.rs:288-317`, `src/mining/mod.rs:70-75`.

**Built incorrectly vs spec**
- Combat is active pickaxe attack, not strictly bump-attack as stated in spec (`GAME_SPEC.md:140`, `src/mining/combat.rs:28-47`).

---

### 11) UI (`ui`) — 74%
**Implemented evidence**
- HUD time/date/gold/stamina/tool and map name/objectives: `src/ui/mod.rs:154-171`.
- Dialogue UI and advance flow: `src/ui/mod.rs:104-111`, `src/ui/mod.rs:266-279`.
- Shop/crafting/pause/map/relationships screens exist: `src/ui/mod.rs:280-320`, `src/ui/mod.rs:327-347`, `src/ui/mod.rs:404-428`.
- Friendship heart-meter view: `src/ui/relationships_screen.rs:134-139`, `src/ui/relationships_screen.rs:279-283`.

**Specced but not built / incomplete**
- Inventory drag-and-drop not found; inventory UI is cursor navigation/use actions (`GAME_SPEC.md:148`, `src/ui/inventory_screen.rs:99-112`, `src/ui/inventory_screen.rs:342-402`).

**Built but not specced**
- Toast system, tutorial hints, cutscene runner, touch overlay, minimap: `src/ui/mod.rs:176-194`, `src/ui/mod.rs:196-207`, `src/ui/mod.rs:72-102`.

**Built incorrectly vs spec**
- None major beyond missing drag-and-drop interaction model.

---

### 12) Save & Settings (`save`) — 72%
**Implemented evidence**
- Full-state serialization is broad and includes core + many extended resources: `src/save/mod.rs:394-457`.
- Exactly 3 save slots: `src/save/mod.rs:28`.
- Autosave on day end (sleep cycle trigger): `src/save/mod.rs:1126-1140`.

**Specced but not built / incomplete**
- Settings parity is incomplete vs spec: music/sfx split, window-size options, and editable keybind remap/persistence are not implemented in save/settings flow (`GAME_SPEC.md:161`; current settings overlay is mainly single volume display and static keybind list: `src/ui/settings_screen.rs:21-31`, `src/ui/settings_screen.rs:81-97`, `src/ui/settings_screen.rs:265-311`).

**Built but not specced**
- Quicksave/quickload keybinds (F5/F9): `src/save/mod.rs:1143-1161`.
- Save metadata cache and slot info presentation model: `src/save/mod.rs:31-58`, `src/save/mod.rs:109-113`.

**Built incorrectly vs spec**
- Settings are not part of `FullSaveFile` schema (no keybind/audio/window setting persistence fields): `src/save/mod.rs:388-457`.

## Global built-but-not-specced highlights
- Quests, romance/marriage, spouse actions, weddings, and relationship stages: `src/main.rs:79-85`, `src/main.rs:118-124`.
- Achievements, tutorial/objective hinting, evaluation systems: `src/main.rs:87-90`, `src/economy/mod.rs:74-103`, `src/ui/mod.rs:196-207`.
- Extended animals/machines/fish beyond baseline spec counts.

## Assumptions
- Percentage estimates treat each spec bullet as roughly equal weight within a domain.
- “Not built” means not found in main gameplay code paths under `src/` (no deep DLC parity audit performed).
- Where implementation extends scope beyond spec, items were marked “built but not specced” unless extension directly contradicts a spec constraint.
