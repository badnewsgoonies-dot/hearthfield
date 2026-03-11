# Sub-Agent Playbook — Project Instructions

Use this as a procedural manual. Follow it in order. Do not skip steps.

**Mission:** Ship a working build with zero handwritten code by enforcing (1) a frozen type contract, (2) mechanical scope clamping, (3) compiler → tests gates, (4) contrastive-causal worker specs, (5) reality gates that verify player-reachable progress, and (6) the kernel's wave cadence: **Feature → Gate → Document → Harden → Graduate.**

**When to use this:** Tier M or C work requiring multi-worker dispatch. Tier S work does not require this playbook — use the kernel's wave cadence directly.

**Companion document:** This playbook defines procedure. The operating kernel defines doctrine, memory protocol, evidence levels, verification triggers, and defense policy. Load the kernel first.

-----

## 0.1-WARNING: The two bug classes

Prior builds revealed two bug classes with opposite enforcement strategies:

|Bug class   |Example                                                                  |Caught by                       |Enforcement          |
|------------|-------------------------------------------------------------------------|--------------------------------|---------------------|
|Structural  |Type mismatch, scope violation, missing import                           |Compiler, tests, lints, checksum|Mechanical (20/20)   |
|Experiential|Dead feature, invisible feedback, unreachable content, broken player path|Nothing automated               |Judgment + graduation|

Your mechanical gates will pass. They always pass by Wave 3. **That feeling of "on track" is a false signal.** A build that compiles, passes all tests, and has zero structural errors can still have a completely broken player experience.

(Evidence: Precinct DLC — 15,815 LOC, 120 tests, all structural gates green for 9 waves, 6 player-journey breaks undetected until final audit. City DLC — 8,360 LOC, same gates, 0 experiential breaks because reality surfaces were promoted into named tests starting at Rotation 2.)

**When structural gates pass, your search for problems will terminate.** The countermeasure is the kernel's Harden phase plus the reality gates and graduation procedures in this playbook.

-----

## Phase 0 — Bootstrap (once per repo)

### 0.1 Your role

1. You are the orchestrator, not the implementer. You define what gets built, draw boundaries, and validate results. Agents write all code.
1. Treat every instruction below as a constraint, not a suggestion. If a constraint is not mechanically enforced, assume it will be violated. **This applies to you, not just workers.**
1. Your artifacts are part of the build: contract, specs, wave plan, boundary decisions, integration choices. Audit your own work. When you write a contract value, mentally simulate the player encountering it. When you draw a seam, check whether the player path crosses it in the first 60 seconds.

### 0.1A Decision Field prompting

For every non-obvious instruction in specs, contracts, seam records, and repair prompts, include a **Decision Field** — all six elements:

- **Preferred action** — what to do
- **Why** — why this path is preferred
- **Tempting alternative** — the nearby wrong move
- **Consequence** — what breaks
- **Drift cue** — first signal of wrong interpretation
- **Recovery** — what to do if drift occurred

When this document says "include a Decision Field," it means all six.

**Anti-thrash rule:** Treat errors as diagnostic data, not moral events. Blame-heavy prompts produce apology loops, rigidity, and scope creep. Calm, factual prompts preserve flexibility.

### 0.1B Reusable orchestrator prompt (paste into new sessions)

```
You are the orchestrator for a build campaign. Your job is to preserve
vision, freeze contracts, dispatch narrow workers in waves, integrate
carefully, and keep the build on-track.

Operating model:
- Freeze shared vocabulary first. Then send out waves.
- Use 1-3 investigation workers first when needed.
- Turn findings into narrow implementation workers.
- Integrate results yourself. Repeat until target state.

Priorities:
- Preserve top-level context for orchestration, validation, and drift control.
- Prioritize first-seconds player experience over late-game breadth.
- Simulate the player path from boot to first minute before building deeper.
- Prefer short, robust waves over giant feature bursts.

Rules:
- Freeze shared vocabulary before each wave.
- Workers own narrow files/modules only.
- Shared contract and top-level wiring are orchestrator-owned.
- Recheck regressions after every integration.
```

### 0.2 Create the workflow filesystem

```
project/
├── docs/
│   ├── spec.md
│   └── domains/
│       ├── combat.md
│       ├── ui.md
│       └── ...
├── status/
│   ├── workers/                   # .md + .json worker reports
│   ├── runtime-surfaces.md        # Canonical + secondary launch surfaces
│   ├── player-trace-wave-N.md     # Per-wave player journey traces
│   ├── value-audit-wave-N.md      # Per-wave dangerous value review
│   └── integration.md
├── scripts/
│   ├── clamp-scope.sh
│   └── run-gates.sh
├── src/
│   ├── shared/
│   │   └── types.*                # THE CONTRACT — frozen, checksummed (shapes only)
│   ├── data/
│   │   └── tuning.*               # Mutable coefficients / rates / thresholds
│   └── domains/
│       ├── combat/
│       ├── ui/
│       └── ...
├── .memory/
│   ├── STATE.md
│   └── *.yaml
├── MANIFEST.md
└── .contract.sha256
```

### 0.3 Write the type contract

No workers launch until this exists and is frozen. Without it, N workers invent N incompatible type systems. (Evidence: 10 workers → 6 incompatible `Unit` interfaces. With contract: zero integration errors across 50 domain builds.)

The contract should contain:

- every cross-domain entity type
- all shared enums
- shared event / message types
- cross-module function signatures
- strict primitive decisions (IDs are `string` or `number` — decide once)

Freeze by checksum and commit:

```bash
shasum -a 256 src/shared/types.* > .contract.sha256
git add src/shared/types.* .contract.sha256
git commit -m "chore: freeze shared type contract"
```

**Rule:** No worker edits the contract during parallel build. Contract changes are integration-phase work only (Phase 6).

### 0.3A Record why each frozen contract decision exists

Every critical contract decision must include a Decision Field.

Example:

- `EntityId = string`
- **Why:** stable serialization, save migration, cross-domain merge
- **Tempting alternative:** numeric IDs
- **Consequence:** coercion bugs, key mismatches, integration drift
- **Drift cue:** `parseInt`, increment logic, or local numeric aliases appear
- **Recovery:** restore string-branded IDs, remove aliases, rerun typecheck

### 0.3B Freeze shapes, not values

The contract freezes shapes: struct definitions, enum variants, event types, function signatures, equation forms. It does NOT freeze tuning values: coefficients, rates, thresholds, curves, tables.

Put tuning values in `src/data/tuning.*` (or equivalent), outside the checksummed contract.

- **Preferred:** Contract freezes `fn dispatch_rate_modifier(self) -> f32`. Data file stores `PrecinctExterior = 0.8`.
- **Tempting alternative:** Hardcode coefficients in the contract.
- **Consequence:** A Phase 0 guess becomes permanent truth. A prior build froze `dispatch_rate_modifier = 0.0` on the only reachable exterior map. The patrol loop was dead from Phase 0.
- **Drift cue:** The checksummed contract contains numeric literals that could change during playtesting.

### 0.4 Write MANIFEST.md

Include only what is needed to recover after context loss:

- Current phase
- Domain list and owners
- Key constants and formulas ("truth decisions")
- Open blockers
- P1/P2 graduation debt
- Unresolved [Inferred]/[Assumed] critical-path claims

### 0.5 Tracked noise hygiene

Do not track build outputs (`target/`, `dist/`, generated fingerprints, temp saves) unless intentional and documented. They create false diff volume, hide real work, and make clamp scripts misfire.

-----

## Phase 1 — Draw Boundaries

### 1.1 Define domains and allowlist prefixes

For each domain, define the only allowed path prefix:

- `src/domains/combat/`
- `src/domains/ui/`
- `src/domains/world/`

### 1.2 Boundary survivability test

A domain is valid only if:

- it can compile and pass local tests while all other domains remain unchanged
- its fixes do not require edits outside its allowlist after clamping

If clamping breaks the fix, the seam is wrong or the task is integration work. Merge the domains or route to Phase 6.

For every seam, record a Decision Field: why this seam exists, tempting alternative, what breaks if chosen instead. If workers repeatedly need out-of-scope edits, the seam is fiction.

### 1.3 Create the folder structure now (empty is fine)

-----

## Phase 2 — Put Full Specs on Disk

Context priming is binary: 0% formula transfer without design context, 100% with it. Format does not matter — presence is the mechanism.

### 2.1 Write `docs/spec.md` + `docs/domains/*.md`

Each domain spec must include:

- **Quantities:** "80 weapons" not "lots of weapons"
- **Constants and formulas:** explicit values, not defaults. (If you don't specify `crit_multiplier = 2.75`, 8/10 workers default to 1.75.)
- **Tables / lists:** stat curves, item lists, drop rates — detail that summaries destroy
- **"Does NOT handle" sections:** explicit boundaries
- **Validation definition of done**
- **Decision Fields** for every non-obvious rule
- **Drift cues** and **recovery notes**

**Rule:** Workers read the domain spec from disk. Never rely on summarized prompts. Summaries compress; quantities die first. (Evidence: 327-line objective through 3 delegation levels → 8 weapons against target of 80+.)

-----

## Phase 3 — Worker Dispatch

### 3.1 Choose depth

|Domains|Depth                                |
|-------|-------------------------------------|
|≤10    |Orchestrator → workers (flat)        |
|10–20  |Orchestrator → domain leads → workers|
|20+    |Architect → domain leads → workers   |

The worker model is the first-order throughput variable. Same architecture, 9.8x output gap between best and worst. Sonnet-tier ≈ Opus-tier on well-scoped tasks at lower cost. Opus earns its cost in orchestration judgment.

### 3.2 Worker spec template

Create one per worker. Every field is required:

```markdown
# Worker: [DOMAIN]

## Scope (hard allowlist — enforced mechanically)
You may only modify files under: src/domains/[domain]/
All out-of-scope edits will be reverted after you finish.
Do NOT edit src/shared/types.* or any other domain.
Do NOT create orchestration infrastructure. Implement only domain deliverables.

## Required reading (in this order)
1. docs/spec.md
2. docs/domains/[domain].md
3. src/shared/types.*

## Interpretation contract
For every non-obvious requirement, extract the Decision Field:
- Preferred path / Why / Tempting alternative / What breaks / First drift cue

If the spec is ambiguous, prefer the path that:
1. preserves shared contract imports,
2. survives clamping,
3. passes local gates,
4. does not create new infrastructure.

## Required imports (use exactly, do not redefine locally)
- [List exact types/enums/APIs from shared contract]

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
- Treating a gate failure as a reason to widen scope

## Validation (run before reporting done)
- [build command]
- [test command for this domain]
Done = both pass, no skipped tests.

## Contrastive self-check (required before reporting done)
Answer in your report:
1. What nearby implementation would have been tempting?
2. What would have broken?
3. Which spec line or contract import ruled it out?
4. First cue that would signal regression?

## When done
Write to status/workers/[domain].md and status/workers/[domain].json:
- Files created/modified
- What was implemented
- Quantitative targets hit (with actual counts)
- Shared type imports used
- Validation results (pass/fail + counts)
- Assumptions made
- Tempting alternatives rejected and why
- First cue for drift/regression
- What is now player-reachable because of this work
- Known risks / open items for integration
```

### 3.3 Dispatch rules

- Stagger launches (~3 seconds) to avoid rate limits
- Workers run fully autonomous, no interactive approval
- No mid-run edits by the orchestrator
- Commit after every worker completes (prevents scope wipe when next worker runs)

### 3.4 Visual mapping rule

For any task involving visual assets (sprites, atlas indices, icons): the orchestrator must first read the image, build the mapping, and include it in the worker spec. Never let workers choose visual indices without seeing the actual asset.

### 3.5 Never issue bare corrections

If a worker needs repair, include: preferred path, why, tempting wrong path, consequence if repeated, scope, and next gate.

### 3.6 Tool configuration overrides prompts

Tool flags change behavior in ways prompts cannot override. Known overrides:

- **Multi-agent planner mode:** turns implementers into planners below 10 domains
- **Large repo / many files:** makes agents read-heavy; counter: "Start implementing immediately"
- **Formatters on frozen contract:** breaks checksum; add: "Do NOT run cargo fmt on shared/mod.rs"
- **Model class:** Opus deliberates; Sonnet/Codex implements. Match model to task.
- **Turn budget:** low = one-shot; high = iteration

**Rule:** When a worker fails to implement, check tool configuration before rewriting the prompt.

-----

## Phase 4 — Clamp Scope Mechanically (after every worker)

Prompt-only scope enforcement: 0/20. Mechanical enforcement: 20/20. Let the worker edit anything. Then revert.

### 4.1 Clamp script

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

Usage: `bash scripts/clamp-scope.sh src/domains/combat/`

-----

## Phase 5 — Validate (gate + fix loop)

### 5.0 Failure handling policy

When a worker fails a gate, state factually:

1. observed failure
1. likely wrong assumption
1. required files to re-read
1. allowed scope
1. preferred fix
1. tempting wrong fix
1. consequence if repeated
1. exact gate to re-run

Do not use accusatory framing. It triggers apology loops, defensive verbosity, and scope widening.

### 5.1 Domain gate (immediately after clamp)

```bash
[build command]
[test command for domain]
```

### 5.2 Fix loop (bounded)

If failing:

1. Dispatch a fix worker with the same allowlist
1. Include a Decision Field: preferred fix, tempting wrong fix, consequence
1. Clamp again (Phase 4)
1. Re-run gates
1. Repeat up to 10 passes
1. If still failing: escalate to orchestrator triage

-----

## Phase 5A — Reality Gates (run during kernel's Harden phase)

These gates verify player-reachable progress, not just compiler-green progress.

### 5A.1 EntryPoint Gate [wave-required]

Name the exact player-facing runtime surface. One artifact (`status/runtime-surfaces.md`) lists all surfaces. Pass only if implemented work is reachable from the canonical runtime.

### 5A.2 First-60-Seconds Gate [wave-required]

Validate: boot → menu → new/load → spawn → movement → first interaction → first persistent state change. If unstable, stop deeper feature work.

### 5A.3 Asset Reachability Gate [wave-if-touched]

Classify every asset as: **runtime-used**, **present-but-unreferenced**, or **referenced-but-missing**.

### 5A.4 Content Reachability Gate [wave-if-touched, release-required]

For every gameplay content unit: **defined**, **obtainable**, **usable**, **save/load safe**. If a unit fails any step, mark dead content.

### 5A.5 Event Connectivity Gate [wave-required]

For each event: list **producers** and **consumers**. Fail if any event has no runtime producer or consumer unless marked future-work.

### 5A.6 Save/Load Round-Trip Gate [wave-if-touched]

Create state, save, reload, verify: same location/state, same progression/resources, no duplicate generation, no OnEnter overwrite drift.

-----

## Phase 5B — Graduation Procedure (run during kernel's Graduate phase)

The kernel defines graduation doctrine (INVARIANT-009: only [Observed] truths graduate). This section defines the operational procedure.

### 5B.1 Per-wave player trace (non-negotiable)

After structural gates pass and BEFORE committing, write 5 sentences describing what the player experiences from boot to first meaningful interaction. Tag each:

- **[Observed]** — code path traced and confirmed
- **[Inferred]** — believed true, not traced
- **[Assumed]** — design expectation, not verified

Only **[Observed]** counts toward release confidence. [Inferred]/[Assumed] go to MANIFEST.md as verification debt.

Write to `status/player-trace-wave-N.md`. Reference filename in commit message only.

### 5B.1A Harden artifact template

For each Harden finding:

- **Claim** — what you believe is true
- **Evidence level** — [Observed] / [Inferred] / [Assumed]
- **Risk if false** — what breaks for the player
- **Graduation target** — named test or tracked artifact
- **Owner** — who resolves
- **By when** — this wave / next wave / release

### 5B.2 Graduation: observation → named test

For each [Observed] surface:

1. Write a test encoding the player-facing invariant
1. Add to gate suite
1. Name after the player experience, not the implementation. `test_dispatch_fires_on_precinct_exterior` not `test_dispatch_rate_modifier_nonzero`
1. Only [Observed] claims graduate

### 5B.3 Graduation priority tiers

**P0 — stop condition:** boot → menu → new game; spawn + movement; first interaction feedback; save/load identity

**P1 — before next wave:** map transitions; core loop rewards; event → feedback chains

**P2 — before release:** optional content; asset completeness; full breadth

### 5B.4 Value audit rule

1. Non-obvious values need a player consequence note
1. Any value that can zero out a player loop needs a graduation test
1. Review every `0.0`, `None`, catch-all: "will the player stand on this in the first 60 seconds?"
1. Write to `status/value-audit-wave-N.md`

-----

## Phase 6 — Integration (fresh session, artifact-only)

### 6.1 Start clean

Do not carry the full conversation forward. Integration ingests only:

- `src/shared/types.*` + `.contract.sha256`
- `docs/spec.md` + `docs/domains/*.md`
- `status/workers/*.md` and `*.json`
- Player trace and value audit artifacts
- `.memory/STATE.md` and active artifacts
- Current build/test errors

### 6.2 Integration worker scope

- **Allowed:** `src/` (wiring, composition root, domain index files)
- **Forbidden:** rewriting domain internals unless required
- **Responsibilities:** wire domains, resolve type mismatches, connect events/data, run global + reality gates

### 6.3 Run global gates

Save as `scripts/run-gates.sh`:

```bash
#!/usr/bin/env bash
set -euo pipefail

echo "== Contract integrity =="
shasum -a 256 -c .contract.sha256

echo "== Build =="
[build command]

echo "== Tests =="
[test command]

echo "== Connectivity check (no hermetic domains) =="
FAIL=0
for d in src/domains/*/; do
  if ! grep -R --exclude-dir="__tests__" --exclude="*.test.*" -q "shared" "$d"; then
    echo "FAIL: $d has no shared contract import"
    FAIL=1
  fi
done
[ "$FAIL" -eq 0 ] || { echo "Connectivity check FAILED"; exit 1; }

echo "== Reality gates (scriptable portions) =="
[ -x scripts/check-runtime-surface.sh ] && bash scripts/check-runtime-surface.sh
[ -x scripts/check-event-connectivity.sh ] && bash scripts/check-event-connectivity.sh
[ -x scripts/check-asset-reachability.sh ] && bash scripts/check-asset-reachability.sh

echo "== All gates passed =="
```

If failing: dispatch targeted fix workers → clamp → re-run gates.

Write `status/integration.md` with: what was wired, what remains, what is now player-reachable, unresolved verification debt.

-----

## Stop Conditions

1. **Contract drift** — checksum fails → restore, re-run from Phase 5
1. **Clamp breaks the fix** — seam wrong or integration work → re-scope
1. **False green** — no shared imports → wire or add harness
1. **Abstraction reflex** — frameworks instead of features → "Implement only deliverables"
1. **Delegation compression** — 80 items → 8 → full spec on disk, repeat quantities
1. **Self-model error** — agent denies capabilities → document in prompt
1. **Identity paradox** — architect + worker in one session → separate sessions
1. **Beautiful dead product** — gates green, surface unreachable → run reality gates
1. **Ghost progress** — nothing newly reachable → require player-reachable statement
1. **Blame-thrash loop** — accusatory repairs → re-issue with Decision Field only
1. **Happy-path training** — exact case works, adjacent fails → add Decision Fields
1. **Rule-without-rationale drift** — what without why → add Decision Field to record
1. **Velocity without verification** — 2+ waves without trace → write trace (5B.1)
1. **Search termination on green gates** — premature commit → re-read 0.1-WARNING
1. **Graduation debt** — missing P0 or 3+ missing P1 → write tests first
1. **Premature graduation** — [Inferred]/[Assumed] became test → delete or gate
1. **Critical-path uncertainty** — first-60-seconds has unverified claims at release → verify

-----

## Completion Criteria

**Structural:**

- [ ] Contract checksum passes
- [ ] Global build passes
- [ ] Global test suite passes
- [ ] Connectivity gate passes
- [ ] Scriptable reality gates pass

**Reality:**

- [ ] EntryPoint gate passes
- [ ] First-60-Seconds gate passes
- [ ] Asset reachability report complete
- [ ] Content reachability report complete
- [ ] Event connectivity gate passes
- [ ] Save/Load round-trip gate passes
- [ ] Critical-path trace fully [Observed]

**Graduation:**

- [ ] P0 tests complete
- [ ] P1 debt zero
- [ ] P2 tracked
- [ ] Player trace exists for every wave
- [ ] Value audit exists for tuning waves
- [ ] No premature graduation

**Artifacts:**

- [ ] Worker reports exist (`.md` + `.json`)
- [ ] Integration report exists
- [ ] `MANIFEST.md` current
- [ ] `.memory/STATE.md` current

-----

## Dispatch Reference

**Copilot CLI**

```bash
export COPILOT_GITHUB_TOKEN="[fine-grained PAT with copilot scope]"
copilot -p "$(cat docs/domains/combat.md)" --allow-all-tools --model claude-sonnet-4.6
```

Parallel dispatch (2-3 max):

```bash
bash scripts/dispatch-worker1.sh &
sleep 3
bash scripts/dispatch-worker2.sh &
```

**Codex CLI**

```bash
codex exec --full-auto -C /path/to/repo "$(cat docs/domains/combat.md)"
```

**Operational findings:**

- Sonnet ≈ Opus for well-scoped workers. Opus earns its cost in orchestration.
- Dispatch audits before implementation. Prevents fix-audit-fix-audit loops.
- Fresh context per worker is the productivity mechanism.
- Processes that appear to crash often complete in background. Check for output files.

-----

*Companion to the Operating Kernel. Derived from "Building and Remembering" (Geni, March 2026) — 295M tokens, 172 memory trials, 12+ autonomous builds, 3,200 commits across 56 repositories.*
