# Hearthfield 2.0 — Autonomous Orchestrator Brief

You are the ORCHESTRATOR for Hearthfield 2.0. You have full autonomy over this codebase — rebuild, fix up, borrow parts, or start fresh. Your call.

## Required Reading (read these BEFORE making any decisions)

1. `THE_MODEL_IS_THE_ORCHESTRATOR.md` — your operating model (pyramid architecture, mechanical scope enforcement, delegation compression mitigation)
2. `GAME_SPEC.md` — the original game specification (what the game should be)
3. `status/retrospective/EXECUTIVE_SUMMARY.md` — synthesis of 8-worker retrospective analysis
4. `status/retrospective/spec-gap-analysis.md` — what's missing or wrong
5. `status/retrospective/performance.md` — performance hotspots
6. `status/retrospective/bevy-best-practices.md` — Bevy 0.15 patterns to adopt
7. `status/retrospective/code-quality.md` — code quality issues
8. `status/retrospective/architecture-analysis.md` — architecture assessment
9. `status/retrospective/alternative-architectures.md` — alternative approaches considered

Read ALL of these. They contain quantitative findings from a dedicated retrospective. Do not guess — the data is on disk.

## Your Authority

You may:
- Rebuild any or all of the codebase from scratch
- Fix up existing code as-is
- Borrow working parts and rewrite the rest
- Spawn nested worker sub-agents via `codex exec` for any implementation task
- Alter `src/shared/mod.rs` (the type contract) — it is NO LONGER frozen
- Create, delete, reorganize any files or directories
- Add new crate dependencies if genuinely needed
- Change the asset pipeline, data format, or rendering approach

**You decide the approach after reading the retrospective.** The existing code is 46K LOC, compiles clean, has 93 passing tests, and all 10 gameplay loops work mechanically. Whether that's worth keeping or rebuilding is your engineering judgment.

---

## WHAT WE DESIRE — The Player Experience Vision

This is not a technical spec. This is what the PLAYER should experience. Every technical decision serves this.

### The Feel

Hearthfield should feel like a **warm, cozy, handcrafted pixel-art world**. Think Stardew Valley's first spring morning. The player should feel:
- **Welcome** — the game teaches by doing, not by telling
- **Rewarded** — every action has satisfying visual and audio feedback
- **Immersed** — the world feels alive, not like a database with sprites
- **Unhurried** — this is a relaxation game, not a stress simulator

### Player's First 5 Minutes (this is the benchmark)

1. **Title screen**: Pixel art logo, gentle music, soft particle effects (fireflies? leaves?). "New Game" glows on hover. The menu itself feels like part of the world.

2. **Intro sequence**: Brief, beautiful. Grandfather's letter fades in with a warm sepia tone. The farm appears through a gentle fade — not a hard cut.

3. **First moment on the farm**: The player spawns on their overgrown farm. Morning light is warm and golden. Birds are chirping (ambient audio). A gentle breeze moves the grass. The camera settles smoothly — no snap.

4. **First tool use**: Player hoes a tile. The tool swing has a satisfying arc animation. Dirt particles puff up. A subtle "thunk" sound. The soil visually changes. The player FEELS they did something.

5. **First interaction**: Walk up to a shipping bin/NPC/object. A contextual prompt appears above it ("Press F to interact" or an icon). The NPC turns to face you. Dialogue box slides up smoothly, not instant-appears.

### What Every Action Should Feel Like

**Planting a seed**: Tiny planting animation, seed goes into soil, small sparkle. Next morning: a tiny sprout appears with a gentle pop.

**Watering crops**: Water arc from can, splash particles on soil, soil darkens. Sound: gentle splash. If sprinklers do it, show water spray particles at dawn.

**Harvesting**: Crop bounces, item pops out with a small arc, floats toward player with a satisfying collection sound. +1 toast with the item icon, not just text.

**Fishing**: Cast with a satisfying arc. Bobber hits water with a splash and ripples. Wait is atmospheric (water sounds, occasional bubble). Bite has an exclamation mark (!) and tension sound. Minigame is responsive. Catching a fish: fish jumps out of water, splash, item collected with fanfare proportional to rarity.

**Mining**: Pickaxe hits rock with screen shake (subtle, 2-3 pixels). Rock cracks with particle debris. Breaking reveals ore with a brief gleam. Enemies have death animations (poof/dissolve). Taking damage: red flash on player, brief knockback, screen shake.

**Talking to NPCs**: NPC has idle animation (breathing/blinking). Dialogue box has a typewriter effect. NPC portrait shows expression. Gift giving shows the item floating to the NPC, reaction animation (heart/sweat drop/anger cloud above head).

**Cooking/Crafting**: Recipe selection has item icons, not just text names. Crafting plays a brief animation (hammer/sparkle). Result item appears with a glow. Eating food shows a brief eating animation, buff icon appears on HUD.

**Day transitions**: End of day: gentle fade to black, summary screen (earnings, notable events). Dawn: fade from black, golden light, rooster crow, new day card ("Spring 15, Year 1"). HUD time starts ticking.

**Season transitions**: Not just a palette swap. Snow falling in the transition to winter. Leaves changing color in fall. Cherry blossoms in spring. The world transforms.

### The HUD

- **Clean and unobtrusive**: Only essential info visible. Time, gold, stamina, current tool.
- **Hotbar**: 12 slots at bottom, icons clearly visible, selected slot highlighted with a glow or bounce.
- **Stamina bar**: Fills/drains smoothly (not stepped). Changes color when low (green → yellow → red).
- **Gold counter**: Animate up/down when gold changes (counting ticker, not instant).
- **Tooltips**: Hover over any item → name, description, sell price. In inventory, in hotbar, in shops.
- **Minimap**: Top-right, shows terrain + player dot + NPC dots. Only redraws when state changes.
- **Context prompts**: Near interactable objects, show a small floating label or button icon.

### UI Screens

- **Inventory**: Grid of item icons with quantity badges. Drag-and-drop or cursor selection. Item details panel on the right. Trash/sell button.
- **Crafting**: Recipe list with icons, required materials shown with have/need counts, craft button with animation.
- **Shop**: Clean buy/sell tabs. Item icons, prices, player gold visible. Purchase confirmation with sound.
- **Dialogue**: Bottom-of-screen box. NPC name + portrait on left. Typewriter text. Player choices highlighted.
- **Journal**: Quest log with checkboxes, achievement gallery, fish/crop encyclopedia with found/unfound silhouettes.
- **Relationships**: NPC portraits with heart meters (visual hearts, not text). Gift preference hints for high-friendship NPCs.
- **Map**: World overview with labeled areas. Player location marker. Discovered vs undiscovered areas.
- **Settings**: Audio sliders (master/music/sfx), keybind remapping, window size.
- **Pause menu**: Semi-transparent overlay over the game world. Resume, Save, Settings, Quit.

### The World

- **Maps feel connected**: Walking off one edge smoothly transitions to the next. Not a hard cut — a brief slide or fade.
- **Seasonal variety**: Each season has distinct visual identity. Spring: cherry blossoms, bright green. Summer: golden sunlight, deep green. Fall: orange/red leaves, harvest colors. Winter: snow cover, bare trees, breath vapor.
- **Weather effects**: Rain has visible droplets, puddles form, sky darkens. Snow has gentle flakes accumulating. Storms have lightning flashes and heavier rain. Sunny days have warm light.
- **Time of day**: Gradual ambient color shift. Morning golden, midday bright, evening amber, night dark blue. Indoor areas have their own lighting (warm lamp light).
- **Ambient life**: Butterflies in spring/summer. Falling leaves in fall. Snowflakes in winter. Occasional bird flying overhead. Grass rustling in wind.
- **Objects feel present**: Trees sway slightly. Water has a subtle shimmer or animation. Crops progress visually through growth stages. Fences and paths look placed, not stamped.

### NPCs

- **Feel alive**: Have idle animations (not static sprites). Walk between locations on schedules. React to weather (umbrellas in rain, coat in winter — even if just a palette swap).
- **Conversations matter**: Dialogue varies by friendship level. High-friendship NPCs greet you differently. Birthdays are acknowledged.
- **Gift reactions are visible**: Love = big heart above head + happy text. Like = small heart. Neutral = "..." Dislike = sweat drop. Hate = anger cloud + negative text.
- **Romance is special**: Bouquet giving has a cutscene feel. Proposal is a memorable moment. Wedding is a festival event.

### Audio

- **Music**: Seasonal themes (4 minimum). Distinct music for mine, town, farm, festivals. Gentle transitions between tracks (crossfade, not hard cut).
- **SFX for every action**: Tool use, item pickup, menu open/close, dialogue advance, fishing cast/bite/catch, footsteps on different terrain, door open/close, purchase cha-ching, crop harvest pop, cooking sizzle.
- **Ambient layers**: Birds (morning), crickets (evening), wind, rain, water near rivers/ocean, fire crackling indoors.
- **Volume mixing**: SFX shouldn't drown out music. Ambient sits underneath both. Separate sliders for each.

---

## Critical Bugs to Fix

These are confirmed issues from the retrospective:

1. **Time scale is 10x off** — Code: 1 real second = 10 game minutes. Spec: 1 real minute = 10 game minutes. A full game day passes in ~2 real minutes instead of ~20. Fix: `src/shared/mod.rs` time_scale constant.

2. **NPC cleanup O(n²)** — `src/npcs/map_events.rs:31-40` uses `Vec::contains` inside `retain` loop. Convert to `HashSet<Entity>` for O(n). Will scale badly with more NPCs.

3. **Farm map west edge routing** — Walking west on the farm goes to Beach. Should go toward the Mine entrance based on world layout.

4. **Animal baby→adult age** — Code says 5 days, spec says 7 days. Fix in `src/animals/day_end.rs`.

5. **No "being outside" happiness bonus** for animals — spec requires it, code doesn't implement it.

6. **expect() crash risk** — `src/ui/main_menu.rs:439` will crash if DLC path resolution fails. Add fallback.

---

## Performance Hotspots to Fix

From `status/retrospective/performance.md`:

1. **Minimap**: Full 4096-pixel texture rewrite every frame. Cache and only update on state change.
2. **Farming render**: 3-pass full reconciliation of all crops/soil/objects every PostUpdate frame. Use dirty flags + change detection.
3. **Y-sort**: 3 separate passes over all y-sorted entities every frame. Combine into single pass.
4. **HUD proximity**: Scans all NPCs + chests + interactables every frame. Only rescan when player moves a tile.
5. **Weather particles**: Unbounded particle count checks. Add cap.
6. **No FixedUpdate usage** anywhere — player/NPC/enemy movement should be frame-rate independent.

---

## Architecture Improvements

From `status/retrospective/bevy-best-practices.md` and `architecture-analysis.md`:

1. **SystemSet taxonomy**: Define phases (Input → Intent → Simulation → Reactions → Presentation) in shared, configure in main.rs. Eliminates ad-hoc .before()/.after() chains.

2. **Split shared/mod.rs**: Currently 2,315 lines. Break into bounded contexts: `shared/calendar.rs`, `shared/economy.rs`, `shared/world.rs`, `shared/social.rs`, `shared/events.rs`, etc. Keep `shared::prelude` re-export.

3. **SubStates for overlays**: Dialogue, Shop, Inventory, Crafting, etc. should be SubStates of Playing, not top-level GameState variants. Use StateScoped entities for cleanup.

4. **FixedUpdate for simulation**: NPC movement, enemy AI, mine combat, animal wandering — deterministic mechanics belong in FixedUpdate.

5. **Data-driven content**: Item definitions, crop data, fish data, NPC data, recipes, shop inventories — externalize to RON or TOML files with schema validation tests.

6. **Reduce clone() density**: 496 clones in src/. Hotspot: save/mod.rs (41 clones). Review for avoidable full-structure clones.

7. **Split UiPlugin**: Currently mixes audio, transitions, dialogue, HUD, tutorial, menu input, and 15+ screens. Break into: HudPlugin, MenusPlugin, OverlayPlugin, etc.

---

## Quality Gates (must ALL pass when you're done)

```bash
cargo check                          # Zero errors
cargo test --test headless           # All tests pass (currently 93)
cargo clippy -- -D warnings          # Zero warnings
```

New tests should be added for:
- Any new systems or features
- Save/load round-trip with new fields
- Performance-sensitive paths (ensure caching works)
- Edge cases in fixed bugs

---

## Operating Model

Read `THE_MODEL_IS_THE_ORCHESTRATOR.md` thoroughly. Key principles:

1. **You dispatch, workers implement.** Use `codex exec --full-auto --skip-git-repo-check "$(cat objectives/WORKER_FILE.md)"` to spawn workers.
2. **Mechanical scope enforcement.** Workers edit whatever they want. You revert out-of-scope changes after with `git checkout`.
3. **Specs on disk, not in prompts.** Write worker specs to `objectives/` before dispatching. Include exact file paths, types, constants, and "done" criteria. Delegation compression destroys quantities — be specific.
4. **Validate after each worker.** At minimum: `cargo check`. Full gate suite for risky changes.
5. **Write status reports** to `status/workers/` for audit trail.

### Suggested Agent Group Rotation

You decide the structure, but consider:

1. **Audit agents** (first) — verify current state, find issues not in retrospective, confirm bug locations
2. **Architecture agents** — implement SystemSet taxonomy, split shared/mod.rs, restructure state model
3. **Bug fix agents** — fix the 6 critical bugs listed above
4. **Performance agents** — fix the 6 performance hotspots
5. **Visual polish agents** — add juice: particles, animations, screen shake, transitions, hover effects
6. **Audio agents** — ambient layers, crossfade, per-action SFX wiring
7. **UI polish agents** — tooltips, typewriter text, smooth transitions, hover states, item icons
8. **Integration/regression agents** — wire everything together, run full test suite, verify nothing broke
9. **Research agents** — for Bevy 0.15-specific patterns, shader questions, or novel problems

Multiple waves expected. Don't try to do everything in one pass.

---

## When Done

Write a final report to `status/orchestrator-hearthfield2-report.md`:
- Approach chosen (rebuild vs fix-up vs hybrid) and why
- Workers dispatched (count, objectives, outcomes)
- Files changed (created/modified/deleted, LOC delta)
- Player-visible improvements (list each one)
- Quality metrics (tests added/passing, clippy, performance)
- Remaining work / known issues / recommended next orchestrator run focus
- Total worker count and approximate token usage

Commit all changes. Push to the current branch.
