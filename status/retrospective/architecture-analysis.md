# Architecture Analysis

## Scope Reviewed
- `src/main.rs`
- `src/shared/mod.rs`
- Domain `mod.rs` files under `src/*/mod.rs`

## Executive Take
- Plugin-per-domain is the right baseline for this game, but two plugins (`ui`, `save`) have become cross-domain integrators and are now architectural hotspots.
- The shared type contract is too large for long-term maintenance and should be split into submodules while preserving a `shared::prelude`-style import surface.
- There is no Rust compile-time circular module dependency loop in the domain `mod.rs` files, but there are architectural coupling cycles via direct cross-domain imports and ordering constraints.
- The architecture is mixed: events are used for coarse domain signaling, but direct shared-resource mutation is dominant for core gameplay state.

## Findings

### 1) Plugin-per-domain fit: mostly good, but uneven in practice
- Main wiring is clearly domain-oriented and readable (`input` first, then gameplay domains): `src/main.rs:134-150`.
- Most domain roots depend only on shared contract types (`use crate::shared::*`), e.g. `animals`, `calendar`, `farming`, `fishing`, `economy`, `mining`, `npcs`, `player`, `world`: `src/animals/mod.rs:1`, `src/calendar/mod.rs:27`, `src/farming/mod.rs:12`, `src/fishing/mod.rs:6`, `src/economy/mod.rs:6`, `src/mining/mod.rs:25`, `src/npcs/mod.rs:6`, `src/player/mod.rs:10`, `src/world/mod.rs:14`.
- However, `ui` is effectively an application layer that touches almost every state and mode (`src/ui/mod.rs:45-442`) and `save` imports internals from multiple domains (`src/save/mod.rs:12-21`, `src/save/mod.rs:208-247`).

Assessment:
- Keep plugin-per-domain as the core pattern.
- Treat `ui` and `save` as orchestrator/platform plugins and formalize that role explicitly (instead of pretending all domains are equally isolated).

### 2) Shared contract size: too large, should be split
- `src/shared/mod.rs` is 2315 lines.
- It currently contains at least:
  - 127 public structs/enums (`rg '^pub (struct|enum)'`)
  - 36 resources (`rg '^#\[derive\(Resource'`)
  - 32 events (`rg '^#\[derive\(Event'`)
- It mixes multiple concerns in one file: core state machine, per-domain data models, rendering constants/utilities, input abstraction, cutscene types, debug overlay, and tests (`src/shared/mod.rs`, sections around `GameState`, events, constants, tool utilities, coordinate conversion, debug tests).
- The file itself states it is the global type contract used by every domain (`src/shared/mod.rs:1-4`), which increases blast radius for every change.

Assessment:
- Split by contract area (example slices):
  - `shared/core_state.rs` (GameState, calendar basics)
  - `shared/player_inventory.rs`
  - `shared/farming_animals.rs`
  - `shared/world_map.rs`
  - `shared/npc_relationships.rs`
  - `shared/economy.rs`
  - `shared/events.rs`
  - `shared/input_contract.rs`
  - `shared/rendering.rs`
- Keep `shared/mod.rs` as a re-export facade to avoid churn at call sites.

### 3) Circular dependencies / tight coupling
No Rust module cycle was observed at the domain root level, but tight coupling exists in several places:
- `npcs` depends on `player` dispatcher ordering via explicit `.before(crate::player::interact_dispatch::dispatch_world_interaction)`: `src/npcs/mod.rs:72-79`.
- `player` imports domain events directly from `crafting`, `farming`, and `economy` in submodules (`src/player/item_use.rs:11-12`, `src/player/interact_dispatch.rs:11-14`).
- `ui` submodules import domain internals directly (`src/ui/shop_screen.rs:3`, `src/ui/dialogue_box.rs:2-3`, `src/ui/chest_screen.rs:10`, `src/ui/pause_menu.rs:3`).
- `save` imports deep internals across calendar/crafting/economy/npcs/world (`src/save/mod.rs:12-21`, plus extended bundles at `src/save/mod.rs:208-247`).

Assessment:
- The coupling is manageable today but trending toward integration-layer gravity around `ui`/`save`/`player` interaction dispatch.
- Prefer moving these crossings behind shared events/resources or explicit adapter APIs.

### 4) Domain boundary discipline
Strong signals:
- Comments in domain roots explicitly reinforce shared-contract-only communication (e.g., `farming` and `economy`): `src/farming/mod.rs:3-10`, `src/economy/mod.rs:3-4`.
- Main initializes shared resources/events centrally (`src/main.rs:61-133`), which gives domains a common contract.

Weak signals:
- Boundary exceptions are not rare in practice (especially UI and save), and some ordering contracts are cross-domain and fragile (`src/npcs/mod.rs:72-79`, `src/player/mod.rs:30-36`).

Assessment:
- Boundary discipline is good as intent, partial in execution.
- Codify boundary tiers:
  - Core gameplay domains: only `shared` contract.
  - Orchestrators (`ui`, `save`): allowed to depend on domain internals, but keep those dependencies explicit and minimal.

### 5) Event-driven vs direct resource mutation
Event-driven evidence:
- Many shared events are globally registered (`src/main.rs:100-133`).
- Calendar uses `DayEndEvent`/`SeasonChangeEvent` as global lifecycle triggers (`src/calendar/mod.rs:61-75`, `src/calendar/mod.rs:193-199`).
- Domains include event listeners for day-end/season and other cross-cut triggers (`src/world/mod.rs:126-139`, `src/farming/mod.rs:184-198`, `src/economy/mod.rs:75-99`).

Direct mutation evidence:
- Domains rely heavily on shared `ResMut` state contracts (`Calendar`, `FarmState`, `PlayerState`, `Inventory`, etc.) initialized centrally in `main` (`src/main.rs:63-99`).
- Cross-domain communication design text repeatedly allows direct shared-resource reads/writes in addition to events (e.g., economy wording: "events/resources" at `src/economy/mod.rs:3-4`).

Assessment:
- Current architecture is event-signaled, resource-mutated.
- Events are primarily lifecycle and notification rails; the dominant business state updates happen via direct shared resource mutation.

## Recommended Next Steps
1. Split `src/shared/mod.rs` into thematic modules with a compatibility re-export layer.
2. Define explicit architectural tiers (domain vs orchestrator) and document allowed dependency directions.
3. Reduce cross-domain ordering dependencies by converting "A must run before B in another domain" patterns into event handoffs where practical.
4. Add a lightweight dependency-check script (e.g., deny direct `crate::<domain>` imports outside approved orchestrator modules).
