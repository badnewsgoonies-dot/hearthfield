# Orchestrator Persistent State
# READ THIS ON EVERY SESSION START / AFTER COMPACTION

## Sub-Agent Dispatch Methods (in order of preference)

### 1. Copilot CLI (PREFERRED — uses user's GitHub Premium requests)
```bash
export COPILOT_GITHUB_TOKEN="$COPILOT_GITHUB_TOKEN"  # Token stored in ~/.bashrc, NOT in repo
copilot -p "$(cat objectives/fix-something.md)" --allow-all-tools --model claude-sonnet-4.6
```
- Token is in ~/.bashrc (github_pat_11BWLAP7A... — do NOT commit to repo)
- Works as of 2026-03-04
- Use `--allow-all-tools` for full autonomous mode
- Use `--model claude-sonnet-4.6` for best results

### 2. Built-in Agent tool (FALLBACK — if copilot fails)
- Use `subagent_type: "general-purpose"` for implementation
- Use `subagent_type: "Explore"` for read-only investigation
- These are Claude sub-agents, more expensive per the user's preference

## Current Phase: Wave 5 — Pilot DLC UI Completion (COMPLETE)

### Branch: claude/setup-orchestration-framework-L8ILN

### Wave 1 (COMPLETED — 0cb9202) — 7 critical bugs fixed
### Wave 2a-d (COMPLETED — 80ba40a → 0510331) — 10 more fixes
### Wave 3a-c (COMPLETED — 566b266 → 5ca809f) — 3 new UI screens (Journal, Relationships, Map)

### Wave 4a: DLC Audit + City Fixes (COMPLETED — 22ad657)
- Removed duplicate city_office_worker_dlc/ directory
- City DLC: task pacing (0.28 base), auto-interruptions, stress persistence, inbox balance
- 40/40 city tests passing

### Wave 4b: Pilot DLC Critical Fixes (COMPLETED — cd77e9a)
- setup_new_game() registered + starter Cessna 172 + starter items
- TransactionLog, GoldMilestones, FuelWarnings added to SaveFile

### Wave 4c: Test Coverage + Main Save (COMPLETED — 7cf470e)
- 12 new pilot headless tests (save roundtrip, airports, data validation) — 76/76 pass
- 5 missing main game resources in FullSaveFile (ToolUpgradeQueue, ShippingBinQuality, FestivalState, FarmVisitTracker, ProcessingMachineRegistry)
- Main game: 88/88 tests pass

### Wave 4d: Pilot DLC Deep Parity (COMPLETED — b6531a8)
- 6 more resources persisted in pilot SaveFile (PilotSkills, StoryProgress, LoanPortfolio, InsuranceState, AirlineBusiness, RelationshipDetails)
- Story missions injected into mission board with ★ STORY prefix
- 3 UI screens wired (RadioComm, CrewLounge, Cutscene)
- Shop buy buttons functional with PurchaseEvent + hover feedback
- All gates green: main 88/88, pilot 76/76, city 40/40

### Wave 5: Pilot DLC UI Completion (COMPLETED — 84cb491)
- 12 GameState variants added (LoadGame, Logbook, Profile, Achievements, Settings, MapView, Notifications, Tutorial, Intro, LoanOffice, InsuranceOffice, BusinessHQ)
- 8 existing screens wired to state transitions
- Load Game screen created (save slot selection + load)
- 3 economy UI screens: Loan Office, Insurance, Business HQ
- Inventory upgraded: selection, use/equip, detail panel, keyboard nav
- Main menu Load TODO resolved
- Audit: 11/12 wave 4 checks PASS (city DLC test naming cosmetic only)
- Full regression: main 88/88, pilot 76/76, city 40/40

### REMAINING PILOT DLC GAPS
**Medium Priority:**
- Aircraft upgrades have no purchase UI
- No dialogue branching/choices (crew conversations are monologue)
- No romance/quest system for crew
- settings.apply_settings is a no-op

**Lower Priority:**
- No crafting/cooking equivalent
- Content volume thin (40 items vs main game's ~80)

### REMAINING MAIN GAME ISSUES (lower priority)
- Hay proximity check, shop sell tracking, Settings/Calendar/Stats screens

### Game Completeness Snapshot
| System | Main Game | Pilot DLC | City DLC |
|--------|-----------|-----------|----------|
| Core Loop | 100% | 95% | 85% |
| Save/Load | 100% | 100% (24 fields) | 100% |
| Tests | 88/88 | 76/76 | 40/40 |
| UI Screens | 85% | 95% (20/21 wired) | 60% |
| Economy | 100% | 95% (3 new UI screens) | 80% |
| NPCs/Crew | 100% | 80% (no romance/quests) | N/A |
| Content Volume | 100% | 75% | 60% |

## DLC Status

### dlc/city/ — City Office Worker
- ~5,000 LOC, 40/40 tests passing
- **Fixed:** task pacing, auto-interruptions, stress persistence, inbox balance
- **Remaining:** main menu UI, HUD screen, day summary screen, content variety

### dlc/pilot/ — Skywarden Pilot Career Sim
- ~31,000 LOC, 76/76 tests passing, 14 domains
- **Fixed this session:** new game init, 9 save fields, story board, 3 UI screens, shop buttons, 12 tests
- **Fixed wave 5:** 12 GameState variants, 12 screens wired, Load Game, 3 economy UI, inventory interactions
- **Remaining:** Aircraft upgrade UI, dialogue branching, romance/quests, content volume

## Architecture Quick Reference
- Rust + Bevy 0.15 ECS, plugin-per-domain (15 domains, ~41k LOC)
- src/shared/mod.rs is FROZEN type contract (2,252 lines, SHA256 checksummed)
- Gates: shasum check → cargo check → cargo test --test headless → cargo clippy -- -D warnings
- TILE_SIZE=16.0, PIXEL_SCALE=3.0, Screen=960x540
- 28 days/season, 4 seasons, max_stamina=100, inventory=36 slots

## User Directives
- NEVER write Rust code directly — always dispatch workers (copilot preferred, Agent fallback)
- "devise a formation and keep INTELLIGENTLY overseeing and dispatching"
- Prefer copilot CLI over built-in Agent tool for implementation
- Focus on fixing existing broken systems before adding new content
- Think from "second 0" — trace the real player experience
- Save context to this file after every major milestone
- Launch subagents for future investigation
- Launch audit subagents for prior work verification
