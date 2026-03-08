# Precinct Domain Spec — Precinct

## Purpose
The indoor hub — replaces Hearthfield's farm buildings. The precinct is where the player
starts each shift, accesses the case board, processes evidence, takes breaks, and manages cases.

## Scope
`src/domains/precinct/` — owns precinct room interactions and hub logic.

## What This Domain Does
- Define interactable objects in the precinct interior map (components on entities)
- Handle F-key interactions with precinct objects:
  - Case Board → emit CaseAssignedEvent (player picks a case)
  - Evidence Room Terminal → start evidence processing (call evidence domain)
  - Break Room Coffee Machine → emit FatigueChangeEvent(+25), StressChangeEvent(-5), advance clock 15 min
  - Break Room Meal Table → emit FatigueChangeEvent(+50), StressChangeEvent(-10), advance clock 30 min
  - Captain's Office Door → placeholder interaction (future: promotions, assignments)
  - Locker → placeholder (future: equipment management)
  - Dispatch Radio → show current dispatch call info
- Spawn interactable entities on map spawn (listen to MapTransitionEvent or coordinate with world domain)
- Provide the hub's functional rooms as interactive game elements

## What This Domain Does NOT Do
- Map rendering or tilemap (world domain)
- Player movement (player domain)
- Case state machine (cases domain)
- Evidence quality/processing logic (evidence domain)
- UI screens for case board or evidence (ui domain, future)

## Key Types (import from crate::shared)
- `CaseBoard` (Resource — read available cases)
- `CaseAssignedEvent`
- `EvidenceLocker` (Resource)
- `FatigueChangeEvent`, `StressChangeEvent`
- `ShiftClock` (ResMut — advance time for breaks)
- `PlayerState`, `MapId`
- `GameState`, `UpdatePhase`
- Constants: `COFFEE_FATIGUE_RESTORE`, `COFFEE_STRESS_RELIEF`, `COFFEE_TIME_COST_MINUTES`,
  `MEAL_FATIGUE_RESTORE`, `MEAL_STRESS_RELIEF`, `MEAL_TIME_COST_MINUTES`

## New Types (define locally)
- `PrecinctInteractable` — Component enum:
  - CaseBoard, EvidenceTerminal, CoffeeMachine, MealTable, CaptainDoor, Locker, DispatchRadio
- `PrecinctObject` — Component marker for cleanup

## Systems to Implement
1. `spawn_precinct_objects` — when player enters PrecinctInterior (listen to MapTransitionEvent or OnEnter(Playing) if starting map is precinct)
   - Spawn entities at known grid positions with PrecinctInteractable component + colored sprite
   - Case Board: grid (4, 12) — yellow square
   - Evidence Terminal: grid (26, 12) — green square
   - Coffee Machine: grid (26, 6) — brown square
   - Meal Table: grid (24, 6) — orange square
   - Captain Door: grid (4, 6) — red square
   - Dispatch Radio: grid (16, 16) — cyan square
2. `handle_precinct_interaction` — UpdatePhase::Reactions, gated on Playing
   - When player is near a PrecinctInteractable and presses F (read PlayerInput.interact):
   - Check proximity (within 2 tiles of object)
   - Match on PrecinctInteractable variant → dispatch appropriate action:
     - CaseBoard: pick first available case from CaseBoard.available, emit CaseAssignedEvent
     - CoffeeMachine: emit FatigueChangeEvent(COFFEE_FATIGUE_RESTORE), StressChangeEvent(-COFFEE_STRESS_RELIEF), advance ShiftClock by COFFEE_TIME_COST_MINUTES
     - MealTable: emit FatigueChangeEvent(MEAL_FATIGUE_RESTORE), StressChangeEvent(-MEAL_STRESS_RELIEF), advance ShiftClock by MEAL_TIME_COST_MINUTES
     - EvidenceTerminal: start processing all Raw evidence in EvidenceLocker
     - Others: emit ToastEvent with placeholder message
3. `cleanup_precinct_objects` — despawn PrecinctObject entities on map transition away from precinct

## Quantitative Targets
- 7 interactable objects in precinct interior
- Coffee: +25 fatigue, -5 stress, costs 15 game-minutes
- Meal: +50 fatigue, -10 stress, costs 30 game-minutes
- Case board shows available cases (reads CaseBoard resource)
- Evidence terminal starts processing all Raw evidence

## Decision Fields
- **Preferred**: precinct as walkable map with spatial interactables
- **Tempting alternative**: precinct as a menu screen
- **Consequence**: menu-only loses the spatial feel that makes sim games engaging
- **Drift cue**: worker builds precinct as a GameState with UI overlay instead of map objects

- **Preferred**: interact via proximity + F-key (matching player domain interaction pattern)
- **Tempting alternative**: click-to-interact or auto-trigger on collision
- **Consequence**: auto-trigger fires unintentionally when player walks past objects
- **Drift cue**: worker uses collision enter events instead of explicit interact input

## Plugin Export
```rust
pub struct PrecinctPlugin;
```

## Tests (minimum 5)
1. Precinct objects spawn at correct grid positions
2. Coffee machine restores correct fatigue and stress amounts
3. Meal table restores correct fatigue and stress amounts
4. Case board interaction emits CaseAssignedEvent
5. Evidence terminal starts processing Raw evidence
6. Objects despawn on map transition
