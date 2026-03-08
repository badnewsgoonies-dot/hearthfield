# 06 — Precinct (Police DLC): Current Project State

Load this SIXTH. With theory, tools, and reference loaded, evaluate this project against known patterns.

## What It Is
Police simulator DLC for Hearthfield. "What kind of cop will you be?" — Harvest Moon spirit.
Designed to become the actual core game base. Architecture must be cleaner than Hearthfield.

## Location
`dlc/police/` in the hearthfield workspace. Crate name: `precinct`.
Binary: `cargo run -p precinct` from repo root.

## Phase 0-2 Complete (merged via PR #40)
- Game spec: `dlc/police/docs/spec.md` (489 lines, 12 domains, full contrastive decision fields)
- Type contract: `dlc/police/src/shared/mod.rs` (848 lines, 73 types, 25 events, 12 resources, 50 constants)
- Main.rs: all events registered, all resources initialized, Wave 1 plugins wired
- MANIFEST: `dlc/police/MANIFEST.md`
- Infrastructure: clamp-scope.sh, run-gates.sh, .contract.sha256
- Contract checksum: `78b9fbc411f4b1b7acd4b39a382e8bf6b1118039519bd6e90e7c41b30525ec0a`

## 12 Domains
1. **calendar** — shift clock, time, weather, rank progression
2. **player** — movement, fatigue, stress, equipment, collision
3. **world** — 12 maps (RON tiles), transitions, day/night, weather overlay
4. **cases** — 25 hand-authored cases, 4 rank tiers, case state machine
5. **evidence** — 30 types, 6 categories, quality scaling, processing pipeline
6. **npcs** — 12 NPCs, trust/pressure dual axis, partner arc, schedules
7. **economy** — salary, reputation, department budget, promotion requirements
8. **skills** — 4 trees × 5 levels (20 perks), XP sources
9. **patrol** — dispatch events, patrol car, fuel, area/time modifiers
10. **precinct** — hub (case board, evidence room, interrogation, break room)
11. **ui** — 15 screens (HUD, case file, interrogation, notebook, skill tree)
12. **save** — full serialization, 3 slots, auto-save on shift end

## Recontextualization Map (Hearthfield → Precinct)
| Hearthfield | Precinct |
|---|---|
| Seasons | Ranks (Patrol/Detective/Sergeant/Lieutenant) |
| Crops (plant→water→harvest) | Cases (assigned→investigate→solve) |
| NPCs (gift/friendship) | NPCs (trust/pressure dual axis) |
| Mining (floors/combat) | Crime scenes (search/evidence/confront) |
| Fishing (minigame) | Interrogation (dialogue minigame) |
| Shipping bin | Case reports (file at shift end) |
| Gold | Salary + reputation + budget |
| Stamina | Fatigue + Stress (dual constraint) |

## Key Constants (from frozen contract)
- TIME_SCALE=2.0 game-min/real-sec, SHIFT_DURATION=8hrs
- MAX_ACTIVE_CASES=3, 25 total cases (8+8+6+3 by rank)
- Evidence quality: base 0.5 + skill*0.05, max 0.95, weather penalty 0.1
- Salary: 80/120/160/200 by rank
- Promotions: Detective(200xp/3cases/10rep), Sergeant(500/8/25), Lieutenant(1000/16/50)
- Dispatch rate: 0.15/game-hour × map modifier × night modifier

## Golden Path (First 60 Seconds)
1. Boot → Loading → MainMenu
2. "New Game" → GameState::Playing
3. Player spawns in Precinct Interior (hub)
4. Walk around precinct (4-dir movement, collision with walls)
5. See HUD: shift clock ticking, fatigue bar, stress bar, gold, rank
6. Explore rooms (case board, break room, evidence room)
7. Shift clock advances in real-time
8. Press Escape → Pause menu → Resume
9. (Future: check case board → accept case → patrol → investigate)

## Wave Plan
- **Wave 1** (first-60-seconds): calendar, player, world, ui — 4 parallel workers
- **Wave 2**: cases, evidence, precinct, npcs
- **Wave 3**: economy, skills, patrol
- **Wave 4**: save, integration, testing
- **Wave 5**: polish, remaining UI screens

## Domain Specs
Written to `dlc/police/docs/domains/{domain}.md` with full contrastive decision fields.
Worker objectives at `dlc/police/objectives/wave1-{domain}.md`.
