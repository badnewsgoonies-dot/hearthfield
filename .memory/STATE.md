# STATE — Hearthfield

**Updated:** 2026-03-10
**HEAD:** b0e301a (fix: move Nora NPC away from farm entrance)
**Branch:** master
**Working tree:** dirty (src/player/spawn.rs — local soil/terrain visual pass)

## Phase

- Macro phase: `finish breadth` (post-Wave-10, late-stage polish + verification)
- Wave phase: `Harden`
- Tier: `M` (multiple interacting surfaces)

## P0 Debt (blocks shipping)

- Building collision not verified end-to-end
- Tutorial flow / first-week guidance not fully closed
- Atlas pre-loading incomplete (affects browser/WASM experience)
- Shop auto-entry requires verification

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

## Retired Debts (previously P0, now fixed)

- ~~Starter items missing hoe~~ — fixed (commit 13594cb)
- ~~Player uses npc_farmer.png placeholder~~ — fixed (character_spritesheet.png)
- ~~Tool animation walk sprite bob only~~ — fixed (held sprites + impact feedback)
- ~~wood_bridge.png row debt~~ — fixed (commit 5195b5f)
- ~~house_roof.png empty rows~~ — fixed (commit f46f372)

## Gate Status

- cargo check: PASS (last verified locally)
- cargo test: 109 base-game headless PASS (MANIFEST.md); 352 repo-wide including DLC
- cargo clippy: 0 warnings (last verified locally)
- WASM build: infrastructure exists (build_wasm.sh), not recently verified

## Critical Path Uncertainties

- [Inferred] All core gameplay loops (fishing, mining, crafting, social, economy) are functional but not verified end-to-end since feature additions
- [Assumed] Season validation on planting works correctly
- [Assumed] Save/load preserves all new map state (Coral Island, Deep Forest, town houses, sailing)
- [Assumed] WASM build still works after sailing + deep forest additions

## Current Runtime Surfaces

| Surface | Status |
|---|---|
| Farm: till → plant → water → grow → harvest | [Inferred] functional |
| Town: walk → enter shops → buy/sell | [Inferred] functional |
| Beach → Coral Island: sailing loop | [Observed] wired and reachable |
| Forest → Deep Forest | [Observed] wired and reachable |
| Mine: enter → descend → mine → exit | [Inferred] functional |
| Fishing: cast → wait → catch | [Inferred] functional |
| Save/Load roundtrip | [Assumed] functional |
| Tool tutorial: Mayor Rex intro | [Observed] wired |
