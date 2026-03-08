# Precinct Wave 1 Audit

Audit scope:
- Requested source/spec files: `dlc/police/src/domains/{calendar,player,world,ui}/mod.rs`
- Shared contract: `dlc/police/src/shared/mod.rs`
- Domain specs: `dlc/police/docs/domains/{calendar,player,world,ui}.md`
- Supporting files read for connectivity/path tracing: `dlc/police/src/main.rs`, plus downstream readers/writers in `cases`, `evidence`, `patrol`, and `precinct`

Verification note:
- `cargo test -p precinct` passed: 53 passed, 0 failed
- The passing test suite does not cover several cross-domain breaks listed below

## Executive Summary

Wave 1 is not shippable as a full gameplay slice yet.

The major blockers are:
- Boot never leaves `GameState::Loading`, so the real game path never reaches the main menu or gameplay.
- `player` and `world` define different local `CollisionMap` resources, so world collision is not actually consumed by player movement.
- The only map-transition trigger is on the wrong edge of the map, while the world domain defines the precinct exit on the south edge.
- World transitions update `PlayerState` but do not reposition the live player entity.
- Pause/resume exits and re-enters `Playing`, which respawns the player at the default spawn and loses current position/map.

## 1. Event Connectivity

Connectivity was checked across the full `dlc/police/src` tree because Wave 1 events are already consumed by later domains.

| Event | Writers | Readers | Connectivity status |
| --- | --- | --- | --- |
| `ShiftEndEvent` | `calendar::check_shift_end` | `cases::advance_case_shifts`, `cases::refresh_available_cases`, `evidence::process_evidence` | Connected |
| `MapTransitionEvent` | `player::check_map_transition_zone` | `world::handle_map_transition`, `precinct::spawn_precinct_objects`, `precinct::cleanup_precinct_objects`, `patrol::manage_fuel` | Connected |
| `FatigueChangeEvent` | `precinct::handle_precinct_interaction`, `patrol::resolve_dispatch` | `player::apply_fatigue_stress` | Connected |
| `StressChangeEvent` | `precinct::handle_precinct_interaction`, `patrol::resolve_dispatch` | `player::apply_fatigue_stress` | Connected |
| `AppExit` | `ui::handle_main_menu_buttons` | No in-crate reader; consumed by Bevy app runtime | No local reader, but expected engine event |

Flags:
- No shared Wave 1 contract event is missing both a writer and a reader.
- `AppExit` has no DLC-side reader, which is normal for a Bevy engine event.

## 2. Spec Compliance

### Calendar

Quantitative targets:
- `TIME_SCALE = 2.0`: hit. Implementation uses shared constant `TIME_SCALE`.
- `SHIFT_DURATION_HOURS = 8`: hit. Implementation uses shared constant `SHIFT_DURATION_HOURS`.
- Rank thresholds `1-28 / 29-56 / 57-84 / 85+`: effectively hit. `rank_for_shift` matches the threshold boundaries.
- Weather weights `60 / 20 / 10 / 10`: hit exactly in `roll_weather`.

Notes:
- Calendar meets its numeric targets.
- Minor contract smell: `ShiftEndEvent` payload fields `cases_progressed`, `evidence_collected`, and `xp_earned` are always sent as `0`.

### Player

Quantitative targets:
- Walk speed `80.0 px/s`: hit.
- Run speed `120.0 px/s`: hit (`80 * 1.5`).
- Fatigue clamp `0.0..100.0`: hit.
- Stress clamp `0.0..100.0`: hit.
- Spawn position `grid (5, 5)` on `MapId::PrecinctInterior`: missed. Implementation spawns at `(16, 20)`.

Notes:
- The player implementation matches the world spec's spawn point `(16, 20)`, not the player spec's `(5, 5)`.
- `read_keyboard_input` sets `F` interact and `Escape` menu, but not `E`, `Tab`, or `R`.
- Running does not generate the spec's fatigue cost over time.

### World

Quantitative targets:
- Precinct interior `32 x 24 = 768` tiles: hit.
- At least `6` distinct rooms: hit exactly `6`.
- `1` transition zone: hit exactly `1`.
- Collision map populated with all wall tiles: hit inside `world::CollisionMap`.

Notes:
- World meets its numeric targets in isolation.
- The transition target map (`PrecinctExterior`) being absent is allowed by the spec.
- The actual cross-domain transition path is still broken; see Risks and First-60-Seconds Path.

### UI

Quantitative targets:
- `3` screens: hit (`MainMenu`, `HUD`, `PauseMenu`).
- HUD updates every frame from `ShiftClock` and `PlayerState`: hit.
- Fatigue bar width proportional to `fatigue / MAX_FATIGUE`: hit.
- Stress bar width proportional to `stress / MAX_STRESS`: hit.

Notes:
- UI meets its numeric targets in isolation.
- Real boot never reaches `MainMenu`, so the screens are currently not reachable from a cold start.

## 3. Shared Type Usage

### Calendar imports

Imports from `crate::shared`:
- `DayOfWeek`
- `GameState`
- `Rank`
- `ShiftClock`
- `ShiftEndEvent`
- `ShiftType`
- `UpdatePhase`
- `Weather`
- `SHIFT_DURATION_HOURS`
- `TIME_SCALE`

### Player imports

Imports from `crate::shared`:
- `Equipment`
- `Facing`
- `FatigueChangeEvent`
- `GameState`
- `GridPosition`
- `InputContext`
- `MapId`
- `MapTransitionEvent`
- `Player`
- `PlayerInput`
- `PlayerMovement`
- `PlayerState`
- `StressChangeEvent`
- `UpdatePhase`
- `MAX_FATIGUE`
- `MAX_STRESS`
- `PIXEL_SCALE`
- `TILE_SIZE`

### World imports

Imports from `crate::shared`:
- `GameState`
- `GridPosition`
- `MapId`
- `MapTransition`
- `MapTransitionEvent`
- `PlayerState`
- `TileKind`
- `UpdatePhase`
- `PIXEL_SCALE`
- `TILE_SIZE`

### UI imports

Imports from `crate::shared`:
- `DayOfWeek`
- `GameState`
- `PlayerInput`
- `PlayerState`
- `ShiftClock`
- `UpdatePhase`
- `Weather`
- `MAX_FATIGUE`
- `MAX_STRESS`
- `SCREEN_HEIGHT`
- `SCREEN_WIDTH`

Shadowing check:
- No audited domain locally redefines a shared-contract type such as `ShiftClock`, `PlayerState`, `GridPosition`, `MapId`, or the Wave 1 events.
- There is a separate cross-domain duplicate, not a shared-type shadow: both `player` and `world` define a local `CollisionMap`. This is a real integration bug because they are different resource types.

## 4. Test Coverage

There is no `dlc/police/tests/` directory. Coverage is entirely module-local unit tests.

### Calendar: 6 tests

- `clock_ticks_forward_when_not_paused`: verifies time advances and `time_scale` is applied.
- `clock_does_not_tick_when_paused`: verifies paused time does not advance.
- `day_advances_when_hour_reaches_twenty_four`: verifies day rollover and weekday advancement.
- `shift_end_event_emits_after_eight_hours_on_duty`: verifies shift end emission and off-duty transition after 8 hours.
- `weather_roll_matches_weight_bands`: verifies weather generation roughly matches 60/20/10/10 bands.
- `rank_progression_updates_at_shift_thresholds`: verifies rank changes at shift threshold boundaries.

### Player: 8 tests

- `player_spawns_at_precinct_entry_point`: verifies spawn position, transform, placeholder sprite, facing, and map id.
- `keyboard_input_sets_move_interact_menu_and_run`: verifies `WASD`/arrows, interact, menu/cancel, and run input fields.
- `movement_updates_transform_and_facing`: verifies movement updates transform, grid position, and facing.
- `movement_does_not_enter_solid_tiles`: verifies collision against the player's local `CollisionMap`.
- `fatigue_changes_apply_and_clamp`: verifies fatigue event application and clamping.
- `stress_changes_apply_and_clamp`: verifies stress event application and clamping.
- `player_despawns_on_exit_playing`: verifies the player entity is removed on exit from `Playing`.
- `running_uses_faster_speed`: verifies run speed and `is_running`.

Coverage gaps:
- No test verifies real integration with `world::CollisionMap`.
- No test verifies actual precinct exit transitions.
- No test verifies pause/resume preserves player location.
- No test verifies notebook/radio input fields or run-fatigue cost.

### World: 5 tests

- `map_spawns_all_precinct_tiles`: verifies exactly `768` map tile entities spawn.
- `collision_map_contains_every_wall_tile`: verifies the world collision map includes every wall tile.
- `collision_map_excludes_walkable_tiles`: verifies walkable tiles are not marked solid and checks spawn/door tiles.
- `map_transition_replaces_existing_tile_entities`: verifies tile entities are replaced and `PlayerState` is updated on transition.
- `precinct_spawn_point_is_walkable`: verifies the spawn tile is not a wall.

Coverage gaps:
- No test verifies the live player entity is repositioned on map transition.
- No test verifies the player's transition trigger lines up with the world's actual exit tile.

### UI: 6 tests

- `main_menu_spawns_title_and_two_buttons`: verifies menu title and button count.
- `new_game_button_transitions_to_playing_and_cleans_main_menu`: verifies state transition and menu cleanup.
- `quit_button_emits_app_exit`: verifies quit emits `AppExit`.
- `hud_spawns_and_reflects_player_and_clock_values`: verifies HUD text and bar sizing from resources.
- `escape_pauses_game_sets_time_paused_and_swaps_screens`: verifies pause state, clock pause flag, and HUD/pause screen swap.
- `resume_button_returns_to_playing_and_unpauses_clock`: verifies resume transitions back and clears `time_paused`.

Coverage gaps:
- No test covers cold boot from `Loading` into `MainMenu`.
- No test verifies pause/resume preserves gameplay entities and player location.

## 5. Cross-Domain Integration Risks

### 1. Boot path is broken at startup

`main.rs` initializes `GameState` with the shared default `Loading`, but no system ever sets `NextState<GameState>` to `MainMenu`.

Implication:
- The real game never reaches the UI main menu, `Playing`, HUD, map spawn, or player spawn from a cold boot.

Relevant code:
- `dlc/police/src/shared/mod.rs`: `GameState::Loading` is the default state
- `dlc/police/src/main.rs`: `init_state::<GameState>()`
- `dlc/police/src/domains/ui/mod.rs`: UI only reacts to `MainMenu`, `Playing`, and `Paused`

### 2. Collision is split across two incompatible resources

`player` defines `player::CollisionMap { solid_tiles: HashSet<(i32, i32)> }`, while `world` defines `world::CollisionMap(HashSet<(i32, i32)>)`.

Implication:
- `world::spawn_map` populates one resource type.
- `player::move_player` reads a different resource type.
- In the actual game, walls are not enforcing movement constraints.

Relevant code:
- `dlc/police/src/domains/player/mod.rs`: local `CollisionMap`, consumed by `move_player`
- `dlc/police/src/domains/world/mod.rs`: different local `CollisionMap`, initialized and populated by `WorldPlugin`

### 3. Player and world disagree on where the precinct exit is

World defines the only transition at south exit tile `(16, 23)`, but player emits `MapTransitionEvent` only at `x == 16 && y <= 0`.

Implication:
- The actual south door never triggers the transition.
- The north-edge trigger does not match map data.

Relevant code:
- `dlc/police/src/domains/world/mod.rs`: `SOUTH_EXIT_POSITION = GridPosition { x: 16, y: 23 }`
- `dlc/police/src/domains/player/mod.rs`: `check_map_transition_zone` uses `grid_position.y <= 0`

### 4. World transitions do not move the live player entity

`world::handle_map_transition` only updates `PlayerState`. It does not query or update the live player entity's `Transform` or `GridPosition`.

Implication:
- After a transition, `PlayerState` and the actual player entity can disagree.
- On the next `move_player` tick with no input, player code writes the stale entity position back into `PlayerState`, effectively undoing the transition target.

Relevant code:
- `dlc/police/src/domains/world/mod.rs`: `handle_map_transition`
- `dlc/police/src/domains/player/mod.rs`: `move_player` treats entity transform as the source of truth

### 5. Pause/resume currently resets the player

Leaving `Playing` triggers `OnExit(GameState::Playing)` cleanup in `player` and `world`. Resuming to `Playing` runs `spawn_player` again, and that function hard-resets `PlayerState` to interior spawn `(16, 20)`.

Implication:
- Pause/resume loses player position and map.
- Any future map transition progress is also lost on resume.

Relevant code:
- `dlc/police/src/domains/player/mod.rs`: `OnExit(GameState::Playing) -> despawn_player`
- `dlc/police/src/domains/player/mod.rs`: `spawn_player` hardcodes interior spawn and map id
- `dlc/police/src/domains/ui/mod.rs`: resume path sets `GameState::Playing`

### 6. Spec drift exists between player and world docs

Player spec says spawn at `(5, 5)`, while world spec says `(16, 20)`. Implementation follows world.

Implication:
- Future fixes could "correct" the player to the wrong location depending on which spec a worker reads first.

## 6. First-60-Seconds Path

Expected player path:
1. Boot the game.
2. Reach `MainMenu`.
3. Press `New Game`.
4. Enter `Playing`.
5. Spawn player/map/HUD.
6. Walk around precinct.
7. Reach precinct exit and transition out.

Actual path:

### 0s: Boot

Observed:
- `App::new()` initializes state with default `GameState::Loading`.
- Only startup camera setup is guaranteed to run.

Broken link:
- No code transitions `Loading -> MainMenu`.

Result:
- The player never reaches the menu from a real boot.

### If state is manually forced to `MainMenu`

Observed:
- UI main menu spawns correctly.
- `New Game` correctly requests `GameState::Playing` via `NextState<GameState>`.

Status:
- This step works in isolated UI tests, but only after bypassing the real boot path.

### Entering `Playing`

Observed:
- `player` spawns the player at `(16, 20)`.
- `world` spawns the precinct map and its collision resource.
- `ui` spawns the HUD.
- `calendar` begins ticking.

Status:
- This also works in isolated state-jump tests.

### First movement inside the precinct

Observed:
- Input and camera follow work.

Broken link:
- Collision is not actually integrated because `player` and `world` use different `CollisionMap` resource types.

Result:
- The player can move through walls in the real game path.

### Attempting to leave the precinct

Observed:
- World's only exit is the south door `(16, 23)`.
- Player's transition writer only fires at the north edge `y <= 0`.

Broken link:
- The intended door never emits `MapTransitionEvent`.

Result:
- The expected precinct exit is non-functional.

### If a transition event somehow does fire

Observed:
- This is only plausible because collision is broken and the player can reach the wrong edge.
- World falls back unsupported `PrecinctExterior` requests back to `PrecinctInterior`.
- Transition code updates `PlayerState`, not the live player entity.

Broken links:
- Unsupported target map fallback hides the intended exit path.
- Player transform/grid are not repositioned, so transition state is unstable.

Result:
- Transition behavior is inconsistent and not gameplay-safe.

### If the player pauses in the first minute

Observed:
- UI sends `Playing -> Paused`.
- On resume, `Playing` re-enters and respawns player/map/HUD.

Broken link:
- The respawn path hard-resets the player to default precinct spawn instead of preserving position.

Result:
- Pause/resume loses location continuity.

## Bottom Line

In isolation, the four Wave 1 domains mostly satisfy their local numeric targets and unit tests. As an integrated slice, the first playable minute is broken by state-machine startup, collision wiring, map-transition wiring, and pause/resume persistence.
