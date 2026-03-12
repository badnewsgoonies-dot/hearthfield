# STATE — Hearthfield

**Updated:** 2026-03-12
**HEAD:** c3ddfbcd (test: add item duplication regression test for crafting)
**Branch:** master
**Working tree:** dirty (graduation tests in progress)

## Phase

- Macro phase: `finish breadth` (post-Wave-10, late-stage polish + verification)
- Wave phase: `Graduate`
- Tier: `M` (multiple interacting surfaces)

## P0 Debt (blocks shipping)

- Atlas pre-loading incomplete (affects browser/WASM experience) — lazy loading confirmed, WASM pop-in risk
- Tutorial flow / first-week guidance — exists (intro cutscene + Mayor Rex + hints) but not runtime-verified

## P1 Debt (next wave)

- Fishing loop — end-to-end verification needed
- Mining loop — end-to-end verification needed
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

## Gate Status

- cargo check: PASS
- cargo test: 180 headless PASS, 0 failures, 2 ignored
- cargo clippy: 0 warnings
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

- [Inferred] Core gameplay loops (fishing, mining, crafting, social, economy) are functional but not runtime-verified end-to-end since feature additions
- [Assumed] WASM build still works after sailing + deep forest additions

## Current Runtime Surfaces

| Surface | Status |
|---|---|
| Farm: till → plant → water → grow → harvest | [Observed] season validation graduated; starter hoe graduated |
| Town: walk → enter shops → buy/sell | [Observed] shop entry position-triggered; collision verified |
| Beach → Coral Island: sailing loop | [Observed] wired and reachable |
| Forest → Deep Forest | [Observed] wired and reachable |
| Mine: enter → descend → mine → exit | [Inferred] functional (rock breaking ECS test passes) |
| Fishing: cast → wait → catch | [Inferred] functional (state reset verified in code) |
| Save/Load roundtrip | [Observed] current_map + grid position graduated |
| Tool tutorial: Mayor Rex intro | [Observed] wired |
| Crafting: bench → select → craft | [Observed] ECS tests pass; dupe fix graduated |
