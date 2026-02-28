# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build Commands

```bash
cargo check          # Type-check only (fastest feedback loop)
cargo run            # Build and launch the game
cargo test           # Run unit tests (shared/mod.rs has 80+ tests)
cargo test -- --test-name  # Run a single test by name
./build_wasm.sh      # Build for web (WASM), outputs to web/ directory
```

Incremental compilation works well — after the first full build (~15 min on HDD), subsequent builds only recompile the `hearthfield` crate (~2-4 min).

## Architecture

Hearthfield is a Harvest Moon-style farming sim built with **Rust + Bevy 0.15**. The codebase is 117 .rs files across 15 domain modules.

### Core Pattern: Plugin-per-Domain

Every gameplay domain is an isolated Bevy plugin in `src/{domain}/`. Domains communicate **only** through:

1. **Shared types** — `use crate::shared::*;` (the only allowed cross-domain import)
2. **Events** — defined in `src/shared/mod.rs` or registered by each plugin
3. **State transitions** — via `NextState<GameState>`

**No domain imports from any other domain.** This is enforced by convention.

### Key Files

- `src/shared/mod.rs` (2,151 lines) — **The type contract.** All shared components, resources, events, enums, and states. Read this first when working on any domain.
- `src/main.rs` — App setup, resource initialization, event registration, plugin registration order. Events defined in `shared` must be registered here with `add_event::<>()`. Domain-local events are registered by their own plugin.
- `src/lib.rs` — Public re-exports of all modules (for integration tests).
- `GAME_SPEC.md` — Full game design document.

### GameState Flow

```
Loading → MainMenu → Playing ↔ Paused
                       ↔ Inventory, Dialogue, Shop, Crafting, Fishing, Mining, Cutscene
```

Guard gameplay systems with `.run_if(in_state(GameState::Playing))`. UI screens spawn on `OnEnter(State)` and despawn on `OnExit(State)`.

### Domain Modules (15 plugins)

| Module     | Plugin Struct    | Key Responsibility                           |
|------------|------------------|----------------------------------------------|
| input      | InputPlugin      | Keyboard/gamepad → PlayerInput abstraction   |
| calendar   | CalendarPlugin   | Time progression, day/season/weather events  |
| player     | PlayerPlugin     | Movement, tool use, stamina, collision       |
| farming    | FarmingPlugin    | Soil tilling, crop growth, harvest, sprinklers |
| animals    | AnimalPlugin     | Animal care, products, buildings             |
| world      | WorldPlugin      | Tilemap loading/rendering, map transitions   |
| npcs       | NpcPlugin        | NPC spawning, schedules, dialogue, gifts, romance |
| economy    | EconomyPlugin    | Shops, shipping bin, blacksmith upgrades     |
| crafting   | CraftingPlugin   | Crafting bench, cooking, processing machines |
| fishing    | FishingPlugin    | Casting, minigame, catch resolution          |
| mining     | MiningPlugin     | Mine floors, rocks, monsters, loot           |
| ui         | UiPlugin         | HUD, inventory, menus, dialogue box, toasts  |
| save       | SavePlugin       | Save/load to JSON, autosave on sleep         |
| data       | DataPlugin       | Populates all registries during Loading state |

### Critical Event Flows

**DayEndEvent** (most important): CalendarPlugin fires when player sleeps. Farming advances crops, animals produce products, economy sells shipping bin, NPCs reset daily flags, save autosaves.

**ToolUseEvent**: PlayerPlugin fires with tool kind + target tile. Farming handles hoe/watering can, world handles axe/pickaxe on overworld, mining handles pickaxe underground.

**MapTransitionEvent**: World despawns/loads maps, player repositions, NPCs spawn/despawn for the new area.

## Common Pitfalls

### Bevy ECS Resource Conflicts
Never take both `Res<T>` and `ResMut<T>` of the same type in one system. Bevy will panic at runtime. Use a single `ResMut<T>` if you need to both read and write.

### UI Despawn
Always use `despawn_recursive()` for UI root entities, not `despawn()`. Plain `despawn()` only removes the root node, leaving all children (buttons, text, panels) orphaned on screen.

### Event Registration
Events defined in `src/shared/mod.rs` must be registered in `src/main.rs` with `.add_event::<EventType>()`. Events defined inside a domain module are registered by that domain's plugin. Missing registration causes a runtime panic: "could not access system parameter".

### Assets
The game expects an `assets/` directory with sprites, fonts, tilesets, and audio from the Sprout Lands asset pack. Missing assets log errors but don't crash. Use `Sprite::from_color()` placeholders when assets aren't available.

## WASM / Web Build

`.cargo/config.toml` configures WASM flags (getrandom wasm_js backend, disabled reference-types for iOS Safari). The `wasm-release` Cargo profile uses opt-level=z with LTO. `build_wasm.sh` handles the full pipeline including wasm-bindgen and optional wasm-opt.

## Windows Development Notes

If builds are slow on Windows, add Defender exclusions for `target/`, `.cargo/`, and `.rustup/` directories, plus `rustc.exe` and `cargo.exe` processes. Real-time scanning on Rust's heavy I/O can 2-3x build times.
