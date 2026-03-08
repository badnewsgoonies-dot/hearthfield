# MANIFEST — Precinct (Police DLC)

## Current Phase
Phase 5 — Waves 7-8 in progress, Wave 9 (WASM) next

## Wave History
- **Wave 1** (COMPLETE): calendar, player, world, ui — 2,256 LOC, 25 tests
- **Wave 2** (COMPLETE): cases, evidence, patrol, precinct — +2,971 LOC, 53 tests
- **Wave 3** (COMPLETE): skills, economy, npcs — +2,254 LOC, 81 tests
- **Wave 4** (COMPLETE): save + integration fixes — 96 tests, all gates green
- **Wave 5** (COMPLETE): event wiring + 5 UI screens (toast, dispatch radio, case file, skill tree, career, interrogation, evidence exam) — 107 tests
- **Wave 6** (COMPLETE): art integration (LimeZu 16x16 sprites for tiles, characters, objects) — 107 tests
- **Wave 7** (COMPLETE): audio system (SFX + music readers wired) — 111 tests
- **Wave 8** (IN PROGRESS): content depth (dialogue, case flavor, NPC personality)
- **Wave 9** (PLANNED): WASM build for browser deployment

## Domain List

| Domain | LOC | Tests | Status |
|--------|-----|-------|--------|
| calendar | ~335 | 6 | Complete + art |
| player | ~600 | 8 | Complete + sprites |
| world | ~900 | 5+ | Complete + tilesets |
| ui | ~3000 | 20+ | Complete (toast, screens, HUD) |
| cases | ~1300 | 7+ | Complete |
| evidence | ~600 | 7+ | Complete |
| patrol | ~600 | 7+ | Complete |
| precinct | ~1000 | 6+ | Complete + sprites |
| skills | ~650 | 6+ | Complete |
| economy | ~500 | 7+ | Complete |
| npcs | ~1200 | 7+ | Complete + sprites |
| save | ~700 | 7+ | Complete |

## Totals
- **LOC**: ~13,928 (target: 55,000+)
- **Tests**: 111 passing
- **Domains**: 12/12 complete
- **Gates**: 4/4 green (contract, check, test, clippy)

## Truth Decisions (frozen)
- All IDs are String
- Fatigue/stress separate f32 (0-100)
- Trust (-100 to +100) / pressure (0 to 100) separate per NPC
- Shift-gated rank progression (28 shifts/tier)
- TIME_SCALE = 2.0, SHIFT_DURATION = 8hrs
- Evidence quality: min(0.95, 0.5 + skill*0.05 - weather*0.1)
- MAX_ACTIVE_CASES = 3
- ALL gold changes via GoldChangeEvent (no direct mutation)
- Contract checksum: 78b9fbc411f4b1b7acd4b39a382e8bf6b1118039519bd6e90e7c41b30525ec0a

## Entrypoint
- Binary: cargo run -p precinct
- Runtime: Loading → MainMenu → Playing → shift loop
