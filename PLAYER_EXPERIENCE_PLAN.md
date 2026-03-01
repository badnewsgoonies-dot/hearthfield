# Hearthfield — Player Experience Fix Plan

## Methodology
Walked the entire player journey through code: every state transition, every system
registration, every input binding, every spawn function. Cross-referenced against
frame captures (taken as directional indicators, not gospel).

---

## Player Journey Audit

### Boot → Menu (seconds 0–5) ✅
- Loading state populates all registries → MainMenu
- Menu renders 3 options: New Game, Load Game, Quit
- **No issues found.**

### New Game → Intro Cutscene (seconds 5–15) ✅
- Click New Game:
  - Resets all resources
  - Gives: 15 turnip seeds, 5 potato seeds, 3 bread, 500g, all 6 tools
  - Fade overlay set to solid black, cutscene queue populated
  - State → Playing → start_pending_cutscene → Cutscene
- Cutscene runs on black screen:
  - "Three years ago, your grandfather left you a letter..." (4s)
  - "'Dear child, I've left you Hearthfield Farm...'" (5s)
  - "Spring 1, Year 1" (2.5s)
  - FadeIn(1.5) reveals farm
  - Wait(1.0)
  - Mayor Rex: 4 dialogue lines (seeds guidance, visit town, sleep before midnight)
  - Skip with Space
- Fade overlay Z(100) < text overlay Z(101) — text visible on black. ✓
- Fade resource persists across state transitions. ✓
- **Code path looks correct.** Needs real-build testing to confirm.

### First Look at Farm (seconds 15–30) ⚠️
- Player spawns at grid (10, 10) — bottom-left of farming dirt area (8–55, 8–47)
- Nearby furniture (all correctly guarded, atlas-backed):
  - Carpenter board: (10, 8) — 2 tiles north
  - Crafting bench: (12, 6) — nearby
  - Shipping bin: (14, 6) — nearby
- House entrance: (31, 0) — ~20 tiles northeast, stone tiles mark it
- HUD shows: "SPRING 1, YEAR 1 - 6:00 AM  SUNNY" | "HOE" | "500 G ████"
- Hotbar shows: ["Turni.", "", "", ...] — text-only, abbreviated names
- Camera now clamped to map bounds [FIXED this session]
- Tile gaps fixed [FIXED this session]
- ClearColor dark green [FIXED this session]

**Issues:**
- 🔴 No proximity labels on furniture — player walks past shipping bin with zero indication
- ⚠️ Hotbar text-only (no item sprites)
- ⚠️ No visual differentiation of interactable objects from scenery

### First Farming Actions (seconds 30–120) 🔴 BLOCKS PLAYER
- Tutorial objective 1: "Till some soil with your hoe (Space)" ✅
  - Hoe is default tool, Space tills dirt. Works.
- Tutorial objective 2: "Plant your turnip seeds (select seeds, press F)" ⚠️
  - "select seeds" means press 1 for hotbar slot 0. Not explained.
  - detect_seed_use checks F key + seed in selected slot + tilled soil. Works if player discovers "1".
- Tutorial objective 3: "Water your crops (select watering can, press Space)" 🔴 **WRONG**
  - Watering can is a TOOL. Cycled with `[` or `]`. NOT a hotbar item.
  - Tutorial text implies it's selectable like an inventory item.
  - Player has zero way to discover `[` `]` as tool cycle keys.
  - **This will block most players from completing the tutorial.**
- Tutorial objective 4: "Visit the town (walk south from the farm)" ✅
  - Path at (30, 48+) leads south. Transition at row 63.
- Tutorial objective 5: "Go home and sleep (press F on your farm)" ⚠️
  - trigger_sleep fires on F key anywhere on Farm/PlayerHouse map. No position check.
  - "Go home" implies they need to find their house — they don't. Any spot on farm works.

### Exploration (minutes 2–5) 🔴 CONFUSING
- NPCs all use character_spritesheet.png — identical appearance to player
- No NPC name tags — can't tell who is who
- No interaction prompts — no "[F] Talk" indicator near NPCs
- No proximity labels on furniture objects
- No controls reference anywhere (pause menu is Resume / Save / Quit only)

### Core Gameplay Loop (minutes 5+) ✅ MOSTLY WORKS
Thanks to Tier 0/1 fixes:
- ✅ Crafting routes through CraftItemEvent (proper unlock checks, SFX, inventory-full refund)
- ✅ Quest toasts fire on daily quest acceptance
- ✅ Tool upgrade UI added to blacksmith shop
- ✅ Achievement toasts fire
- ✅ Tool impact SFX fires
- ✅ Save system confirmed complete (10 resources, all in FullSaveFile)
- ✅ Event graph clean (0 unintentional orphaned events)
- ✅ 22 new SFX IDs mapped to audio files

---

## Prioritized Fix Plan

### P0: Player Gets Stuck (MUST FIX)

**P0-1: Fix tutorial text**
- File: `src/ui/tutorial.rs` line 144-150
- Change objective 3 from:
  `"Water your crops (select watering can, press Space)"`
  to:
  `"Water your crops (press [ or ] to switch to watering can, then Space)"`
- Change objective 2 from:
  `"Plant your turnip seeds (select seeds, press F)"`
  to:
  `"Plant your turnip seeds (press 1 to select seeds, then F)"`
- ~5 minutes, 0 risk

**P0-2: Add controls overlay on first play**
- When: During intro cutscene or as first toast sequence after Mayor Rex dialogue
- Show: WASD=Move, Space=Use Tool, F=Interact, []=Cycle Tool, 1-9=Hotbar, E=Inventory, C=Craft, Esc=Pause
- Options:
  - A) Add to intro sequence as ShowText step (simplest, ~10 min)
  - B) Add controls panel to pause menu (more useful long-term, ~30 min)
  - C) Both (best UX)
- Recommend: A first, then B

### P1: Player Confused But Not Stuck

**P1-1: Proximity interaction prompts**
- New system: `show_interaction_prompts` in `src/ui/hud.rs`
- When player is within TILE_SIZE*1.5 of an Interactable, show floating "[F] {label}" text
- Labels already exist on Interactable components: "Ship Items", "Crafting Bench", "Building Upgrades"
- Also show for NPCs: "[F] Talk to {npc_name}"
- ~1 worker, medium risk (new UI rendering)

**P1-2: NPC name tags**
- New system: render NPC name as floating text 8px above sprite
- Uses existing `npc.id` → look up display name from NpcRegistry
- ~1 worker, low risk (spawn Text2d child on NPC entities)

**P1-3: Fix tutorial objective 5 text**
- Change: `"Go home and sleep (press F on your farm)"`
  to: `"End the day (press F anywhere on your farm to sleep)"`
- ~2 minutes, 0 risk

### P2: Ugly But Functional

**P2-1: NPC visual differentiation**
- Quick approach: tint each NPC sprite a unique color
  - Margaret: warm brown, Marco: olive, Lily: pink, etc.
  - Set `sprite.color` per NPC during spawning based on npc.id
- Better approach: add NPC-specific atlas frames (requires new art)
- Recommend: tinting as immediate fix, ~1 worker

**P2-2: Hotbar item icons**
- Currently: text-only ("Turni.", "x15")
- Ideal: render item atlas sprites in each slot
- Complex: need to load items atlas, get sprite index per item, render as ImageNode in UI
- Prerequisite: ItemDef needs atlas_index field (check if it exists)
- ~2 workers, medium risk

**P2-3: Controls section in pause menu**
- Add 4th option "Controls" to PAUSE_OPTIONS
- Show keybindings panel when selected
- ~1 worker, low risk

### P3: Polish

**P3-1: Season-aware music** — Switch BGM on SeasonChangeEvent and MapTransitionEvent
**P3-2: Interactable highlight** — Subtle glow/pulse on objects when player is nearby
**P3-3: Crop growth indicators** — Visual stages beyond color change
**P3-4: Map/minimap** — Help with navigation between areas

---

## Execution Order

```
P0-1  Fix tutorial text                    [direct edit, 5 min]
P0-2a Add controls to intro cutscene       [direct edit, 10 min]
P1-3  Fix sleep tutorial text              [direct edit, 2 min]
      ─── commit + push ───
P1-1  Proximity interaction prompts        [1 worker, 30 min]
P1-2  NPC name tags                        [1 worker, 20 min]
P2-1  NPC tint colors                      [1 worker, 15 min]
      ─── commit + push ───
P0-2b Controls in pause menu               [1 worker, 30 min]
P2-2  Hotbar item icons                    [1-2 workers, 45 min]
      ─── commit + push ───
P3-*  Polish items as time allows
```

## Risk Assessment

| Fix | Risk | Why |
|-----|------|-----|
| P0-1 Tutorial text | None | String literal change |
| P0-2a Intro controls | None | Add CutsceneStep::ShowText |
| P1-1 Proximity prompts | Medium | New UI entities that spawn/despawn per frame |
| P1-2 NPC name tags | Low | Simple Text2d child entities |
| P2-1 NPC tinting | Low | Set sprite.color in spawn function |
| P2-2 Hotbar icons | Medium | UI rendering changes, atlas loading in UI context |
| P2-3 Pause controls | Low | Static UI panel |

## Known-Good Systems (no changes needed)
- Intro cutscene code path (fade overlay Z-ordering correct, resource persistence correct)
- Save/load system (all 10 resources confirmed in FullSaveFile)
- Furniture spawning (guards, atlas sprites, positions)
- Seed planting (F-key, facing tile detection, season check)
- Tool cycling ([ ] keys, TOOL_ORDER array)
- Sleep system (F anywhere on farm, cutscene for day transition)
- Input routing (InputPlugin, InputContext, all state mappings)
- State transitions (Playing ↔ Cutscene guards verified, no double-spawn risks)
