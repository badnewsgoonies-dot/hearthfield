# MANIFEST — Hearthfield Orchestrator

## Current Phase: 3 (worker dispatch ready)

## Architecture
- Orchestrator: Claude Opus (1M context, holds full codebase)
- Workers: dispatched via `codex exec --full-auto --skip-git-repo-check`
- Coordination: frozen type contract + mechanical scope clamping + compiler gates
- Scaffold: zero — orchestration logic lives in this conversation + disk artifacts

## Type Contract
- File: `src/shared/mod.rs` (2,252 lines)
- Checksum: `a6b95251fcc56437ba0b21ec73b407b50861b2cf06183a2689a93d82ea4d80dc`
- Status: FROZEN — no worker may modify
- Verify: `shasum -a 256 -c .contract.sha256`

## Gate Commands (all must pass after every worker)
```bash
bash scripts/run-gates.sh              # Unified: contract + check + test + clippy + connectivity
# Or individually:
shasum -a 256 -c .contract.sha256      # Contract integrity
cargo check                            # Type-check
cargo test --test headless             # Integration tests (93 tests, no GPU)
cargo clippy -- -D warnings            # Lint gate
```

## Domains (15) — Status

| # | Domain | Path | Spec | Objective | Worker Status |
|---|--------|------|------|-----------|---------------|
| 1 | calendar | src/calendar/ | docs/domains/calendar.md | objectives/calendar.md | Ready |
| 2 | player | src/player/ | docs/domains/player.md | objectives/player.md | Ready |
| 3 | farming | src/farming/ | docs/domains/farming.md | objectives/farming.md | Ready |
| 4 | animals | src/animals/ | docs/domains/animals.md | objectives/animals.md | Ready |
| 5 | world | src/world/ | docs/domains/world.md | objectives/world.md | Ready |
| 6 | npcs | src/npcs/ | docs/domains/npcs.md | objectives/npcs.md | Ready |
| 7 | economy | src/economy/ | docs/domains/economy.md | objectives/economy.md | Ready |
| 8 | crafting | src/crafting/ | docs/domains/crafting.md | objectives/crafting.md | Ready |
| 9 | fishing | src/fishing/ | docs/domains/fishing.md | objectives/fishing.md | Ready |
| 10 | mining | src/mining/ | docs/domains/mining.md | objectives/mining.md | Ready |
| 11 | ui | src/ui/ | docs/domains/ui.md | objectives/ui.md | Ready |
| 12 | save | src/save/ | docs/domains/save.md | objectives/save.md | Ready |
| 13 | data | src/data/ | docs/domains/data.md | objectives/data.md | Ready |
| 14 | input | src/input/ | docs/domains/input.md | objectives/input.md | Ready |
| 15 | shared | src/shared/ | N/A (contract) | N/A (frozen) | Frozen |

## Map Bounds (truth — all coordinates must respect these)
- Farm: 32×24 (x: 0-31, y: 0-23)
- Town: 28×22 (x: 0-27, y: 0-21)
- Beach: 20×14 (x: 0-19, y: 0-13)
- Forest: 22×18 (x: 0-21, y: 0-17)
- MineEntrance: 14×12 (x: 0-13, y: 0-11)
- PlayerHouse: 16×16 (x: 0-15, y: 0-15)
- GeneralStore: 12×12 (x: 0-11, y: 0-11)
- AnimalShop: 12×12 (x: 0-11, y: 0-11)
- Blacksmith: 12×12 (x: 0-11, y: 0-11)

## Coordinate System
- tiles[y * width + x] row-major
- y=0 is back wall (north), y=h-1 is front/door (south)
- grid_to_world_center(x, y) converts to pixel coords
- world_to_grid(wx, wy) converts back (floor-based)

## Key Constants (truth decisions)
- Tile size: 16px
- Pixel scale: 3.0
- Screen: 960×540
- Days per season: 28
- Seasons per year: 4
- Max stamina: 100.0
- Max health: 100.0
- Inventory: 36 slots (12 hotbar + 24 backpack)
- Friendship: 100 points per heart, 10 hearts max
- Gift points: Loved +80, Liked +45, Neutral +20, Disliked -20, Hated -40
- Birthday multiplier: 8x
- Tool upgrade costs: 2000/5000/10000/25000g + 5 bars
- Quality sell multipliers: Normal 1.0, Silver 1.25, Gold 1.5, Iridium 2.0
- Player speed: 80.0 px/sec
- NPC speed: 40.0 px/sec

## Worker Dispatch Protocol
1. Write spec → `docs/domains/{domain}.md` ✓
2. Write objective → `objectives/{domain}.md` ✓
3. Dispatch → `codex exec --full-auto --skip-git-repo-check "$(cat objectives/{domain}.md)"`
4. Wait for completion
5. Clamp → `bash scripts/clamp-scope.sh src/{domain}/`
6. Verify contract → `shasum -a 256 -c .contract.sha256`
7. Run gates → `bash scripts/run-gates.sh`
8. If gates fail → dispatch fix worker (same scope), clamp, re-gate (max 10 passes)
9. Write report → `status/workers/{domain}.md`
10. Commit → descriptive message
11. Update this file

## Tier Plan (from UPGRADE_STRATEGY.md)
- **Tier 0:** Unblock core loops (crafting, quests, tool upgrades)
- **Tier 1:** Complete event graph (achievements, fishing level-up, tool upgrade completion, dead event audit)
- **Tier 2:** Content & depth (dialogue, quests, festivals, mine polish, recipes, animals)
- **Tier 3:** Polish & deploy (audio, transitions, save audit, WASM, touch input)

## Open Blockers
- None — framework is ready for worker dispatch

## Completed Phases
- Phase 0: Bootstrap (repo, contract, checksum, clamp script) ✓
- Phase 1: Domain boundaries drawn ✓
- Phase 2: Full specs on disk for all 15 domains ✓
- Phase 3: Worker objectives written for all 14 dispatchable domains ✓
