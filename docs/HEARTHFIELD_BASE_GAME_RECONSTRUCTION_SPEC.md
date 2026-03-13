# Hearthfield Base Game Reconstruction Spec

Purpose: define the concrete reconstruction target for the **base game as it exists now** in the repo, using the current code and tests as the primary source of truth.

This document is not a nostalgia brief or a loose design note.
It is the operational target for rebuilding or re-orchestrating the current base game to an objective quality floor.

## 1. Source of Truth

Primary truth order for this spec:

1. `src/`
2. `tests/headless.rs`
3. `tests/keybinding_duplicates.rs`
4. `assets/maps/*.ron`
5. `ACCEPTANCE.md`
6. `GAME_SPEC.md`
7. `ROADMAP_TO_SHIP.md`

If these disagree, code and tests win.

## 2. Current Measurable Baseline

Current observed baseline from the repo:

- 15 top-level base-game domains/plugins in `src/`
- 18 shipped map RON files in `assets/maps/`
- 212 headless integration tests
- 1 keybinding regression test
- 32 shared events
- 37 shared resources

Current hard-coded content registries:

- 16 crop definitions
- 31 fish definitions
- 237 item definitions
- 11 NPC definitions
- 64 recipes
- 63 shop listings

Current asset surface counts in `assets/`:

- 18 map RON files
- 122 PNG files under `assets/sprites/`
- 17 PNG files under `assets/tilesets/`
- 32 PNG files under `assets/ui/`
- 31 `.ogg` audio files

These are baseline counts only.
The machine-readable parity baselines now live under `status/research/`.

## 3. Structural Reconstruction Target

The reconstruction must preserve the base-game lattice:

- `calendar`
- `player`
- `farming`
- `animals`
- `world`
- `npcs`
- `economy`
- `crafting`
- `fishing`
- `mining`
- `ui`
- `save`
- `data`
- `input`
- `shared`

The target is not “same approximate shape.”
The quality floor is:

- same domain coverage
- same relative function
- same style of coordination through shared resources/events/state

## 3A. Allowed Structural Improvements

These are allowed if the quality floor is preserved:

- file/module splits and moves
- clearer internal naming
- stronger manifests, inventories, and audit artifacts
- stronger regression tests and hardening coverage
- cleaner scheduling/setup code
- asset preloading / cache improvements
- internal refactors that do not change preserved runtime surfaces

These are not allowed unless explicitly logged as a target deviation:

- reducing or collapsing runtime surface coverage
- changing map graph behavior
- changing the meaning of current resource/event contracts
- changing content counts or registry semantics
- replacing correct visuals with placeholders, generics, or “close enough” substitutions
- shifting timings, progression, economy, or transition behavior under the label of cleanup

## 4. Actual Domain/System Families To Preserve

### Calendar

Files:

- `src/calendar/mod.rs`
- `src/calendar/festivals.rs`

Must preserve:

- day progression
- season cycling
- calendar/time state
- festival activation and cleanup
- day-end and season-change events

### Player

Files:

- `spawn.rs`
- `movement.rs`
- `camera.rs`
- `interaction.rs`
- `interact_dispatch.rs`
- `item_use.rs`
- `tools.rs`
- `tool_anim.rs`

Must preserve:

- spawn state
- logical/world position handling
- collision-aware movement
- map transitions
- interaction routing
- tool use and stamina coupling
- tool feedback and animation
- camera behavior

### Farming

Files:

- `soil.rs`
- `crops.rs`
- `harvest.rs`
- `render.rs`
- `events_handler.rs`
- `sprinkler.rs`
- `sprinklers.rs`
- `tool_fx.rs`

Must preserve:

- till / plant / water / grow / harvest loop
- crop season validation
- crop death/wither rules
- sprinklers
- crop render state
- farming tool feedback

### Animals

Files:

- `spawning.rs`
- `movement.rs`
- `feeding.rs`
- `products.rs`
- `day_end.rs`
- `interaction.rs`
- `rendering.rs`

Must preserve:

- animal spawning and housing
- hunger/happiness state
- petting / feeding effects
- product generation
- aging
- rendering and movement

### World

Files:

- `map_data.rs`
- `maps.rs`
- `mod.rs`
- `objects.rs`
- `lighting.rs`
- `weather_fx.rs`
- `tree_fx.rs`
- `grass_decor.rs`
- `seasonal.rs`
- `chests.rs`
- `ysort.rs`

Must preserve:

- map registry and map loading
- reachable world graph
- tile and object spawning
- outdoor collision rules
- door / edge transitions
- indoor vs outdoor lighting
- weather particles and cleanup
- tree / seasonal effects
- chest placement and interaction
- y-sort ordering

### NPCs

Files:

- `definitions.rs`
- `spawning.rs`
- `schedule.rs`
- `schedules.rs`
- `dialogue.rs`
- `gifts.rs`
- `quests.rs`
- `romance.rs`
- `map_events.rs`
- `animation.rs`
- `emotes.rs`
- `idle_behavior.rs`

Must preserve:

- NPC definitions and portraits/sprites
- schedule data and map placement
- dialogue system
- gifting/friendship
- quests
- romance progression
- emotes and idle behavior
- map-specific spawn/despawn behavior

### Economy

Files:

- `gold.rs`
- `shipping.rs`
- `shop.rs`
- `blacksmith.rs`
- `tool_upgrades.rs`
- `buildings.rs`
- `achievements.rs`
- `play_stats.rs`
- `evaluation.rs`
- `stats.rs`

Must preserve:

- gold change logic
- shipping bin sale flow
- shop buy/sell flow
- blacksmith/tool upgrade flow
- building upgrades
- achievements and play stats
- year-3 evaluation flow

### Crafting

Files:

- `recipes.rs`
- `bench.rs`
- `cooking.rs`
- `machines.rs`
- `buffs.rs`
- `unlock.rs`

Must preserve:

- recipe registry usage
- crafting bench loop
- cooking loop
- machine processing
- food buffs
- recipe unlocks

### Fishing

Files:

- `cast.rs`
- `bite.rs`
- `minigame.rs`
- `resolve.rs`
- `render.rs`
- `fish_select.rs`
- `skill.rs`
- `treasure.rs`
- `legendaries.rs`

Must preserve:

- cast/bite/resolve sequence
- minigame
- fish selection by conditions
- fishing skill progression
- treasure behavior
- legendary fish handling
- fishing UI/rendering

### Mining

Files:

- `floor_gen.rs`
- `spawning.rs`
- `components.rs`
- `movement.rs`
- `rock_breaking.rs`
- `rock_impact.rs`
- `combat.rs`
- `ladder.rs`
- `transitions.rs`
- `hud.rs`
- `anim.rs`

Must preserve:

- mine floor generation
- mine entity spawning
- rock breaking and drops
- movement and collision
- combat
- ladder/floor transitions
- mine HUD
- mine animation

### UI

Files:

- `main_menu.rs`
- `pause_menu.rs`
- `settings_screen.rs`
- `hud.rs`
- `inventory_screen.rs`
- `journal_screen.rs`
- `relationships_screen.rs`
- `map_screen.rs`
- `calendar_screen.rs`
- `stats_screen.rs`
- `shop_screen.rs`
- `crafting_screen.rs`
- `chest_screen.rs`
- `building_upgrade_menu.rs`
- `dialogue_box.rs`
- `intro_sequence.rs`
- `cutscene_runner.rs`
- `transitions.rs`
- `tutorial.rs`
- `tool_tutorial.rs`
- `toast.rs`
- `audio.rs`
- `debug_overlay.rs`
- `menu_input.rs`
- `menu_kit.rs`

Must preserve:

- all named screens and overlays
- menu flow and pause/settings flow
- dialogue and cutscene presentation
- tutorial surfaces
- toast/audio feedback
- map and relationship screens
- shop/crafting/chest flows

### Save

Files:

- `src/save/mod.rs`

Must preserve:

- save/load roundtrip for the current shared/base-game resources
- slot behavior
- current map + player position persistence
- combined resource serialization

### Data

Files:

- `items.rs`
- `crops.rs`
- `fish.rs`
- `npcs.rs`
- `recipes.rs`
- `shops.rs`

Must preserve:

- startup population of all registries
- exact content classes
- registry availability before gameplay

### Input

Files:

- `src/input/mod.rs`

Must preserve:

- gameplay/menu input context abstraction
- interaction claiming and blocking
- keybinding defaults

### Shared

Files:

- `src/shared/mod.rs`
- `src/shared/schedule.rs`

Must preserve:

- cross-domain resource types
- events
- enums/states
- contract semantics

## 5. Map / World Graph Baseline

Current shipped maps:

- `farm`
- `town`
- `town_west`
- `beach`
- `forest`
- `deep_forest`
- `mine_entrance`
- `mine`
- `player_house`
- `town_house_west`
- `town_house_east`
- `general_store`
- `animal_shop`
- `blacksmith`
- `library`
- `tavern`
- `coral_island`
- `snow_mountain`

The replica must preserve:

- the same reachable graph
- the same interiors/exteriors
- the same key transitions
- the same named map identities

## 6. Current UI / Screen Baseline

Current screen/overlay families present in code:

- main menu
- pause menu
- settings
- HUD
- inventory
- journal
- relationships
- map
- calendar
- stats
- shop
- crafting
- chest
- building upgrade
- dialogue box
- intro sequence
- cutscene runner
- transitions
- tutorial and tool tutorial
- toast
- audio-backed feedback
- debug overlay

Replica failure condition:

- any shipped player-facing screen family is missing or downgraded to placeholder-only behavior

## 7. Content / Registry Baseline

Current registry-backed content classes:

- items
- crops
- fish
- NPCs
- recipes
- shop listings

Replica failure condition:

- any content class disappears
- registry population order or availability breaks gameplay start
- registry counts or semantics regress without explicit, justified target changes

## 8. Test / Invariant Baseline

Current test baseline:

- 212 headless tests
- 1 keybinding regression test

Current auditable test families include:

- boot smoke
- crop growth / wither / rain / season validation
- shipping / economy / gold clamp
- animal happiness / feeding / products / aging
- calendar / season / festival behavior
- watering can tiers / sprinkler patterns
- play stats / achievements
- relationships / bouquet / proposal / wedding
- quests / evaluation
- y-sort and position math
- player animation/tool state
- crafting and recipe logic
- mining state and rock interactions
- save/load roundtrips across many resources
- item sprite index bounds
- fishing skill / legendary behavior
- input blocks and contexts
- map transitions and current-map persistence
- snow mountain / town west / library / tavern / tutorial later-day regressions

Replica failure condition:

- any existing graduated invariant is lost
- any existing regression test family becomes unrepresentable

## 9. Asset / Visual Baseline

Current directly loaded asset families include:

- character sprite sheets
- NPC sprite files and portraits
- crop sheets / plants atlas
- animal sheets
- fishing atlas
- mining atlas / mine enemies
- furniture atlas
- world object atlases
- building sprites
- tilesets
- UI atlases and icon sheets
- weather icons
- fonts
- music
- SFX

The replica must preserve:

- loaded asset family coverage
- sprite/atlas role coverage
- current correct visual mappings
- no placeholder substitution where the baseline already has a correct concrete visual

## 10. Machine-Readable Baselines

The current baseline set is:

1. `status/research/runtime_used_asset_manifest.csv`
   - assets directly referenced by current runtime loaders, mappings, or map registry loading
   - source file
   - loader kind
   - role hint

2. `status/research/asset_manifest.csv`
   - full repo asset inventory
   - asset family
   - role class
   - file size

3. `status/research/visual_mapping_manifest.csv`
   - fixed loaded visual assets
   - item/crop/fish/NPC sprite mappings
   - source module

4. `status/research/reachable_surface_manifest.csv`
   - map definitions
   - door / transition / edge links
   - map sizes and spawn positions

5. `status/research/runtime_surface_manifest.csv`
   - player-facing loop inventory
   - entry conditions
   - primary files
   - preserve requirements

6. `status/research/test_baseline_manifest.csv`
   - current test name
   - suite
   - inferred invariant family

7. `status/research/plugin_resource_event_inventory.csv`
   - plugin/module registrations
   - resource registrations
   - event registrations
   - shared contract resources/events

These baselines make the reconstruction target materially more auditable.

Validation command:

```bash
python3 scripts/validate_reconstruction_baselines.py
```

Current limitation:

- `runtime_used_asset_manifest.csv` is stronger than the broad inventory, but it is still derived from code-level loaders and mapping tables rather than runtime tracing
- `asset_manifest.csv` remains a broad repo inventory, useful for omission audits but not sufficient alone
- `visual_mapping_manifest.csv` is a seed baseline for role coverage, not a complete pixel-cell truth table
- `runtime_surface_manifest.csv` is a strong gameplay-loop floor, but not a full per-state playback proof

So zero omission / zero visual regression is now partially auditable, but not yet maximally strict.

## 11. Reconstruction Acceptance Standard

The rebuild passes only if:

- domain/plugin parity holds
- runtime surface parity holds
- content registry parity holds
- reachable world graph parity holds
- screen/overlay parity holds
- baseline gates stay green
- existing invariants stay green
- zero asset omission holds
- zero new bugs are introduced
- zero visual regressions occur where baseline visuals are already correct

## 12. Recommended Task Decomposition

This is the minimum bounded-task program I would trust:

1. bootstrap / contract / state / manifests
2. registry population parity
3. input abstraction + keybindings
4. player spawn + camera
5. player movement + interaction
6. player tools + animations
7. calendar + festivals
8. soil + crop state
9. farming render + tool FX
10. harvest + season validation + sprinklers
11. animal spawn/render/movement
12. feeding + happiness + products + aging
13. world map registry + loading
14. world tile/object spawning
15. world collision + transitions
16. lighting + weather + seasonal + y-sort
17. NPC definitions + spawning
18. schedules + map events
19. dialogue + gifts + quests + romance + emotes
20. economy gold + shipping + shop
21. blacksmith + tool upgrades + buildings
22. achievements + stats + evaluation
23. recipe registry + crafting bench
24. machines + cooking + buffs + unlocks
25. fishing cast/bite/minigame/resolve
26. fishing skill + treasure + legendaries
27. mining generation + spawning + components
28. mining rock/combat/ladder/transitions/HUD/anim
29. UI shell: menus, pause, settings
30. UI gameplay: HUD, inventory, journal, relationships, map, calendar, stats
31. UI interaction: shop, crafting, chest, building upgrade, dialogue, cutscenes, tutorial, transitions, audio, toast
32. save/load parity
33. asset/visual parity pass
34. regression and hardening pass

This is the minimum shape.
Fewer tasks will likely over-compress distinct system families and hide regressions.

## 13. Bottom Line

The target is not a game that feels “close enough” to Hearthfield.

The target is the current base game, reconstructed or improved to at least:

- the same breadth
- the same relative function
- the same system-family coverage
- the same or better visual correctness
- with zero missing runtime-used asset roles
- zero new bugs
- zero baseline-correct visual downgrades

Anything below that is a failed reconstruction.
