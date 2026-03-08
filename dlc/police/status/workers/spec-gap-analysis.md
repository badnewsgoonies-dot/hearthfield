# Police DLC Spec Gap Analysis

Scope: compared `dlc/police/docs/spec.md` against only the requested domain files:

- `dlc/police/src/domains/calendar/mod.rs`
- `dlc/police/src/domains/cases/mod.rs`
- `dlc/police/src/domains/economy/mod.rs`
- `dlc/police/src/domains/evidence/mod.rs`
- `dlc/police/src/domains/npcs/mod.rs`
- `dlc/police/src/domains/patrol/mod.rs`
- `dlc/police/src/domains/player/mod.rs`
- `dlc/police/src/domains/precinct/mod.rs`
- `dlc/police/src/domains/save/mod.rs`
- `dlc/police/src/domains/skills/mod.rs`
- `dlc/police/src/domains/ui/mod.rs`
- `dlc/police/src/domains/world/mod.rs`

Method:

- Focused on player-reachable behavior.
- Treated authored data plus runnable ECS systems as implemented.
- Treated stubs, helper functions with no player path, and training shortcuts as partial.
- Did not inspect `ui/screens.rs`, `ui/notifications.rs`, or any non-`mod.rs` files beyond the requested spec.

## Calendar

References: `spec.md` calendar section; `calendar/mod.rs:11-156`.

Implemented features:

- Game time advances at the spec time scale and stops when `time_paused` is set.
- Shift end is detected from the current shift type and the fixed 8-hour duration.
- Day rollover advances `day` and `day_of_week`.
- Weather rerolls on a new day.
- Rank progression follows the authored shift thresholds for Patrol Officer, Detective, Sergeant, and Lieutenant.

Partial features:

- The pause hook exists, but this module only honors `clock.time_paused`; it does not itself guarantee all spec pause contexts are wired.
- Morning, afternoon, and night shift start hours exist, but this module does not expose a player-facing shift scheduler or shift selection flow.
- Calendar rank progression is driven directly from `shift_number`, while the economy domain separately models promotion thresholds from XP, solved cases, and reputation. That means the reviewed `mod.rs` files currently describe two competing progression authorities instead of one clean spec-faithful promotion path.

Missing features:

- The 3-shifts-per-week structure and 3 on-duty plus 4 off-duty rhythm are not enforced here.
- The Mon/Wed/Fri or Tue/Thu/Sat schedule pattern is not implemented here.
- Night shift unlock at Detective rank is not enforced here.
- Major scripted case calendar events per rank tier are not implemented here.
- Weather effects on patrol visibility and NPC behavior are not implemented here.

## Player

References: `spec.md` player section; `player/mod.rs:24-311`.

Implemented features:

- 4-direction movement and running are implemented.
- Interaction, menu, notebook, radio, and career-view hotkeys are captured.
- Terrain collision works against the world collision map.
- Door-triggered map transitions are emitted when the player steps onto a transition tile.
- Fatigue and stress values are updated and clamped through events.
- The badge is auto-equipped on spawn.

Partial features:

- Movement exists, but the player is still a flat colored sprite with no walk/run animation states in this module.
- Notebook and radio are currently just inputs and screen hotkeys, not modeled equipment behaviors.
- Equipment exists only in a minimal sense here; only the badge is ensured.

Missing features:

- No player-reachable sidearm, flashlight, or holster/draw behavior.
- No modeled inventory behavior for 12 evidence slots plus 6 personal item slots in this module.
- No per-action fatigue costs for investigation, chase, interrogation, or patrol walking.
- No stress-over-80 penalties.
- No NPC/object collision beyond tile blocking.

## World

References: `spec.md` world section; `world/mod.rs:9-548`.

Implemented features:

- The precinct interior and precinct exterior are playable tilemaps.
- The precinct interior has authored room boundaries and interactable tiles.
- Bidirectional interior/exterior transitions exist.
- Wall collision is built into a collision map.

Partial features:

- The precinct hub layout roughly reflects captain, break room, case board, evidence, lobby, and locker spaces, but only as simple tile geometry.
- The module declares tile kinds like `Water` and `CrimeTape`, but only walls are actually used for collision in the current maps.

Missing features:

- Ten of the twelve spec maps are not implemented here.
- Unsupported maps explicitly fall back to the precinct interior.
- No dynamic crime-scene dressing system.
- No patrol-car fast travel between exteriors.
- No day/night visual variants.
- No weather overlay system.
- No restricted traversal behavior for crime-scene tape or water in practice.

## Cases

References: `spec.md` cases section; `cases/mod.rs:33-906`.

Implemented features:

- The case registry contains 25 authored cases across all four ranks.
- Cases are rank-gated and replenished into the available list.
- Max 3 active cases is enforced on assignment.
- Active cases track evidence collected, suspects interrogated, witnesses interviewed, elapsed shifts, and notes.
- Case statuses progress through `Active`, `Investigating`, `EvidenceComplete`, `Interrogating`, `Solved`, and `Cold`.
- Evidence collected events advance case progress.
- Shift end advances case timers.
- Time-limited cases can go cold.
- Solved cases pay out XP, gold, and reputation rewards.
- Major cases exist as authored data via the `is_major` flag.

Partial features:

- Witnesses, suspects, scenes, and major-case flags are authored, but this module does not turn most of that authored structure into distinct player-facing case flow by itself.
- There is a training evidence pickup path at one exterior position that simply grants the next missing evidence item for an active case.
- Case close only requires `EvidenceComplete`; there is no report-filing or stronger solve gate here.
- The `New` and `Failed` portions of the spec state model are not meaningfully expressed as player-facing active-case states here.
- At least one authored case/evidence link is inconsistent inside the reviewed files: the arson case requires `forensic_report`, but that evidence ID does not exist in `evidence/mod.rs`.

Missing features:

- No witness interview loop in this domain.
- No suspect-identification step beyond authored suspect lists.
- No narrative-specific multi-shift major case logic beyond authored data fields.
- No player-facing case report filing flow.
- No explicit enforcement of the spec's average 1-3 shift case length or 4-8 shift major case length.

## Evidence

References: `spec.md` evidence section; `evidence/mod.rs:26-347`.

Implemented features:

- All 30 evidence types exist across the 6 specified categories.
- Collected evidence stores category, name, description, quality, linked case, processing state, collected shift, and collected map.
- Quality is computed from the shared base-quality and weather/night penalties.
- Evidence processing supports `Raw -> Processing -> Analyzed`.
- Processing completion happens after a shift passes.

Partial features:

- The quality formula exists, but this module hardcodes skill level to `0`, so the skill-based quality portion of the spec is not actually connected here.
- The incoming `EvidenceCollectedEvent.quality` value is ignored in favor of recalculating quality locally.
- Case linking is just `Some(case_id)` for non-empty IDs; there is no richer evidence-chain validation.

Missing features:

- No player-facing orphaned-evidence flagging.
- No explicit raw-vs-processed gameplay differences beyond the processing-state enum.
- No actual skill-driven evidence quality scaling in live play from the skills domain.
- No scene search flow or differentiated collection mechanics in this module beyond receiving collection events.

## NPCs

References: `spec.md` NPC section; `npcs/mod.rs:39-983`.

Implemented features:

- All 12 named NPCs are authored with roles, descriptions, and three schedule entries.
- NPCs spawn on the current map and respawn/update on map changes and schedule changes.
- Trust and pressure are both tracked and clamped separately.
- Dialogue and interrogation states can be entered from player interaction.
- Suspect interrogations can add confession evidence and interrogation XP.
- Partner arc stage progression is driven by trust thresholds.
- Weather/weekend offsets slightly affect final NPC positions.

Partial features:

- Schedules are coarse three-slot schedules, not rich time/day/weather schedule logic.
- Dialogue is a generic state transition with context text, not authored trust/pressure-reactive dialogue behavior.
- Partner arc progression stores stage and triggered markers, but does not grant the spec's gameplay bonuses in this module.
- NPCs can be authored as witnesses/suspects for cases, but witness interview progress is not meaningfully advanced from regular interaction here.

Missing features:

- No favor system.
- No trust/pressure-based branching outcomes like volunteered info vs false info.
- No side quests from high trust.
- No 10 partner-specific dialogue events.
- No deeper witness interview mechanics.

## Economy

References: `spec.md` economy section; `economy/mod.rs:16-226`.

Implemented features:

- Salary is paid per shift based on rank.
- Case solved events award gold and reputation.
- Failed cases deduct gold and reputation.
- Weekly rent and maintenance are deducted.
- Promotion thresholds are checked against XP, cases solved, and reputation.
- Promotions update rank automatically.
- Reputation is clamped to the spec bounds.

Partial features:

- Reputation matters for promotion, but this module does not make it affect case assignment, NPC availability, or access restrictions.
- Salary is fixed by rank here; performance-based salary modification from the spec is not implemented.
- Expenses are only weekly rent and maintenance in this module.
- Department budget may exist on the shared economy state, but this reviewed module does not create a player-facing budget request or spending loop.

Missing features:

- No coffee cost.
- No department budget gameplay.
- No desk duty or restricted access penalties for bad reputation.
- No player-requested promotion flow; promotion is automatic once thresholds are met.

## Skills

References: `spec.md` skills section; `skills/mod.rs:22-333`.

Implemented features:

- All 20 named perks exist across the 4 trees.
- XP is earned from case solves, evidence collection, and patrol events.
- Skill points are awarded at 1 point per 100 XP.
- Spending a point increases the appropriate tree level.
- Patrol level 4 applies the one-time community trust bonus.
- Investigation quality bonus helper values exist for level 3 and 5 thresholds.

Partial features:

- Most perks are definitions only; they do not change player-reachable behavior in this module.
- Several perk descriptions explicitly say `future`.
- Investigation quality bonuses exist here, but the evidence domain does not consume them.

Missing features:

- No XP source for favors.
- No active gameplay effect for most interrogation perks.
- No active gameplay effect for most patrol perks.
- No active gameplay effect for leadership perks like budget requests, partner synergy, task delegation, or multi-unit command.

## Patrol

References: `spec.md` patrol section; `patrol/mod.rs:98-349`.

Implemented features:

- Dispatch generation runs while on duty.
- All 6 specified dispatch event types are authored with fatigue, stress, XP, and description values.
- Dispatch chance uses base rate plus map and night modifiers.
- Ignored calls expire after time passes.
- Patrol car fuel is consumed between exterior-map transitions and refilled at the precinct.
- Resolving a dispatch produces fatigue, stress, and XP effects.

Partial features:

- Dispatches are descriptions attached to the player's current map, not full response scenes.
- `may_generate_evidence` exists in data but is not acted on in this module.
- There is no player-facing queue or choice among multiple calls.
- There is no explicit 1-3 calls-per-shift cap here.
- Fuel cost is a single fixed trip cost here, not the spec's variable 10-20 range.

Missing features:

- No actual patrol-car driving or fast-travel interface in this module.
- No response-scene gameplay for traffic stops, domestics, backup calls, or suspicious vehicles.
- No direct evidence follow-up from suspicious vehicle calls.
- No dispatch-desk selection flow.

## Precinct

References: `spec.md` precinct section; `precinct/mod.rs:52-430`.

Implemented features:

- Precinct interactables spawn and hide/show based on the current map.
- The case board can assign the first available case or open the case file if one is already active.
- The evidence terminal can open the evidence examination state.
- Coffee and meal interactions restore fatigue, relieve stress, and advance time.
- The captain door routes to the career view.
- The dispatch radio shows the current dispatch description if one exists.
- The exterior evidence scene can emit evidence collection events.

Partial features:

- The evidence terminal only opens a screen here; it does not start evidence processing from player interaction in this module.
- A helper exists to mark all raw evidence as processing, but it is not wired into player interaction here.
- The locker interaction is a placeholder toast.
- The exterior evidence scene is a training shortcut that dumps all 30 evidence IDs onto the first active case.
- The dispatch radio is informational only.

Missing features:

- No records room interaction.
- No cold-case access flow.
- No interrogation-room interaction path in this module.
- No promotion request or reputation-check interaction.
- No locker equipment management.
- No partner conversations in the break room.
- No dispatch-call choice/triage interaction.

## UI

References: `spec.md` UI section; `ui/mod.rs:15-380`.

Implemented features:

- Loading boots into the main menu.
- Main menu supports New Game, Load Game, and Quit.
- HUD shows time, day, weather, rank, duty state, gold, fatigue, and stress.
- Pause menu supports resume, save, load, skill tree, career view, and quit to menu.
- Hotkeys route to skill tree, case file, and career-view states.
- Notification and screen submodules are installed by this module.

Partial features:

- This `mod.rs` proves that screen states are routed and installed, but the actual detailed screen implementations live outside the requested review scope.
- Save/load UI is hardcoded to slot 0 even though the save domain supports 3 slots.

Missing features visible from `ui/mod.rs`:

- No HUD active-case indicator.
- No flashing radio icon for dispatch.
- No notebook UI visible in this file.
- No map screen visible in this file.
- No settings screen/menu visible in this file.
- No shift summary screen visible in this file.
- No main-menu 3-slot load flow visible in this file.

## Save

References: `spec.md` save section; `save/mod.rs:19-198`.

Implemented features:

- Save and load are implemented with serde JSON.
- The save payload includes shift clock, player state, inventory, case board, evidence locker, NPC mutable state, partner arc, economy, skills, and patrol state.
- There are 3 logical save slots via slot normalization.
- Auto-save is triggered on shift end.
- Load restores all serialized resources and returns the game to `Playing`.

Partial features:

- Save coverage is broad, but it only stores mutable NPC data from the registry, not immutable definitions.
- Three slots exist at the data layer, but this is not surfaced as a player-facing slot-selection flow in `ui/mod.rs`.

Missing features:

- No settings serialization for volume, window size, or keybinds.
- No explicit world object state serialization beyond the resources included here.
- No explicit save payload for map-object state or broader world simulation beyond the captured resources.

## Cross-domain summary

Strongest implemented player path:

- Walk around the precinct.
- Take authored cases from the board.
- Trigger evidence collection shortcuts.
- Build up evidence/case progress.
- Use core HUD, pause, save/load, and some menu routing.
- Resolve dispatches at the state/event level.

Most important spec gaps:

- World breadth is far below spec: only 2 maps are truly playable from `world/mod.rs`.
- Many systems have strong authored data but thin player-facing behavior: cases, evidence, NPC relationships, and skills are the clearest examples.
- Several precinct rooms are represented spatially but not functionally.
- UI routing exists, but `ui/mod.rs` alone does not demonstrate many of the spec's promised player-facing screens.
- Save/load is solid for core state, but settings and broader world-state coverage are still missing.
