# STATE — Hearthfield

**Updated:** 2026-03-13T17:52Z
**HEAD:** c49d080 (graduate: incorporate Codex cross-vendor validation into Constitution v4)
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
- ~~Crafting loop~~ — graduated to [Observed], full loop traced
- Social loop — NPC friendship progression verification
- ~~Economy loop~~ — graduated to [Observed], earn/spend/persist traced
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
- [Observed] Fishing double stamina bug FIXED: removed duplicate StaminaDrainEvent from resolve.rs (commit fa54fa9) — worker dispatched, scope clamped, verified
- [Observed] Contract violation defense: two-layer (hook blocks dirty diff + SHA-256 checksum gate) — tested live, both caught tampering
- [Observed] Crafting loop: bench interaction→recipe check→ingredient consume→item add→full-inventory guard, all wired (bench.rs, tested in headless)
- [Observed] Economy loop: earn (shop sell + shipping bin) → spend (shop buy + blacksmith) → persist (PlayerState serde), gold.rs:16 central handler
- [Observed] Cross-domain GoldChangeEvent wiring: 8 producers, 5 consumers, negative gold clamped to 0 (corrected from 10 — Trial G verified)
- [Observed] Cold restart reconstruction: fresh agent correctly rebuilt full state from artifacts alone (tier, phase, debts, uncertainties, HEAD drift)
- [Observed] Artifact-only vs transcript comparison (Trial E): both agents found real bugs; artifact-only used 50% fewer tokens (19k vs 38k) with equivalent accuracy
- [Observed] Stale artifact causes misdirection (Trial F): 5-commit-stale STATE.md had 9/9 data points wrong, would cause redundant bug fixes and wasted verification
- [Observed] STATE.md claim-to-code accuracy (Trial G): 4/5 claims confirmed against source, 1 partial (producer count 8 not 10). 80% full-accuracy on numeric claims
- [Observed] Artifact transfers decision context (Trial H): fresh agent derived core doctrine ("demote evidence levels, verification-first") from artifacts alone, correctly prioritized P0 > P1
- [Observed] 2x2 AB trial (10-question quiz): A1(STATE+git)=10/10@33k, A2(STATE)=9/10@20k, B1(git)=10/10@24k, B2(code)=8/10@40k. STATE.md is an efficiency cache: same accuracy as git-only at 60% token cost and 46% wall time. Staleness caused A2's only miss (Q10). Code-only cannot answer session-state questions at all
- [Observed] Codex CLI multi-agent (Trial I): native spawn_agent+wait worked — orchestrator spawned 2 explorer sub-agents, both returned correct answers (8,625 tokens)
- [Observed] Parallel codex exec isolation (Trial J): 3 workers ran simultaneously with CODEX_HOME isolation, all correct, no session interference
- [Observed] Cross-LLM nesting (Trial K): Claude→Codex orchestrator→2 Codex sub-agents — full audit with source citations, 2-layer nesting, 16k tokens. Confirmed 8 GoldChangeEvent producers and shop quality gap
- [Observed] Cross-vendor state reconstruction (2x2 repeat with Codex/gpt-5.4): identical scores to Claude — A1=10/10, A2=9/10, B1=10/10, B2=8/10. Artifact reconstruction is vendor-independent. Codex B2 correctly refused unanswerable questions (more disciplined than Claude B2 which cheated)
- [Observed] BREAK FOUND: STATE.md covers only ~55% of critical paths (Attack 7). Blind spots: animals, festivals, weather, romance, chests, building upgrades, sleep/day-end, achievements, DLC
- [Observed] Attack 7 found 2 real bugs STATE.md missed: festival save/load soft-lock + animal state not surviving save/load
- [Observed] Poisoned STATE (Attack 1): agent caught 10/10 deliberate lies by verifying against code — artifact cannot override source
- [Observed] Conflicting artifacts (Attack 6): agent correctly identified true artifact at 100% vs false at 0% — code is the tiebreaker
- [Observed] FIX: Coverage Manifest added — explicit negative knowledge ("NOT covered" list). Agent usefulness rating went from 5/10 → 8/10. Agent found 3rd bug (achievement off-by-one) faster with manifest guiding search
- [Observed] BUG P2: Achievement "Community Pillar" says "all 11 NPCs" but unlock condition only requires 10 (achievements.rs:64,218)
- [Observed] 2x2 rerun (Sonnet 4.6): A1=9/10@18.5k, A2=8/10@18k, B1=9/10@49k, B2=2/10@32k. Staleness cost A1 and A2 each 1pt vs previous run. B2 now honest (2/10 vs inflated 8/10). No cell hit 10/10. Structural facts stable after 7 commits; numerics drift. (status/research/trial-2x2-rerun-2026-03-13.md)
- [Observed] Multi-hop fork trial: 4 agents in chain, each compressing checkpoint for next. 5/5 at every hop. 169→30→15→8 lines (21x compression) with zero fidelity loss on structural facts. Minimum viable checkpoint = 8 lines. (status/research/reconstruction-fidelity-and-forks-2026-03-13.md)
- [Observed] Reconstruction fidelity ratios: A1 mean 96.7%, A2 mean 86.7%, B1 mean 96.7% (at 2.6x cost), B2 honest 20%. Staleness decay = -1.4%/commit. Structural facts immune; numerics drift after 5 commits.

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
- ~~Fishing double stamina drain~~ — FIXED (commit fa54fa9), duplicate StaminaDrainEvent removed from resolve.rs
- ~~Crafting loop e2e~~ — [Observed] bench→recipe check→consume→add→full-inventory guard (audit confirmed)
- ~~Economy loop e2e~~ — [Observed] earn (shop+shipping) → spend (shop+blacksmith) → persist (serde) (audit confirmed)
- ~~Cooking path item dupe on full inventory~~ — FIXED (commit 7e4a25b), partial try_add removed before refund

## Gate Status

- Gate 1 (contract integrity): PASS (mod.rs + schedule.rs checksums)
- Gate 2 (cargo check): PASS (requires libudev/alsa — fails in headless container)
- Gate 3 (cargo test): 214 headless PASS, 0 failures, 2 ignored (requires system libs)
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
- [Observed] ItemPickupEvent→inventory cross-domain wiring confirmed (interaction.rs:482-509)
- [Inferred] Mining combat subsystem (no ECS tests for player attack, enemy AI, knockout)
- [Inferred] Mining floor transitions (no ECS tests for entry/exit/descent — code traced only)
- [Observed] Crafting and economy loops verified end-to-end via code tracing (this session)
- [Inferred] Social loop functional but not runtime-verified end-to-end since feature additions
- [Assumed] WASM build still works after sailing + deep forest additions
- ~~[Observed] BUG: Fishing double stamina drain~~ — FIXED in commit fa54fa9
- ~~[Observed] BUG P0: Cooking path item dupe on full inventory~~ — FIXED in commit 7e4a25b
- [Observed] DESIGN GAP: Perfect catch toast says "Quality upgraded!" but ItemPickupEvent has no quality field — upgrade is cosmetic only (minigame.rs:246-254, shared/mod.rs:871-875)
- [Observed] DESIGN: C key opens crafting without proximity check — player can craft from anywhere (bench.rs:274)
- [Assumed] Mining atlas tile indices (cave_tiles constants) match actual fungus_cave.png — see PRINCIPLE-world-tileset-silent-overflow
- [Observed] ECONOMY GAP: try_buy/try_sell in shop.rs are dead code (#[allow(dead_code)]) — runtime shop UI reimplements buy/sell inline in shop_screen.rs, so unit tests don't cover actual runtime path
- [Observed] ECONOMY GAP: Shop sell ignores ItemQuality::sell_multiplier() — only shipping bin (shipping.rs:124) respects quality pricing
- [Observed] ECONOMY GAP: Dual gold mutation — shop UI directly mutates player.gold while blacksmith/shipping use GoldChangeEvent → apply_gold_changes (gold.rs:16). EconomyStats only tracks event-based path

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
| Crafting: bench → select → craft | [Observed] full loop traced: bench interaction (bench.rs:75-114), recipe check (bench.rs:228-239), consume (bench.rs:257-270), add (bench.rs:174-198), dupe fix graduated |
| Economy: earn → spend → persist | [Observed] shop sell (shop_screen.rs:670), shipping (shipping.rs:124), buy (shop_screen.rs:624), gold handler (gold.rs:16), serde roundtrip |

## Coverage Manifest (what STATE.md knows vs doesn't)

**Covered domains (verified or traced):**
farming, fishing, mining, crafting, economy, player/tools, world/maps, save/load (partial), calendar (day/season), ui/shops, sailing

**NOT covered (no verification, no tracing, silence ≠ working):**
- animals (spawning, products, day-end, save/load fidelity)
- festivals (triggers, minigames, save/load during festival)
- weather (effects, crop impact, visual fx)
- romance/social (dating, marriage, friendship progression)
- chest storage (placement, transfer, persistence)
- building upgrades (house, coop, barn, silo progression)
- sleep/day-end flow (stamina collapse, bed transition)
- NPC dialogue/quests (quest completion, reward flow)
- achievements (unlock triggers, persistence)
- DLC content (pilot, police — separate crates)

**Coverage estimate: ~55% of critical paths. Agents MUST NOT assume uncovered domains work.**

## Bugs Found Via Coverage Gap Analysis (Attack 7)

- [Observed] BUG P1: Festival save/load soft-lock — saving mid-Egg Hunt loses timer (festivals.rs:29 skips serialization), but hunt refuses to restart if started=true (festivals.rs:132) and refuses to run without timer (festivals.rs:191). Player permanently stuck.
- [Observed] BUG P1: Animal state doesn't survive save/load — SheepWoolCooldown and PendingProductQuality are ECS-only components (animals/day_end.rs:32), not serialized in AnimalState (shared/mod.rs:575). Position, cooldown, quality all lost on reload.
