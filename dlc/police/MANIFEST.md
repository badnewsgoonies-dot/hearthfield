# MANIFEST — Precinct (Police DLC)

## Current Phase
Phase 3 — Wave 3 in progress (skills, economy, npcs)

## Wave History
- **Wave 1** (COMPLETE): calendar, player, world, ui — 2,256 LOC, 25/25 tests
- **Wave 2** (COMPLETE): cases, evidence, patrol, precinct — 2,971 LOC, 53/53 tests total
- **Integration fix** (IN PROGRESS): boot path, collision wiring, transition fix, pause/resume, clippy
- **Wave 3** (IN PROGRESS): skills, economy, npcs — dispatched via 2-layer Codex orchestrator

## Domain List

| Domain | Allowlist | Status | LOC | Tests |
|--------|-----------|--------|-----|-------|
| calendar | src/domains/calendar/ | Complete | 335 | 6 |
| player | src/domains/player/ | Complete (integration fix pending) | 506 | 8 |
| world | src/domains/world/ | Complete (integration fix pending) | 569 | 5 |
| ui | src/domains/ui/ | Complete (integration fix pending) | 856 | 6 |
| cases | src/domains/cases/ | Complete | 1,151 | part of 53 |
| evidence | src/domains/evidence/ | Complete | 569 | part of 53 |
| patrol | src/domains/patrol/ | Complete | 589 | 7 |
| precinct | src/domains/precinct/ | Complete | 652 | part of 53 |
| skills | src/domains/skills/ | Wave 3 in progress | — | — |
| economy | src/domains/economy/ | Wave 3 in progress | — | — |
| npcs | src/domains/npcs/ | Wave 3 in progress | — | — |
| save | src/domains/save/ | Wave 4 (deferred) | — | — |

## Truth Decisions (frozen)
- All IDs are `String` (CaseId, EvidenceId, NpcId, ItemId)
- Fatigue and stress are separate f32 resources (0–100 each)
- Trust (-100 to +100) and pressure (0 to 100) are separate i32 per NPC
- Rank progression is shift-gated (28 shifts per tier), not open-ended
- Time scale: 2.0 game-minutes per real-second
- Evidence quality: base 0.5 + (skill_level × 0.05), capped at 0.95
- Max 3 active cases simultaneously
- ALL gold changes via GoldChangeEvent (no direct mutation — Hearthfield shop bypass anti-pattern)

## Key Formulas
```
evidence_quality = min(0.95, 0.5 + investigation_level * 0.05 - weather_penalty)
weather_penalty = 0.1 * (is_rainy + is_foggy + is_night)
case_xp = difficulty * 15
case_gold = difficulty * 25
salary = rank.salary()  // 80, 120, 160, 200
skill_points_available = total_xp / 100 - points_spent
dispatch_rate = 0.15 * map.dispatch_rate_modifier() * (1.5 if night_shift else 1.0)
```

## Known Integration Issues (from Wave 1 audit)
1. ~~Boot path: Loading → MainMenu transition missing~~ (fix dispatched)
2. ~~CollisionMap split between player and world~~ (fix dispatched)
3. ~~Map transition trigger mismatch~~ (fix dispatched)
4. ~~Pause/resume respawns player~~ (fix dispatched)
5. ShiftEndEvent payload fields always zero (cosmetic, future fix)

## Remaining Work
- Wave 3: skills, economy, npcs
- Wave 4: save (deferred until gameplay resources stabilize)
- Wave 5: integration pass, remaining UI screens, polish
- Future: art assets, audio, WASM build

## Entrypoint
- Binary: `cargo run -p precinct`
- Runtime: Loading → MainMenu → Playing → shift loop
