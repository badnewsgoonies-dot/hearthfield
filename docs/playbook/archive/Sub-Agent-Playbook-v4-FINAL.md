# Sub-Agent Playbook — Project Instructions (v4: Contrastive-Causal + Reality Gates Edition)

Use this as a procedural manual. Follow it in order. Do not skip steps.

**Mission:** Ship a working build with zero handwritten code by enforcing (1) a frozen type contract, (2) mechanical scope clamping, (3) compiler → tests gates, (4) contrastive-causal worker specs that teach the correct path, the tempting wrong path, and the consequence boundary between them, and (5) reality gates that verify player-reachable progress, not just compilation.

*Derived from "The Model Is the Orchestrator" (Geni, February 2026) — 295M tokens, 98 agent sessions, 11 autonomous builds, 8 controlled experiments, 3,200 commits across 56 repositories. Contrastive-causal infusion derived from empirical worker training methodology and pattern-matching research across cognitive science, phenomenology, and naturalistic decision-making. Reality gates derived from failure classes observed in the City Office Worker DLC audit.*

---

## Phase 0 — Bootstrap (once per repo)

### 0.1 Your role

1. You are the orchestrator, not the implementer. You define what gets built, draw boundaries, and validate results. Agents write all code.
2. Treat every instruction below as a constraint, not a suggestion. If a constraint is not mechanically enforced, assume it will be violated.

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
│   └── integration.md             # Integration report (written in Phase 6)
├── scripts/
│   ├── clamp-scope.sh             # Mechanical scope enforcement
│   └── run-gates.sh               # Validation pipeline
├── src/
│   ├── shared/
│   │   └── types.ts               # THE CONTRACT — frozen, checksummed
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
- npx tsc --noEmit
- npm test -- src/domains/[domain]/
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

## Phase 5 — Validate Each Domain (gate + fix loop)

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
npx tsc --noEmit
npm test -- src/domains/[domain]/
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
- npx tsc --noEmit
- npm test -- src/domains/[domain]/

In your report, state:
- smallest wrong assumption corrected
- alternative rejected
- cue you will watch for next time
```

---

## Phase 5A — Reality Gates (required before any wave is considered complete)

These gates verify that work is player-reachable, not just compiler-green. They are derived from failure classes observed in the City Office Worker DLC audit: entrypoint drift, asset dead-ends, content that exists but is not reachable, save/load drift, and event buses that emit into nothing.

### 5A.1 EntryPoint Gate

Name the exact player-facing runtime surface:
- binary / crate / branch / folder / launch command

Pass only if the implemented work is reachable from that runtime. Code that compiles in an unwired side surface does not count as progress.

- **Tempting alternative:** "it compiles somewhere, so it's fine."
- **Consequence:** false progress; shipped branch contains code that is not actually launch-path reachable.
- **Drift cue:** worker creates new entry points, side crates, or standalone binaries instead of wiring into the existing runtime.

### 5A.2 First-60-Seconds Gate

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

### 5A.3 Asset Reachability Gate

Classify every asset as:
- **runtime-used** — loaded and rendered during normal play
- **present-but-unreferenced** — file exists, no code path loads it
- **referenced-but-missing** — code references it, file doesn't exist

No asset-heavy wave closes without this report.

- **Why:** "asset exists" is not the same as "player can ever see it."
- **Tempting alternative:** "the art is in the repo, so the art is done."
- **Consequence:** dead art/UI inventory that inflates perceived progress.

### 5A.4 Content Reachability Gate

For every gameplay content unit:
- **defined** — exists in code/data
- **obtainable** — player can acquire it through normal gameplay
- **usable / sellable / consumable / progressable** — player can do something meaningful with it
- **save/load safe** — survives round-trip serialization

If a unit fails any step, mark it as dead content.

- **Why:** this is the cleanest way to kill fake depth.
- **Tempting alternative:** "the items are defined in the data file."
- **Consequence:** a game with 200 items where the player can only ever touch 12.

### 5A.5 Event Connectivity Gate

For each event:
- list **producers** (systems that emit it)
- list **consumers** (systems that read it)

Fail if any event has no runtime producer or no runtime consumer unless explicitly marked future-work.

- **Why:** disconnected events feel "implemented" in code review but are inert in play.
- **Drift cue:** event types defined in the contract with no `emit()` or `on()` calls in any domain.

### 5A.6 Save/Load Round-Trip Gate

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

Save as `scripts/run-gates.sh`:

```bash
#!/usr/bin/env bash
set -euo pipefail

echo "== Contract integrity =="
shasum -a 256 -c .contract.sha256

echo "== Typecheck =="
npx tsc --noEmit

echo "== Tests =="
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

echo "== All gates passed =="
```

The connectivity check is a grep proxy. For stronger verification: use AST parsing (`ts-morph`) or require each domain to export an `index.ts` that imports at least one shared type in a value position (not type-only, which gets tree-shaken).

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

---

## Completion Criteria

You are done **only** when:

- [ ] Contract checksum passes
- [ ] Global typecheck passes (`npx tsc --noEmit` / `cargo check`)
- [ ] Global test suite passes
- [ ] Connectivity gate passes (no hermetic domains)
- [ ] EntryPoint gate passes (all work reachable from player-facing runtime)
- [ ] First-60-Seconds gate passes (boot → menu → spawn → move → interact → persist)
- [ ] Asset reachability report complete (no referenced-but-missing)
- [ ] Content reachability report complete (no dead content units)
- [ ] Event connectivity gate passes (no orphaned producers/consumers)
- [ ] Save/Load round-trip gate passes
- [ ] Each worker report exists (`status/workers/*.md` + `*.json`)
- [ ] Integration report exists (`status/integration.md`)
- [ ] `MANIFEST.md` updated with final status
