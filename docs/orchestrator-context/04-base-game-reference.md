# 04 — Hearthfield Base Game: Architecture + Audit

Load this FOURTH. This is the reference implementation — patterns to follow and anti-patterns to avoid.

## Project Overview
- **Rust/Bevy 0.15.3** farming sim (Stardew Valley-inspired), 16x16 pixel art, 3x upscale
- **~144k total LOC**: main game ~50k, pilot DLC ~75k, city DLC ~17k, police DLC ~2k
- **429 commits** over ~9 days, heavy AI-assisted development
- Plugin-per-domain architecture, 15 main domains, event/resource-driven integration
- Shared type contract at `src/shared/mod.rs` (2,320 LOC, SHA256 checksummed)

## Architecture
- Workspace: main crate + 3 DLC crates (skywarden, city_office_worker_dlc, precinct)
- No cross-domain imports — all via `crate::shared::*` (events + resources)
- SystemSet phases: Input → Intent → Simulation → Reactions → Presentation
- Plugin registration order matters (Input first, UI/Save last)

## Build Gates
```bash
bash scripts/run-gates.sh  # 5 gates: contract, check, test, clippy, connectivity
```

## Codebase Audit (March 2026) — Current State

### Events: ALL 51 WIRED (was 17 orphans in old audit)
- 32 shared events + 19 domain-local events
- Zero orphaned events
- Heaviest: DayEndEvent (5 writers, 22 readers), ItemPickupEvent (17 writers, 6 readers)
- Pattern: `ItemUseEvents` SystemParam bundles related EventWriters — clean, adopt for Precinct

### UI Bypass: 1 SIGNIFICANT
- `shop_screen.rs` directly mutates `player.gold` and `Inventory` instead of routing through events
- Also emits ShopTransactionEvent — but for tracking, not for the actual mutation
- **Anti-pattern for Precinct**: route ALL state mutations through domain events from day one

### GameState: 1 DEAD VARIANT
- `GameState::Mining` has zero references (mining works via MapTransitionEvent)
- **Lesson**: don't define state variants without wiring them

### Save/Load: COMPREHENSIVE
- ~25 resources serialized covering all player-mutable state
- Correctly excludes read-only registries and runtime UI state

### System Ordering: PARTIAL
- 24 explicit ordering constraints exist
- 22 DayEndEvent consumers have no relative ordering (latent nondeterminism)
- **Lesson for Precinct**: spec explicit ordering for ShiftEndEvent consumers from the start

### Tests: 254 TOTAL
- 130 base headless + 1 keybinding + 47 city DLC + 76 pilot DLC
- Headless tests use MinimalPlugins (no GPU required)

## Key Constants
- TILE_SIZE=16, PIXEL_SCALE=3.0, SCREEN=960x540
- DAYS_PER_SEASON=28, SEASONS=4, MAX_STAMINA/HEALTH=100
- INVENTORY_SLOTS=36, FRIENDSHIP_PER_HEART=100, MAX_HEARTS=10
- Quality multipliers: 1.0/1.25/1.5/2.0 (Normal/Silver/Gold/Iridium)

## Retrospective Key Findings
- **Keep**: domain decomposition, event rails, quality gates, delivery speed
- **Fix**: monolithic shared contract (split by bounded context), O(n²) patterns, 30% fix-loop churn
- **Add**: SystemSet taxonomy, SubStates, FixedUpdate, change-driven updates, data-driven content
