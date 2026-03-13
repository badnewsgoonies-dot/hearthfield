# Pyramid Trial Spec — Depth-3, Fan-Out 3

**Hypothesis:** Root-level token cost decreases as depth increases because each layer compresses before passing up. A true 3×3 pyramid (1 root → 3 foremen → 9 workers) should produce the cheapest root layer yet while shipping real bug fixes.

**Topology:**

```
Root (synthesizer + committer)
├── Foreman-Animals (reads dir, decomposes, spawns 3)
│   ├── Worker-A1: products.rs — quality propagation fix
│   ├── Worker-A2: spawning.rs — same-frame purchase guard
│   └── Worker-A3: day_end.rs — pig outdoor check + feeding.rs hay guard
├── Foreman-Calendar (reads dir, decomposes, spawns 3)
│   ├── Worker-C1: mod.rs — festival name → enum alignment
│   ├── Worker-C2: festivals.rs — egg spawn bounds validation
│   └── Worker-C3: write regression tests for C1 + C2 fixes
└── Foreman-Farming (reads dir, decomposes, spawns 3)
    ├── Worker-F1: sprinklers.rs — DayEndEvent double-tick audit + guard
    ├── Worker-F2: harvest.rs — edge cases (0-quantity, out-of-season)
    └── Worker-F3: crops.rs — season-change crop death completeness
```

**Agent count:** 1 root + 3 foremen + 9 workers = 13 agents total

## Bug Assignments (all from prior audit, [Observed])

### Domain: animals/ (Foreman-Animals)

**Worker-A1 — products.rs quality propagation (P1)**
- Bug: `products.rs:96` emits `ItemPickupEvent { item_id, quantity: 1 }` but `ItemPickupEvent` has no quality field → quality computed at lines 82-84 is silently dropped after the toast
- Root cause: `ItemPickupEvent` (shared/mod.rs:872) lacks `quality: ItemQuality`
- Fix: Since contract is frozen, CANNOT modify ItemPickupEvent. Instead: emit a NEW domain-local event `QualityItemPickupEvent` that wraps item_id + quality + quantity, and add a system that reads it and calls `inventory.try_add_quality()` if that API exists — OR document the gap with a `// TODO(phase-6): add quality to ItemPickupEvent` and add a test proving the quality IS computed but NOT persisted
- Scope: `src/animals/products.rs` only
- Deliverable: either a working domain-local quality path OR a documented gap with regression test

**Worker-A2 — spawning.rs same-frame purchase bypass (P1)**
- Bug: `spawning.rs:498` reads `ShopPurchaseEvent` in a loop. If two purchase events fire on the same frame, housing cap check uses stale `animal_query` count — second purchase slips through
- Fix: Count already-spawned animals THIS frame by tracking a local counter that increments after each successful spawn, not just querying existing entities
- Scope: `src/animals/spawning.rs` only
- Deliverable: frame-local spawn counter preventing cap bypass

**Worker-A3 — day_end.rs pig truffle + feeding.rs hay guard (P2)**
- Bug 1: `day_end.rs:298` — pig truffle production checks `happiness >= 50` but ignores outdoor requirement. Pigs should only find truffles when `is_outdoor == true`
- Bug 2: `feeding.rs:46` — hay removal event fires for every hay item removed but the loop feeds ALL animals per event. If player removes 3 hay, all animals get fed 3 times (idempotent on state but wasteful)
- Fix 1: Add `&& animal.is_outdoor` check at line 299
- Fix 2: Add early-return guard if animals are already fed this day (check a `fed_today` flag or use a `Local<bool>` system param)
- Scope: `src/animals/day_end.rs`, `src/animals/feeding.rs`

### Domain: calendar/ (Foreman-Calendar)

**Worker-C1 — mod.rs festival name alignment (P2)**
- Bug: `mod.rs:523` announces "Spring Dance" but `festivals.rs` uses `FestivalKind::EggFestival` for Spring 13. The toast says "Spring Dance" while the actual festival is an egg hunt
- Fix: Change `"Spring Dance"` to `"Egg Festival"` at line 523, or align all 4 entries with their FestivalKind variant names
- Scope: `src/calendar/mod.rs` only

**Worker-C2 — festivals.rs egg spawn bounds (P2)**
- Bug: `festivals.rs:180-181` spawns eggs at `rng.gen_range(-8..8)` × TILE_SIZE. This is arbitrary and doesn't check against actual farm bounds — eggs can spawn on water, buildings, or off-map
- Fix: Clamp spawn positions to valid farm tile coordinates. Read farm bounds from FarmState or use known farm dimensions. At minimum, reject positions that fall on non-walkable tiles
- Scope: `src/calendar/festivals.rs` only

**Worker-C3 — regression tests for calendar fixes**
- Write tests in `tests/headless.rs` that:
  1. Verify festival announcement string matches FestivalKind for all 4 seasons
  2. Verify egg spawn positions are within valid bounds (if bounds are queryable)
- Scope: `tests/headless.rs` only

### Domain: farming/ (Foreman-Farming)

**Worker-F1 — sprinklers.rs DayEndEvent audit (P2)**
- Risk: `auto_water_sprinklers` listens for `DayEndEvent`. If `DayEndEvent` fires more than once (which other audits flagged as a risk), sprinklers water twice → no gameplay bug but wasted compute
- Fix: Add a `Local<u32>` day tracker that skips if already watered today
- Scope: `src/farming/sprinklers.rs` only

**Worker-F2 — harvest.rs edge cases (P2)**
- Audit + fix: check for 0-quantity harvest, harvesting non-mature crops, harvesting in wrong season, null item_id from crop data
- Scope: `src/farming/harvest.rs` only

**Worker-F3 — crops.rs season-change completeness (P2)**
- Audit: verify that ALL crop types are killed on season change, not just the ones in a hardcoded list. Check if any crop survives across seasons when it shouldn't
- Scope: `src/farming/crops.rs` only

## Constraints

- **Contract frozen:** No edits to `src/shared/mod.rs`. If a fix requires contract changes, document the gap and write a test proving the bug exists
- **Clamp after every worker:** `bash scripts/clamp-scope.sh src/{domain}/` (and `tests/` for C3)
- **Workers must NOT run cargo fmt** on files outside their scope
- **Each worker reports:** files modified, bug fixed (yes/no/partial), lines changed, assumptions made

## Measurements

Track at each layer:
- Token count (from Codex response metadata)
- Wall time
- Files read vs files modified
- Whether the worker autonomously found the bug location or needed the spec to point it out

## Success Criteria

1. Root layer tokens < 12k (beating depth-3 narrow trial)
2. All 9 workers produce diffs
3. At least 6/9 fixes are valid (pass gates after clamping)
4. Total wall time < 8 minutes
5. Each successive depth layer is cheaper at its root than the layer below
