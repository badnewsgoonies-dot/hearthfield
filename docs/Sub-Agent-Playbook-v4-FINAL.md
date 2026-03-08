# Sub-Agent Playbook v4 — FINAL
# Contrastive-Causal + Reality Gates Edition

**Mission:** Ship working builds via: (1) frozen type contract, (2) mechanical scope clamping, (3) compiler+test gates, (4) contrastive-causal specs, (5) reality gates.

*Derived from "The Model Is the Orchestrator" (Geni, February 2026) — 295M tokens, 98 agent sessions, 11 autonomous builds, 8 controlled experiments.*

---

## Phase 0 — Bootstrap (once per repo)

### 0.1 Your role

1. You are the orchestrator, not the implementer. You define what gets built, draw boundaries, and validate results. Agents write all code.
2. Treat every instruction below as a constraint, not a suggestion. If a constraint is not mechanically enforced, assume it will be violated.

### 0.1A Contrastive-causal prompting methodology

Bare directives build brittle obedience. Every non-obvious instruction includes:

- **Preferred action** — what to do
- **Why** — why preferred
- **Tempting alternative** — the nearby wrong move a capable worker might choose
- **Consequence** — what breaks if the alternative is taken
- **Drift cue** — the first observable signal of wrong interpretation
- **Recovery** — the smallest correction path back

**Weighting heuristic for transferable competence:**
- 35% causal explanation (why this works)
- 25% tempting alternatives (what looks right but isn't)
- 20% consequence mapping (what breaks and how)
- 10% recovery guidance (how to fix if drifted)
- 10% ownership/self-assessment cues

**Anti-thrash rules:**
- Blame-heavy prompts produce: apology loops, rigidity, scope creep, framework-building
- Calm factual prompts produce: flexibility, narrow repair, forward progress
- Treat errors as diagnostic data, not moral events
- Good correction template: observed failure → likely wrong assumption → re-read these files → scope this fix to → preferred fix → tempting wrong fix → consequence of wrong fix → run this gate next

### 0.1B Reusable orchestrator prompt template

When starting a new orchestration session, prime the orchestrator with:
- Preserve context for orchestration/integration/validation
- Don't waste top-level context on coding tasks
- Prioritize first-seconds player experience
- Mentally simulate the player journey before building deeper systems
- Short robust waves > giant feature bursts
- Freeze shared vocabulary before each wave

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
│   ├── workers/                   # Worker completion reports (written by workers)
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
   - Every cross-domain entity type
   - All shared enums
   - Shared event/message types
   - Cross-module function signatures
   - Strict primitive decisions — IDs are `string` or `number`, decide once, do not mix

2. **Rule:** No domain may redefine these types locally. Every domain must import from the contract.

3. Freeze by checksum + commit:

```bash
shasum -a 256 src/shared/types.ts > .contract.sha256
git add src/shared/types.ts .contract.sha256
git commit -m "chore: freeze shared type contract"
```

**Rule:** No worker edits the contract during parallel build. Contract changes are integration-phase work only (Phase 6).

### 0.3A Decision Records

Every frozen decision includes: choice + why + tempting alternative + consequence + drift cue + recovery.

"Decisions with rationale are remembered as patterns. Decisions without are remembered as rules."

### 0.4 Write MANIFEST.md

The orchestrator's brain on disk. Include:

- Current phase
- Domain list + owners
- Key constants/formulas ("truth decisions")
- Open blockers
- Recurring drift patterns (updated after each wave)
- Failed seam decisions (updated after each wave)

### 0.5 Tracked Noise Hygiene

Explicitly forbid tracked build outputs (target/, dist/, generated fingerprints, temp saves). Add to .gitignore. They create false diff volume, hide real work, and make orchestration slower.

---

## Phase 1 — Draw Boundaries That Survive Clamping

### 1.1 Define domains and allowlist prefixes

For each domain, define the only allowed path prefix:
- `src/domains/combat/`
- `src/domains/ui/`
- etc.

### 1.2 Boundary survivability test (non-negotiable)

A domain is valid **only** if:
- It can compile + pass local tests while all other domains remain unchanged.
- Its fixes do not require edits outside its allowlist after clamping.

**If clamping breaks the fix:**
- Your seam is wrong, OR the task is integration work.
- Merge the domains or route to an integration worker (Phase 6).

Two tightly coupled modules are one module. Draw the seam where architectural independence holds.

**Record:** Why this seam, tempting alternative seam, what breaks if alternative chosen.

### 1.3 Create the folder structure now (empty is fine)

---

## Phase 2 — Put Full Specs on Disk (No Summaries)

Hierarchies compress information. Numbers die first. (Evidence: 327-line objective through 3 delegation levels → 8 weapons against target of 80+.)

Context priming is binary: 0% formula transfer without design context, 100% with it. Format doesn't matter — a static document equals a synthetic dialogue. Presence is the mechanism.

### 2.1 Write `docs/spec.md` + `docs/domains/*.md`

Each domain spec **must** include:

- **Quantities:** "80 weapons" not "lots of weapons." "25 chapters" not "a full campaign."
- **Constants and formulas:** with exact values. (If you don't specify `crit_multiplier = 2.75`, 8/10 workers default to 1.75.)
- **Tables/lists:** stat curves, item lists, drop rates — enumerative detail that summaries destroy.
- **"Does NOT handle" sections:** explicit boundaries.
- **Validation definition of "done."**
- **Why / tempting alternative / consequence blocks** for every non-obvious rule.
- **Drift cues** — first signals of wrong interpretation.
- **Recovery notes** — smallest correction path.

### 2.1A Decision Field (required for every critical spec decision)

Every non-obvious spec decision must include:
- Preferred approach + why
- Tempting alternative + what breaks
- First warning sign/drift cue + recovery path

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

## Interpretation contract
Before writing any code, extract from the domain spec:
- The preferred approach for each major decision
- Why that approach is preferred
- The tempting alternative and what it breaks
- The drift cue that signals wrong interpretation

## Required imports (use these exactly, do not redefine locally)
- [List exact types/enums/APIs from src/shared/types.ts]

## Deliverables
- [Exports, files, features]

## Quantitative targets (non-negotiable)
- [Explicit counts]
- [All constants/formulas with values]

## Failure patterns to avoid
- Local type redefinition (import from contract instead)
- Hidden cross-domain edits (will be reverted)
- "Local green / global red" (your tests pass but you broke the contract)
- Framework-building (implement features, not abstractions)
- Gate-failure scope widening (fix the failure, don't refactor surrounding code)

## Validation (run before reporting done)
- [language-specific check command]
- [language-specific test command]
Done = both commands pass, no skipped tests.

## Contrastive self-check (include in report)
- What tempting alternative did you consider and reject?
- What would have broken if you'd taken it?
- Which spec line ruled it out?
- What's the first regression cue to watch for?

## When done
Write completion report to status/workers/[domain].md containing:
- Files created/modified
- What was implemented
- Quantitative targets hit (with actual counts)
- Shared type imports used
- Validation results (pass/fail + counts)
- Contrastive self-check answers
- **What is now player-reachable because of this work**
- Known risks / open items for integration

Also write status/workers/[domain].json (machine-readable).
```

### 3.3 Dispatch rules

- Stagger launches (~3 seconds) to avoid rate limits.
- Workers run fully autonomous, no interactive approval.
- No mid-run edits by the orchestrator.

### 3.4 Never issue bare corrections

When a worker fails or drifts, always specify:
- Preferred path + why
- Tempting wrong path + consequence
- Scope of fix
- Next gate to run

---

## Phase 4 — Clamp Scope Mechanically (after every worker)

Prompt-only scope enforcement: 0/20 under compiler pressure. Mechanical enforcement: 20/20.

**Rule:** You are not preventing scope violations in the moment. Let the worker edit anything. Then revert everything outside its allowlist.

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

---

## Phase 5 — Validate Each Domain (gate + fix loop)

### 5.0 Failure Handling Policy

- Errors are diagnostic data, not moral events
- Bad: "What were you thinking?" → apology loops, rigidity, scope creep
- Good: observed failure → likely wrong assumption → re-read these files → scope → preferred fix → tempting wrong fix → consequence → next gate
- One failure is data, not identity

### 5.1 Domain gate (immediately after clamp)

```bash
[language-specific check]
[language-specific test]
```

### 5.2 Fix loop (bounded, contrastive)

If failing:

1. Dispatch a fix worker with the same allowlist using contrastive prompt:
   - Gate failure: [exact error]
   - Likely wrong assumption: [diagnosis]
   - Re-read: [specific files]
   - Scope: only edit [allowlist]
   - Preferred fix: [what to do]
   - Why: [rationale]
   - Tempting wrong fix: [what not to do]
   - Consequence: [what breaks]
   - Re-run: [gate command]
2. Clamp again (Phase 4).
3. Re-run gates.
4. Repeat up to 10 passes.
5. If still failing: escalate to orchestrator triage.

---

## Phase 5A — Reality Gates (required before any wave is considered complete)

### 5A.1 EntryPoint Gate
Name the exact player-facing runtime surface:
- binary / crate / branch / folder / launch command

Pass only if the implemented work is reachable from that runtime.
Code that compiles in an unwired side surface does not count as progress.

- **Tempting alternative:** "it compiles somewhere, so it's fine."
- **Consequence:** false progress; shipped binary contains none of the new work.
- **Drift cue:** worker builds in a test harness or side crate that isn't wired to main.

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

- **Tempting alternative:** build mid/late-game systems first because they're more interesting.
- **Consequence:** "beautiful dead game" — rich systems nobody can reach.
- **Drift cue:** workers building systems that only matter after minute 10 while first-seconds path is fragile.

### 5A.3 Asset Reachability Gate
Classify every asset as:
- runtime-used
- present-but-unreferenced
- referenced-but-missing

No asset-heavy wave closes without this report.

### 5A.4 Content Reachability Gate
For every gameplay content unit:
- defined
- obtainable
- usable / sellable / consumable / progressable
- save/load safe

If a unit fails any step, mark it as dead content.

### 5A.5 Event Connectivity Gate
For each event:
- list producers
- list consumers

Fail if any event has no runtime producer or no runtime consumer unless explicitly marked future-work.

- **Tempting alternative:** declare events in shared types and assume someone will wire them.
- **Consequence:** disconnected events feel "implemented" in code review but are inert in play.
- **Drift cue:** event types exist in contract but grep finds zero emit/send calls.

### 5A.6 Save/Load Round-Trip Gate
Create state, save, reload, verify:
- same location/state
- same progression/resources
- no duplicate generation
- no `OnEnter` overwrite drift

- **Tempting alternative:** test save and load separately rather than round-trip.
- **Consequence:** save works, load works, but loaded state diverges from saved state.
- **Drift cue:** any resource that implements `Default` but not `Serialize`.

---

## Phase 6 — Integration (fresh session, artifact-only)

### 6.1 Start clean

**Do not carry the full orchestration conversation forward.** (~95% of orchestrator cost is re-reading conversation history. Integration is where context is largest.)

Integration session ingests **only:**
- `src/shared/types.ts` + `.contract.sha256`
- `docs/spec.md` + `docs/domains/*.md`
- `status/workers/*.md` (as captured reasoning boundaries, not just status)
- Current compiler/test errors (if any)

### 6.2 Integration worker scope
- **Allowed:** `src/` (wiring files, composition root, domain index files)
- **Forbidden:** rewriting domain internals unless compilation requires it
- **Responsibilities:** wire domains together, resolve type mismatches via contract, ensure events/data flows connected, run global + reality gates

### 6.3 Run global gates

```bash
#!/usr/bin/env bash
set -euo pipefail

echo "== Contract integrity =="
shasum -a 256 -c .contract.sha256

echo "== Typecheck =="
[language-specific check]

echo "== Tests =="
[language-specific test]

echo "== Connectivity check (no hermetic domains) =="
FAIL=0
for d in src/domains/*/; do
  if ! grep -R --exclude-dir="__tests__" --exclude="*.test.*" -q "shared" "$d"; then
    echo "FAIL: $d has no shared contract import"
    FAIL=1
  fi
done
[ "$FAIL" -eq 0 ] || { echo "Connectivity FAILED"; exit 1; }

echo "== All gates passed =="
```

If failing: dispatch targeted fix workers → clamp → re-run gates.
Write `status/integration.md` with what was wired + what remains.

---

## Stop Conditions (do not push through these)

1. **Contract drift** (checksum fails) → Stop. Restore contract. Re-run from Phase 5.
2. **Clamp breaks the fix** → Stop. Boundaries are wrong. Re-scope or merge domains (Phase 1.2).
3. **False green** (domains compile but don't import shared types) → Stop. Wire imports (Phase 6).
4. **Abstraction reflex** (worker builds frameworks instead of features) → Stop. Re-issue: "Implement only domain deliverables."
5. **Delegation compression** (asked for 80 items, got 8) → Stop. Worker reading summary not spec. Ensure disk read. Repeat quantities.
6. **Self-model error** (agent claims it cannot do things it can) → Add capability list to prompt.
7. **Identity paradox** (one agent playing architect + worker) → Use separate sessions per role.
8. **Blame-thrash loop** (worker stuck in apology/retry) → Re-issue with factual failure + scope + preferred/wrong fix + next gate.
9. **Happy-path training** (worker succeeds on exact case, fails on adjacent) → Add contrastive notes to spec and rerun.
10. **Rule-without-rationale drift** (worker follows letter but not spirit) → Add decision record (why/alternative/consequence/cue/recovery).
11. **Beautiful dead game** (rich systems, unplayable) → Run Phase 5A. Stabilize first-60-seconds before deeper waves.
12. **Ghost progress** (code exists but player can't reach it) → Require statement: "what is now player-reachable because of this work."

---

## Completion Criteria

You are done **only** when:

- [ ] Contract checksum passes
- [ ] Global typecheck passes
- [ ] Global test suite passes
- [ ] Connectivity gate passes (no hermetic domains)
- [ ] EntryPoint gate passes
- [ ] First-60-Seconds gate passes
- [ ] Asset reachability report complete
- [ ] Content reachability report complete
- [ ] Event connectivity gate passes
- [ ] Save/Load round-trip gate passes
- [ ] Each worker report exists (`.md` + `.json`)
- [ ] Integration report exists (`status/integration.md`)
- [ ] `MANIFEST.md` updated with final status
