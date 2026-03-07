# MANIFEST — Precinct (Police DLC)

## Current Phase
Phase 0 — Bootstrap (contract frozen, specs written, ready for Wave 1)

## Domain List

| Domain | Allowlist | Owner | Status |
|--------|-----------|-------|--------|
| calendar | src/domains/calendar/ | — | Not started |
| player | src/domains/player/ | — | Not started |
| world | src/domains/world/ | — | Not started |
| cases | src/domains/cases/ | — | Not started |
| evidence | src/domains/evidence/ | — | Not started |
| npcs | src/domains/npcs/ | — | Not started |
| economy | src/domains/economy/ | — | Not started |
| skills | src/domains/skills/ | — | Not started |
| patrol | src/domains/patrol/ | — | Not started |
| precinct | src/domains/precinct/ | — | Not started |
| ui | src/domains/ui/ | — | Not started |
| save | src/domains/save/ | — | Not started |

## Truth Decisions (frozen)

- All IDs are `String` (CaseId, EvidenceId, NpcId, ItemId)
- Fatigue and stress are separate f32 resources (0–100 each)
- Trust (-100 to +100) and pressure (0 to 100) are separate i32 per NPC
- Rank progression is shift-gated (28 shifts per tier), not open-ended
- Time scale: 2.0 game-minutes per real-second
- Evidence quality: base 0.5 + (skill_level × 0.05), capped at 0.95
- Max 3 active cases simultaneously
- 25 total cases: 8 Patrol + 8 Detective + 6 Sergeant + 3 Lieutenant
- Salary: 80/120/160/200 per shift by rank
- XP per case: difficulty × 15
- Skill point every 100 XP

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

## Open Blockers
- None (ready for Wave 1 dispatch)

## Recurring Drift Patterns
- (none yet — will update after Wave 1)

## Failed Seam Decisions
- (none yet)

## Entrypoint
- Binary: `cargo run` from `dlc/police/`
- Runtime: Bevy app → MainMenu → Playing → shift loop
- All work must be reachable from this binary's main.rs

## Wave History
- (none yet)
