# MANIFEST — Hearthfield Orchestrator

## Current Phase: 3 (worker dispatch ready)

## Architecture
- Orchestrator: Claude Opus 4.6 (1M context, holds full codebase)
- Workers: dispatched via Copilot CLI (preferred) or claude -p (fallback)
- Coordination: frozen type contract + mechanical scope clamping + compiler gates
- Scaffold: zero — orchestration logic lives in this conversation + disk artifacts

## Copilot CLI Dispatch Recipe (VERIFIED WORKING in this container)
```bash
# Auth (tokens in ~/.env_tokens)
source ~/.env_tokens
# COPILOT_GITHUB_TOKEN must be fine-grained PAT (github_pat_...), NOT classic PAT

# Orchestrator (GPT 5.4)
copilot -p "orchestrator prompt" --model gpt-5.4 --yolo --add-dir /home/user/hearthfield

# Worker (Opus 4.6 — 3 premium requests per call)
copilot -p "$(cat objectives/{domain}.md)" --model claude-opus-4.6 --yolo --add-dir /home/user/hearthfield

# Worker (Sonnet 4.6 — 1 premium request per call, cheaper)
copilot -p "$(cat objectives/{domain}.md)" --model claude-sonnet-4.6 --yolo --add-dir /home/user/hearthfield

# Parallel dispatch (max 2-3 concurrent, stagger 3s)
copilot -p "..." --model claude-opus-4.6 --yolo --add-dir /home/user/hearthfield 2>&1 | tee status/workers/worker1.json &
sleep 3
copilot -p "..." --model claude-opus-4.6 --yolo --add-dir /home/user/hearthfield 2>&1 | tee status/workers/worker2.json &
```

### Available Models (copilot --model)
claude-opus-4.6, claude-opus-4.6-fast, claude-sonnet-4.6, claude-sonnet-4.5, claude-haiku-4.5,
gpt-5.4, gpt-5.3-codex, gpt-5.2-codex, gpt-5.2, gpt-5.1-codex, gpt-5.1, gpt-4.1,
gemini-3-pro-preview

### Auth Notes
- `COPILOT_GITHUB_TOKEN` env var (NOT GITHUB_TOKEN or GH_TOKEN)
- Must be fine-grained PAT with copilot scope
- Classic PAT (ghp_...) works for git but NOT for Copilot auth
- Tokens stored in `~/.env_tokens`

### Fallback: Claude sub-agents (if Copilot network fails)
```bash
claude -p "$(cat objectives/{domain}.md)" \
  --allowedTools "Read,Edit,Write,Bash,Grep,Glob" \
  --max-turns 45 --output-format json \
  --cwd /home/user/hearthfield
```

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
3. Dispatch → `source ~/.env_tokens && copilot -p "$(cat objectives/{domain}.md)" --model claude-opus-4.6 --yolo --add-dir /home/user/hearthfield`
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
- Waves 5-7: Domain implementation (crafting, economy, fishing, npcs, mining, ui, animals, world, calendar, player, farming, save, data, input) ✓
- Visual Audit: 5-worker parallel audit of all sprite/animation/z-order systems ✓
- Visual Fix Pass 1: BottomCenter anchor, tool direction, water anim, grass variation, bed/stove y-sort, NPC face-player, portraits, forageables, floor tiles ✓

## Remaining Visual Issues (from status/workers/visual_audit_report.md)
- C1: Fish sprite_index OOB (indices 48-52 for 48-frame atlas) — no render consumer yet
- C3: 8/10 animal types have no dedicated sprites (colored rectangles or tinted humans)
- H4: No animal animation system
- H6: Emote atlas indices are guesses
- M2: Path/fence autotile bitmask assumption unverified
- M4: Crop growth has no animation (instant jump between stages)
- M6: No water edge transitions
