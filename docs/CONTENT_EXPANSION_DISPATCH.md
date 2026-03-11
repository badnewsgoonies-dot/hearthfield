# Hearthfield Parallel Content Expansion — Dispatch Package

## Architecture

```
Layer 0: Geni (meta-orchestrator)
  ├── Layer 1A: Orchestrator — House Interiors (Claude Code session)
  │     ├── Worker: PlayerHouse redesign
  │     ├── Worker: TownHouseWest interior
  │     ├── Worker: TownHouseEast interior  
  │     ├── Worker: Tavern interior
  │     └── Worker: Library interior
  │
  ├── Layer 1B: Orchestrator — Lake Plaza (Claude Code session)
  │     ├── Worker: Plaza map layout + terrain
  │     ├── Worker: Lakeside Bar exterior + NPC
  │     ├── Worker: Plaza shops + fountain
  │     └── Worker: Lake dock + fishing spots
  │
  └── Layer 1C: Orchestrator — Mountain Range (Claude Code session)
        ├── Worker: Mountain map layout + trails
        ├── Worker: Summit lookout + weather
        ├── Worker: Mountain cave entrance
        └── Worker: Alpine meadow + rare spawns
```

## Pre-Dispatch: Contract Amendment (YOU do this before launching anything)

Add these to `src/shared/mod.rs` MapId enum, then re-checksum:

```rust
pub enum MapId {
    Farm,
    Town,
    Beach,
    Forest,
    DeepForest,
    CoralIsland,
    MineEntrance,
    Mine,
    PlayerHouse,
    GeneralStore,
    AnimalShop,
    Blacksmith,
    // === NEW — Content Expansion ===
    TownHouseWest,
    TownHouseEast,
    Tavern,
    Library,
    LakePlaza,
    LakesideBar,     // interior
    MountainTrail,
    MountainSummit,
    MountainCave,
}
```

Also add any new ObjectTypes needed:

```rust
// In ObjectType enum, add:
BarStool,
TavernTable,
TavernBarrel,
BeerTap,
Bookshelf,
ReadingDesk,
Fountain,
PlazaBench,
StreetLamp,
MountainRock,
AlpineFlower,
TrailSign,
CaveCrystal,
Telescope,
```

Then:
```bash
shasum -a 256 src/shared/mod.rs > .contract.sha256
git add src/shared/mod.rs .contract.sha256
git commit -m "contract: extend MapId + ObjectType for content expansion"
git push
```

Also update `is_indoor_map()` in `src/world/lighting.rs`:

```rust
fn is_indoor_map(map_id: MapId) -> bool {
    matches!(
        map_id,
        MapId::PlayerHouse | MapId::GeneralStore | MapId::AnimalShop | MapId::Blacksmith
        | MapId::TownHouseWest | MapId::TownHouseEast | MapId::Tavern | MapId::Library
        | MapId::LakesideBar | MapId::MountainCave
    )
}
```

Commit that too. Now the contract is frozen and all three orchestrators share it.

---

## Orchestrator A — House Interiors

### Launch command (Claude Code on PC)

```bash
cd ~/swarm/hearthfield
claude --resume  # or new session
```

### Paste this as the orchestrator prompt:

```
You are Orchestrator A for the Hearthfield content expansion. Your domain is
HOUSE INTERIORS. You own all indoor maps and their contents.

ROLE: You dispatch workers via Copilot CLI. You do NOT implement code yourself
except for top-level wiring (map transitions, is_indoor_map updates). Workers
write all domain code.

READ FIRST (in this order):
1. src/shared/mod.rs (the frozen contract — do NOT edit)
2. src/world/map_data.rs (how maps are defined)
3. src/world/maps.rs (how maps are rendered)
4. src/world/objects.rs (how objects are placed)
5. src/world/lighting.rs (indoor lighting behavior)
6. Existing interior: look at how PlayerHouse is implemented as your template

YOUR MAPS (allowlist):
- TownHouseWest — cozy cottage, lived-in feel. Warm wood floors, fireplace,
  bookshelf, bed, table with chairs, potted plants. 12x10 tiles.
- TownHouseEast — fancy merchant house. Polished floors, display cases,
  expensive furniture, upstairs balcony area. 14x12 tiles.
- Tavern — the social hub. Bar counter with stools, tables, stage area for
  events, back room with storage barrels. 16x14 tiles. NPC bartender.
- Library — quiet knowledge. Floor-to-ceiling bookshelves, reading desks,
  cozy reading nook with lantern, card catalog. 14x12 tiles.
- PlayerHouse redesign — currently bare. Add: kitchen area, bedroom area,
  storage chest area, decoration slots. Keep existing 12x10 size.

EACH INTERIOR NEEDS:
- Map data entry in map_data.rs (tiles, objects, transitions)
- Furniture objects using ONLY ObjectTypes from shared/mod.rs
- A door transition FROM Town map (add the door trigger zone)
- Indoor lighting (is_indoor_map already updated for these)
- At least one interactable object per room
- Candle/lantern light sources for ambiance

WORKER DISPATCH PATTERN:
For each interior, create a worker spec file, then dispatch:

  export COPILOT_GITHUB_TOKEN="[YOUR_COPILOT_GITHUB_TOKEN]"
  copilot -p "$(cat objectives/interior-tavern.md)" --allow-all-tools --model claude-sonnet-4.6

After EACH worker:
  bash scripts/clamp-scope.sh src/world/
  cargo check
  cargo test
  git add -A && git commit -m "feat(world): Tavern interior"

SCOPE RULE: Workers may ONLY edit files in src/world/. They must NOT edit
src/shared/mod.rs. All ObjectTypes and MapIds are already in the contract.

INTEGRATION: After all 5 interiors are done, wire the door transitions from
the Town map yourself. Test each transition. Verify indoor lighting works.

BRANCH: You are on branch `content/interiors`. Commit and push to this branch.
Do NOT merge to master — the meta-orchestrator handles that.

DONE WHEN:
- All 5 interiors have map data, objects, transitions, and lighting
- Player can walk into each from Town
- Each has at least 1 interactable object
- All gates pass (cargo check, cargo test, cargo clippy)
- Write status/workers/interiors-complete.md
```

---

## Orchestrator B — Lake Plaza

### Launch command

```bash
cd ~/swarm/hearthfield
claude --resume  # separate session from A
```

### Paste this as the orchestrator prompt:

```
You are Orchestrator B for the Hearthfield content expansion. Your domain is
the LAKE PLAZA — a new outdoor area south of Town.

ROLE: You dispatch workers via Copilot CLI. You do NOT implement code yourself
except for top-level wiring.

READ FIRST:
1. src/shared/mod.rs (frozen contract)
2. src/world/map_data.rs (map definition pattern)
3. src/world/maps.rs (rendering)
4. src/world/objects.rs (object placement)
5. How Beach or Town maps are built — use as template for outdoor maps

YOUR MAPS:
- LakePlaza (outdoor) — 30x24 tiles. A lakefront plaza area:
  * Central fountain (Fountain object) surrounded by PlazaBench objects
  * Cobblestone paths radiating from fountain
  * StreetLamp objects along paths
  * Shop facades on the north side (decorative — future expansion)
  * Lake water on the south/east edge (Water tiles)
  * Fishing spots at the dock (reuse existing fishing system)
  * Trees and flower beds for ambiance
  * Map transition to Town (north edge)

- LakesideBar (indoor) — 14x10 tiles. Interior of the bar:
  * Bar counter with BarStool objects
  * TavernTable + chair clusters
  * TavernBarrel in the back
  * BeerTap behind the counter
  * NPC bartender (reuse NPC spawn pattern)
  * Warm indoor lighting
  * Door transition back to LakePlaza

VIBE: Relaxed lakeside feel. Late afternoon light reflecting off water.
The plaza is where villagers hang out in the evening. The bar is the
social hub after dark.

WORKER DISPATCH:
  export COPILOT_GITHUB_TOKEN="[YOUR_COPILOT_GITHUB_TOKEN]"
  copilot -p "$(cat objectives/lake-plaza.md)" --allow-all-tools --model claude-sonnet-4.6

After EACH worker:
  bash scripts/clamp-scope.sh src/world/
  cargo check
  cargo test
  git add -A && git commit -m "feat(world): Lake Plaza map"

SCOPE: Workers edit src/world/ ONLY. Contract is frozen.

FISHING INTEGRATION: The lake should have 2-3 fishing spots using the
existing fishing system. Read src/fishing/ to understand the spot format,
then place FishingSpot objects in the map data. Do NOT modify src/fishing/.

BRANCH: You are on branch `content/lake-plaza`. Commit and push to this branch.
Do NOT merge to master.

DONE WHEN:
- LakePlaza outdoor map renders with fountain, paths, lake, dock
- LakesideBar interior has bar, tables, barrels, lighting
- Player can walk from Town → LakePlaza → LakesideBar → back
- Fishing spots work at the lake dock
- All gates pass
- Write status/workers/lake-plaza-complete.md
```

---

## Orchestrator C — Mountain Range

### Launch command

```bash
cd ~/swarm/hearthfield
claude --resume  # separate session from A and B
```

### Paste this as the orchestrator prompt:

```
You are Orchestrator C for the Hearthfield content expansion. Your domain is
the MOUNTAIN RANGE — a new area east of the Forest.

ROLE: You dispatch workers via Copilot CLI. You do NOT implement code yourself
except for top-level wiring.

READ FIRST:
1. src/shared/mod.rs (frozen contract)
2. src/world/map_data.rs (map definition pattern)
3. How DeepForest or Beach maps are built — template for your maps

YOUR MAPS:
- MountainTrail (outdoor) — 24x30 tiles (tall, vertical layout). A winding
  trail up the mountainside:
  * Stone/dirt path winding upward (use Path + Stone tiles)
  * MountainRock objects scattered along edges
  * AlpineFlower clusters in sheltered spots
  * TrailSign objects at forks
  * Sparse trees at lower elevation, none at top
  * Map transition from Forest (south edge)
  * Map transition to MountainSummit (north edge)
  * Map transition to MountainCave (east, mid-trail)

- MountainSummit (outdoor) — 16x16 tiles. The peak:
  * Rocky terrain, minimal vegetation
  * Telescope object at the viewpoint
  * Dramatic wind (reuse tree sway system at high amplitude)
  * A few AlpineFlower objects in sheltered crevices
  * Transition back to MountainTrail (south)

- MountainCave (indoor) — 20x16 tiles. A natural cave:
  * Stone floor throughout
  * CaveCrystal objects that shimmer (reuse ore shimmer system)
  * Dark ambient lighting (indoor map, low light)
  * Narrow passages and a larger central chamber
  * Possible mining nodes (reuse mine ore system if compatible)
  * Transition back to MountainTrail (west)

VIBE: Rugged, elevated, sparse. The trail feels like a climb. The summit
is windswept and rewarding. The cave is mysterious and dark with crystal
gleams.

WORKER DISPATCH:
  export COPILOT_GITHUB_TOKEN="[YOUR_COPILOT_GITHUB_TOKEN]"
  copilot -p "$(cat objectives/mountain-trail.md)" --allow-all-tools --model claude-sonnet-4.6

After EACH worker:
  bash scripts/clamp-scope.sh src/world/
  cargo check
  cargo test
  git add -A && git commit -m "feat(world): Mountain Trail map"

SCOPE: Workers edit src/world/ ONLY. Contract is frozen.

BRANCH: You are on branch `content/mountain-range`. Commit and push to this branch.
Do NOT merge to master.

DONE WHEN:
- All 3 mountain maps render correctly
- Player can walk Forest → MountainTrail → Summit or Cave → back
- Cave has crystals and dark lighting
- Summit has telescope and wind effects
- All gates pass
- Write status/workers/mountain-range-complete.md
```

---

## Merge Plan (after all three orchestrators finish)

All three orchestrators edit src/world/ files. The maps are independent data
(different MapId values, different map_data entries) so they should merge
cleanly. But the wiring files (map transitions, the main map registry) will
have merge conflicts.

### Merge sequence:

1. Orchestrator A finishes → push to branch `content/interiors`
2. Orchestrator B finishes → push to branch `content/lake-plaza`
3. Orchestrator C finishes → push to branch `content/mountain-range`

4. Meta-orchestrator (you) merges in order:
   ```bash
   git checkout master
   git merge content/interiors
   # resolve any conflicts in map_data.rs, maps.rs
   cargo check && cargo test

   git merge content/lake-plaza
   # resolve conflicts
   cargo check && cargo test

   git merge content/mountain-range
   # resolve conflicts
   cargo check && cargo test
   ```

5. Final integration pass:
   - Wire Town → LakePlaza transition (south edge of Town)
   - Wire Forest → MountainTrail transition (east edge of Forest)
   - Verify all map transitions work end-to-end
   - Run full gate suite
   - Visual verification of every new map

### Expected conflicts (plan for these):
- `src/world/map_data.rs` — all three add entries to the map registry
- `src/world/maps.rs` — all three add rendering entries
- `src/world/objects.rs` — all three add object placement logic
- `src/world/mod.rs` — possible system registration conflicts

These are additive conflicts (each adds new code, nobody modifies existing code)
so resolution is concatenation, not choice.

---

## Timing Estimate

Each orchestrator dispatches 3-5 workers sequentially (clamp after each).
Worker runtime: ~1-3 min each via Copilot CLI.
Orchestrator overhead: ~2 min per worker (spec writing, clamping, gates).

Total per orchestrator: ~15-25 min
All three in parallel: ~25 min wall clock
Merge + integration: ~15 min

**Total: ~40 min for 10 new maps/interiors.**

---

## Quick-Start Commands (copy-paste ready)

### Terminal 1 — Contract amendment (do this FIRST)
```bash
cd ~/swarm/hearthfield
# Edit src/shared/mod.rs to add new MapId variants and ObjectTypes
# Edit src/world/lighting.rs to update is_indoor_map
shasum -a 256 src/shared/mod.rs > .contract.sha256
git add -A && git commit -m "contract: extend for content expansion"
git push
```

### Terminal 2 — Orchestrator A (interiors)
```bash
cd ~/swarm/hearthfield
git checkout -b content/interiors
claude  # paste Orchestrator A prompt
```

### Terminal 3 — Orchestrator B (lake plaza)
```bash
cd ~/swarm/hearthfield
git worktree add ../hearthfield-lake-plaza content/lake-plaza -b content/lake-plaza
cd ../hearthfield-lake-plaza
claude  # paste Orchestrator B prompt
```

### Terminal 4 — Orchestrator C (mountain range)
```bash
cd ~/swarm/hearthfield
git worktree add ../hearthfield-mountain content/mountain-range -b content/mountain-range
cd ../hearthfield-mountain
claude  # paste Orchestrator C prompt
```

Each orchestrator works in its own git worktree — same repo, separate working directories, separate branches. No stepping on each other. All three read from the same frozen contract (committed to master before branching). Merge at the end.
