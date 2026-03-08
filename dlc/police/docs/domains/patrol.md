# Patrol Domain Spec — Precinct

## Purpose
The outdoor gameplay loop — replaces Hearthfield's farming outdoor loop.
Dispatch radio fires events during patrol, player responds to calls.

## Scope
`src/domains/patrol/` — owns dispatch generation, patrol state, call resolution.

## What This Domain Does
- Generate dispatch calls during shifts based on DISPATCH_BASE_RATE × area × time modifiers
- Manage PatrolState resource (fuel, calls responded/ignored, current dispatch)
- Create DispatchCall structs with appropriate fatigue/stress/XP values
- Emit DispatchCallEvent when a new call arrives
- Handle dispatch resolution: emit DispatchResolvedEvent, award XP, deduct fatigue/stress
- Track fuel for patrol car (100 max, 10-20 per trip, free refuel at precinct)
- Some dispatch calls generate evidence opportunities (may_generate_evidence flag)

## What This Domain Does NOT Do
- Player movement (player domain)
- Map transitions for responding to calls (world domain)
- UI for dispatch radio (ui domain, future)
- Evidence collection at dispatch scenes (evidence domain)
- Case management (cases domain)

## Key Types (import from crate::shared)
- `DispatchEventKind`, `DispatchCall`, `PatrolState` (Resource)
- `DispatchCallEvent`, `DispatchResolvedEvent`
- `FatigueChangeEvent`, `StressChangeEvent`, `XpGainedEvent`
- `ShiftClock` (read on_duty, hour, weather)
- `PlayerState` (read position_map for area modifier)
- `MapId`, `GameState`, `UpdatePhase`
- Constants: `DISPATCH_BASE_RATE`, `DISPATCH_NIGHT_MODIFIER`, `XP_PER_PATROL_EVENT`, `FUEL_MAX`, `FUEL_COST_PER_TRIP`

## Dispatch Event Definitions (6 types)

| Kind | Fatigue | Stress | XP | Evidence? | Description |
|------|---------|--------|-----|-----------|-------------|
| TrafficStop | 5.0 | 0.0 | 10 | No | Routine traffic violation |
| NoiseComplaint | 3.0 | 0.0 | 5 | No | Residential noise issue |
| ShoplifterInProgress | 8.0 | 5.0 | 15 | Yes | Active shoplifting |
| DomesticDisturbance | 10.0 | 10.0 | 20 | No | Domestic situation |
| SuspiciousVehicle | 5.0 | 5.0 | 10 | Yes | May lead to case evidence |
| OfficerNeedsBackup | 15.0 | 15.0 | 25 | No | High-stress response |

## Systems to Implement
1. `generate_dispatch` — UpdatePhase::Simulation, gated on GameState::Playing
   - Only when ShiftClock.on_duty == true
   - Rate: DISPATCH_BASE_RATE * map_modifier * night_modifier * delta_secs (in game hours)
   - map_modifier from PlayerState.position_map.dispatch_rate_modifier()
   - night_modifier: 1.5 if ShiftClock.hour >= 22 or hour < 6, else 1.0
   - If PatrolState.current_dispatch is Some, don't generate new
   - Roll random DispatchEventKind (weighted: 30% traffic, 20% noise, 15% shoplifter, 15% domestic, 10% suspicious, 10% backup)
   - Create DispatchCall, set PatrolState.current_dispatch, emit DispatchCallEvent
2. `resolve_dispatch` — triggered by player action (for Wave 2: public fn that external systems can call)
   - Emit DispatchResolvedEvent with XP
   - Emit FatigueChangeEvent with negative delta (cost)
   - Emit StressChangeEvent with negative delta (cost)
   - Emit XpGainedEvent
   - Clear PatrolState.current_dispatch
   - Increment calls_responded
3. `ignore_dispatch` — if dispatch times out (10 game-minutes) without resolution
   - Clear current_dispatch
   - Increment calls_ignored
4. `manage_fuel` — on map transitions, deduct fuel; at precinct, refuel to FUEL_MAX

## Quantitative Targets
- DISPATCH_BASE_RATE = 0.15 events per game-hour
- Night modifier = 1.5x
- Map modifiers: Downtown 1.5, Industrial 1.2, Residential 1.0, Highway 0.8, ForestPark 0.5, interiors 0.0
- Fuel: 100 max, 10-20 per trip (use FUEL_COST_PER_TRIP = 15 average)
- Dispatch timeout: 10 game-minutes
- 1-3 dispatch events expected per 8-hour shift

## Decision Fields
- **Preferred**: dispatch events as random radio calls with area/time modifiers
- **Tempting alternative**: random NPC spawns / combat encounters
- **Consequence**: random combat feels like an action RPG, not a police sim
- **Drift cue**: worker implements enemy spawning or attack mechanics

- **Preferred**: dispatch calls have a timeout (10 minutes) then auto-expire
- **Tempting alternative**: calls persist until resolved
- **Consequence**: accumulated calls create impossible backlog, feels punishing
- **Drift cue**: worker queues multiple simultaneous dispatches

## Plugin Export
```rust
pub struct PatrolPlugin;
```

## Tests (minimum 5)
1. No dispatch generated when off_duty
2. No dispatch generated when current_dispatch is Some
3. Dispatch rate modified by map area (Downtown = 1.5x)
4. Dispatch rate modified by night (1.5x when hour >= 22)
5. Resolving dispatch emits correct XP and clears current_dispatch
6. Fuel decrements on travel and refills at precinct
