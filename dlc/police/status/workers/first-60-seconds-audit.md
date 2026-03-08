# Precinct DLC First 60 Seconds Audit

Scope:
- Read all required entry files: `dlc/police/src/main.rs`, `dlc/police/src/domains/calendar/mod.rs`, `dlc/police/src/domains/cases/mod.rs`, `dlc/police/src/domains/economy/mod.rs`, `dlc/police/src/domains/evidence/mod.rs`, `dlc/police/src/domains/npcs/mod.rs`, `dlc/police/src/domains/patrol/mod.rs`, `dlc/police/src/domains/player/mod.rs`, `dlc/police/src/domains/precinct/mod.rs`, `dlc/police/src/domains/save/mod.rs`, `dlc/police/src/domains/skills/mod.rs`, `dlc/police/src/domains/ui/mod.rs`, and `dlc/police/src/domains/world/mod.rs`.
- Also inspected `dlc/police/src/domains/ui/screens.rs` and `dlc/police/src/domains/ui/notifications.rs` because `UiPlugin` delegates screen and feedback wiring there.

Legend:
- `WIRED`: concrete code path exists end to end.
- `BROKEN`: code exists but there is a clear break in the flow.
- `MISSING`: no concrete implementation found.

| Step | Status | Evidence | Notes |
| --- | --- | --- | --- |
| 1. BOOT `Loading -> MainMenu` | `WIRED` | `GameState` defaults to `Loading` in `dlc/police/src/shared/mod.rs:20-24`; `main()` initializes state in `dlc/police/src/main.rs:44-50`; `UiPlugin` runs `boot_to_main_menu` on `OnEnter(GameState::Loading)` in `dlc/police/src/domains/ui/mod.rs:19-21`; handler sets `MainMenu` in `dlc/police/src/domains/ui/mod.rs:136-138`. | Cold boot path is explicitly present. |
| 2. MENU buttons | `WIRED` | Main menu UI spawns `New Game`, `Load Game`, and `Quit` buttons in `dlc/police/src/domains/ui/mod.rs:274-348`; button handler drives `Playing`, `LoadRequestEvent { slot: 0 }`, and `AppExit::Success` in `dlc/police/src/domains/ui/mod.rs:350-391`. | Menu buttons are real interactive Bevy buttons, not placeholders. |
| 3. SPAWN player + map + HUD | `WIRED` | Domain plugins are registered in `dlc/police/src/main.rs:101-113`; `PlayerPlugin` spawns player on `OnEnter(GameState::Playing)` in `dlc/police/src/domains/player/mod.rs:24-28` and `:99-140`; `WorldPlugin` spawns map on `OnEnter(GameState::Playing)` in `dlc/police/src/domains/world/mod.rs:145-149` and `:277-299`; `UiPlugin` spawns HUD on `OnEnter(GameState::Playing)` in `dlc/police/src/domains/ui/mod.rs:32-34` and `:399-578`. | Entering `Playing` has explicit spawn hooks for all three pieces. |
| 4. MOVE `WASD` + collision | `WIRED` | Input maps `WASD`/arrows in `dlc/police/src/domains/player/mod.rs:56-88`; movement applies that vector in `dlc/police/src/domains/player/mod.rs:143-196`; collision blocks movement via `tile_is_blocked` in `dlc/police/src/domains/player/mod.rs:303-305`; walls are inserted into `CollisionMap` in `dlc/police/src/domains/world/mod.rs:379-396`. | Movement and wall collision are both present. |
| 5. INTERACT `F` key + toast | `WIRED` | `F` sets `player_input.interact` in `dlc/police/src/domains/player/mod.rs:84`; precinct interaction handler gates on that flag in `dlc/police/src/domains/precinct/mod.rs:170-189`; interactions emit `ToastEvent` for case board, evidence terminal, coffee, meal, locker, radio, and exterior evidence scene in `dlc/police/src/domains/precinct/mod.rs:191-306`. | There is a concrete input-to-interaction-to-toast path. |
| 6. CLOCK ticks + HUD | `WIRED` | `CalendarPlugin` runs `tick_clock` during `Playing` in `dlc/police/src/domains/calendar/mod.rs:11-24`; `tick_clock` advances minutes unless `time_paused` is set in `dlc/police/src/domains/calendar/mod.rs:28-49`; HUD clock/day/weather/rank/duty values are refreshed every frame in `dlc/police/src/domains/ui/mod.rs:581-613`. | Clock simulation and HUD presentation are connected. |
| 7. FEEDBACK toast reader | `WIRED` | Notification systems are installed from `UiPlugin` in `dlc/police/src/domains/ui/mod.rs:58-59`; feedback overlay and toast stack are spawned in `dlc/police/src/domains/ui/notifications.rs:66-80` and `:107-176`; `ToastEvent` is consumed into toast UI cards in `dlc/police/src/domains/ui/notifications.rs:301-339`; lifetimes are ticked in `:342-352`. | Toast events are actually rendered, not just emitted. |
| 8. DISPATCH radio display | `WIRED` | Patrol generates `DispatchCallEvent` and stores `current_dispatch` in `dlc/police/src/domains/patrol/mod.rs:119-147`; notification layer captures dispatch events in `dlc/police/src/domains/ui/notifications.rs:287-299` and renders/flashes a dispatch banner in `:355-413`; precinct radio interaction also surfaces the current call text via toast in `dlc/police/src/domains/precinct/mod.rs:263-277`. | Both passive banner display and active radio interaction exist. |
| 9. PAUSE preserve state | `WIRED` | `Escape` pauses gameplay and sets `clock.time_paused = true` in `dlc/police/src/domains/ui/mod.rs:621-630`; pause menu spawns on `OnEnter(GameState::Paused)` in `:48-55` and `:643-714`; resuming clears pause in `:716-779` and `:797-799`; player/map cleanup only happens on `MainMenu` in `dlc/police/src/domains/player/mod.rs:27-28`, `:254-257` and `dlc/police/src/domains/world/mod.rs:148-155`, `:335-340`. | Pause overlays the game and pauses the clock without despawning the live player/map state. |
| 10. SAVE write + load | `BROKEN` | Save/write path is present: pause menu emits save/load requests in `dlc/police/src/domains/ui/mod.rs:752-760`; `SavePlugin` serializes to JSON in `dlc/police/src/domains/save/mod.rs:77-117` and `:176-183`; load restores resources and sets `Playing` in `dlc/police/src/domains/save/mod.rs:120-165`. Break: `spawn_player` early-returns if a player already exists in `dlc/police/src/domains/player/mod.rs:103-106`, and `spawn_map` early-returns if map tiles already exist in `dlc/police/src/domains/world/mod.rs:281-285`. | Main-menu load looks viable, but in-session pause-menu load does not clearly resync already spawned player/map entities to the newly loaded resources before returning to `Playing`. `world` has a resync helper in `dlc/police/src/domains/world/mod.rs:512-523`, but `handle_load` never calls it. |
| 11. SCREENS `Tab/J/C` hotkeys | `WIRED` | Input maps `Tab`, `J`, and `C` in `dlc/police/src/domains/player/mod.rs:87-96`; `UiPlugin::open_player_views` routes those flags to `SkillTree`, `CaseFile`, and `CareerView` in `dlc/police/src/domains/ui/mod.rs:633-640`; screen systems are installed in `dlc/police/src/domains/ui/screens.rs:220-280`, with concrete screen spawns for case file in `:406-442`, skill tree in `:493-580`, and career view in `:683-695`. | Hotkeys open real screen states. `Tab` is named `radio` in input but is currently wired to `SkillTree`. |

## Bottom Line

Result:
- `WIRED`: 1, 2, 3, 4, 5, 6, 7, 8, 9, 11
- `BROKEN`: 10
- `MISSING`: none

Highest-risk issue:
- Save/load is only partially safe. Resource restoration exists, but the pause-menu load path appears to leave already-spawned live entities out of sync with the loaded snapshot.
