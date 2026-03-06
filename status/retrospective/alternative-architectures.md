# Alternative Architectures (Worker 8)

## Baseline Observations From This Codebase

- The project is already a hybrid ECS architecture, not ECS-pure: gameplay state is heavily resource-centric (`Inventory`, `FarmState`, `AnimalState`, `Relationships`, `MineState`) in [`src/shared/mod.rs`](/home/user/hearthfield/src/shared/mod.rs), while ECS entities are often synchronized projections (for example farming render sync in [`src/farming/render.rs`](/home/user/hearthfield/src/farming/render.rs) and animal resource sync in [`src/animals/rendering.rs`](/home/user/hearthfield/src/animals/rendering.rs)).
- The shared contract is monolithic: [`src/shared/mod.rs`](/home/user/hearthfield/src/shared/mod.rs) is 2315 lines and imported broadly (`use crate::shared::*` appears across most domains).
- Event/message passing exists and is useful (many `EventReader`/`EventWriter` systems), but direct shared-state mutation is also common (`ResMut<...>` to shared resources across domains).
- Claimed domain isolation is partially violated in practice: comments in shared/NPC modules describe strict boundaries, but there are direct cross-domain imports (for example [`src/player/item_use.rs`](/home/user/hearthfield/src/player/item_use.rs), [`src/player/interact_dispatch.rs`](/home/user/hearthfield/src/player/interact_dispatch.rs), [`src/save/mod.rs`](/home/user/hearthfield/src/save/mod.rs), UI modules importing economy/world/save internals).
- Game data is code-first: content definitions are hardcoded in Rust population functions (`src/data/*.rs`), loaded by [`src/data/mod.rs`](/home/user/hearthfield/src/data/mod.rs). There are no gameplay `.ron`/`.toml` data files in this repo.
- Save format is a large aggregate struct (`FullSaveFile`) in [`src/save/mod.rs`](/home/user/hearthfield/src/save/mod.rs), tightly coupled to many domain types.
- Testing is strongest where logic is deterministic and data-structure oriented (`tests/headless.rs`, many unit tests in shared), which fits the hybrid resource style.

## ECS-Pure vs Hybrid (Current) vs Traditional OOP

| Axis | ECS-pure | Hybrid (current) | Traditional OOP |
|---|---|---|---|
| Complexity | High upfront: many components/systems needed for simple state | Moderate: simple state in resources, ECS where useful | Moderate initially, grows high with inheritance/coupled managers |
| Dev velocity | Slower early, better long-term consistency | Fast for feature shipping (matches current history) | Fast for isolated features, slower when cross-cutting systems grow |
| Testability | Good for pure systems, can be noisy to set up full worlds | Strong for pure functions/resources (already used) | Good for object-level tests, weaker for emergent interactions |
| Performance | Best scaling for large entity counts and parallelism | Good enough for current scale; some sync overhead | Often worst for many simulated entities unless heavily optimized |
| Onboarding | Hardest (ECS mental model) | Easiest for mixed-experience teams | Easy for OOP developers, but harder to reason globally |

### What this game would look like under each

- ECS-pure version:
  - `Inventory`, crops, relationships, and even progression data would move from large resources into components on entities (player entity, crop entities, NPC entities, etc.).
  - `FarmState` hash maps would be replaced by archetype queries and sparse components.
  - Save/load would become query-driven snapshots instead of one large struct.
  - This would reduce resource bottlenecks but increase system count and scheduling complexity.

- Hybrid version (current):
  - Keep resource-centric simulation data for coarse subsystems and persistence.
  - Keep ECS for rendering, movement, interaction surfaces, and transient effects.
  - This is exactly what the code does now and explains why it shipped broad features quickly.

- OOP version:
  - Likely central `Game`, `World`, `Player`, `NpcManager`, `FarmManager` classes with method calls.
  - Would simplify local reasoning but fight Bevy’s scheduler/data model.
  - Integration with Bevy states/events would become adapter-heavy and likely less idiomatic than current code.

### Assessment for this project

- For current scope (single-player sim, moderate entity counts, broad feature surface), hybrid is the best fit.
- Full ECS-pure is only justified if future scope requires high simulation scale (many NPCs/entities concurrently) or stricter parallelism.
- Traditional OOP would likely reduce fit with Bevy and increase long-term integration cost.

## Monolithic Shared Types vs Trait-Based Contracts vs Message Passing

| Axis | Monolithic shared types (current) | Trait-based contracts | Message passing (events/commands)
|---|---|---|---|
| Complexity | Low initial, high long-term due to central growth | Moderate/high design overhead | Moderate; requires event taxonomy discipline |
| Velocity | Very high early (single place for types) | Slower early | Moderate (good once patterns stabilize) |
| Testability | Good for state-level tests, weak boundary enforcement | Very good mockability at boundaries | Good for integration behavior tests |
| Performance | Efficient direct reads/mutations | Usually neutral | Slight overhead for event buffering/indirection |
| Onboarding | Easy at first, harder as file grows | Harder initially, clearer ownership later | Moderate; requires understanding event flow |

### Grounded implications here

- Current monolith (`shared/mod.rs`) enabled speed, but now creates coupling pressure and ownership ambiguity.
- Trait contracts are currently minimal (almost no architectural trait layer); introducing traits at domain seams (save adapters, UI-facing read models, economy calculators) would improve isolation without forcing a rewrite.
- Message passing already exists and works for cross-domain actions (`GoldChangeEvent`, `DayEndEvent`, `MapTransitionEvent`, etc.), but is mixed with direct resource mutation. A stricter command/event policy for cross-domain writes would reduce incidental coupling.

### What an alternative would look like

- Trait-based variant:
  - Split `shared` into smaller contract modules (`shared/calendar`, `shared/economy`, `shared/social`, etc.).
  - Define traits like `EconomyLedger`, `QuestReadModel`, `WorldCollisionView` and have domains depend on trait interfaces, not concrete resources.

- Message-first variant:
  - Cross-domain writes only via command events.
  - Domains own their resources/components and expose read-only projections.
  - Save plugin reads from projection resources instead of importing many internals directly.

## Code-First vs Data-Driven (RON/TOML configs)

| Axis | Code-first (current) | Data-driven (RON/TOML)
|---|---|---|
| Complexity | Low tooling complexity | Higher loader/validation/tooling complexity |
| Velocity | Fast for programmers | Faster for designers/content iteration after setup |
| Testability | Good via Rust unit tests | Good if schema validation + golden-data tests exist |
| Performance | Excellent startup/runtime predictability | Slight parse/load overhead at boot (usually acceptable) |
| Onboarding | Easier for Rust contributors | Better for non-programmer content contributors |

### Grounded implications here

- Current data in `src/data/*.rs` is reliable and type-safe, but content edits require code changes and recompilation.
- String IDs are already used widely (`ItemId`, recipe references), which is a good prerequisite for externalized data.
- `ron` is in dependencies but gameplay content is not externalized today; saves are JSON.

### What a data-driven version would look like

- Move item/crop/fish/npc/shop definitions to `assets/data/*.ron` (or `.toml`).
- Keep strongly typed Rust structs in `shared` for deserialization.
- Add schema/consistency validation at load (duplicate IDs, dangling references, seasonal constraints), preserving the existing test rigor.
- Keep high-risk logic (economy formulas, quest progression rules) code-first while externalizing content tables.

## Recommended Direction (Given Current Realities)

- Keep hybrid ECS as the core architecture.
- Refactor the monolithic shared contract into smaller bounded modules instead of one giant file.
- Increase message/command boundaries for cross-domain writes, while allowing direct reads where practical.
- Introduce traits only at high-value seams (save serialization boundaries, UI read models, economy/stat calculators).
- Move content definitions to data files incrementally (items/crops/fish first), backed by strict validation tests.

This path preserves existing velocity and Bevy alignment while reducing coupling and improving long-term maintainability, onboarding, and content iteration.
