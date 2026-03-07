# Wave 11 — UX Feedback Coverage (Autonomous Orchestration)

## Mission
Fill remaining player feedback gaps. Every player action that can fail or change state
should produce visible/audible feedback. All changes are ADDITIVE (send toast/SFX events).
No control flow changes. No shared contract edits.

## Workers to dispatch (sequential, via copilot-dispatch.sh)

### Worker A: Crafting Feedback (src/crafting/)
Scope: src/crafting/ only
Tasks:
1. In bench.rs handle_craft_item: when ingredients insufficient, send toast "Missing ingredients!" + error SFX
2. In bench.rs handle_craft_item: on successful craft, send toast "{item_name} crafted!" + SFX "sfx_coin_single1"
3. In cooking.rs: same pattern — missing ingredients toast, success toast
4. In machines.rs: when player collects machine output, send toast "{item} collected!" + pickup SFX

### Worker B: Farming Feedback (src/farming/)
Scope: src/farming/ only
Tasks:
1. In soil.rs handle_till: if tile is already tilled, send toast "Already tilled!" (prevent confusion)
2. In soil.rs handle_water: if tile already watered, send toast "Already watered today."
3. In harvest.rs: on successful harvest, send toast "Harvested {crop}!" (the event fires but no player-facing text)
4. In events_handler.rs: when crop dies (wrong season after season change), send toast "{crop} withered..."

### Worker C: Fishing + Mining Feedback (src/fishing/ + src/mining/)
Scope: src/fishing/ AND src/mining/ only
Tasks:
1. In fishing/cast.rs: when cast fails (wrong location/no water), send toast "Can't fish here."
2. In fishing/cast.rs: when fish escapes (reaction window missed), send toast "The fish got away!"
3. In fishing/minigame.rs: on successful catch, send toast "Caught a {fish_name}!" (if not already there)
4. In mining/combat.rs: when player takes damage, send toast "Ouch! -{damage} HP" (brief, 1.5s)
5. In mining/ladder.rs: on floor transition, send toast "Floor {n}" (brief, 1.5s)

### Worker D: Weather + Season Notifications (src/world/ + src/calendar/)
Scope: src/world/ AND src/calendar/ only
Tasks:
1. In world/weather_fx.rs: when weather changes (rain starts/stops), send toast "It started raining." / "The rain stopped."
2. In calendar/mod.rs handle_season_change: send toast "{Season} has arrived!" if not already done by cutscene
   (Check: trigger_sleep already shows season text in cutscene. Only add toast if season changes WITHOUT sleep, e.g. loading a save.)

## Validation per worker
- cargo check must pass (zero errors)
- No files modified outside worker's scope
- Write completion report to status/workers/feedback-{domain}.md

## Global validation after all workers
- cargo check
- cargo clippy -- -D warnings  
- shasum -a 256 -c .contract.sha256

## On completion
- git add all changed files + reports
- git commit with descriptive message
- Write summary to status/wave11-complete.md
