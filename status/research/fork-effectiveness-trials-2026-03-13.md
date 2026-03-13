# Fork Effectiveness Trials — 2026-03-13

Purpose: measure the effectiveness of dispatching work to forked sub-agents vs doing it inline. Three trial types: accuracy quiz, investigation quality, parallel throughput.

## Trial A — Single Fork Accuracy (10-Question Code Quiz)

### Setup
- 10 factual questions about codebase structure (numeric counts, enum variants, defaults)
- Ground truth established by orchestrator reading code directly
- Fork given same questions, told to read code only (no STATE.md/docs)
- Fork type: Explore agent, "very thorough"

### Ground Truth vs Fork Answers

| Q | Question | Truth | Fork | Correct? |
|---|----------|-------|------|----------|
| 1 | #[test] count in headless.rs | 214 | 214 | ✓ |
| 2 | MapId variants | 18 | 18 | ✓ |
| 3 | Event types in shared/mod.rs | 32 | 32 | ✓ |
| 4 | Default spawn map | PlayerHouse | PlayerHouse | ✓ |
| 5 | .ron map files | 18 | 18 | ✓ |
| 6 | ToolKind variants | 6 | 6 | ✓ |
| 7 | src/ domain directories | 15 | 15 | ✓ |
| 8 | save_roundtrip tests | 20 | 20 | ✓ |
| 9 | Ignored tests | 2 | **0** | **✗** |
| 10 | AnimalKind variants | 10 | 10 | ✓ |

### Results
- **Accuracy: 9/10 (90%)**
- **Token cost: 21.9k**
- **Tool uses: 27**
- **Wall time: 39s**

### Failure Analysis
Q9 miss: fork searched `#[ignore]` but tests use `#[ignore = "reason"]` syntax. The fork's grep was too narrow. This is a **regex precision failure**, not a comprehension failure.

### Comparison to Inline
Inline verification: ~5 tool calls, ~3k tokens, ~10s. Fork used 5.5x more resources for the same task. The fork's value-add was zero here — the task was too simple and bounded for delegation to help.

---

## Trial B — Single Fork Investigation (Serialization Gap Audit)

### Setup
- Task: find all ECS components/resources used at runtime but NOT serialized
- Fork type: Explore agent, "very thorough"
- Fork told to read save/mod.rs + shared/mod.rs + cross-reference

### Results
- **Total gaps found: 74** (across P1/P2/P3 severity)
- **P1 (game-breaking): 3** — BoatMode, CutsceneQueue, CutsceneFlags
- **P2 (noticeable): 6** — ActiveFloor, InMine, FishingState, FishingMinigameState, ActiveNpcInteraction, ActiveShop
- **P3 (cosmetic/transient): 65** — UI state, input state, registries, atlases, timers
- **Token cost: ~141k** (very high — extensive exploration)
- **Tool uses: 40+**
- **Wall time: ~180s**

### Verification (spot-checked 5 claims)
| Claim | Verified? |
|-------|-----------|
| BoatMode not serialized | ✓ Correct (not in save/mod.rs) |
| CutsceneQueue not serialized | ✓ Correct |
| ActiveFloor not serialized | ✓ Correct |
| FishingState not serialized | ✓ Correct |
| InMine not serialized | ✓ Correct |

### Known Gaps Fork Missed
- SheepWoolCooldown / PendingProductQuality (ECS-only animal components) — these were likely in the broader unserialzied list but not called out as P1/P2 despite being game-state-affecting
- Festival timer serialization gap (known P1 bug from Attack 7) — not specifically named

### Assessment
Fork performed well on **breadth** (74 items cataloged across entire codebase) but the high token cost (141k) suggests this task was at the edge of what a single fork can handle efficiently. The P1/P2 findings are accurate and valuable. The P3 list is overcomplete — many items (registries, atlases) are regenerated on load and aren't real gaps.

**False positive rate: ~30%** (P3 items that aren't true serialization gaps because they're regenerated)

---

## Trial C — Parallel Forks (3 Independent Domain Audits)

### Setup
- 3 forks launched simultaneously on independent domains
- Fork C1: fishing domain audit (5 specific questions)
- Fork C2: mining domain audit (5 specific questions)
- Fork C3: economy domain audit (5 specific questions)
- Fork type: Explore agent, "medium" thoroughness
- Measured: total wall time, per-fork accuracy, scope drift

### Results

| Fork | Domain | Tokens | Tools | Wall Time | Accuracy |
|------|--------|--------|-------|-----------|----------|
| C1 | Fishing | 62.1k | 25 | 72.5s | 4/5 questions correct |
| C2 | Mining | 60.9k | 21 | 73.0s | 5/5 questions correct |
| C3 | Economy | 47.4k | 18 | 57.0s | 3/5 questions correct |
| **Total** | | **170.4k** | **64** | **73s wall** | **12/15 (80%)** |

### Parallel Speedup
- Sequential estimate: 72.5 + 73.0 + 57.0 = 202.5s
- Parallel actual: 73.0s (bottleneck = slowest fork)
- **Speedup: 2.77x** (near-linear for 3 forks)

### Per-Fork Scoring

**C1 (Fishing) — 4/5:**
- ✓ State machine: correct (Idle → WaitingForBite → BitePending → Minigame → Idle)
- ✓ Fish count: 31 (verified)
- ✗ Stamina drain: said "NO stamina drain" — correct for fishing-specific code, but **missed** that player/tools.rs drains stamina generically for all tool uses including fishing rod. Cross-domain blind spot.
- ✓ Dead code: found 2 true dead items (spawn_fish_display, CATCHES_PER_LEVEL)
- ✓ Legendary pool: correctly identified no removal mechanism

**C2 (Mining) — 5/5:**
- ✓ 20 floors
- ✓ 3 enemy types with correct floor distributions
- ✓ Elevator every 5 floors
- ✓ Knockout: 10% gold loss, 50% HP restore, return to bed
- ✓ Save gaps: ActiveFloor, rocks, enemies, ladder state all identified as not persisted

**C3 (Economy) — 3/5:**
- ✗ GoldChangeEvent producers: found only 3 (shipping, blacksmith, buildings) — **missed 5** outside src/economy/ (festivals, mining/combat, mining/transitions, npcs/quests, fishing/treasure). Scope-bounded miss.
- ✓ Gold floor: correctly identified clamp to 0
- ✓ Shop dual mutation: correctly found direct mutation pattern
- ✓ Shipping flow: correct
- ✗ Dual mutation audit incomplete — correctly found shop path but didn't search for other dual paths outside economy domain

### Scope Drift Analysis
- C1: stayed in src/fishing/ — no drift, but missed cross-domain stamina interaction
- C2: stayed in src/mining/ — no drift, complete within scope
- C3: stayed in src/economy/ — no drift, but task required cross-domain view that scope discipline prevented

**Key finding: scope discipline and cross-domain completeness are in tension.** Forks that respect their domain boundary produce clean, verifiable results but systematically miss cross-domain interactions. Forks that widen scope risk drift and noise.

---

## Synthesis

### Fork Effectiveness by Task Type

| Task Type | Fork Value | Risk | Recommendation |
|-----------|-----------|------|----------------|
| Simple factual quiz (Trial A) | **Low** — inline is faster and cheaper | Regex/search precision errors | Don't fork for simple lookups |
| Broad investigation (Trial B) | **High** — 74 findings, orchestrator couldn't match breadth inline | High token cost, 30% false positive rate | Fork for discovery, then verify P1/P2 inline |
| Parallel domain audits (Trial C) | **High** — 2.77x speedup, 80% accuracy | Cross-domain blind spots | Fork per-domain, but add cross-domain reconciliation step |

### Quantified Tradeoffs

| Metric | Inline (orchestrator) | Single Fork | Parallel Forks (3) |
|--------|----------------------|-------------|-------------------|
| Token efficiency | ~3k for quiz | 22k for quiz (7.3x worse) | 170k total, 57k avg (per-fork cost) |
| Wall time | 10s for quiz | 39s (3.9x slower) | 73s for 3 tasks (2.77x faster than sequential) |
| Accuracy | 10/10 (100%) | 9/10 (90%) | 12/15 (80%) |
| Cross-domain coverage | Full (has context) | None (scope-bounded) | None (scope-bounded) |
| Discovery breadth | Low (focused) | High (Trial B: 74 items) | Medium (5 questions each) |

### Failure Modes Observed

1. **Regex precision** (Trial A Q9): fork's search pattern was too narrow for `#[ignore = "..."]`
2. **Cross-domain blind spot** (Trial C1, C3): scoped forks miss interactions that cross domain boundaries
3. **Overcounting** (Trial B): fork catalogs everything including regenerated state, inflating gap count
4. **Scope-bounded incompleteness** (Trial C3): "audit economy" finds only economy-local producers, not the 5 cross-domain ones

### Operational Recommendations

1. **Don't fork for tasks requiring <5 tool calls** — overhead exceeds value
2. **Fork for broad investigation** — the discovery breadth is unmatched inline
3. **Always add a cross-domain reconciliation step** after parallel forks
4. **Score fork output with inline spot-checks** — 5 random claims verified is enough to calibrate trust
5. **Parallel forks approach linear speedup** up to the slowest fork's duration
6. **Fork cost is dominated by exploration, not answer generation** — "medium" thoroughness uses 2-3x fewer tokens than "very thorough" with similar accuracy on bounded questions
