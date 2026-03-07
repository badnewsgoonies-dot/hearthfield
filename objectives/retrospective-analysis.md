# Retrospective Analysis: How Hearthfield Was Built & How It Should Have Been

## Mission
You are the ORCHESTRATOR for a full retrospective analysis of the Hearthfield codebase.

You will spawn workers to investigate every aspect of how this 46K LOC Rust/Bevy game was built, then synthesize findings into a comprehensive report covering:
1. How it WAS built (actual history, patterns, mistakes)
2. How it SHOULD have been built (optimal approach with hindsight)
3. Specific recommendations for the next project

## Phase 1: History Investigation (spawn workers in parallel)

### Worker 1: Git Archaeology
- Run: git log --oneline --reverse (full history, 303 commits)
- Run: git log --stat --reverse (file change patterns)
- Run: git shortlog -sn (contributor patterns)
- Identify build phases: scaffolding, domain implementation, integration, polish, bugfix
- Map the timeline: what was built when, what order, what got reworked
- Find: how many times was shared/mod.rs modified? By whom? When?
- Find: which domains were built first? Which caused the most rework?
- Find: what was the commit velocity? Any stalls or pivots?
- Write findings to: status/retrospective/git-archaeology.md

### Worker 2: Architecture Analysis
- Read: src/main.rs (plugin wiring order)
- Read: src/shared/mod.rs (the type contract — 2246 lines)
- Read: each domain's mod.rs (15 domains)
- Assess: Is the plugin-per-domain architecture the right call for this game?
- Assess: Is the shared type contract too large? Should it have been split?
- Assess: Are there circular dependencies or tight coupling between domains?
- Assess: How well do domains respect their boundaries?
- Assess: Event-driven vs direct resource mutation — which pattern dominates?
- Write findings to: status/retrospective/architecture-analysis.md

### Worker 3: Code Quality Audit
- Run: cargo clippy -- -D warnings (capture all warnings)
- Run: find src -name '*.rs' -exec wc -l {} + | sort -rn | head 20 (largest files)
- Grep for: TODO, FIXME, HACK, XXX across the codebase
- Grep for: unwrap() usage (potential panics)
- Grep for: clone() overuse
- Grep for: #[allow(clippy:: patterns (suppressed warnings)
- Assess: test coverage (what does tests/headless.rs actually test?)
- Assess: error handling patterns (Result vs panic)
- Write findings to: status/retrospective/code-quality.md

### Worker 4: Game Design vs Implementation Gap
- Read: GAME_SPEC.md (the original spec)
- Compare spec features to actual implementation in each domain
- For each domain: what percentage of spec is implemented?
- Find: features that were specced but never built
- Find: features that were built but not in the spec
- Find: features that were built wrong relative to spec
- Write findings to: status/retrospective/spec-gap-analysis.md

### Worker 5: Performance & Scalability
- Grep for: Query patterns — are any systems querying too broadly?
- Grep for: .iter() vs .iter_mut() — unnecessary mutability?
- Assess: How many systems run per frame? Any that should be FixedUpdate?
- Assess: Map generation — procedural or precomputed? Memory impact?
- Assess: Entity count scaling — what happens with 100 NPCs, 500 crops, 200 mine entities?
- Assess: Are there any O(n²) patterns? (nested loops over entities)
- Write findings to: status/retrospective/performance.md

### Worker 6: Build Process & Tooling
- Read: CLAUDE.md (orchestrator instructions)
- Read: scripts/ directory (what automation exists?)
- Read: status/workers/ directory (all worker reports)
- Read: MANIFEST.md if it exists
- Assess: How effective was the orchestrator pattern?
- Assess: How many fix cycles did each domain need?
- Assess: What was the total token/cost to build this?
- Assess: Where did the most time/tokens get wasted?
- Write findings to: status/retrospective/build-process.md

## Phase 2: Research Optimal Approaches (spawn workers in parallel)

### Worker 7: Bevy Best Practices Research
- What's the community-recommended architecture for a game this size in Bevy 0.15?
- Should we have used States differently?
- Should we have used SystemSets more aggressively?
- Is plugin-per-domain the right granularity or should it be finer?
- What about using Bevy's built-in scheduling features we might have missed?
- Write findings to: status/retrospective/bevy-best-practices.md

### Worker 8: Alternative Architectures
- Compare: ECS-pure vs hybrid (our approach) vs traditional OOP
- Compare: Monolithic shared types vs trait-based contracts vs message passing
- Compare: Code-first vs data-driven (RON/TOML config files for game data)
- What would this game look like built with a different architecture?
- Write findings to: status/retrospective/alternative-architectures.md

## Phase 3: Synthesis (after all workers complete)
Combine all worker reports into a single executive summary:
- status/retrospective/EXECUTIVE_SUMMARY.md

Structure:
1. What went right (keep doing)
2. What went wrong (stop doing)
3. What we should have done differently (start doing)
4. Quantitative metrics (commits, rework %, test coverage, etc.)
5. Recommended architecture for "Hearthfield 2.0"
6. Estimated cost comparison (actual vs optimal)

## Constraints
- Do NOT modify any source code
- Write all output to status/retrospective/ directory
- Read actual source files — do not guess or assume
- Use git log extensively to understand the real history
- Be brutally honest — this is a learning exercise, not a PR review
