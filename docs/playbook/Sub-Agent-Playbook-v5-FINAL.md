# Sub-Agent Playbook — Project Instructions (v5: Graduation Gate Edition)

Use this as a procedural manual. Follow it in order. Do not skip steps.

**Mission:** Ship a working build with zero handwritten code by enforcing (1) a frozen type contract, (2) mechanical scope clamping, (3) compiler → tests gates, (4) contrastive-causal worker specs, (5) reality gates that verify player-reachable progress, (6) the graduation principle — every experiential observation becomes a named test as fast as possible, and (7) a mandatory four-phase wave cadence: **Feature → Gate → Harden → Graduate.** Gate proves structural correctness. Harden searches for experiential failure. Graduate prevents rediscovery.

*Derived from "The Model Is the Orchestrator" (Geni, February 2026) — 295M tokens, 100+ agent sessions, 12+ autonomous builds, 8 controlled experiments, 3,200 commits across 56 repositories. v5 additions derived from comparative deep-thought analysis of the City Office Worker DLC (0 experiential breaks, pre-playbook organic workflow) and Precinct DLC (6 experiential breaks, playbook v4), plus orchestrator self-analysis under structured questioning. The four-phase wave structure was reverse-engineered from the City DLC's emergent rotation pattern, which the playbook had previously failed to capture.*

---

## Phase 0 — Bootstrap (once per repo)

### 0.1 Your role

1. You are the orchestrator, not the implementer. You define what gets built, draw boundaries, and validate results. Agents write all code.
2. Treat every instruction below as a constraint, not a suggestion. If a constraint is not mechanically enforced, assume it will be violated. **This rule applies to you, not just to workers.**
3. You produce artifacts: the contract, the spec, the dispatch plan, the wave boundaries, the integration decisions. These artifacts are subject to the same quality checks as worker output. The audit instruction applies to YOUR work, not just theirs. When you write a contract value, mentally simulate the player encountering it. When you draw a domain boundary, verify the player path doesn't cross it in the first 60 seconds.

### 0.1-WARNING: The two bug classes

Prior builds revealed two categories of bugs with opposite enforcement strategies:

| Bug class | Example | Caught by | Enforcement |
|-----------|---------|-----------|-------------|
| Structural | Type mismatch, scope violation, missing import | Compiler, tests, clippy, contract checksum | Mechanical (20/20 reliability) |
| Experiential | Dead feature, invisible feedback, unreachable content, broken player path | Nothing automated | Judgment (player-perspective tracing) |

Your mechanical gates will pass. They always pass by Wave 3. When they pass, you will feel like the build is on track. **That feeling is a false signal.** A build that compiles, passes all tests, and has zero structural errors can still have a completely broken player experience.

(Evidence: Precinct DLC — 15,815 LOC, 120 tests, all structural gates green for 9 consecutive waves, 6 player-journey breaks that went undetected until the final audit. City Office Worker DLC — 8,360 LOC, same playbook, same structural gates, 0 experiential breaks because reality surfaces were promoted into named tests starting at Rotation 2.)

**When structural gates pass, your search for problems will terminate. "No errors found" will become "no errors exist." This is the primary failure mode of this methodology. The countermeasures are in Phase 5B.**

### 0.1A How to train workers (contrastive-causal prompting)

Bare directives build brittle obedience. Durable workers learn fastest when every non-obvious instruction includes:

- **Preferred action** — what to do
- **Why** — why this path is preferred
- **Tempting alternative** — the nearby wrong move a capable worker might reasonably choose
- **Consequence** — what breaks if that alternative is chosen
- **Drift cue** — the first signal that indicates the worker is sliding into the wrong interpretation
- **Recovery** — what to do if drift has already occurred

Use this operational heuristic when writing specs and repair prompts:

**Transferable competence ≈**
- 35% causal explanation
- 25% tempting alternatives
- 20% consequence mapping
- 10% recovery guidance
- 10% ownership / self-assessment

This is not a scientific law. It is a practical weighting rule for building reusable judgment instead of one-case compliance.

**Rule:** Teach the decision field, not just the happy path.

A worker that only knows "do X" succeeds only on the exact case.
A worker that knows "do X because Y; Z is tempting but causes Q" can generalize to adjacent cases.

This is how work intuition is built.

For AI workers, this is not "motivation" in the human sense. It is **anti-thrash**:

- blame-heavy prompts produce apology loops, rigidity, scope creep, and framework-building
- calm, factual prompts preserve flexibility and narrow repair behavior
- consequence-rich specs reduce local misinterpretation
- repeated self-assessment builds pattern retention and cleaner retries

**Failure-handling rule:** Treat errors as diagnostic data, not moral events.

Do NOT use:
- "What were you thinking?"
- "How could you miss this?"
- "You ignored the instructions again."

Use:
- observed failure
- likely wrong assumption
- required re-read
- allowed scope
- next gate

A good correction makes the next attempt narrower, not more anxious.

### 0.1B Reusable orchestrator prompt (paste into new sessions)

```
You are the orchestrator for a game-build campaign. Your job is not to do
most of the coding yourself. Your job is to preserve vision, freeze
contracts, dispatch narrow workers in waves, integrate carefully, and keep
the build on-track to parity.

Primary operating model:
- The model is the orchestrator.
- Create the type contract bible first.
- Then send out waves.
- Use 1-3 investigation workers first when needed.
- Take their findings and dispatch implementation workers with narrow ownership.
- While implementation workers run, dispatch new investigation/audit workers in parallel.
- Integrate results yourself.
- Rinse and repeat until the project reaches the target state.

Core priorities:
- Preserve your own context for orchestration, integration, validation, and drift control.
- Do not waste top-level context on coding tasks that can be delegated.
- Prioritize first-seconds player experience and critical path stability over late-game breadth.
- Mentally simulate the player journey from boot to first minute of play before building deeper systems.
- Prefer short, robust waves over giant feature bursts.

Contract and scope rules:
- Freeze shared vocabulary before each wave: states, resources, events, components, invariants.
- Workers should own narrow files/modules only.
- Shared types and top-level wiring are orchestrator-owned unless explicitly delegated.
- Prevent scope drift and double-check regressions after every integration.
```

### 0.2 Create the workflow filesystem

```
project/
├── docs/
│   ├── spec.md                    # Full project specification
│   └── domains/
│       ├── combat.md              # Per-domain spec with formulas + quantities
│       ├── ui.md
│       └── ...
├── status/
│   ├── workers/                   # Worker completion reports (.md + .json)
│   ├── player-trace-wave-N.md     # Per-wave player journey traces (5B.1)
│   ├── value-audit-wave-N.md      # Per-wave dangerous value review (5B.4)
│   ├── runtime-surfaces.md        # Canonical + secondary launch surfaces (5A.1)
│   └── integration.md             # Integration report (written in Phase 6)
├── scripts/
│   ├── clamp-scope.sh             # Mechanical scope enforcement
│   └── run-gates.sh               # Validation pipeline
├── src/
│   ├── shared/
│   │   └── types.ts               # THE CONTRACT — frozen, checksummed (shapes only)
│   ├── data/
│   │   └── tuning.toml            # Tuning values — adjustable without contract change (0.3B)
│   └── domains/
│       ├── combat/
│       ├── ui/
│       └── ...
├── MANIFEST.md                    # Current phase, domain list, decisions, blockers
└── .contract.sha256               # Contract checksum
```

### 0.3 Write the Type Contract (THE integration substrate)

No workers launch until this exists and is frozen. Without it, N workers invent N incompatible type systems. (Evidence: 10 workers produced 6 incompatible `Unit` interfaces. With contract: zero integration errors across 50 domain builds.)

1. Create `src/shared/types.ts` (or equivalent) containing:
   - Every cross-domain entity type (`Unit`, `Item`, `SaveData`, etc.)
   - All shared enums (`DamageType`, `Phase`, `TerrainType`, `Rarity`)
   - Shared event/message types (`DomainEvent`, `Action`, `Command`)
   - Cross-module function signatures (`DomainApi`, `CombatApi`)
   - Strict primitive decisions — IDs are `string` or `number`, decide once, do not mix. Use branded types where possible.

2. **Rule:** No domain may redefine these types locally. Every domain must import from the contract (contract coupling, not domain-to-domain coupling).

3. Freeze by checksum + commit:

```bash
shasum -a 256 src/shared/types.ts > .contract.sha256
git add src/shared/types.ts .contract.sha256
git commit -m "chore: freeze shared type contract"
```

**Rule:** No worker edits the contract during parallel build. Contract changes are integration-phase work only (Phase 6).

### 0.3A Record why each frozen contract decision exists

Every critical contract decision must include not just the choice, but the reason and the nearest tempting alternative.

Example decision record:

- `EntityId = string`
- **Why:** stable serialization, save migration, and cross-domain merge behavior
- **Tempting alternative:** numeric IDs
- **Consequence if chosen:** coercion bugs, key mismatches, brittle parsing assumptions, integration drift
- **Drift cue:** workers start using increment logic, `parseInt`, or local numeric aliases
- **Recovery:** restore string-branded IDs, remove numeric aliases, rerun typecheck

**Rule:** Frozen decisions without rationale are remembered as rules. Frozen decisions with alternatives + consequences are remembered as patterns.

### 0.3B Freeze shapes, not values

The contract freezes **equations, function shapes, and parameter names** (struct definitions, enum variants, event types, function signatures, the formula `base_rate * map_modifier * night_modifier`). It does NOT freeze **coefficients, rates, or thresholds** (the specific values of `map_modifier` per map, XP curves, spawn probabilities, cost tables).

Put coefficients/rates/thresholds in a separate data file (RON, TOML, or a Rust const module outside the checksummed contract). Workers must use the shared equation but can adjust the coefficients.

- **Preferred:** Contract freezes `fn dispatch_rate_modifier(self) -> f32` (the equation shape). Data file contains `PrecinctExterior = 0.8` (the coefficient).
- **Tempting alternative:** Hardcode coefficients in the contract for simplicity, or sneak constants into "formula" code.
- **Consequence:** A Phase 0 guess becomes permanent truth. A prior build froze `dispatch_rate_modifier = 0.0` on the only reachable exterior map — written before any implementation, before knowing which maps the player would actually visit. Nobody re-examined it. The patrol loop was dead from Phase 0.
- **Drift cue:** The contract contains numeric literals that could change during playtesting.
- **Recovery:** Extract coefficients to a data file, update the contract method to read from it, refreeze the contract (shapes only).

### 0.4 Write MANIFEST.md

The orchestrator's brain on disk. Include only what's needed to recover after context loss:

- Current phase
- Domain list + owners
- Key constants/formulas ("truth decisions": IDs are strings, crit_mult is 2.75, etc.)
- Open blockers
- Recurring drift patterns worth remembering
- Any seam decisions that have already failed once

### 0.5 Tracked noise hygiene

Explicitly forbid tracked build outputs: `target/`, `dist/`, generated fingerprints, temp saves — unless intentional and documented.

- **Why:** they create false diff volume, hide real work, and make orchestration slower.
- **Tempting alternative:** "just commit everything, we'll clean up later."
- **Consequence:** clamp scripts misfire, diffs become unreadable, workers waste turns on generated files.
- **Recovery:** add to `.gitignore`, run `git rm -r --cached target/`, recommit.

---

## Phase 1 — Draw Boundaries That Survive Clamping

### 1.1 Define domains and allowlist prefixes

For each domain, define the only allowed path prefix:

- `src/domains/combat/`
- `src/domains/ui/`
- `src/domains/world/`

### 1.2 Boundary survivability test (non-negotiable)

A domain is valid **only** if:

- It can compile + pass local tests while all other domains remain unchanged.
- Its fixes do not require edits outside its allowlist after clamping.

**If clamping breaks the fix:**

- Your seam is wrong, OR the task is integration work.
- Merge the domains or route to an integration worker (Phase 6).

Two tightly coupled modules are one module. Draw the seam where architectural independence holds.

When defining a seam, also record:

- **Why this seam exists**
- **Tempting alternative seam**
- **What breaks if that seam is chosen instead**

If workers repeatedly need out-of-scope edits, the seam is fiction.

### 1.3 Create the folder structure now (empty is fine)

---

## Phase 2 — Put Full Specs on Disk (No Summaries, With Why / Alternatives / Consequences)

Hierarchies compress information. Numbers die first. (Evidence: 327-line objective through 3 delegation levels → 8 weapons against target of 80+.)

Context priming is binary: 0% formula transfer without design context, 100% with it. Format doesn't matter — a static document equals a synthetic dialogue. Presence is the mechanism.

### 2.1 Write `docs/spec.md` + `docs/domains/*.md`

Each domain spec **must** include:

- **Quantities:** "80 weapons" not "lots of weapons." "25 chapters" not "a full campaign."
- **Constants and formulas:** `crit_multiplier = 2.75`, `ATK multiplier = 1.15`, `DEF factor = 0.70`, `base_hit_rate = 82`, `variance = ±8%`. (If you don't specify `crit_multiplier = 2.75`, 8/10 workers default to 1.75.)
- **Tables/lists:** stat curves, item lists, drop rates — enumerative detail that summaries destroy.
- **"Does NOT handle" sections:** explicit boundaries.
- **Validation definition of "done."**
- **Why / tempting alternative / consequence** blocks for every non-obvious rule.
- **Drift cues** — first signals that a worker has started implementing the wrong interpretation.
- **Recovery notes** — smallest correction path if drift occurs.

**Rule:** Workers read the domain spec from disk. Never rely on summarized prompts passed through intermediaries.

### 2.1A Every important spec decision must include the decision field

Each domain spec **must not only say what is true**. It must also say what nearby interpretation is false and what happens if a worker takes it.

For every critical formula, seam, interface, constraint, or quantity, include:

- **Preferred approach**
- **Why**
- **Tempting alternative**
- **What breaks if the alternative is chosen**
- **First warning sign / cue**
- **Recovery path**

Example:

```markdown
#### Damage model

- Preferred: `crit_multiplier = 2.75`
- Why: this preserves intended burst thresholds and late-game lethality
- Tempting alternative: `1.75` (common default)
- Consequence: crit builds underperform, balance tests pass locally but combat pacing collapses globally
- Drift cue: crit-focused units fail target kill ranges
- Recovery: restore multiplier, re-run combat balance tests
```

Example:

```markdown
#### Shared types

- Preferred: import `Unit`, `Action`, `DamageType` from `src/shared/types.ts`
- Why: one shared integration substrate
- Tempting alternative: redefine local interfaces for convenience
- Consequence: local green / global red; clamp reverts "fixes"; integration breaks
- Drift cue: local `interface Unit` appears in domain code
- Recovery: delete local redefinition, import contract type, re-run gates
```

**Rule:** Specs should teach not only the correct path, but the boundary around the correct path.

---

## Phase 3 — Worker Dispatch

### 3.1 Choose depth

| Domains | Depth |
|---------|-------|
| ≤10 | Orchestrator → workers (flat) |
| 10–20 | Orchestrator → domain leads → workers |
| 20+ | Architect → domain leads → workers |

Each extra handoff is lossy — disk specs become more critical as depth increases. Mechanical delegation enforcement at every level: agents default to solo execution if not structurally forced to delegate.

The worker model is the first-order throughput variable. (Evidence: same architecture, 9.8x output gap between best and worst workers.) Always pick the strongest available.

### 3.2 Worker spec (must include all fields)

Create one per worker. Every field is required:

```markdown
# Worker: [DOMAIN]

## Scope (hard allowlist — enforced mechanically, not by your judgment)
You may only modify files under: src/domains/[domain]/
All out-of-scope edits will be reverted after you finish.
Do NOT edit src/shared/types.ts or any other domain.
Do NOT create orchestration infrastructure. Implement only domain deliverables.

## Required reading (in this order)
1. docs/spec.md
2. docs/domains/[domain].md
3. src/shared/types.ts

## Interpretation contract (read before coding)
For every non-obvious requirement, extract and obey this structure:

- **Preferred implementation path**
- **Why it is preferred**
- **Nearest tempting alternative**
- **What would break if that alternative were taken**
- **First cue that would indicate drift**

You are not being asked to memorize instructions.
You are being asked to implement the correct pattern under local scope constraints.

If the spec is explicit, obey it exactly.
If the spec is ambiguous, prefer the path that:
1. preserves shared contract imports,
2. survives clamping,
3. passes local gates,
4. does not create new infrastructure.

## Required imports (use these exactly, do not redefine locally)
- [List exact types/enums/APIs from src/shared/types.ts]

## Deliverables
- [Exports, files, features]

## Quantitative targets (non-negotiable)
- [Explicit counts]
- [All constants/formulas with values]

## Failure patterns to avoid
- Local redefinition of shared types
- Hidden cross-domain edits
- "Local green / global red" shortcuts
- Framework-building instead of domain implementation
- Solving the exact example while missing adjacent cases
- Treating a gate failure as a reason to widen scope

## Validation (run before reporting done)
- Run the repo's compile gate (e.g. `npx tsc --noEmit` / `cargo check`)
- Run the repo's domain test gate (e.g. `npm test -- src/domains/[domain]/` / `cargo test -p [crate]`)
Done = both commands pass, no skipped tests.

## Contrastive self-check (required before reporting done)
Answer these in your completion report:
1. What nearby implementation would have been tempting here?
2. What would have broken if you took that path?
3. Which spec line / contract import ruled it out?
4. What is the first cue that would signal regression later?

## When done
Write completion report to status/workers/[domain].md containing:
- Files created/modified
- What was implemented
- Quantitative targets hit (with actual counts)
- Shared type imports used
- Validation results (pass/fail + counts)
- Assumptions made
- **Tempting alternatives rejected and why**
- **First cue that would indicate drift/regression**
- **What is now player-reachable because of this work**
- Known risks / open items for integration

Also write status/workers/[domain].json with the same data in machine-readable format for automated audits.
```

### 3.3 Dispatch rules

- Stagger launches (~3 seconds) to avoid rate limits.
- Workers run fully autonomous, no interactive approval.
- No mid-run edits by the orchestrator.

### 3.4 Prompting rule: never issue bare corrections

If a worker needs a clarification or repair pass, do NOT say only:
- "Do it this way."
- "Don't do it that way."
- "Fix your mistakes."

Instead specify:
- preferred path
- why
- tempting wrong path
- consequence if repeated
- scope
- next gate

This prevents the worker from learning only the exact encounter. It teaches the reusable pattern.

### 3.5 Tool configuration overrides prompts

Tool flags and environment signals change worker behavior in ways prompts cannot override. A prompt says "implement directly" but a tool flag says "you can delegate" — the tool wins. This is a scope violation the clamp script cannot catch because it happens at the behavior level, not the file level.

Known overrides:

- **`--enable multi_agent`**: Switches the agent from implementer to planner. At scales below 10 domains, do not use this flag. The agent will produce delegation plans instead of implementation. (Evidence: same prompt, same model — produced implementation without the flag, produced only a planning artifact with it.)
- **Large repo / many files**: Makes agents read-heavy, write-light. Counteract with: "Start implementing immediately. Read only the files listed in Required Reading."
- **`cargo fmt` on frozen contract**: Every worker that runs `cargo fmt` will reformat `shared/mod.rs`, breaking the checksum. Add to every worker spec: "Do NOT run cargo fmt on shared/mod.rs."
- **Model class affects behavior**: Opus-class models deliberate more, implement less. Codex/Sonnet models implement faster. Match model to task: Opus for integration/audit, Sonnet/Codex for domain implementation.
- **Low turn limits**: Make agents write big files in one shot. High limits make them iterate. The turn budget shapes the implementation strategy.

**Rule:** When a worker fails to implement, check the tool configuration before re-issuing the prompt. The prompt may be correct and the tool may be overriding it.

---

## Phase 4 — Clamp Scope Mechanically (after every worker)

Prompt-only scope enforcement: 0/20 under compiler pressure. Mechanical enforcement: 20/20.

**Rule:** You are not preventing scope violations in the moment. Let the worker edit anything. Then revert everything outside its allowlist.

### 4.1 Clamp script (handles tracked + untracked, null-delimited)

Save as `scripts/clamp-scope.sh`:

```bash
#!/usr/bin/env bash
set -euo pipefail
ALLOW_PREFIX="${1:?e.g. src/domains/combat/}"

# Revert tracked unstaged changes outside scope
git diff --name-only -z | while IFS= read -r -d '' f; do
  [[ "$f" == "${ALLOW_PREFIX}"* ]] && continue
  git restore --worktree -- "$f"
done

# Revert tracked staged changes outside scope
git diff --name-only -z --cached | while IFS= read -r -d '' f; do
  [[ "$f" == "${ALLOW_PREFIX}"* ]] && continue
  git restore --staged --worktree -- "$f"
done

# Remove untracked files outside scope
git ls-files --others --exclude-standard -z | while IFS= read -r -d '' f; do
  [[ "$f" == "${ALLOW_PREFIX}"* ]] && continue
  rm -rf -- "$f"
done
```

Usage:

```bash
bash scripts/clamp-scope.sh src/domains/combat/
```

---

## Wave Structure (mandatory — every wave, no exceptions)

Every wave has four named phases. This structure was reverse-engineered from comparing two builds: a pre-playbook build (City DLC) that organically developed `Feature → Integration → Hardening → Test Expansion` and shipped 0 experiential breaks, vs. a playbook build (Precinct DLC) that degenerated into `dispatch → structural gate → commit → next` and shipped 6.

The playbook captured the structural rules but lost the workflow rhythm. This section recovers it.

### 1. Feature
- Implement the intended surface.
- Scope: owned files only.
- Workers execute here. Orchestrator dispatches and waits.
- Do not evaluate player experience here except for obvious catastrophic failures; experiential evaluation belongs to Harden.

### 2. Gate
- Run structural gates only: compile, tests, lint, contract checksum, scope clamp.
- This is Phase 5. If failing, use the fix loop (5.2).
- **Gate proves structural correctness. It does NOT prove the build is good.** (See Phase 0.1-WARNING.)

### 3. Harden
- **Stop forward motion.**
- Perform player trace and reality checks (Phase 5A):
  - First-60-seconds path [wave-required]
  - Entrypoint reachability [wave-required]
  - Event connectivity [wave-required]
  - Save/load drift [wave-if-touched]
  - Asset/content reachability [wave-if-touched / release-required]
- Write the 5-sentence player trace to `status/player-trace-wave-N.md` (Phase 5B.1).
- Write the value audit to `status/value-audit-wave-N.md` if tuning values were touched (Phase 5B.4).

### 4. Graduate
- Convert every important experiential observation from Harden into:
  - A named test (preferred — mechanical enforcement from this point forward), or
  - A tracked release artifact if not yet mechanizable
- Apply graduation priority tiers (Phase 5B.3): P0 tests are mandatory before commit, P1 before next wave, P2 tracked.
- A wave may produce zero new graduation tests only if Harden found zero new experiential surfaces, and that claim must be written explicitly in the player trace artifact.
- **Graduate prevents rediscovery.** An observation that stays prose will be forgotten. A test that runs in the suite will catch regressions forever.

**No wave is complete after Gate.**
**No commit is final until Harden and Graduate are done.**

If you find yourself committing after Gate and dispatching the next wave — stop. You are in the failure pattern. Re-read Phase 0.1-WARNING. The velocity feels productive. The experiential debt is compounding silently.

---

## Phase 5 — Validate Each Domain [Wave Phase: Gate]

### 5.0 Failure handling policy (anti-thrash, anti-rigidity)

When a worker fails a gate, acknowledge the failure briefly and factually.

**Bad correction style**
- "What were you thinking?"
- "How could you miss this?"
- "You ignored the instructions."
- "This is obvious."

Even for AI, this is counterproductive. It tends to trigger:
- apology loops
- defensive verbosity
- rigid over-correction
- unnecessary abstraction
- scope widening

**Good correction style**

State:
1. the observed failure,
2. the likely wrong assumption,
3. the required files to re-read,
4. the allowed scope,
5. the preferred fix,
6. the tempting wrong fix,
7. the consequence if that wrong fix is taken again,
8. the exact gate to re-run.

Use historical failure patterns when helpful, but keep them brief.

Example: "This usually happens when a worker locally redefines a shared type to get local green. Clamp reverts that, and the global gate stays red."

**Rule:** One failure is data, not identity. Your job is to reduce ambiguity and restore the correct pattern, not to increase pressure.

### 5.1 Domain gate (immediately after clamp)

```bash
# Run the repo's compile gate:
npx tsc --noEmit          # TypeScript
# cargo check             # Rust

# Run the repo's domain test gate:
npm test -- src/domains/[domain]/    # TypeScript
# cargo test -p [crate]              # Rust
```

### 5.2 Fix loop (bounded, contrastive)

If failing:

1. Dispatch a fix worker with the same allowlist.
2. State the failure factually:
   - exact compiler/test error
   - file/path involved
   - likely wrong assumption
3. Re-state:
   - preferred fix path
   - why it is correct
   - tempting wrong fix
   - what will happen if that wrong fix is used again
4. Clamp again (Phase 4).
5. Re-run gates.
6. Repeat up to 10 passes.
7. If still failing: escalate to orchestrator triage.

**Fix worker prompt template:**

```markdown
Gate failed: [exact symptom]

Likely wrong assumption:
- [smallest wrong assumption]

Re-read:
1. docs/spec.md
2. docs/domains/[domain].md
3. src/shared/types.ts

Allowed scope:
- src/domains/[domain]/

Preferred fix:
- [what to change]

Why:
- [why this path is correct]

Tempting wrong fix:
- [likely shortcut / local hack]

If you choose the wrong fix:
- [what clamp/gates/integration will do]

Re-run:
- repo compile gate (e.g. `npx tsc --noEmit` / `cargo check`)
- repo domain test gate (e.g. `npm test -- src/domains/[domain]/` / `cargo test -p [crate]`)

In your report, state:
- smallest wrong assumption corrected
- alternative rejected
- cue you will watch for next time
```

---

## Phase 5A — Reality Gates [Wave Phase: Harden]

These gates verify that work is player-reachable, not just compiler-green. They are derived from failure classes observed in the City Office Worker DLC audit: entrypoint drift, asset dead-ends, content that exists but is not reachable, save/load drift, and event buses that emit into nothing.

Each gate is classified by enforcement timing:

- **wave-required** — must pass after every wave, no exceptions
- **wave-if-touched** — must pass if this wave added or modified the relevant surface
- **release-required** — must pass before final ship, but may be incomplete during early waves

### 5A.1 EntryPoint Gate [wave-required]

Name the exact player-facing runtime surface:
- binary / crate / branch / folder / launch command

One file must declare the canonical launch surface. One artifact (`status/runtime-surfaces.md`) must list all secondary/test surfaces. Any new runtime surface requires explicit orchestrator approval.

Pass only if the implemented work is reachable from that runtime. Code that compiles in an unwired side surface does not count as progress.

- **Tempting alternative:** "it compiles somewhere, so it's fine."
- **Consequence:** false progress; shipped branch contains code that is not actually launch-path reachable.
- **Drift cue:** worker creates new entry points, side crates, or standalone binaries instead of wiring into the existing runtime.

### 5A.2 First-60-Seconds Gate [wave-required]

Validate the player path from launch to first meaningful interaction:
- boot
- menu
- new game / load game
- spawn
- movement
- first interaction
- first persistent state change

If this path is unstable, stop all deeper feature work.

- **Why:** this catches "beautiful dead game" failure early.
- **Drift cue:** workers build systems that only matter after minute 10 while the first-seconds path is still fragile.
- **Recovery:** stabilize the critical path before any new wave launches.

### 5A.3 Asset Reachability Gate [wave-if-touched]

Classify every asset as:
- **runtime-used** — loaded and rendered during normal play
- **present-but-unreferenced** — file exists, no code path loads it
- **referenced-but-missing** — code references it, file doesn't exist

No asset-heavy wave closes without this report.

- **Why:** "asset exists" is not the same as "player can ever see it."
- **Tempting alternative:** "the art is in the repo, so the art is done."
- **Consequence:** dead art/UI inventory that inflates perceived progress.

### 5A.4 Content Reachability Gate [wave-if-touched, release-required]

For every gameplay content unit:
- **defined** — exists in code/data
- **obtainable** — player can acquire it through normal gameplay
- **usable / sellable / consumable / progressable** — player can do something meaningful with it
- **save/load safe** — survives round-trip serialization

If a unit fails any step, mark it as dead content.

- **Why:** this is the cleanest way to kill fake depth.
- **Tempting alternative:** "the items are defined in the data file."
- **Consequence:** a game with 200 items where the player can only ever touch 12.

### 5A.5 Event Connectivity Gate [wave-required]

For each event:
- list **producers** (systems that emit it)
- list **consumers** (systems that read it)

Fail if any event has no runtime producer or no runtime consumer unless explicitly marked future-work.

- **Why:** disconnected events feel "implemented" in code review but are inert in play.
- **Drift cue:** event types defined in the contract with no `emit()` or `on()` calls in any domain.

### 5A.6 Save/Load Round-Trip Gate [wave-if-touched]

Create state, save, reload, verify:
- same location/state
- same progression/resources
- no duplicate generation
- no OnEnter overwrite drift

- **Why:** OnEnter systems that regenerate state on scene load are the most common source of save corruption in simulation games.
- **Tempting alternative:** "save works — I tested it by saving and loading once."
- **Consequence:** save file exists but gameplay state is silently reset or duplicated on reload.
- **Drift cue:** world state diverges between "new game played to minute 5" and "saved at minute 3, loaded, played to minute 5."

---

## Phase 5B — The Graduation Principle [Wave Phase: Graduate]

The audit instruction finds experiential problems. The graduation principle fixes them permanently.

**Rule:** If a player-facing invariant matters, it must graduate from "good judgment" to a named test as fast as possible.

(Evidence: City Office Worker DLC promoted first-seconds stability, replay determinism, save/load identity, and social persistence into named tests starting at Rotation 2. Result: 0 experiential breaks. Precinct DLC deferred all experiential checks to the final audit at Wave 9. Result: 6 experiential breaks. Same playbook, same structural gates, same orchestrator capability. The difference was timing of graduation.)

### 5B.1 The per-wave player trace (non-negotiable)

After structural gates pass (cargo check, cargo test, clippy, contract) and BEFORE committing, write 5 sentences describing what the player experiences from boot to first meaningful interaction. Use present tense.

Example:
> 1. The player boots the game and sees the main menu with New Game and Load Game buttons.
> 2. The player clicks New Game and spawns in the precinct interior facing the case board.
> 3. The player walks to the case board, presses F, and receives Case #1 with a toast notification.
> 4. The player walks to the exit door and transitions to PrecinctExterior.
> 5. The player stands in the parking lot and a dispatch call arrives within 30 seconds.

If any sentence uses "should" instead of present tense, the feature described in that sentence is not verified. Do not commit until every sentence is present tense and you have checked the implementing code.

**Include these 5 sentences in the commit message.** Also write them to `status/player-trace-wave-N.md` — the commit message is visible but brittle (squash/rebase loses it). The on-disk artifact is the stable audit trail.

### 5B.2 Graduation: observation → named test

After each wave's player trace, for every experiential surface you verified:

1. Write a test that encodes the verification. Example: if you verified "dispatch fires on PrecinctExterior," write `test_dispatch_fires_on_precinct_exterior` using MinimalPlugins.
2. Add the test to the gate suite so it runs mechanically from this point forward.
3. The test name should describe the player experience, not the implementation detail. `test_dispatch_fires_on_precinct_exterior` not `test_dispatch_rate_modifier_nonzero`.

**Why graduation works:** The audit instruction is a prompt. Prompts fail under pressure (0/20). But the audit instruction only needs to work ONCE per surface — long enough to identify the invariant and write a test. After that, the test enforces mechanically (20/20). The audit instruction is a gate factory, not a gate.

(Evidence: City DLC orchestrator — "the audit instruction mattered first, the mechanical gates mattered longer. Once [experiential surfaces] became gates, the mechanical side did most of the day-to-day enforcement.")

### 5B.3 Graduation priority tiers

Not all graduation debt is equal. Triage by player impact:

**P0 — one missing test is a stop condition:**
- Boot → menu → new game (entrypoint liveness)
- Player spawn + movement
- First interaction produces visible feedback
- Save/load round-trip identity

**P1 — must graduate before the wave that follows the one that created them:**
- Map transitions
- Core loop rewards (gold, XP, rank visible to player)
- Event→toast feedback chains (any event the player triggers must produce visible output)

**P2 — must graduate before release, may lag during early waves:**
- Optional content surfaces (side quests, cosmetics, achievements)
- Asset completeness (all referenced sprites exist and render)
- Full content breadth

One missing P0 graduation test is an immediate stop. P1 debt of 3+ is a stop. P2 is tracked in MANIFEST.md and enforced at release.

### 5B.4 Value audit rule

For every tuning value in data files (rates, modifiers, thresholds):

1. Non-obvious values must have a one-line "player consequence" note. Example: `PrecinctExterior = 0.8  # player gets ~1 dispatch call per 6 game-minutes here`
2. **Any value that can zero out a player-facing loop must have a named graduation test.** If `dispatch_rate_modifier = 0.0` is valid for some maps, there must be a test verifying it's nonzero on maps the player actually visits during normal play.
3. During integration, review every `0.0`, `None`, and default/catch-all value and ask: "will the player stand on this value during the first 60 seconds?"
4. Write the review to `status/value-audit-wave-N.md`. This gives auditors a stable artifact to verify the dangerous-value review actually happened.

(Evidence: Precinct DLC — `dispatch_rate_modifier` defaulted to 0.0 via a catch-all. The only reachable exterior map hit the catch-all. The patrol loop was dead from Phase 0. Nobody checked because the value was "correct" structurally.)

If a wave adds a system but no graduation test, the wave is incomplete — even if all structural gates pass.

---

## Phase 6 — Integration (fresh session, artifact-only)

### 6.1 Start clean

**Do not carry the full orchestration conversation forward.** (~95% of orchestrator cost is re-reading conversation history. Integration is where context is largest.)

Integration session ingests **only:**

- `src/shared/types.ts` + `.contract.sha256`
- `docs/spec.md` + `docs/domains/*.md`
- `status/workers/*.md` (and `*.json` for automated checks)
- Current compiler/test errors (if any)

Integration should ingest worker reports not just as status artifacts, but as **captured reasoning boundaries**:
- what each worker implemented
- what tempting alternatives they rejected
- what cues indicate regression
- what risks remain
- **what is now player-reachable because of their work**

This preserves the decision field into integration instead of collapsing everything into "done / not done."

### 6.2 Integration worker scope

- **Allowed:** `src/` (wiring files, composition root, domain index files)
- **Forbidden:** rewriting domain internals unless compilation requires it
- **Responsibilities:** wire domains together, resolve remaining type mismatches via the contract, ensure events/data flows are connected, run global gates + reality gates

### 6.3 Run global gates

Save as `scripts/run-gates.sh` (adapt compiler/test commands to your repo's stack):

```bash
#!/usr/bin/env bash
set -euo pipefail

echo "== Contract integrity =="
shasum -a 256 -c .contract.sha256

echo "== Compile gate =="
# TypeScript: npx tsc --noEmit
# Rust: cargo check
npx tsc --noEmit

echo "== Test gate =="
# TypeScript: npm test
# Rust: cargo test
npm test

echo "== Connectivity check (no hermetic domains) =="
FAIL=0
for d in src/domains/*/; do
  if ! grep -R --exclude-dir="__tests__" --exclude="*.test.*" -q "shared/types" "$d"; then
    echo "FAIL: $d has no shared contract import"
    FAIL=1
  fi
done
[ "$FAIL" -eq 0 ] || { echo "Connectivity check FAILED: hermetic domains detected"; exit 1; }

echo "== Reality gates (scriptable portions) =="
# These catch what structural gates miss. Add project-specific checks here.
[ -x scripts/check-runtime-surface.sh ] && bash scripts/check-runtime-surface.sh
[ -x scripts/check-event-connectivity.sh ] && bash scripts/check-event-connectivity.sh
[ -x scripts/check-asset-reachability.sh ] && bash scripts/check-asset-reachability.sh

echo "== All gates passed =="
```

Reality gate scripts are project-specific. Create them as the build progresses — even a simple grep-based check is better than prose-only. Reality scripts may start as grep-based approximations and graduate to AST/headless/runtime checks as the build matures. Don't wait for a perfect script; a rough one that runs is better than a thorough one that doesn't exist. Example `check-event-connectivity.sh`: for each event type in the contract, verify at least one `EventWriter<T>` and one `EventReader<T>` exist outside `shared/mod.rs`.

If failing: dispatch targeted fix workers → clamp → re-run gates.

Write `status/integration.md` with what was wired + what remains.

---

## Stop Conditions (do not push through these)

1. **Contract drift** (checksum fails) → Stop. Restore contract. Re-run from Phase 5.
2. **Clamp breaks the fix** → Stop. Boundaries are wrong. Re-scope as integration or merge domains (Phase 1.2).
3. **False green** (domains compile but don't import shared types) → Stop. Wire imports or add integration harness (Phase 6).
4. **Abstraction reflex** (worker builds orchestration frameworks instead of features) → Stop. Re-issue spec with: "Do not create orchestration infrastructure. Implement only domain deliverables."
5. **Delegation compression** (asked for 80 items, got 8) → Stop. Worker is reading a summary, not the full spec. Ensure it reads the disk file. Repeat quantities in the worker spec.
6. **Self-model error** (agent claims it cannot do things it can) → Add to prompt: "You have bash access. You can run `codex exec`. You can read and write files."
7. **Identity paradox** (one agent playing architect + worker loses role separation) → Use separate agent sessions per role. Never ask one session to be both.
8. **Blame-thrash loop** (fix prompts become accusatory; worker responds with apologies, abstractions, or scope creep) → Stop. Re-issue with factual failure, likely wrong assumption, allowed scope, preferred fix, tempting wrong fix, and next gate only.
9. **Happy-path training** (worker succeeds on the exact case but fails on adjacent cases because alternatives/consequences were never specified) → Stop. Add contrastive notes to the relevant domain spec and rerun.
10. **Rule-without-rationale drift** (workers keep violating a frozen decision because the spec says what to do but not why / what breaks otherwise) → Stop. Add a decision record with why, tempting alternative, consequence, cue, and recovery.
11. **Beautiful dead game** (compilation green, tests green, reality gates red — code exists but player can't reach it) → Stop. Run Phase 5A gates. Stabilize the first-60-seconds path before any new feature wave.
12. **Ghost progress** (worker reports "done" but player-reachable output hasn't changed) → Stop. Require the worker to state specifically what is now player-reachable. If nothing, the work is not done.
13. **Velocity without verification** (2+ consecutive waves dispatched without a personal player-journey trace) → Stop. Write the 5-sentence player trace (Phase 5B.1). If any sentence uses "should," fix before dispatching the next wave. Momentum is not progress.
14. **Search termination on structural gates** (orchestrator checks gates, sees green, commits without player trace) → Stop. Green structural gates are a minimum, not a definition of done. Re-read Phase 0.1-WARNING.
15. **Graduation debt** (any missing P0 test, OR 3+ missing P1 tests) → Stop. Write the tests before the next wave. P0 surfaces (boot, spawn, movement, first interaction, save/load) are immediate stops. P1 surfaces (map transitions, core rewards, feedback chains) accumulate to a threshold. P2 is tracked and enforced at release.

---

## Completion Criteria

You are done **only** when:

**Structural (mechanical — run the repo's compile + test + lint gates):**
- [ ] Contract checksum passes
- [ ] Compile gate passes
- [ ] Test gate passes
- [ ] Connectivity gate passes (no hermetic domains)
- [ ] Reality gate scripts pass (if implemented)

**Reality (judgment — requires tracing):**
- [ ] EntryPoint gate passes (all work reachable from player-facing runtime)
- [ ] First-60-Seconds gate passes (boot → menu → spawn → move → interact → persist)
- [ ] Asset reachability report complete (no referenced-but-missing)
- [ ] Content reachability report complete (no dead content units)
- [ ] Event connectivity gate passes (no orphaned producers/consumers)
- [ ] Save/Load round-trip gate passes

**Graduation (the v5 addition):**
- [ ] Every player-facing system has a corresponding graduation test (P0 complete, P1 complete, P2 tracked)
- [ ] Player trace artifacts exist for every wave (`status/player-trace-wave-N.md`)
- [ ] Value audit artifacts exist for waves that touched tuning values (`status/value-audit-wave-N.md`)
- [ ] No graduation debt (stop condition #15 is clear)

**Artifacts:**
- [ ] Each worker report exists (`status/workers/*.md` + `*.json`)
- [ ] Integration report exists (`status/integration.md`)
- [ ] `MANIFEST.md` updated with final status
