# Precinct DLC Final Miracle Audit

Date: 2026-03-08

Scope:
- Read `dlc/police/src/main.rs`
- Read every source file under `dlc/police/src/domains/`
- Pulled `dlc/police/src/shared/mod.rs` as supporting contract context
- Ran `cargo test -p precinct --quiet` -> `120 passed, 0 failed`

Method:
- Static trace of the boot/state/input/event/resource flow
- Cross-check against the crate's unit tests
- Asset existence check for player, tile, object, and audio resources

## Miracle Path

| # | Step | Status | Why |
| --- | --- | --- | --- |
| 1 | Browser loads WASM -> MainMenu appears | WORKING | `GameState` defaults to `Loading`; UI immediately transitions `Loading -> MainMenu` in `boot_to_main_menu` and spawns the main menu on `OnEnter(MainMenu)` (`src/domains/ui/mod.rs:140`, `src/domains/ui/mod.rs:357`). The unit test `loading_boots_into_main_menu` passes. |
| 2 | Click New Game -> Playing state | WORKING | Main menu button handling sets `NextState<GameState>` to `Playing` for `New Game` (`src/domains/ui/mod.rs:357`). |
| 3 | Player spawns in precinct with SPRITE (not colored rect) | WORKING | `spawn_player` uses `player_sprite()` first and only falls back to `Sprite::from_color` if sprite loading is unavailable (`src/domains/player/mod.rs:108`, `src/domains/player/mod.rs:348`). `assets/characters/player_officer.png` exists. |
| 4 | Map tiles rendered with ATLAS SPRITES (not colored rects) | WORKING | Interior precinct tiles use `precinct_floor_sprite` plus overlay atlas sprites when the atlas is available (`src/domains/world/mod.rs:430`, `src/domains/world/mod.rs:654`). `assets/tilesets/precinct_office.png` exists. Note: this is true for `PrecinctInterior`; `PrecinctExterior` is still rendered as colored rects by design. |
| 5 | HUD shows clock, fatigue, stress, gold, rank — all updating | WORKING | HUD spawns all listed fields and updates from live resources every presentation frame (`src/domains/ui/mod.rs:406`, `src/domains/ui/mod.rs:614`). Clock changes come from calendar; fatigue/stress from events applied into `PlayerState`; gold/rank are read from `PlayerState` and `ShiftClock`. |
| 6 | Walk to case board, press F -> case assigned -> TOAST shows | WORKING | Precinct interaction on `CaseBoard` emits `CaseAssignedEvent` and `ToastEvent` when an available case exists (`src/domains/precinct/mod.rs:205`, `src/domains/precinct/mod.rs:227`). Cases domain consumes the assignment and moves the case to `active` (`src/domains/cases/mod.rs:74`). Notifications spawn the toast. |
| 7 | Walk to exit -> PrecinctExterior map loads | WORKING | Player transition-zone detection emits `MapTransitionEvent`; world consumes it and swaps interior/exterior maps, updating both resource state and live player position (`src/domains/player/mod.rs:254`, `src/domains/world/mod.rs:339`). |
| 8 | Dispatch call arrives -> radio flashes on HUD | BROKEN | Dispatch generation depends on `MapId::dispatch_rate_modifier()`. `PrecinctExterior` gets the default `0.0`, so step 7's destination cannot generate a dispatch at all (`src/domains/patrol/mod.rs:119`, `src/shared/mod.rs:575`). The HUD/banner flash path exists (`src/domains/ui/notifications.rs:291`, `src/domains/ui/notifications.rs:359`), but this miracle path never reaches a dispatch-producing map. |
| 9 | Walk back inside -> precinct objects still there | WORKING | Precinct objects are spawned once on entering `Playing`, hidden/shown by current map, and only cleaned up on `MainMenu` (`src/domains/precinct/mod.rs:176`, `src/domains/precinct/mod.rs:357`, `src/domains/precinct/mod.rs:377`). Walking back inside restores their visibility. |
| 10 | Walk to evidence terminal, press F -> evidence processing | BROKEN | Pressing `F` on the evidence terminal does not start processing. It only opens `GameState::EvidenceExam` if the locker has items; otherwise it shows `Evidence locker is empty.` (`src/domains/precinct/mod.rs:253`). Actual processing starts only after clicking `Process All Raw` inside the evidence exam screen (`src/domains/ui/screens.rs:1043`, `src/domains/ui/screens.rs:1099`, `src/domains/ui/screens.rs:1405`). The listed miracle path also never guarantees raw evidence exists. |
| 11 | Walk to coffee machine -> fatigue restores, TOAST shows | WORKING | Coffee interaction emits fatigue/stress changes, advances the clock by 15 minutes, and sends a toast (`src/domains/precinct/mod.rs:266`). HUD updates off the changed resources. |
| 12 | Press Tab -> skill tree screen with 20 perks | WORKING | `Tab` maps to `open_skill_tree`, then UI transitions to `SkillTree` (`src/domains/player/mod.rs:65`, `src/domains/ui/mod.rs:666`). The screen spawns 4 trees x 5 tiers = 20 perk buttons (`src/domains/ui/screens.rs:504`). |
| 13 | Press J -> case file with evidence status | WORKING | `J` maps to `open_case_file`; the case file screen renders collected/missing evidence status and quality text from active case + locker state (`src/domains/player/mod.rs:65`, `src/domains/ui/mod.rs:666`, `src/domains/ui/screens.rs:423`, `src/domains/ui/screens.rs:1251`). |
| 14 | Press C -> career view with promotion requirements | WORKING | `C` maps to `open_career_view`; career screen shows current rank plus next-rank XP/cases/reputation requirements (`src/domains/player/mod.rs:91`, `src/domains/ui/mod.rs:666`, `src/domains/ui/screens.rs:694`, `src/domains/ui/screens.rs:709`). |
| 15 | Shift ends -> salary paid -> TOAST shows | BROKEN | Salary payment works: `ShiftEndEvent` is emitted by calendar and consumed by economy to create `GoldChangeEvent` (`src/domains/calendar/mod.rs:51`, `src/domains/economy/mod.rs:78`). But no system turns shift end or salary payment into a `ToastEvent`. You get gold and SFX, not the requested salary toast. |
| 16 | Escape -> pause -> save works | BROKEN | Native path: structurally working. Browser/WASM path: broken. The save plugin writes with `std::fs::create_dir_all`, `std::fs::write`, and `std::env::var_os` (`src/domains/save/mod.rs:3`, `src/domains/save/mod.rs:32`, `src/domains/save/mod.rs:85`, `src/domains/save/mod.rs:186`), so the requested browser flow has no browser-compatible persistence backend. |
| 17 | Quit to menu -> Load Game -> state restored exactly | BROKEN | Native path: structurally working. Browser/WASM path: broken for the same reason as step 16. Loading depends on `std::fs::read_to_string` from a local save slot (`src/domains/save/mod.rs:127`, `src/domains/save/mod.rs:206`), not browser storage. |
| 18 | All 25 events have writers AND readers (zero dead events) | BROKEN | Structurally, all 25 registered events have at least one writer and one reader. End-to-end, the graph is not zero-dead: `DispatchResolvedEvent` depends on `patrol::resolve_dispatch()` (`src/domains/patrol/mod.rs:194`), and nothing schedules or calls that helper during normal play, so the dispatch-resolution event path is runtime-dead. |

## Main Breaks

1. Step 8 fails because the only reachable map after leaving the precinct is `PrecinctExterior`, and that map has a `0.0` dispatch spawn modifier.
2. Step 10 fails because `F` at the evidence terminal opens an examination screen instead of starting processing, and the listed path never guarantees raw evidence exists.
3. Step 15 fails because salary payment has no toast emitter.
4. Steps 16 and 17 fail in the requested browser/WASM path because persistence is implemented with `std::fs`, not browser storage.
5. Step 18 fails as an end-to-end runtime audit because dispatch resolution is stranded behind an uncalled helper.

## Event Writer/Reader Matrix

| Event | Writers | Readers | Verdict |
| --- | --- | --- | --- |
| `ShiftEndEvent` | `calendar::check_shift_end` | `economy::pay_salary`, `cases::advance_case_shifts`, `cases::refresh_available_cases`, `evidence::process_evidence`, `save::auto_save`, `ui::audio::emit_shift_end_sfx` | WORKING |
| `CaseAssignedEvent` | `precinct::handle_precinct_interaction` | `cases::handle_case_assignment`, `npcs::emit_investigation_commentary`, `ui::audio::emit_case_assigned_sfx` | WORKING |
| `CaseSolvedEvent` | `cases::handle_case_close` | `economy::apply_case_rewards`, `skills::emit_case_xp`, `ui::notifications::emit_case_solved_toasts`, `ui::audio::emit_case_solved_sfx` | WORKING |
| `CaseFailedEvent` | `cases::check_case_expiry` | `economy::apply_case_penalties`, `ui::notifications::emit_case_failed_toasts`, `ui::audio::emit_case_failed_sfx` | WORKING |
| `EvidenceCollectedEvent` | `precinct::handle_precinct_interaction`, `cases::handle_training_evidence_pickup`, `npcs::handle_interrogation_events` | `cases::track_evidence_for_cases`, `evidence::collect_evidence`, `skills::emit_evidence_xp`, `ui::notifications::emit_evidence_collected_toasts`, `ui::audio::emit_evidence_collected_sfx`, `npcs::emit_investigation_commentary` | WORKING |
| `EvidenceProcessedEvent` | `evidence::process_evidence` | `ui::notifications::emit_processed_evidence_toasts` | WORKING |
| `InterrogationStartEvent` | `npcs::handle_npc_interaction` | `npcs::handle_interrogation_events`, `ui::screens::track_interrogation_session` | WORKING |
| `InterrogationEndEvent` | `ui::screens::handle_interrogation_buttons` | `npcs::handle_interrogation_events`, `ui::screens::track_interrogation_session` | WORKING |
| `DispatchCallEvent` | `patrol::generate_dispatch` | `ui::notifications::capture_dispatch_events`, `ui::audio::emit_dispatch_call_sfx` | WORKING |
| `DispatchResolvedEvent` | `patrol::resolve_dispatch` | `skills::accumulate_xp` | BROKEN end-to-end; writer is not invoked by scheduled gameplay |
| `PromotionEvent` | `economy::check_promotions` | `economy::apply_promotions`, `cases::refresh_available_cases`, `ui::notifications::emit_promotion_toasts`, `ui::audio::emit_promotion_sfx` | WORKING |
| `NpcTrustChangeEvent` | `ui::screens::handle_interrogation_buttons`, `skills::apply_one_time_perks` | `npcs::apply_trust_pressure` | WORKING |
| `DialogueStartEvent` | `npcs::handle_npc_interaction` | `npcs::handle_dialogue_events`, `ui::screens::track_dialogue_session` | WORKING |
| `DialogueEndEvent` | `npcs::handle_dialogue_cancel_input`, `ui::screens::handle_dialogue_buttons` | `npcs::handle_dialogue_events`, `ui::screens::track_dialogue_session` | WORKING |
| `MapTransitionEvent` | `player::check_map_transition_zone` | `world::handle_map_transition`, `patrol::manage_fuel`, `npcs::spawn_npcs_for_map`, `npcs::update_npc_schedules`, `npcs::emit_patrol_commentary`, `ui::audio::queue_transition_audio` | WORKING |
| `FatigueChangeEvent` | `precinct::handle_precinct_interaction`, `patrol::resolve_dispatch` | `player::apply_fatigue_stress` | WORKING |
| `StressChangeEvent` | `precinct::handle_precinct_interaction`, `patrol::resolve_dispatch` | `player::apply_fatigue_stress` | WORKING |
| `GoldChangeEvent` | `economy::pay_salary`, `economy::apply_case_rewards`, `economy::apply_case_penalties`, `economy::apply_weekly_expenses` | `economy::process_gold_changes` | WORKING |
| `XpGainedEvent` | `skills::emit_case_xp`, `skills::emit_evidence_xp`, `patrol::resolve_dispatch`, `npcs::handle_interrogation_events` | `skills::accumulate_xp`, `ui::notifications::emit_xp_toasts` | WORKING |
| `SkillPointSpentEvent` | `ui::screens::handle_skill_tree_buttons` | `skills::handle_skill_spend`, `ui::notifications::emit_skill_spent_toasts` | WORKING |
| `PlaySfxEvent` | precinct/UI/notification/audio reaction systems | `ui::audio::play_requested_sfx` | WORKING |
| `PlayMusicEvent` | `ui::queue_main_menu_music`, `ui::queue_playing_music`, `ui::audio::queue_transition_audio` | `ui::audio::handle_music_requests` | WORKING |
| `ToastEvent` | precinct/UI screens/notifications/NPC commentary systems | `ui::notifications::spawn_toasts_from_events`, `ui::audio::emit_toast_pop_sfx` | WORKING |
| `SaveRequestEvent` | `ui::handle_pause_buttons`, `save::auto_save` | `save::handle_save` | WORKING |
| `LoadRequestEvent` | `ui::handle_main_menu_buttons`, `ui::handle_pause_buttons` | `save::handle_load` | WORKING |

## Final Verdict

The miracle path is not fully intact, especially in the requested browser/WASM context.

Current score:
- WORKING: 12
- BROKEN: 6
- MISSING: 0

Blocking breaks:
- Step 8: no dispatch can spawn in `PrecinctExterior`
- Step 10: evidence terminal opens a screen instead of processing, and the listed path does not guarantee raw evidence
- Step 15: salary payment has no toast
- Step 16: save backend is native filesystem only
- Step 17: load backend is native filesystem only
- Step 18: dispatch resolution path is runtime-dead
