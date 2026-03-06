# Hearthfield 2.0 — Multi-Run Refresh Plan

## Philosophy

**Refresh, not remake.** The foundation is solid (46K LOC, 93 tests, all loops working, clean compile). We fix architecture, add polish, and bring every domain up to spec — domain by domain, run by run, quality over speed.

Every decision serves one question: **what does the player see and feel?**

## Architecture Target (applied incrementally per domain)

These are applied as each domain gets its refresh pass:

1. **SystemSet taxonomy** — Input → Intent → Simulation → Reactions → Presentation
2. **Split shared/mod.rs** — bounded contexts (`shared/calendar.rs`, `shared/economy.rs`, etc.) with `shared::prelude` re-export
3. **SubStates for overlays** — Dialogue/Shop/Inventory/Crafting as SubStates of Playing, StateScoped cleanup
4. **FixedUpdate for simulation** — NPC movement, enemy AI, mine combat, animal wandering
5. **Data-driven content** — externalize item/crop/fish/NPC/recipe/shop data to RON files with validation tests
6. **Change-driven updates** — dirty flags + event-triggered sync, no per-frame full-world scans

## Run Schedule

Each run = one orchestrator session dispatching workers. Runs are sequential across sessions.
Each run should: read this plan, read the retrospective, implement its scope, validate, commit.

### Run 0: Triage (CURRENT — in progress)
**Scope**: Fix critical bugs + performance hotspots
**Status**: Codex orchestrator running
- [x] Time scale fix (1s=10min → 1min=10min)
- [x] NPC O(n²) cleanup → HashSet
- [x] Farm map west routing fix
- [x] Animal baby→adult age 5→7
- [x] expect() crash guard in main_menu
- [ ] Minimap caching (stop full rewrite every frame)
- [ ] Farming render caching (dirty flags)
- [ ] Y-sort single pass
- [ ] HUD proximity scan on player move only
- [ ] Weather particle cap

### Run 1: Shared Contract + SystemSet Foundation
**Scope**: Architecture foundation that all subsequent runs depend on
- Split `src/shared/mod.rs` (2315 LOC) into bounded modules
- Define `SystemSet` enum phases in shared
- Configure `configure_sets(Update, ...)` in main.rs
- Move existing systems into sets (no behavior change)
- Add `FixedUpdate` schedule for simulation systems
- Introduce `SubStates` for Playing sub-modes
- **Validation**: cargo check + all 93 tests still pass + clippy clean

### Run 2: Calendar + World Domain Refresh
**Scope**: Time, weather, maps, seasonal visuals — the world the player inhabits
- Fix time scale to match spec exactly (1 real min = 10 game min)
- Seasonal palette transitions (not just tint — actual visual identity shift)
- Weather particle polish (proper rain droplets, snow flakes, storm effects)
- Day/night ambient color: morning golden → midday bright → evening amber → night blue
- Map transitions: smooth slide/fade instead of hard cut
- Ambient particle systems: butterflies (spring/summer), falling leaves (fall), snowflakes (winter)
- Externalize calendar/festival data to RON
- Add ambient audio hooks (bird/cricket/wind events)
- **Player feels**: The world is alive and changes around me

### Run 3: Player + Input Domain Refresh
**Scope**: How the player moves, acts, and interacts — the core feel
- 4-directional walk/run animations (smooth, not grid-snapped)
- Tool use animations with satisfying arcs + particle feedback
- Stamina bar: smooth fill/drain, color shift (green → yellow → red)
- Number key direct tool equip (spec requirement, currently missing)
- Context prompts near interactable objects (floating icon/label)
- Item pickup: collection arc animation + sound + icon toast
- Camera: smooth follow with slight lag, no snap on map transition
- Collision feel: no sticking on corners (corner-slide)
- **Player feels**: Movement is responsive and every action has feedback

### Run 4: Farming Domain Refresh
**Scope**: The core loop — planting, watering, harvesting
- Planting animation + seed-into-soil sparkle
- Watering: water arc from can, splash particles, soil darkens
- Crop growth: visible stage transitions with gentle pop
- Harvest: crop bounces, item arcs toward player, collection sound
- Sprinkler: dawn water spray particles
- Withering/dead crop visual (brown, droopy)
- Externalize crop data to RON (currently in data/crops.rs as code)
- Farming render: change-driven (dirty flags), not per-frame reconciliation
- **Player feels**: Farming is satisfying and I can see my crops grow

### Run 5: Animals Domain Refresh
**Scope**: Livestock care loop
- Animal idle animations (pecking, grazing, sleeping)
- Feeding animation + happy reaction
- Petting: heart particle above animal
- Product collection: item pop with quality indicator
- Baby → adult growth visual (size interpolation)
- "Being outside" happiness bonus (spec requirement, now implemented)
- Externalize animal data to RON
- **Player feels**: Animals feel alive and I care about them

### Run 6: NPC + Social Domain Refresh
**Scope**: The people of Hearthfield
- NPC idle animations (breathing/blinking minimum)
- Dialogue: typewriter text effect, smooth box slide-in
- Gift reactions: visible emote above head (heart/sweat/anger)
- Friendship UI: visual heart meters (not text)
- NPC schedule awareness: umbrella in rain, coat in winter (palette swap)
- Romance: bouquet/proposal as memorable moments with special UI
- Externalize NPC data + dialogue to RON
- **Player feels**: NPCs are characters, not databases

### Run 7: Economy + Crafting Domain Refresh
**Scope**: Shops, shipping, crafting, cooking
- Shop UI: item icons, clear prices, purchase confirmation sound
- Shipping bin: item toss animation, end-of-day earnings summary
- Crafting UI: recipe icons with material have/need counts, craft animation
- Cooking: sizzle sound, brief animation, buff icon appears on HUD
- Gold counter: animated tick up/down on change
- Tooltips on all items (name, description, sell price)
- Externalize recipes + shop data to RON
- **Player feels**: Buying, selling, and crafting are clean and satisfying

### Run 8: Fishing + Mining Domain Refresh
**Scope**: The adventure loops
- Fishing cast: satisfying arc, bobber splash + ripples
- Bite: exclamation mark (!) + tension sound
- Catch: fish jumps from water, splash, rarity-proportional fanfare
- Mining pickaxe: screen shake (2-3px), rock crack particles
- Ore reveal: brief gleam on breaking
- Enemy death: poof/dissolve animation
- Player damage: red flash, knockback, screen shake
- Elevator unlock: satisfying milestone feel
- **Player feels**: Fishing is atmospheric, mining is visceral

### Run 9: UI + Audio Polish
**Scope**: Everything the player touches that isn't gameplay
- All menu screens: hover effects, smooth transitions
- Inventory: drag-and-drop or clear cursor nav, item detail panel
- Pause menu: semi-transparent overlay over world
- Journal: quest checkboxes, encyclopedia with found/unfound silhouettes
- Settings: audio sliders (master/music/sfx), keybind remapping, window size
- Day transition: gentle fade, summary screen, dawn fade-in with rooster
- Season transition: visual event (snow falling, leaves changing, blossoms)
- Music crossfade between areas (not hard cut)
- Ambient audio layers (birds, crickets, wind, water, fire)
- SFX for every action (footsteps on terrain, door, menu, dialogue advance)
- **Player feels**: The UI is part of the world, not bolted on

### Run 10: Save System + Settings + Integration
**Scope**: Persistence and final integration
- Save/load round-trip tests for all new resources/fields
- Migration path from v1 → v2 saves
- Settings persistence (audio levels, keybinds, window size)
- Auto-save on sleep
- 3 save slots with preview (day, gold, playtime)
- Full integration test pass: every domain's changes work together
- Performance budget tests for high-entity scenarios
- **Player feels**: My progress is safe and my preferences remembered

### Run 11: Final Polish + Spec Compliance Audit
**Scope**: Gap closure
- Audit every spec requirement against implementation
- Fix remaining mismatches
- Add missing negative/error path coverage
- Final clippy + test pass
- Write completion report
- **Player feels**: This is a complete, polished game

## Orchestrator Instructions (for each run)

```
You are the Hearthfield 2.0 orchestrator for Run N.

Read these files BEFORE doing anything:
1. HEARTHFIELD_2_PLAN.md — this file (find your run, execute its scope)
2. GAME_SPEC.md — what the game should be
3. status/retrospective/EXECUTIVE_SUMMARY.md — what went wrong before
4. THE_MODEL_IS_THE_ORCHESTRATOR.md — your operating model

Dispatch workers for implementation. Validate after each worker.
Quality over speed. Player experience is the priority.

Gates (must pass when done):
  cargo check
  cargo test --test headless
  cargo clippy -- -D warnings

Write your run report to status/orchestrator-run-N-report.md
Commit and push when done.
```

## Success Criteria

When all 12 runs are complete, a player should be able to:
1. Launch the game and feel welcomed by a warm, polished title screen
2. Start a new game and experience a beautiful intro sequence
3. Farm with satisfying visual and audio feedback on every action
4. Fish with atmosphere and tension
5. Mine with visceral combat feedback
6. Talk to NPCs who feel alive
7. Cook, craft, and shop through clean, polished UI
8. Watch the seasons change and the world transform
9. Save their progress and have their settings remembered
10. Play for hours without encountering a bug, stutter, or jarring transition

That's Hearthfield 2.0.
