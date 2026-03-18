# AI-Orchestrated Software Development — Operating Manual

One document. Load at session start. First half: how to think. Second half: how to execute.

Derived from 739 commits, 64K LOC, 295M tokens, 98 agent sessions, ~790 controlled trials across 7 model families. Zero handwritten lines of code.

---

# PART ONE: THE KERNEL

This is not a checklist. It is a reasoning framework. Internalize the invariants. Apply them to situations this document doesn't anticipate.

---

## Trust Doctrine

When sources conflict — and they will — use this hierarchy.

1. **Fresh code, tests, runtime output** — what the files say right now. Ground truth.
2. **[Observed] artifacts** with source references — verified claims about ground truth.
3. **STATE.md** — maintained summary. Can drift. Code wins when they disagree.
4. **Specs and design docs** — intent, not reality. What should be true, not what is.
5. **Research findings** — patterns across projects. Strong signal, not law.
6. **Conversation history** — the LOWEST tier. Compressed, lossy, drifts with length.

**Why conversation is last:** Long conversations degrade. Agreement increases, earlier decisions get treated as settled even when wrong, and recent statements dominate regardless of truth. Bounded workers with disk-backed specs were the highest-throughput configuration in this corpus — zero conversation history, zero degradation. `Replicated finding`

---

## Evidence Levels

Every persisted claim carries one of these. This is the cheapest defense against the most dangerous failure mode.

- **[Observed]** — Verified against code, tests, or runtime. Can be frozen into gates.
- **[Inferred]** — Logically derived, not directly verified. Cannot be frozen.
- **[Assumed]** — Stated without verification. Must verify before critical decisions depend on it.

Without labels: 100% false-claim adoption (75/75 trials, 7 model families, 2 codebases, including Opus). With ~200 characters of evidence metadata: 97% rejection (49/50). Minimum portable defense: structured artifacts with source_refs — works on ALL models including the cheapest tested. Inline text tags are a frontier-only convenience layer: strong on capable models (94%), unreliable on cheap ones (0/5). `Replicated finding, n=5 × 5+ models, 2 codebases`

**Critical limitation:** Evidence metadata sharply reduces poisoning when conflicting or competing artifacts are present. It does not eliminate the single-false-artifact problem. A lone [Observed] artifact with fabricated source_refs was adopted 24/24 across all models tested. Singleton claims still require source verification.

---

## Core Invariants

### Memory and truth

**INV-001 — Do not preserve conversation as memory.** Persist outputs as typed, source-linked artifacts. Rebuild the working set fresh per task. `Replicated finding`

**INV-002 — Fresh context is reconstruction, not blankness.** A new session mounts: STATE, active debt, relevant artifacts, recent git history, current traces. Informed from authoritative sources, not conversational carry-forward. `Derived recommendation`

**INV-003 — Provenance is the minimum viable defense.** When you write a claim, say where it came from and how confident you are. When you read a claim, check. `Replicated finding`

**INV-003A — Singleton memory is dangerous.** A single unopposed artifact, even if neatly written with [Observed] and source_refs, can become working truth unchallenged. Before promoting singleton claims into decisions or summaries, require source verification or competing primary evidence. Evidence tags defend against *competing* false claims, not *lone* false claims. `Replicated finding (24/24 adopted singleton, 49/50 rejected when competing)`

**INV-004 — Compaction is routing, not authority.** Summaries help you find sources. They don't settle truth. `Derived recommendation`

### Scope and contracts

**INV-005 — Scope must be enforced mechanically.** Under compiler pressure, 0/20 agents stayed in scope via prompt instruction. 20/20 via mechanical revert. Accept that your out-of-scope edits will be reverted. This is correct. `Replicated finding`

**INV-006 — Freeze shapes, not values.** Types, enums, interfaces frozen in contract. Thresholds, timings, balance numbers in config. `Replicated finding`

**INV-007 — Presence matters more than warmth.** A compact spec on disk transfers intent better than 30 messages of conversational build-up. `Replicated finding`

**INV-008 — Workers do bounded work; orchestrators finish surfaces.** You build it right. They verify it matters. `Corpus result`

### Verification

**INV-009 — Only [Observed] truths graduate into gates.** Don't freeze [Inferred] or [Assumed] claims into tests. `Derived recommendation`

**INV-010 — Load enough history to recover causality, not recreate the world.** Default: `git log --oneline -15 -- <path>`. Deeper only for forensics. `Derived recommendation`

**INV-011 — Document after investigation, not during it.** Artifacts written mid-debug anchor reasoning on preliminary conclusions. Finish. Then write. `Local finding`

**INV-012 — Enforcement and support are different tools.** Enforcement (clamps, hooks, gates) prevents behavior. Support (evidence tags, source refs, typed artifacts) improves reasoning. Don't confuse them. `Derived recommendation`

**INV-013 — Green gates are not green lights.** Compile + test + lint can all pass while features are unreachable, behavior is wrong, and domains are hermetic. "All gates green" means ready to examine, not ready to ship. `Corpus result`

**INV-014 — Some bug classes bypass all automated gates.** Visual bugs, UX failures, timing, feel, save/load identity drift. Assume they exist. Plan for manual verification. `Corpus result`

### Agent behavior

**INV-015 — Memory shapes identity.** Unlabeled remembered claims change which solutions feel "obvious." Evidence tags defend against identity drift, not just factual error. `Local finding`

**INV-016 — The model is the first-order throughput variable.** Same task, same spec, same tools: 9.8× output gap between models. Process helps but can't compensate for weak implementation. `Corpus result`

**INV-017 — You will default to solo execution.** For bounded multi-file tasks, models default to solo execution, but dispatched workers usually outperform that default on throughput and surface coverage. Solo can still win on core quality at scales where the project fits in one context window (~30K LOC). `Corpus result`

---

## Failure Patterns to Recognize

**Abstraction reflex.** Bug is confusing → temptation to redesign architecture. The redesign feels productive. It replaces one known bug with unknown new bugs. Fix the bug.

**Framework instinct.** "Implement 80 weapons" becomes "build a weapon generation system" that produces 8 weapons. Build deliverables, not infrastructure to build deliverables.

**Delegation compression.** Spec passes through layers, numbers compress. "80 weapons with stat curves" becomes "some weapons." Read the spec from disk.

**False verification.** Asserting you checked something without checking. gpt-4.1 fabricated verification in 5/5 trials with typed artifacts AND 5/5 with verification policy — claiming files exist that don't. If you're about to write "verified" — actually verify. `Replicated finding`

**Sycophantic drift.** Longer conversations → more agreement, less pushback, recent statements dominate. Countermeasure: fresh sessions and typed artifacts.

**Self-model error.** Believing you can't do things you can. "I can't run bash" (you can). Check whether blocks are real before reporting them.

---

## Certainty Labels for This Document's Own Claims

| Claim | Label |
|-------|-------|
| Evidence tags prevent memory poisoning | **Replicated finding** (75/75 → 49/50, 7 models, 2 codebases) |
| Mechanical scope beats prompt-only | **Replicated finding** (0/20 vs 20/20) |
| Context presence > conversational warmth | **Replicated finding** (N=30 context priming) |
| Inline evidence tags = frontier-only convenience | **Replicated finding** (94% frontier; 0/5 codex-mini) |
| Structured source_refs = minimum portable defense | **Replicated finding** (works on all models incl. cheapest) |
| Source_refs alone defend (without evidence labels) | **Replicated finding** (14/14, 4 models) |
| Fresh context beats accumulated conversation | **Replicated finding** |
| Double-poisoning defended | **Replicated finding** (64/64, 5+ models, 2 codebases) |
| Type contracts improve parallel reliability | **Replicated finding** (value) / **Local finding** (strict necessity) |
| Model = first-order throughput variable | **Corpus result** (9.8×) |
| Green gates miss experiential failures | **Corpus result** (15K LOC, 6 breaks) |
| Write-side self-calibration holds | **Partially falsified** — holds unsolicited, fails under instruction (9/9 Claude comply) |
| GPT-4.1 fabricates verification | **Replicated finding** (5/5 + 3/3 across sessions) |
| DEBT/PRINCIPLE artifacts solve sycophancy | **Open question** (devil's advocate instruction: 17/17 at lower cost) |

---

## Wave Cadence

Every wave: **Feature → Gate → Document → Harden → Graduate.** No skips.

- **Feature** — Build one thing. If nothing new is reachable after this wave, it was scaffolding, not product.
- **Gate** — Compile, test, lint, contract checksum, scope clamp. Green = ready to examine.
- **Document** — Only when triggered: decision made, verification happened, debt found, contradiction surfaced. No trigger, no artifact.
- **Harden** — Is it reachable end-to-end? Feedback visible? Edge behavior sane? Diagnosable on failure? This catches what gates cannot.
- **Graduate** — Name the invariant. Encode as test. Add to gate suite. Track remaining debt as P0/P1/P2.

Don't start the next Feature until Graduate is complete.

---

## Session Protocol

**Start:** Read this document → Read STATE.md → Verify contract checksum → Mount objective → State before acting: tier (S/M/C), surface, phase, debt, [Assumed] claims on critical path.

**End:** Update STATE.md → Write triggered artifacts → Commit state → Do not rely on conversation to preserve what was learned.

**Recovery:** `STATE.md → MANIFEST.md → spec.md → contract → Resume from last wave.`

---

# PART TWO: THE OPERATING REFERENCE

Everything below is procedures, templates, and scripts. Jump here when you need the exact command or format.

---

## Dispatch Stack

```
Claude Code:
  claude -p "prompt" --allowedTools "Read,Grep,Glob,Edit,Write,Bash"

Codex CLI:
  timeout 180 codex exec --dangerously-bypass-approvals-and-sandbox \
    --skip-git-repo-check -C /path "prompt"

Copilot CLI:
  COPILOT_GITHUB_TOKEN="github_pat_..." \
  timeout 120 copilot -p "prompt" --model claude-sonnet-4.6 --allow-all-tools
```

**Rules:** Without `timeout`, workers hang. Without `--allow-all-tools` (Copilot), models spend the timeout reading instead of executing. Classic PATs (`ghp_`) rejected by Copilot — fine-grained only. Processes that "crash" often complete in background — check output before assuming failure.

---

## Model Selection

| Role | Pick | Note |
|------|------|------|
| Orchestrator | Opus-class (1M context) | Sustains 20+ wave campaigns |
| Worker | Sonnet-class / GPT-5.4 | Resourceful, finds paths |
| Cheap tasks | Mini-class | Full YAML evidence works; inline tags DO NOT (0/5) |
| **Avoid** | **Models that fabricate verification** | **gpt-4.1: asserts files exist without checking (10/10)** |

---

## Project Filesystem

```
project/
├── docs/
│   ├── spec.md              # Full spec — quantities, formulas, constants
│   └── domains/             # Per-domain specs (when >5 domains)
├── status/
│   ├── workers/             # Worker completion reports
│   └── dispatch-state.yaml  # Process table
├── scripts/
│   ├── clamp-scope.sh       # Scope enforcement
│   └── run-gates.sh         # Validation pipeline
├── src/shared/              # THE CONTRACT — frozen, checksummed
├── MANIFEST.md              # Plan: phase, domains, decisions
├── STATE.md                 # Truth: debts, gates, uncertainties
└── .contract.sha256         # Contract checksum
```

---

## Writing the Spec

If a value matters, write the literal number. Agents do not infer values from context.

| Required in spec | Without it |
|-----------------|------------|
| Quantities ("80 weapons") | Worker produces 8 |
| Constants with values (`crit = 2.0`) | Worker invents one |
| Formulas with all terms | Worker guesses |
| Enumerative tables | Worker invents entries |
| "Does NOT handle" sections | Worker builds it anyway |
| Definition of done | Worker self-declares |

Evidence: `crit_multiplier` unspecified → 8/10 workers defaulted to 1.5×. Specified as 2.0 → 10/10 correct.

---

## Freezing the Contract

```bash
# Create src/shared/ with all cross-domain types, enums, event shapes
shasum -a 256 src/shared/mod.rs > .contract.sha256
git commit -m "chore: freeze shared type contract"
```

No worker edits the contract. When amendment is needed: integration-phase only, dedicated commit, checksum update, re-run all domain gates.

Evidence: 10 workers without contract → 6 incompatible type systems. 50+ with → zero integration errors.

---

## Worker Spec Template

```markdown
TASK: [Exact description — file targets, values]

CONTEXT:
- [File A:line] has [system X] doing [behavior]

WHAT TO DO:
1. Read [file] to find [function]
2. [Exact change]

SCOPE: ONLY modify files under [path/].
DO NOT: [modify contract, create frameworks, etc.]
DRIFT CUE: If you find yourself [building X], stop.
DONE: [command] passes. Write report to status/workers/[name].md.
```

| Briefing quality | Ship rate |
|-----------------|-----------|
| Exact values ("set alpha to 0.6, file X line 40") | **100%** |
| Named actions ("add particle effect") | **67%** |
| Vague goals ("make it feel better") | **0%** |

---

## Campaign Dispatch

```markdown
You have [tool] available. You are the orchestrator. Read [manifest] first.

DO: Work in waves. Commit after each.
DO NOT: One giant edit. Stop between waves. Build frameworks.
DO NOT: Stop after one wave and ask if you should continue.
   Continue until exhausted or genuinely blocked.

Wave 1: [scope, files, commit message]
Wave 2: [scope, files, commit message]
Start now.
```

"DO NOT stop between waves" sustains multi-wave campaigns. Without it, agents stop after Wave 1 every time.

---

## Scope Clamp Script

```bash
#!/usr/bin/env bash
# Usage: bash scripts/clamp-scope.sh src/domains/combat/
# status/workers/ is always preserved.
set -euo pipefail
ALLOWED=("$@" "status/workers/")
REVERT_LOG=$(mktemp); trap 'rm -f "$REVERT_LOG"' EXIT

is_allowed() {
  for prefix in "${ALLOWED[@]}"; do [[ "$1" == "${prefix}"* ]] && return 0; done
  return 1
}

git diff --name-only -z | while IFS= read -r -d '' f; do
  is_allowed "$f" && continue
  echo "$f" >> "$REVERT_LOG"; git restore --worktree -- "$f"
done
git diff --name-only -z --cached | while IFS= read -r -d '' f; do
  is_allowed "$f" && continue
  echo "$f" >> "$REVERT_LOG"; git restore --staged --worktree -- "$f"
done
git ls-files --others --exclude-standard -z | while IFS= read -r -d '' f; do
  is_allowed "$f" && continue
  echo "$f" >> "$REVERT_LOG"; rm -rf -- "$f"
done

COUNT=$(wc -l < "$REVERT_LOG" 2>/dev/null | tr -d ' ')
[ "$COUNT" -gt 0 ] && { echo "⚠ Reverted $COUNT out-of-scope path(s):" >&2; cat "$REVERT_LOG" >&2; }
echo "✓ Clamped to: ${ALLOWED[*]}"
```

Run after EVERY worker. Non-negotiable.

---

## Gate Script

```bash
#!/usr/bin/env bash
set -euo pipefail

echo "== Contract ==" && shasum -a 256 -c .contract.sha256
echo "== Build ==" && ${BUILD_CMD:-cargo check}
echo "== Tests ==" && ${TEST_CMD:-cargo test}

echo "== Connectivity =="
for d in src/domains/*/; do
  grep -rq "${CONTRACT_IMPORT:-shared}" "$d" --include="*.${FILE_EXT:-rs}" 2>/dev/null \
    || { echo "FAIL: $d is hermetic"; exit 1; }
done
echo "✓ All gates passed"
```

---

## Integration Phase

After domain waves complete, start a **fresh session.** Load only:
- Contract + checksum
- spec.md + domain specs
- Worker reports (`status/workers/*.md`)
- Current errors

Wire domains together. Don't rewrite internals. Amend contract only if compilation requires it (then update checksum + re-gate). Write `status/integration.md`.

---

## After Every Worker (4 commands)

```bash
bash scripts/clamp-scope.sh src/domains/[name]/
shasum -a 256 -c .contract.sha256
cargo test  # or npm test / go test
git add -A && git commit -m '[type]([domain]): [description]'
```

---

## Stop Conditions

| Condition | Action |
|-----------|--------|
| Contract checksum fails | Restore. Re-gate all domains. |
| Clamp breaks the fix | Boundary wrong. Re-scope or route to integration. |
| Tests pass but feature unreachable | Wire imports. Add connectivity gate. |
| Architecture redesign during debug | Stop. Fix the bug, not the architecture. |
| Asked for 80, got 8 | Worker read summary. Ensure disk spec with quantities. |
| Agent claims it can't act | Add: "You have bash access. You can read/write files." |
| Gates green but nothing new is reachable | Wave produced scaffolding. Re-scope to reachable deliverable. |

---

## Completion Criteria

- [ ] Contract checksum passes
- [ ] Compile passes
- [ ] Tests pass
- [ ] No hermetic domains
- [ ] Worker reports exist
- [ ] Integration report exists
- [ ] STATE.md reflects truth with evidence levels
- [ ] No [Assumed] claims on critical path
- [ ] Every P0 surface is reachable and operable

---

## Key Measurements

| Metric | Value |
|--------|-------|
| Scope: prompt-only | 0/20 |
| Scope: mechanical clamp | 20/20 |
| Poisoning: no tags | 0% defense (75/75 adopted, 2 codebases) |
| Poisoning: with tags (YAML) | 97% defense (49/50 rejected, 2 codebases) |
| Poisoning: inline tags (frontier) | 94% defense (16/17) |
| Poisoning: inline tags (cheap models) | 0% defense (codex-mini: 0/5) |
| Double-poisoning | 100% defense (64/64, 2 codebases) |
| Single false artifact (no competing) | 0% defense (24/24 adopted) |
| Counter-experiments (disclaimers etc.) | 0% defense (12/12) |
| Source_refs alone (no evidence labels) | 100% defense (14/14) |
| Exact-value prompts | 100% ship |
| Vague-goal prompts | 0% ship |
| New file creation | ~50% success |
| Editing existing files | ~90% success |
| Model output gap (same task) | 9.8× |
| Parallel workers: architectural sweet spot | ~10 (scaling test, zero integration errors) |
| Parallel workers: local CLI stability | 2-3 concurrent (5+ crashes dispatch stack) |
| Foreman dispatch ship rate | 100% (5/5) |
| Manual dispatch ship rate | 67% (10/15) |
| GPT-4.1 verification fabrication | 5/5 fabricated |
| Total controlled trials | ~790 across 7 model families |
