# Codex Nesting Trials — Productive Audits, 2026-03-13

## Task
Audit `src/animals/` (8 files, 18 pub functions) for bugs and coverage gaps.
Same domain, same codebase state, three different topologies.

## Round 2A: Depth-2 Foreman (dynamic decomposition)

```
Root (foreman) ──┬── Worker: interaction.rs
                 ├── Worker: mod.rs
                 ├── Worker: feeding.rs
                 ├── Worker: movement.rs
                 ├── Worker: day_end.rs
                 ├── Worker: rendering.rs
                 ├── Worker: products.rs
                 └── Worker: spawning.rs
```

| Metric | Value |
|--------|-------|
| Topology | Root reads dir → spawns 8 workers in parallel |
| Tokens (root) | 24,023 |
| Wall time | 2m 40s |
| Bugs found | 11 P2, 6 P3 = 17 total |
| All [Observed] | 14/17 (82%) |
| Root read any files? | No — only `rg --files` for dir listing |

**Key behavior:** Foreman listed files, spawned 1 agent per file, waited in rounds (6 concurrent, then 2 more), synthesized. It autonomously decided the decomposition — I never told it which files exist.

## Round 2B: Flat Single Agent (no nesting, same task)

```
Single agent ── reads all 8 files sequentially
```

| Metric | Value |
|--------|-------|
| Topology | One agent reads everything |
| Tokens (total) | 58,941 |
| Wall time | 3m 23s |
| Bugs found | 2 P1, 3 P2, 2 P3 = 7 total |
| All [Observed] | 7/7 (100%) |
| Cross-file analysis? | Yes — found quality loss across product→inventory boundary |

**Key behavior:** Sequential reads, but could cross-reference between files. Found the P1 quality-loss bug (products.rs emits ItemPickupEvent without quality → quality silently dropped) that the foreman's per-file workers missed.

## Round 3: Depth-3 Hierarchy (root → 2 foremen → workers)

```
Root ──┬── Foreman: animals/ ──┬── Worker per file (8)
       │                       └── ...
       └── Foreman: calendar/ ──┬── Worker: mod.rs
                                └── Worker: festivals.rs
```

| Metric | Value |
|--------|-------|
| Topology | Root → 2 parallel foremen → workers per file |
| Tokens (root) | 11,692 |
| Wall time | 4m 54s |
| Bugs found | ~13 P2, 3 P3 = ~16 total |
| Cross-domain? | Yes — identified shared-state drift + DayEndEvent ordering risk |
| New findings? | Spring 13 "Spring Dance" vs EggFestival name mismatch, egg spawn bounds |

**Key behavior:** Root spawned 2 foremen simultaneously, waited on both, produced cross-domain synthesis. The cross-domain report found structural overlap (split authority, event wiring drift) that per-domain audits couldn't surface alone.

## Comparison Matrix

| Metric | 2A Foreman (d=2) | 2B Flat (d=1) | 3 Hierarchy (d=3) |
|--------|-----------------|---------------|-------------------|
| Tokens (root reported) | 24k | 59k | 12k |
| Wall time | 2m40s | 3m23s | 4m54s |
| Total bugs | 17 | 7 | ~16 |
| P1 bugs | 0 | 2 | 0 |
| Cross-file bugs | 0 | 2 | 0 (cross-domain structural only) |
| Unique discoveries | pig indoor truffle, starter herd state sync | quality loss P1, same-frame purchase P1 | festival name mismatch, egg spawn bounds, DayEndEvent race |
| Decomposition by | Agent (dynamic) | N/A (single agent) | Agent (dynamic, 2 levels) |

## Where Depth Actually Helps

### 1. TOKEN EFFICIENCY: Depth wins
The foreman (24k) used 59% fewer tokens than flat (59k) because each worker only loaded one file into context, not all eight. Depth-3 was even cheaper at root level (12k) because the root only saw summaries.

### 2. WALL TIME: Flat wins (barely)
Flat (3m23s) beat foreman (2m40s is faster actually). Depth-3 (4m54s) is slowest due to 3 layers of transport overhead. But the foreman was faster than flat because 8 parallel workers complete faster than 8 sequential reads.

**Correction:** Foreman was actually faster (2m40s < 3m23s). Parallelism beat sequential reading.

### 3. BUG COUNT: Foreman wins on volume, flat wins on severity
Foreman found 17 bugs vs flat's 7. But flat found the 2 P1 bugs that foreman missed (quality loss, same-frame purchase bypass). The foreman's per-file workers couldn't see cross-file data flow.

### 4. CROSS-DOMAIN: Hierarchy is the only option
Only depth-3 produced cross-domain analysis (animals × calendar). It found the DayEndEvent ordering risk and shared-state drift pattern that neither per-domain approach could surface. This is the unique value of the tree topology.

### 5. DYNAMIC DECOMPOSITION: Real and reliable
The foreman read `src/animals/`, found 8 files, and spawned 8 workers without me telling it the file list. This is the "foreman decides how to decompose" capability that Claude Code cannot do (because sub-agents can't spawn).

## Where Depth Hurts

### 1. CROSS-FILE BLIND SPOTS
Per-file workers can't see that `products.rs` emits an event consumed by `spawning.rs`. The flat agent read both files and caught the quality-loss bug. The foreman's workers each saw only their assigned file.

### 2. BUG SEVERITY ACCURACY
Flat found 2 P1s. Foreman found 0 P1s (classified the same-frame purchase bug as P2). When an agent has full context, it can better judge impact.

### 3. WALL TIME AT DEPTH 3
Each nesting layer adds ~30s of transport overhead. Depth-3 took 4m54s vs foreman's 2m40s — the extra layer cost 2m14s for cross-domain synthesis that could have been done cheaper by running 2 flat agents and merging manually.

## The Thread / Road Forward

The optimal topology for audit tasks is **hybrid**:

1. **Foreman for decomposition** (depth-2): Let it discover files and spawn per-file workers. Saves tokens, enables parallelism.
2. **Cross-file pass after** (depth-1 or orchestrator): Run a second agent that reads the per-file audit reports + key interface files to find cross-boundary bugs.
3. **Cross-domain synthesis** (orchestrator level): The orchestrator (Claude Code or Codex root) merges domain audits and checks event wiring.

This gives you:
- Foreman's token efficiency (24k vs 59k)
- Foreman's parallel speed (2m40s vs 3m23s)
- Flat's cross-file accuracy (the P1 bugs)
- Hierarchy's cross-domain synthesis (the structural findings)

The playbook already prescribes this (Phase 3.1): "10-20 domains → Orchestrator → domain leads → workers." The depth-2 foreman IS the domain lead. But the playbook doesn't explicitly say "run a cross-file pass after the foreman" — that's the gap this trial revealed.

## Bugs to Track (deduped across all rounds)

### P1 (from flat agent, not found by foreman)
1. `products.rs:82` — animal product quality lost at ItemPickupEvent boundary [Observed]
2. `spawning.rs:498` — same-frame multi-purchase bypasses housing caps [Observed]

### P2 (union of all rounds)
3. `day_end.rs:297` — pig truffle ignores outdoor requirement [Observed]
4. `rendering.rs:71-75` — sync_animal_state_resource drops cooldown/starvation ECS state [Observed]
5. `mod.rs:251` — atlas animation indexing ignores row offsets [Observed]
6. `mod.rs:243` — frame_count==0 causes modulo-by-zero panic [Observed]
7. `feeding.rs:46` — duplicate hay removal with no guard [Observed]
8. `spawning.rs:410` — starter herd ECS entities not synced to AnimalState [Observed]
9. `calendar/mod.rs:522` — "Spring Dance" vs EggFestival name mismatch [Observed]
10. `festivals.rs:176` — egg spawn positions not validated against farm bounds [Observed]
11. `save/mod.rs:1218` — DayEndEvent ordering risk between animal mutation and autosave [Inferred]

### P3
12. `interaction.rs:56` — first-in-range, not closest animal [Observed]
13. `products.rs:56` — same issue in product collection [Observed]
14. `calendar/mod.rs:9` — stale sleep docs (B key vs F key) [Observed]
