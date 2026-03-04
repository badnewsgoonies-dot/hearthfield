# Domain Spec: Calendar & Time

## Scope
`src/calendar/` — `mod.rs`, `festivals.rs`

## Responsibility
Advance game time, manage day/season/year progression, trigger DayEndEvent and SeasonChangeEvent, weather generation, and festival scheduling.

## Shared Contract Types (import from `crate::shared`)
- `GameState` (pause time when not `Playing`)
- `Calendar` (Resource — year, season, day, hour, minute, weather, time_scale, time_paused, elapsed_real_seconds)
- `Season`, `DayOfWeek`, `Weather`
- `DayEndEvent`, `SeasonChangeEvent`
- `PlayerInput` (pause detection)
- `MapId` (festival location transitions)
- Constants: `DAYS_PER_SEASON` (28), `SEASONS_PER_YEAR` (4)

## Quantitative Targets
- 4 seasons × 28 days = 112-day year
- Time: 6:00 AM → 2:00 AM (hour 6..25), default time_scale = 10.0 game-minutes/real-second
- Weather distribution per season:
  - Spring: 60% Sunny, 30% Rainy, 10% Stormy
  - Summer: 70% Sunny, 20% Rainy, 10% Stormy
  - Fall: 50% Sunny, 35% Rainy, 15% Stormy
  - Winter: 40% Sunny, 20% Snowy, 30% Snowy, 10% Stormy (Snowy only in Winter)
- 4 festival days: Spring 13, Summer 11, Fall 16, Winter 25
- Festival names: Spring Dance, Summer Luau, Fall Harvest, Winter Star

## Constants & Formulas
- `DAYS_PER_SEASON = 28`
- `SEASONS_PER_YEAR = 4`
- `time_scale = 10.0` (game-minutes per real second)
- Day-of-week: `total_days % 7` → Mon–Sun
- `total_days_elapsed = (year-1) * 112 + season_index * 28 + (day - 1)`

## Key Systems
1. `tick_clock` — accumulate real delta time, advance minute/hour/day
2. `check_day_end` — when hour >= 26 (2 AM) or player sleeps, fire `DayEndEvent`
3. `advance_day` — reset hour to 6, increment day, roll weather, check season boundary
4. `advance_season` — fire `SeasonChangeEvent`, roll new weather pattern
5. `festival_check` — detect festival days, set up cutscene/dialogue

## Does NOT Handle
- Crop growth (farming domain listens to `DayEndEvent`)
- Animal feeding/aging (animals domain listens to `DayEndEvent`)
- Time display in HUD (ui domain reads `Calendar`)
- Weather visual effects (world domain reads `Calendar.weather`)
- Save/load of Calendar (save domain handles serialization)
