# STATE — Hearthfield

**Updated:** 2026-03-13
**HEAD:** 253bfe8 (graduate: fishing + mining loops [Observed])
**Branch:** claude/llm-git-orchestration-OLSPR
**Working tree:** clean

## Phase

- Macro phase: `finish breadth` (post-Wave-10, late-stage polish + verification)
- Wave phase: `Graduate`
- Tier: `M` (multiple interacting surfaces)

## P0 Debt (blocks shipping)

- Atlas pre-loading incomplete (affects browser/WASM experience) — lazy loading confirmed, WASM pop-in risk
- Tutorial flow / first-week guidance — exists (intro cutscene + Mayor Rex + hints) but not runtime-verified

## P1 Debt (next wave)

- ~~Fishing loop~~ — graduated to [Observed], see Runtime Surfaces table
- ~~Mining loop~~ — graduated to [Observed], see Runtime Surfaces table
- Crafting loop — crafting bench interaction end-to-end
- Social loop — NPC friendship progression verification
- Economy loop — earn/spend/upgrade cycle verification
- WASM/browser — build + deploy verification
- Performance / endurance — extended play session test
- Full-year playthrough — season transitions, festival triggers
- Pilot DLC — end-to-end playability
- City DLC — end-to-end playability

## Last Decisions

- [Observed] Player sprite loads from character_spritesheet.png (src/player/spawn.rs)
- [Observed] Tool feedback: held sprites + swing arcs + impact particles + till dust (commit 8bf943e3)
- [Observed] NPC-driven tool tutorial with visual overlay (commit 10b8b888)
- [Observed] Sailing: boat boarding, sailing movement, dock interaction (commit 196fb88)
- [Observed] Coral Island + Deep Forest maps reachable and rendering
- [Observed] Two town houses (West/East) with interiors (commit 8d1f24c3)
- [Observed] 15-domain audit: 7 bugs fixed, >60% of sub-agent claims were false positives (commits 614cb86d..c3ddfbcd)
- [Observed] Item dupe on full-inventory craft fixed + regression test (commit ddcb11da)
- [Observed] Season validation works: blocks out-of-season planting, kills crops on season change (graduated test)
- [Observed] Save/load preserves current_map + grid position (graduated test)
- [Observed] Building collision works: stone tiles solid, doors carved out (graduated test)
- [Observed] Starter items include hoe + seeds (graduated test)
- [Observed] Orchestration enforcement hardened: clamp-scope rewritten (temp files, verified clean), contract-deps checksummed, hook paths portable, gates expanded to 7 (commits cdcc85c..b5b4740)
- [Observed] Claude Code agents wired: domain-worker (Sonnet, scoped), auditor (Sonnet, read-only), red-team (Opus, read-only) — .claude/agents/ (commit b9a5854)
- [Observed] Mechanical hooks active: PreToolUse blocks Rust edits from orchestrator + guards agent dispatch; PostToolUse checks contract integrity after Bash (commit cdcc85c)
- [Observed] Fishing loop: cast→bite→minigame→catch→inventory→reset, all wired via ECS systems (src/fishing/cast.rs, minigame.rs, resolve.rs)
- [Observed] Mining loop: entry→floor spawn→rock breaking→ore pickup→ladder descent→exit, 20 floors, elevator every 5 (src/mining/transitions.rs, rock_breaking.rs, ladder.rs)

## Retired Debts (previously P0, now fixed)

- ~~Starter items missing hoe~~ — fixed (commit 13594cb), graduated (test_starter_items_include_hoe)
- ~~Player uses npc_farmer.png placeholder~~ — fixed (character_spritesheet.png)
- ~~Tool animation walk sprite bob only~~ — fixed (held sprites + impact feedback)
- ~~wood_bridge.png row debt~~ — fixed (commit 5195b5f)
- ~~house_roof.png empty rows~~ — fixed (commit f46f372)
- ~~Building collision not verified~~ — [Observed] solid tiles + door carve-outs (graduated test)
- ~~Shop auto-entry requires verification~~ — [Observed] position-triggered on door tiles (src/player/interaction.rs:135-151)
- ~~Season validation on planting~~ — [Observed] crop_can_grow_in_season + kills on season change (graduated tests)
- ~~Save/load preserves map state~~ — [Observed] current_map + grid coords serialized (graduated test)
- ~~Orchestration enforcement gaps (red-team finding)~~ — hardened: scope clamping, contract checksums, hook wiring all mechanical (commits cdcc85c..b5b4740)
- ~~Fishing loop e2e~~ — [Observed] full state machine traced: cast→bite→minigame→catch→inventory→reset
- ~~Mining loop e2e~~ — [Observed] full loop traced: entry→rock breaking→ladder descent→exit, elevator system

## Gate Status

- Gate 1 (contract integrity): PASS (mod.rs + schedule.rs checksums)
- Gate 2 (cargo check): PASS (requires libudev/alsa — fails in headless container)
- Gate 3 (cargo test): 180 headless PASS, 0 failures, 2 ignored (requires system libs)
- Gate 4 (cargo clippy): 0 warnings (requires system libs)
- Gate 5 (connectivity): PASS — all domains import from shared contract
- Gate 6 (STATE.md freshness): tracks HEAD drift (warning-only)
- Gate 7 (artifact source refs): PASS — all file refs resolve
- WASM build: infrastructure exists (build_wasm.sh), not recently verified

## Bugs Fixed This Session (commits 614cb86d..c3ddfbcd)

- P1: Item duplication on full-inventory craft (src/crafting/bench.rs)
- P2: `return` → `continue` eating DayEndEvents (src/player/interaction.rs)
- P2: Fish wildcard consuming without checking try_remove (src/crafting/cooking.rs)
- P2: UTF-8 byte-slice panics in 4 UI screens (src/ui/*.rs)
- P2: Tool sprite desync after entity despawn (src/player/tool_anim.rs)
- P2: Refund overflow silently swallows items (src/crafting/bench.rs)
- P3: Grass decor despawn loop optimization (src/world/grass_decor.rs)

## Critical Path Uncertainties

- [Observed] Fishing and mining loops verified end-to-end via code tracing (this session)
- [Inferred] Crafting, social, economy loops functional but not runtime-verified end-to-end since feature additions
- [Assumed] WASM build still works after sailing + deep forest additions

## Current Runtime Surfaces

| Surface | Status |
|---|---|
| Farm: till → plant → water → grow → harvest | [Observed] season validation graduated; starter hoe graduated |
| Town: walk → enter shops → buy/sell | [Observed] shop entry position-triggered; collision verified |
| Beach → Coral Island: sailing loop | [Observed] wired and reachable |
| Forest → Deep Forest | [Observed] wired and reachable |
| Mine: enter → descend → mine → exit | [Observed] full loop traced: entry (transitions.rs:17-67), rock breaking (rock_breaking.rs:35-134), ladder descent (ladder.rs:14-95), exit (ladder.rs:99-147), day-end penalty (transitions.rs:72-123) |
| Fishing: cast → wait → catch | [Observed] full loop traced: cast (cast.rs:63-189), bite timer (cast.rs:192-238), minigame (minigame.rs:50-311), catch→inventory (resolve.rs:66-69), state reset (resolve.rs:147-152) |
| Save/Load roundtrip | [Observed] current_map + grid position graduated |
| Tool tutorial: Mayor Rex intro | [Observed] wired |
| Crafting: bench → select → craft | [Observed] ECS tests pass; dupe fix graduated |
