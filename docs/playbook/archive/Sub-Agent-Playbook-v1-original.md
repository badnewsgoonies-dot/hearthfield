# Sub-Agent Playbook — Project Instructions

Use this as a procedural manual. Follow it in order. Do not skip steps.

**Mission:** Ship a working build with zero handwritten code by enforcing (1) a frozen type contract, (2) mechanical scope clamping, and (3) compiler → tests gates.

---

## Phase 0 — Bootstrap (once per repo)

### 0.1 Your role

1. You are the orchestrator, not the implementer. You define what gets built, draw boundaries, and validate results. Agents write all code.
2. Treat every instruction below as a constraint, not a suggestion. If a constraint is not mechanically enforced, assume it will be violated.

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

### 0.4 Write MANIFEST.md

The orchestrator's brain on disk. Include only what's needed to recover after context loss:

- Current phase
- Domain list + owners
- Key constants/formulas ("truth decisions": IDs are strings, crit_mult is 2.75, etc.)
- Open blockers

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

### 1.3 Create the folder structure now (empty is fine)

---

## Phase 2 — Put Full Specs on Disk (No Summaries)

Hierarchies compress information. Numbers die first. (Evidence: 327-line objective through 3 delegation levels → 8 weapons against target of 80+.)

Context priming is binary: 0% formula transfer without design context, 100% with it. Format doesn't matter — a static document equals a synthetic dialogue. Presence is the mechanism.

### 2.1 Write `docs/spec.md` + `docs/domains/*.md`

Each domain spec **must** include:

- **Quantities:** "80 weapons" not "lots of weapons." "25 chapters" not "a full campaign."
- **Constants and formulas:** `crit_multiplier = 2.75`, `ATK multiplier = 1.15`, `DEF factor = 0.70`, `base_hit_rate = 82`, `variance = ±8%`. (If you don't specify `crit_multiplier = 2.75`, 8/10 workers default to 1.75.)
- **Tables/lists:** stat curves, item lists, drop rates — enumerative detail that summaries destroy.
- **"Does NOT handle" sections:** explicit boundaries.
- **Validation definition of "done."**

**Rule:** Workers read the domain spec from disk. Never rely on summarized prompts passed through intermediaries.

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

## Required imports (use these exactly, do not redefine locally)
- [List exact types/enums/APIs from src/shared/types.ts]

## Deliverables
- [Exports, files, features]

## Quantitative targets (non-negotiable)
- [Explicit counts]
- [All constants/formulas with values]

## Validation (run before reporting done)
- npx tsc --noEmit
- npm test -- src/domains/[domain]/
Done = both commands pass, no skipped tests.

## When done
Write completion report to status/workers/[domain].md containing:
- Files created/modified
- What was implemented
- Quantitative targets hit (with actual counts)
- Shared type imports used
- Validation results (pass/fail + counts)
- Assumptions made
- Known risks / open items for integration
```

### 3.3 Dispatch rules

- Stagger launches (~3 seconds) to avoid rate limits.
- Workers run fully autonomous, no interactive approval.
- No mid-run edits by the orchestrator.

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

### 5.1 Domain gate (immediately after clamp)

```bash
npx tsc --noEmit
npm test -- src/domains/[domain]/
```

### 5.2 Fix loop (bounded)

If failing:

1. Dispatch a fix worker with the same allowlist.
2. Clamp again (Phase 4).
3. Re-run gates.
4. Repeat up to 10 passes.
5. If still failing: escalate to orchestrator triage.

---

## Phase 6 — Integration (fresh session, artifact-only)

### 6.1 Start clean

**Do not carry the full orchestration conversation forward.** (~95% of orchestrator cost is re-reading conversation history. Integration is where context is largest.)

Integration session ingests **only:**

- `src/shared/types.ts` + `.contract.sha256`
- `docs/spec.md` + `docs/domains/*.md`
- `status/workers/*.md`
- Current compiler/test errors (if any)

### 6.2 Integration worker scope

- **Allowed:** `src/` (wiring files, composition root, domain index files)
- **Forbidden:** rewriting domain internals unless compilation requires it
- **Responsibilities:** wire domains together, resolve remaining type mismatches via the contract, ensure events/data flows are connected, run global gates

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

---

## Completion Criteria

You are done **only** when:

- [ ] Contract checksum passes
- [ ] Global typecheck passes (`npx tsc --noEmit` / `cargo check`)
- [ ] Global test suite passes
- [ ] Connectivity gate passes (no hermetic domains)
- [ ] Each worker report exists (`status/workers/*.md`)
- [ ] Integration report exists (`status/integration.md`)
- [ ] `MANIFEST.md` updated with final status

---

*Derived from "The Model Is the Orchestrator" (Geni, February 2026) — 295M tokens, 98 agent sessions, 11 autonomous builds, 8 controlled experiments, 3,200 commits across 56 repositories.*
