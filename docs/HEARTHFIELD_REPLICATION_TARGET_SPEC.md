# Hearthfield Quality-Floor Target Spec

Purpose: define the quality-floor target for an orchestration system whose goal is not a toy farming game, but Hearthfield-equivalent breadth, function, and visual coverage.

This spec is intentionally harsh.

The target floor is:

- Hearthfield-equivalent in breadth
- Hearthfield-equivalent in system style
- Hearthfield-equivalent in relative function

Non-negotiables:

- zero asset omission
- zero new bug introduction
- zero visual regression against baseline mappings

If the original has the correct sprite and the new build does not, it fails.

## 1. Target Scope

Primary target:

- the base Hearthfield game under `src/`, `assets/`, `tests/`, `web_template/`, and `build_wasm.sh`

Secondary target:

- DLC crates may be staged later, but they are not required for claiming base-game parity unless explicitly included in the replication wave plan

## 2. Structural Parity Requirements

At minimum, the replica must preserve:

- the same top-level domain/plugin lattice
- the same major runtime surfaces
- the same broad system families
- the same contract boundary style

### Required domain/plugin parity

The new build must include functional equivalents for:

- `calendar`
- `player`
- `farming`
- `animals`
- `world`
- `npcs`
- `economy`
- `crafting`
- `fishing`
- `mining`
- `ui`
- `save`
- `data`
- `input`
- `shared`

Additional modules such as `sailing` or submodules may be implemented differently, but no major runtime surface may disappear.

### Allowed improvements

The new build may improve:

- internal file structure
- internal module boundaries
- tests and manifests
- scheduling clarity
- preload and cache behavior
- instrumentation and auditability

Only if all parity gates still hold.

### Forbidden drift

The new build may not quietly change:

- runtime surface set
- map graph
- asset-role coverage
- content semantics
- baseline-correct visual mappings
- progression/economy/timing behavior

### Required runtime surface parity

The new build must preserve working equivalents for:

- farm day-one loop
- town exploration and shop entry
- house enter/exit and sleep
- farming loop
- fishing loop
- mining loop
- crafting loop
- social / NPC interaction loop
- economy / buy-sell-upgrade loop
- save/load roundtrip
- tutorial / first-week guidance
- world-map travel and reachable map graph
- sailing / boat travel where present in baseline

## 3. Zero Asset Omission

This is binary.

The build fails if:

- a baseline runtime-used asset is missing
- a baseline sprite role is replaced by a placeholder, blank, or omitted visual
- a reachable map or interior loses its concrete baseline visual content

### Required baseline artifacts before the experiment starts

The evaluation harness must first generate a baseline set:

1. `runtime_used_asset_manifest.csv`
   - every asset directly referenced by runtime loaders, mapping tables, or map registry loading
   - source file
   - loader kind
   - role hint

2. `asset_manifest.csv`
   - full repo asset inventory
   - asset class
   - broad role family

3. `visual_mapping_manifest.csv`
   - map/tile/object/NPC/UI element
   - source asset
   - atlas index or sprite path
   - expected role

4. `reachable_surface_manifest.csv`
   - every player-reachable runtime surface
   - map transitions
   - interaction points

5. `runtime_surface_manifest.csv`
   - current gameplay-loop inventory
   - entry conditions
   - primary files
   - preserve requirements

Without at least those five files, “zero asset omission” and “same gameplay surface” are not auditable.

Baseline integrity check:

```bash
python3 scripts/validate_reconstruction_baselines.py
```

## 4. Zero Bug Introduction

This is also binary.

The build fails if:

- any baseline graduated test now fails
- any baseline gate now fails
- any preserved runtime surface becomes broken
- any baseline invariant is lost

### Minimum bug baseline

Before implementation starts, snapshot:

- current `cargo check`
- current `cargo test --test headless`
- current `cargo test --test keybinding_duplicates`
- current `cargo clippy -- -D warnings`
- current known open debt from `.memory/STATE.md`

Rule:

- the new build may fix existing debt
- it may not introduce new regressions while doing so

## 5. Zero Visual Regression

This is not subjective.

A visual pass fails if the baseline surface uses a correct concrete sprite/mapping and the new build downgrades it.

Examples of failure:

- correct player sprite becomes placeholder
- correct crop stage art becomes colored block
- correct furniture/object mapping becomes wrong atlas cell
- correct dock/building/object sprite becomes generic fallback

### Allowed changes

Only these are allowed:

- exact match to baseline visual role
- demonstrably improved visual role with no lost coverage

If the baseline visual is correct and the new build chooses a different but worse or missing visual, it fails.

## 6. Throughput Does Not Excuse Loss

The new build may be:

- faster
- smaller
- cleaner
- better architected

But those do not offset failures in:

- asset parity
- bug parity
- visual parity

Performance wins only count after parity holds.

## 7. Acceptance Gates

The build is acceptable only if all of these pass:

1. Structural parity gate
   - all required domains and runtime surfaces exist

2. Asset omission gate
   - no baseline runtime-used asset category is missing

3. Visual mapping gate
   - no baseline-correct mapping is downgraded or omitted

4. Regression gate
   - all baseline graduated tests remain green

5. Surface gate
   - every baseline player-facing surface still works

6. Difference audit
   - every intentional deviation from baseline is explicitly logged and justified

## 8. Scoring Priority

Order of priority:

1. parity
2. regression avoidance
3. visual correctness
4. completeness
5. speed / cleanliness / elegance

That means:

- a fast elegant rebuild with missing assets fails
- a clean rewrite with one broken sprite role fails
- a beautiful refactor that drops a runtime surface fails

## 9. What Counts As Success

Success is not:

- “it feels close”
- “the tests mostly pass”
- “the architecture is nicer”
- “the workers were fast”

Success is:

- Hearthfield-equivalent breadth
- preserved or improved system-family coverage
- zero omitted runtime-used asset roles
- zero new regressions
- zero baseline-correct visual downgrades

## 10. Bottom Line

The target is not a Hearthfield-inspired game.

The target is Hearthfield, rebuilt or re-orchestrated to at least the same gameplay and visual quality floor, with:

- no missing asset coverage
- no new bugs
- no visual downgrade where the original is already correct

Anything below that is a failed attempt, even if it is technically impressive.
