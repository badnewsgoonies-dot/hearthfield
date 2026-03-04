# Worker: MAIN GAME — 3 Missing UI Screens (Calendar, Stats, Settings)

## Context
The main Hearthfield game at `/home/user/hearthfield` has 3 missing UI screens. The data exists but there are no visual screens for Calendar, Statistics, or Settings.

## CRITICAL: Read these files first
1. `src/shared/mod.rs` — GameState enum, Calendar, PlayStats, all types
2. `src/ui/mod.rs` — How existing screens (Journal, Relationships, Map) are wired
3. `src/ui/journal_screen.rs` — Reference for screen structure pattern
4. `src/ui/relationships_screen.rs` — Another reference
5. `src/calendar/mod.rs` — Calendar data, seasons, festivals
6. `src/data/mod.rs` — Static data tables

## IMPORTANT: src/shared/mod.rs is FROZEN
Check if GameState already has CalendarView, StatsView, Settings variants. If NOT, you CANNOT add them (the file is checksummed). Instead, check `.contract.sha256` — if the checksum check is enforced, you must work within existing variants or skip adding states.

If the variants already exist, wire screens normally. If they DON'T exist:
- Check if there are unused GameState variants you can repurpose
- Or use overlay/popup patterns that don't need new states (show during Playing with a key toggle)

## Screen 1: Calendar Screen (`src/ui/calendar_screen.rs`, ~150 lines)

Display the game calendar:
- Month grid: 4 weeks × 7 days (28 days per season)
- Current day highlighted
- Season name + year at top
- Festival days marked with special color/icon
- Player birthday marked
- NPC birthdays on their days (if data available)
- Keyboard: Esc to close

## Screen 2: Statistics Screen (`src/ui/stats_screen.rs`, ~120 lines)

Display PlayStats resource:
- "STATISTICS" header
- crops_harvested, fish_caught, items_shipped, gifts_given
- mine_floors_cleared, animal_products_collected, food_eaten
- total_gold_earned, days_played, festivals_attended
- Format as two-column layout: label | value

## Screen 3: Settings Screen (`src/ui/settings_screen.rs`, ~100 lines)

Basic settings screen:
- "SETTINGS" header
- Audio volume slider (or +/- buttons) — can be placeholder that stores value
- Show current keybinds (read-only display)
- "Back" button to return to previous state

## Wiring
Add screens to `src/ui/mod.rs` following the same OnEnter/OnExit/Update pattern.

## Validation
```bash
cd /home/user/hearthfield && cargo check 2>&1
cd /home/user/hearthfield && cargo test --test headless 2>&1
cd /home/user/hearthfield && cargo clippy -- -D warnings 2>&1
shasum -a 256 -c .contract.sha256 2>&1
```
Done = all four pass. The shasum check is critical — do NOT modify src/shared/mod.rs.

## When done
Write completion report to `/home/user/hearthfield/status/workers/main-game-missing-screens.md`
