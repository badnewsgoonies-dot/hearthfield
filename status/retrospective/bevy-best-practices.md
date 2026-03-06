# Bevy 0.15 Best Practices for Hearthfield

## Scope and source quality
This report prioritizes Bevy-official sources: Bevy 0.15 release notes, Bevy 0.15 official examples, and official docs pages.

## Recommended architecture patterns (Bevy 0.15)
- Keep feature/domain plugins as the primary composition unit.
- Keep plugins focused; split large concerns into smaller plugins when responsibilities diverge.
- Build app flow around states, but avoid a single monolithic state enum for all orthogonal concerns.
- Use `SubStates`/`ComputedStates` to model derived/parallel modes instead of duplicating `run_if` logic across many systems.
- Prefer state-scoped entities/events for state-owned runtime data to reduce manual cleanup and stale event handling.

Evidence:
- Official plugin example explicitly recommends “small scope” plugins: https://docs.rs/crate/bevy/0.15.3/source/examples/app/plugin.rs
- Bevy 0.15 release highlights `StateScoped` entities and state-scoped events as architecture tools: https://bevy.org/news/bevy-0-15/
- Official state examples include `SubStates` and `ComputedStates`: https://docs.rs/crate/bevy/0.15.3/source/examples/state/sub_states.rs and https://docs.rs/crate/bevy/0.15.3/source/examples/state/computed_states.rs

## State usage best practices
- Keep one high-level flow state (boot/menu/gameplay), then layer mode/detail states via `SubStates` or `ComputedStates`.
- Use `OnEnter`/`OnExit` for setup/teardown; use state predicates for per-frame logic.
- Prefer composing state predicates once (derived/computed states) rather than repeating large `.run_if(in_state(...).or(...))` expressions.
- Use `StateScoped` entities where UI/world objects should strictly live with a state lifecycle.
- Use state-scoped events when event lifetimes should not cross state boundaries.

Evidence:
- Sub-states and scoped entities example: https://docs.rs/crate/bevy/0.15.3/source/examples/state/sub_states.rs
- Computed state example: https://docs.rs/crate/bevy/0.15.3/source/examples/state/computed_states.rs
- State-scoped events in 0.15 release notes: https://bevy.org/news/bevy-0-15/

## SystemSets usage patterns
- Define domain/system-phase sets with `#[derive(SystemSet)]` and place systems into sets (`.in_set(...)`) instead of long flat tuples.
- Configure ordering once with `app.configure_sets(ScheduleLabel, (...).chain())`.
- Use sets as the main place for run conditions, ordering, and transition boundaries (input -> simulation -> reactions -> presentation).
- Add explicit ordering edges only where needed (`.before`/`.after`) to keep parallelism.

Evidence:
- Official ECS guide example shows set definitions, assignment, and `configure_sets(...chain())`: https://docs.rs/crate/bevy/0.15.3/source/examples/ecs/ecs_guide.rs
- Official custom transitions example shows transition phases via sets and chained order: https://docs.rs/crate/bevy/0.15.3/source/examples/state/custom_transitions.rs

## Plugin granularity guidance
- Keep domain plugins (good for ownership), but split “god plugins” by concern when they mix unrelated lifecycles (audio, transitions, HUD, menu logic, screen state, etc.).
- Group lower-level plugins under a thin parent plugin for ergonomics.
- Aim for plugin boundaries that match data ownership and state ownership.

Evidence:
- Plugin example commentary and plugin-group usage: https://docs.rs/crate/bevy/0.15.3/source/examples/app/plugin.rs
- Bevy quickstart plugin section (official docs): https://bevy.org/learn/quick-start/getting-started/plugins/

## Scheduling features likely missed
- `SystemSet` taxonomy + `configure_sets`: currently absent in `src/`.
- Derived state tools (`SubStates`, `ComputedStates`): currently absent.
- State-scoped lifecycle tools (`StateScoped`, `add_state_scoped_event`): currently absent.
- Transition-oriented scheduling patterns (set-driven transition phases, including dedicated transition schedules): not used.
- Fixed-step scheduling (`FixedUpdate`) for deterministic simulation islands (if desired for mechanics like growth/combat windows): not currently used.

Evidence:
- No matches in `src/` for `derive(SystemSet)`, `configure_sets`, `in_set`, `add_sub_state`, `add_computed_state`, `StateScoped`, `add_state_scoped_event`, `FixedUpdate`.
- Relevant official references:
  - sets: https://docs.rs/crate/bevy/0.15.3/source/examples/ecs/ecs_guide.rs
  - sub/computed states: https://docs.rs/crate/bevy/0.15.3/source/examples/state/sub_states.rs and https://docs.rs/crate/bevy/0.15.3/source/examples/state/computed_states.rs
  - transitions and set phases: https://docs.rs/crate/bevy/0.15.3/source/examples/state/custom_transitions.rs
  - 0.15 state-scoped entities/events: https://bevy.org/news/bevy-0-15/

## Mapping to this codebase

### What is already aligned
- Plugin-per-domain architecture is already in place and generally healthy (`src/main.rs`).
- Widespread use of state predicates and `OnEnter`/`OnExit` patterns is already present.
- Some schedule separation exists (`PreUpdate`, `First`, `PostUpdate`) in targeted domains.

### High-impact improvements for Hearthfield
1. Introduce shared gameplay phase sets in `src/shared`.
- Example shape: `Input`, `Intent`, `Simulation`, `Reactions`, `Presentation`.
- Configure once in `main` for `Update`, then move domain systems into sets.
- Expected benefit: less ad-hoc `.before(...)` and fewer order regressions when adding systems.

2. Decompose `GameState` into layered state model.
- Keep high-level flow in `GameState` (loading/menu/playing).
- Add sub/computed states for overlays/modes (dialogue, shop, inventory, cutscene, map, journal) instead of encoding all as top-level mutually-exclusive variants.
- Expected benefit: simpler transition logic and fewer broad `.or(in_state(...))` conditions.

3. Split `UiPlugin` into smaller state-owned plugins.
- Current `src/ui/mod.rs` is a large integration point handling audio, transitions, dialogue, HUD, tutorial, menu input, and many screens.
- Split by ownership/lifecycle (for example `UiAudioPlugin`, `HudPlugin`, `MenuScreensPlugin`, `OverlayPlugin`, `TutorialPlugin`).
- Keep a `UiPlugin` as composition wrapper.

4. Adopt state-scoped entities/events for UI-heavy screens and ephemeral overlays.
- Replace manual spawn/despawn bookkeeping where possible with state-scoped lifecycle.
- Use state-scoped events for events that should never leak across menu/gameplay mode boundaries.

5. Evaluate fixed-step islands for deterministic logic.
- Candidate islands: time-critical simulation loops and combat windows where frame-rate variance can cause subtle behavior drift.
- Keep rendering/input in variable-step schedules.

### Suggested rollout order (low risk)
1. Add shared `SystemSet` enums + `configure_sets(Update, ...)` without moving behavior.
2. Move systems into sets incrementally domain by domain.
3. Split `UiPlugin` into internal sub-plugins while preserving public behavior.
4. Introduce sub/computed states for one vertical slice (for example overlay/menu stack), then expand.
5. Add state-scoped entities/events for selected screens and temporary world overlays.

## Sources
- Bevy 0.15 release notes: https://bevy.org/news/bevy-0-15/
- Bevy 0.15 official plugin example: https://docs.rs/crate/bevy/0.15.3/source/examples/app/plugin.rs
- Bevy 0.15 official ECS guide example (sets/configure_sets): https://docs.rs/crate/bevy/0.15.3/source/examples/ecs/ecs_guide.rs
- Bevy 0.15 official custom transitions example: https://docs.rs/crate/bevy/0.15.3/source/examples/state/custom_transitions.rs
- Bevy 0.15 official sub-states example: https://docs.rs/crate/bevy/0.15.3/source/examples/state/sub_states.rs
- Bevy 0.15 official computed-states example: https://docs.rs/crate/bevy/0.15.3/source/examples/state/computed_states.rs
- Bevy official quickstart plugins page: https://bevy.org/learn/quick-start/getting-started/plugins/
