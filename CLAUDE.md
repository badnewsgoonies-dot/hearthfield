# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Hearthfield is a Harvest Moon-style farming simulator built in Rust with Bevy 0.15. The full game design is in `GAME_SPEC.md`. ~116 source files, ~35k lines of Rust.

## Build Commands

```bash
cargo check                  # Type-check without building (fast iteration)
cargo build                  # Full debug build
cargo run                    # Run the game (requires GPU/window)
cargo test --test headless   # Run headless integration tests (no GPU needed)
```

WASM target is configured (`profile.wasm-release` in Cargo.toml, `webgl2` feature enabled) but no wasm-pack/trunk scripts exist yet.

## Architecture

### Plugin-per-domain with shared type contract

The game uses a strict domain isolation pattern:

- `src/shared/mod.rs` is the **type contract** — all components, resources, events, and states shared across domains are defined here (~1800 lines). **Do not casually modify** — changes ripple across every domain.
- `src/main.rs` is the **wiring hub** — registers all resources, events, and plugins. **Do not casually modify** — order matters.
- `src/lib.rs` re-exports all modules as a library crate so `tests/headless.rs` can import game systems.

Each domain lives in `src/{domain}/mod.rs` and exports a `{Domain}Plugin`. There are 13 domain directories plus `src/data/`.

### Isolation rule

Domains communicate **only** through `crate::shared::*`. No domain imports from another domain. Cross-domain coordination happens via:
1. Shared resources (e.g., `Calendar`, `FarmState`, `Inventory`)
2. Events (e.g., `DayEndEvent`, `ToolUseEvent`, `MapTransitionEvent`)
3. State transitions (`NextState<GameState>`)

### GameState machine

```
Loading → MainMenu → Playing ⇄ Paused
                        ↕
          Dialogue / Shop / Fishing / Crafting / Inventory / Cutscene / BuildingUpgrade
```

Gameplay systems must be guarded with `.run_if(in_state(GameState::Playing))` or the appropriate state.

### Input abstraction

`src/input/mod.rs` (`InputPlugin`) runs in `PreUpdate` and converts raw keyboard/mouse into a `PlayerInput` resource. All other systems read `PlayerInput` instead of `ButtonInput<KeyCode>` directly. `InputContext` controls which actions are available (Gameplay, Menu, Disabled).

### Key event flows

**DayEndEvent** (most important): Player sleeps → CalendarPlugin fires `DayEndEvent` → farming advances crops, animals produce, economy sells shipping bin, NPCs reset, save autosaves.

**ToolUseEvent**: PlayerPlugin emits with target tile → farming tills/waters, world chops/breaks, mining breaks rocks, fishing starts minigame.

**MapTransitionEvent**: Triggers world map swap, player repositioning, NPC spawn/despawn, screen fade.

## Domain Reference

| Domain   | Plugin Struct  | Key Sub-modules |
|----------|----------------|-----------------|
| calendar | CalendarPlugin | festivals |
| player   | PlayerPlugin   | movement, tools, tool_anim, camera, interaction, interact_dispatch, item_use, spawn |
| farming  | FarmingPlugin  | soil, crops, harvest, render, sprinklers, sprinkler, events_handler |
| animals  | AnimalPlugin   | feeding, products, movement, interaction, rendering, day_end |
| world    | WorldPlugin    | maps, objects, chests, seasonal, lighting, weather_fx, ysort |
| npcs     | NpcPlugin      | spawning, schedule, schedules, dialogue, gifts, animation, definitions, map_events, quests, romance |
| economy  | EconomyPlugin  | shop, shipping, gold, tool_upgrades, blacksmith, buildings, evaluation, achievements, play_stats, stats |
| crafting | CraftingPlugin | bench, recipes, cooking, machines, buffs, unlock |
| fishing  | FishingPlugin  | cast, bite, minigame, resolve, render, fish_select, skill, legendaries, treasure |
| mining   | MiningPlugin   | floor_gen, spawning, rock_breaking, combat, movement, ladder, transitions, hud, components |
| ui       | UiPlugin       | hud, main_menu, pause_menu, dialogue_box, inventory_screen, shop_screen, crafting_screen, chest_screen, building_upgrade_menu, cutscene_runner, intro_sequence, tutorial, transitions, toast, debug_overlay, menu_kit, menu_input, audio |
| save     | SavePlugin     | (single mod.rs) |
| data     | DataPlugin     | items, crops, fish, recipes, npcs, shops |

## Testing

Headless tests (`tests/headless.rs`) exercise ECS logic without a window or GPU using `MinimalPlugins`. They import individual systems from the library crate (e.g., `hearthfield::farming::crops::advance_crop_growth`) and run them in a test `App`. To add a new headless test, register only the systems and resources your test needs in `build_test_app()`.

## Conventions

- Use `Sprite::from_color()` for placeholder visuals when sprites aren't available; mark with `TODO` comments.
- Bevy 0.15 API — e.g., `Camera2d` (not `Camera2dBundle`), `Sprite` component, `required_components`.
- Camera uses `PIXEL_SCALE` for pixel-art scaling: `Transform::from_scale(Vec3::splat(1.0 / PIXEL_SCALE))`.
- Dependencies: `bevy 0.15`, `serde`, `serde_json`, `rand 0.8`, `ron 0.8`. No other runtime deps.
- Dev profile: `opt-level = 1` for game code, `opt-level = 3` for dependencies (fast compile + fast deps).

## Common Pitfalls

- Adding a new shared resource or event requires updating **both** `src/shared/mod.rs` (definition) and `src/main.rs` (registration via `.init_resource` or `.add_event`). Also update `tests/headless.rs:build_test_app()` if used in tests.
- The `src/lib.rs` must re-export any new domain module for headless tests to access it.
- System ordering: `InputPlugin` runs in `PreUpdate` before all domain plugins. Domain plugins should register gameplay systems in `Update` with state guards.
- `InteractionClaimed` is reset each frame by the input system — use it to prevent multiple systems from handling the same player interaction.
