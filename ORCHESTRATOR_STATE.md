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

## Current Phase: Wave 6 — Cross-Project UI Parity (COMPLETE)

### Branch: claude/setup-orchestration-framework-L8ILN

### Waves 1-3 (COMPLETED) — 7 bugs + 10 fixes + 3 UI screens
### Wave 4a-d (COMPLETED) — DLC audit, pilot critical fixes, test coverage, deep parity
### Wave 5 (COMPLETED — 84cb491) — Pilot DLC: 12 GameState variants, 12 screens wired, economy UI, inventory

### Wave 6a: Main Game UI Completion (COMPLETED — 7b508ca)
- Calendar overlay (F1): 4x7 grid, festivals, NPC birthdays, current day highlight
- Statistics overlay (F2): all 10 PlayStats fields, two-column layout
- Settings overlay (F4): volume control + keybinds display
- Overlay pattern: no shared/mod.rs changes, contract intact (shasum OK)
- 88/88 tests pass

### Wave 6b: City DLC Full UI Layer (COMPLETED — 65dd8c7)
- Created entire game/ui/ module from scratch (+1,227 lines, 7 new files)
- Main Menu: New Game, Load Game, Quit buttons (replaces auto-transition)
- HUD: energy/stress/focus/rep/money bars, clock, inbox count, key hints
- Pause Menu: Resume, Save Game, Quit to Menu
- Day Summary: salary, tasks, reputation, level up, Continue button (replaces auto-advance)
- Task Board: active tasks with progress bars, priority badges, deadlines
- Interruption popup: Calm/Panic choice when pending interruptions > 0
- Removed println! → info!, removed auto-transitions
- 40/40 tests pass, clippy clean

### REMAINING GAPS (all lower priority)
**Pilot DLC:**
- Aircraft upgrades purchase UI
- Dialogue branching/choices for crew
- Romance/quest system for crew
- settings.apply_settings is a no-op
- Content volume (40 items vs main game ~80)

**City DLC:**
- Content variety (only 4 task kinds, 6 interruption scenarios)
- NPC entity spawning (names exist, no visual entities)
- Office world/navigation (rooms, movement)
- Dialogue system beyond interrupt choices
- Extended endurance testing (only 5-day tested)

**Main Game:**
- Hay proximity check (cosmetic)
- Shop sell gold not split from earned gold in PlayStats (cosmetic)

### Game Completeness Snapshot
| System | Main Game | Pilot DLC | City DLC |
|--------|-----------|-----------|----------|
| Core Loop | 100% | 95% | 95% |
| Save/Load | 100% | 100% (24 fields) | 100% |
| Tests | 88/88 | 76/76 | 40/40 |
| UI Screens | 100% (3 overlays added) | 95% (20/21 wired) | 95% (6 screens from scratch) |
| Economy | 100% | 95% (3 UI screens) | 95% (salary + progression visible) |
| NPCs/Crew | 100% | 80% (no romance/quests) | 70% (names only, no entities) |
| Content Volume | 100% | 75% | 60% |

## DLC Status

### dlc/city/ — City Office Worker
- ~6,600 LOC (+1,227 UI), 40/40 tests passing
- **Fixed waves 4-6:** task pacing, auto-interruptions, stress persistence, inbox balance, FULL UI LAYER
- **Remaining:** content variety, NPC entities, office navigation, dialogue, endurance testing

### dlc/pilot/ — Skywarden Pilot Career Sim
- ~33,000 LOC, 76/76 tests passing, 14 domains
- **Fixed waves 4-5:** new game init, 9 save fields, story board, 15 UI screens wired, economy UI, inventory
- **Remaining:** aircraft upgrade UI, dialogue branching, romance/quests, content volume

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
