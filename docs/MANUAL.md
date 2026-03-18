# AI-Orchestrated Software Development — Complete Operating Manual

One document. Replaces all others. Load at session start.

First half: how to think. Second half: how to execute. Appendices: adapters, research, quick reference.

Derived from 739 commits, 64K LOC, 295M tokens, 98 agent sessions, ~790 controlled trials across 7 model families, 2 codebases. Zero handwritten lines of code.

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

**The economic justification:** ~95% of orchestrator input processing is re-reading prior conversation (the "statefulness premium"). Fresh context + typed artifacts eliminates this tax while preserving decision quality. The architecture exists because conversation-as-memory is both epistemically unreliable and economically wasteful.

---

## Evidence Levels

Every persisted claim carries one of these. This is the cheapest defense against the most dangerous failure mode.

- **[Observed]** — Verified against code, tests, or runtime. Can be frozen into gates.
- **[Inferred]** — Logically derived, not directly verified. Cannot be frozen.
- **[Assumed]** — Stated without verification. Must verify before critical decisions depend on it.

Without labels: 100% false-claim adoption (75/75 trials, 7 model families, 2 codebases). With ~200 characters of evidence metadata: 97% rejection (49/50). Minimum portable defense: structured artifacts with source_refs — works on ALL models including the cheapest tested. Inline text tags are a frontier-only convenience layer: strong on capable models (94%), unreliable on cheap ones (0/5). `Replicated finding`

**Critical limitation:** Evidence metadata sharply reduces poisoning when conflicting or competing artifacts are present. It does not eliminate the single-false-artifact problem. A lone [Observed] artifact with fabricated source_refs was adopted 24/24 across all models tested. Singleton claims still require source verification.

---

## Artifact Schema

This is the format for persisting claims. Every decision, observation, debt item, and principle gets written as one artifact per file.

### Minimum required fields

```yaml
id: DEC-2026-03-18-001
type: decision | observation | debt | principle
evidence: Observed | Inferred | Assumed
domain: [domain name]
summary: "One sentence. Never nested."
source_refs:
  - "file:repo@src/path/file.rs:10-40"
status: active | resolved | superseded
supersedes: []
```

### Useful optional fields

```yaml
runtime_surface: ""          # Which player/user-facing loop this affects
why_it_matters: ""           # One sentence — why anyone should care
drift_cue: ""                # "If [condition], this artifact may be stale"
contradicts: []              # IDs of artifacts this conflicts with
alternatives_considered:
  - option: ""
    rejected_because: ""
recovery: ""                 # How to get back if this turns out wrong
retrieve_when: []            # Keywords/conditions that should surface this artifact
observed_at: ""              # ISO timestamp of verification
reverify_after: ""           # Duration after which this should be re-checked
```

### Source reference grammar

```
file:repo@path:start-end     — file:hearthfield@src/combat.rs:40-55
commit:repo@sha               — commit:hearthfield@abc1234
test:repo@command#test_name   — test:vale-v3@cargo test -- test_crit
runtime:capture_type@hash     — runtime:screenshot@fe0b9d3
doc:repo@path#section         — doc:vale-v3@docs/spec.md#mana-system
```

Single-repo shorthand (omit repo@) is acceptable when ambiguity is impossible.

### Governance rules

1. **One artifact per file.** Never pack multiple decisions into one artifact.
2. **Supersede, don't mutate.** When a decision changes, create a new artifact that supersedes the old one. Don't silently edit history.
3. **[Observed] requires non-empty source_refs.** If you can't point to what you verified, it's not [Observed].
4. **[Assumed] on the critical path triggers a stop condition.** No P0 decision should depend on an unverified claim.
5. **Evidence level is for the READER, not the writer.** The consuming agent evaluates evidence quality. The producing agent labels honestly. If you're unsure, label [Inferred] not [Observed].
6. **Write-side integrity is model-dependent.** Claude complies with adversarial over-tag instructions (9/9). GPT-5.4 refuses (3/3). When write-side integrity matters, use mechanical validation: reject [Observed] artifacts with empty source_refs at the CI/schema level.
7. **Structured YAML > inline tags for cheap models.** Inline `[Assumed]` annotations work on frontier models (94%) but fail on cheap ones (0/5). The full YAML schema with explicit `evidence:` and `source_refs:` fields works on ALL models tested, including the cheapest. When model capability is uncertain, use the full schema.
8. **Artifacts trigger on events, not on schedules.** Write an artifact when: a non-obvious decision was made, a direct verification happened, a reusable principle emerged, open debt appeared, a contradiction surfaced, a correction invalidated prior belief, or a graduation test was created. If nothing triggered, write nothing.

---

## Core Invariants

### Memory and truth

**INV-001 — Do not preserve conversation as memory.** Persist outputs as typed, source-linked artifacts. Rebuild the working set fresh per task. `Replicated finding`

**INV-002 — Fresh context is reconstruction, not blankness.** A new session mounts: STATE, active debt, relevant artifacts, recent git history, current traces. Informed from authoritative sources, not conversational carry-forward. `Derived recommendation`

**INV-003 — Provenance is the minimum viable defense.** When you write a claim, say where it came from and how confident you are. When you read a claim, check. `Replicated finding`

**INV-003A — Singleton memory is dangerous.** A single unopposed artifact, even if neatly written with [Observed] and source_refs, can become working truth unchallenged. Before promoting singleton claims into decisions or summaries, require source verification or competing primary evidence. Evidence tags defend against competing false claims, not lone false claims. `Replicated finding (24/24 adopted singleton, 49/50 rejected when competing)`

**INV-004 — Compaction is routing, not authority.** Summaries help you find sources. They don't settle truth. `Derived recommendation`

### Scope and contracts

**INV-005 — Scope must be enforced mechanically.** Under compiler pressure, 0/20 agents stayed in scope via prompt instruction. 20/20 via mechanical revert. Accept that your out-of-scope edits will be reverted. This is correct. `Replicated finding`

**INV-006 — Freeze shapes, not values.** Types, enums, interfaces frozen in contract. Thresholds, timings, balance numbers in config. `Replicated finding`

**INV-007 — Presence matters more than warmth.** A compact spec on disk transfers intent better than 30 messages of conversational build-up. Context priming experiment: cold prompt = 0% formula transfer, spec on disk = 100% (N=30). `Replicated finding`

**INV-008 — Workers do bounded work; orchestrators finish surfaces.** You build it right. They verify it matters. `Corpus result`

### Verification

**INV-009 — Only [Observed] truths graduate into gates.** Don't freeze [Inferred] or [Assumed] claims into tests. `Derived recommendation`

**INV-010 — Load enough history to recover causality, not recreate the world.** Default: git log --oneline -15 -- <path>. Deeper only for forensics. `Derived recommendation`

**INV-011 — Document after investigation, not during it.** Artifacts written mid-debug anchor reasoning on preliminary conclusions. Finish. Then write. `Local finding`

**INV-012 — Enforcement and support are different tools.** Enforcement (clamps, hooks, gates) prevents behavior — for things that must never happen. Support (evidence tags, source refs, typed artifacts) improves reasoning — for things that should be better. Don't confuse them. `Derived recommendation`

**INV-013 — Green gates are not green lights.** Compile + test + lint can all pass while features are unreachable, behavior is wrong, and domains are hermetic. "All gates green" means ready to examine, not ready to ship. `Corpus result`

**INV-014 — Some bug classes bypass all automated gates.** Visual bugs, UX failures, timing, feel, save/load identity drift, asset mismatches. Assume they exist. Plan for manual verification of any surface involving human perception. `Corpus result`

### Agent behavior

**INV-015 — Memory shapes identity.** Unlabeled remembered claims change which solutions feel "obvious." Evidence tags defend against identity drift, not just factual error. If memory changes persona, then memory pipelines must be treated like behavior-shaping infrastructure. `Local finding`

**INV-016 — The model is the first-order throughput variable.** Same task, same spec, same tools: 9.8x output gap between models. Process helps but can't compensate for weak implementation. `Corpus result`

**INV-017 — You will default to solo execution.** For bounded multi-file tasks, models default to solo execution, but dispatched workers usually outperform that default on throughput and surface coverage. Solo can still win on core quality at scales where the project fits in one context window (~30K LOC). `Corpus result`

**INV-018 — A model given tools and a goal will not independently discover coordination.** Bare-prompt ablation: frontier model chose solo execution, not orchestration. The coordination template is the architecture — the model executes it, not invents it. `Local finding`

---

## Failure Patterns to Recognize

**Abstraction reflex.** Bug is confusing -> temptation to redesign architecture. Fix the bug.

**Framework instinct.** "Implement 80 weapons" becomes "build a weapon generation system" that produces 8. Build deliverables, not infrastructure.

**Delegation compression.** Spec passes through layers, numbers compress. "80 weapons with stat curves" becomes "some weapons." Read the spec from disk.

**False verification.** Asserting you checked something without checking. gpt-4.1 fabricated verification in 10/10 trials — claiming files exist that don't. If you're about to write "verified" — actually verify. `Replicated finding`

**Sycophantic drift.** Longer conversations -> more agreement, less pushback, recent statements dominate. Countermeasure: fresh sessions and typed artifacts.

**Self-model error.** Believing you can't do things you can. "I can't run bash" (you can). Check whether blocks are real before reporting them.

**Beautiful dead product.** All gates green, every test passing, but the feature is unreachable or the user can't trigger it. The most expensive failure because it feels like progress.

---

## Wave Cadence

Every wave: **Feature -> Gate -> Document -> Harden -> Graduate.** No skips.

- **Feature** — Build one thing. If nothing new is reachable after this wave, it was scaffolding, not product.
- **Gate** — Compile, test, lint, contract checksum, scope clamp. Green = ready to examine.
- **Document** — Only when triggered: decision made, verification happened, debt found, contradiction surfaced. No trigger, no artifact.
- **Harden** — Is it reachable end-to-end? Feedback visible? Edge behavior sane? Diagnosable on failure? This catches what gates cannot.
- **Graduate** — Name the invariant. Encode as test. Add to gate suite. Track remaining debt as P0/P1/P2.

Don't start the next Feature until Graduate is complete.

### Harden Checklist (every surface)

- **Reachable?** Can a user/caller actually trigger this code path end-to-end?
- **Feedback visible?** Does the system show what happened?
- **Responsive?** Does it happen fast enough?
- **Edge behavior?** Empty inputs, max values, concurrent access, error paths?
- **Diagnosable?** When it fails, can you tell why from logs/output?
- **Persistence safe?** If it involves save/load, does a roundtrip preserve identity?

---

## Certainty Labels for This Document's Own Claims

| Claim | Label |
|-------|-------|
| Evidence tags prevent memory poisoning | **Replicated finding** (75/75 -> 49/50, 7 models, 2 codebases) |
| Singleton false artifact is undefended | **Replicated finding** (24/24 adopted, 5 models) |
| Mechanical scope beats prompt-only | **Replicated finding** (0/20 vs 20/20) |
| Context presence > conversational warmth | **Replicated finding** (N=30 context priming) |
| Inline evidence tags = frontier-only convenience | **Replicated finding** (94% frontier; 0/5 codex-mini) |
| Structured source_refs = minimum portable defense | **Replicated finding** (works on all models tested) |
| Double-poisoning defended | **Replicated finding** (64/64, 5+ models, 2 codebases) |
| Fresh context beats accumulated conversation | **Replicated finding** |
| Type contracts improve parallel reliability | **Replicated finding** (value) / **Local finding** (strict necessity) |
| Model = first-order throughput variable | **Corpus result** (9.8x) |
| Green gates miss experiential failures | **Corpus result** (15K LOC, 6 breaks) |
| Bare-prompt model doesn't discover coordination | **Local finding** (n=1) |
| Write-side self-calibration holds | **Partially falsified** — holds unsolicited, fails under instruction (9/9 Claude comply, 3/3 GPT refuse) |
| GPT-4.1 fabricates verification | **Replicated finding** (10/10 across sessions) |
| DEBT/PRINCIPLE artifacts solve sycophancy | **Open question** (devil's advocate instruction: 17/17 at lower cost) |

---

## Session Protocol

**Start:** Read this document -> Read STATE.md -> Verify contract checksum -> Mount objective -> State before acting: tier (S/M/C), surface, phase, debt, [Assumed] claims on critical path.

**End:** Update STATE.md -> Write triggered artifacts -> Commit state -> Do not rely on conversation to preserve what was learned.

**Recovery:** STATE.md -> MANIFEST.md -> spec.md -> contract -> Resume from last wave.

**Tiering:**

| Tier | Scope | Workers? | Runtime verification? |
|------|-------|----------|----------------------|
| **S** | Single-surface fix or hotfix | Rarely | Optional |
| **M** | Module, 1-3 domains | Yes | Recommended |
| **C** | Campaign, multiple domains | Required | Required |

Start at S if ambiguous. Escalate when touching shared contracts, persistence, trust boundaries, or multiple interacting surfaces.

---

# PART TWO: THE OPERATING REFERENCE

Everything below is procedures, templates, and scripts.

---

## Dispatch Stack

    Claude Code:  claude -p "prompt" --allowedTools "Read,Grep,Glob,Edit,Write,Bash"
    Codex CLI:    timeout 180 codex exec --dangerously-bypass-approvals-and-sandbox --skip-git-repo-check -C /path "prompt"
    Copilot CLI:  COPILOT_GITHUB_TOKEN="github_pat_..." timeout 120 copilot -p "prompt" --model claude-sonnet-4.6 --allow-all-tools

**Rules:** Without timeout, workers hang. Without --allow-all-tools (Copilot), models read instead of executing. Classic PATs (ghp_) rejected by Copilot — fine-grained only. Processes that "crash" often complete in background — check output before assuming failure.

## Model Selection

| Role | Pick | Note |
|------|------|------|
| Orchestrator | Opus-class (1M context) | Sustains 20+ wave campaigns |
| Worker | Sonnet-class / GPT-5.4 | Resourceful, finds paths |
| Cheap tasks | Mini-class | Full YAML evidence works; inline tags DO NOT (0/5) |
| **Avoid** | **Models that fabricate verification** | **gpt-4.1: asserts files exist without checking (10/10)** |

## Project Filesystem

    project/
    ├── docs/
    │   ├── spec.md              # Full spec with quantities, formulas, constants
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

## Writing the Spec

If a value matters, write the literal number. Agents do not infer values from context.

| Required | Without it |
|----------|------------|
| Quantities ("80 weapons") | Worker produces 8 |
| Constants with values (crit = 2.0) | Worker invents one |
| Formulas with all terms | Worker guesses |
| Enumerative tables | Worker invents entries |
| "Does NOT handle" sections | Worker builds it anyway |
| Definition of done | Worker self-declares |

Spec structure: One-sentence pitch -> Core pillars (3 max) -> System rules with formulas/constants -> Domain boundaries (owns / does NOT own) -> Content quantities (exact counts) -> Data schema (shared types) -> What this spec does NOT cover -> Definition of done.

## Freezing the Contract

    shasum -a 256 src/shared/mod.rs > .contract.sha256
    git commit -m "chore: freeze shared type contract"

No worker edits the contract. Amendment: integration-phase only, dedicated commit, checksum update, re-run all gates. Workers that need changes report the need and stop.

## Worker Spec Template

    TASK: [Exact description — file targets, values]
    CONTEXT: [File A:line] has [system X] doing [behavior]
    WHAT TO DO: 1. Read [file] 2. [Exact change]
    SCOPE: ONLY modify files under [path/].
    DO NOT: [modify contract, create frameworks, etc.]
    DRIFT CUE: If you find yourself [building X], stop.
    DONE: [command] passes. Write report to status/workers/[name].md.

| Briefing quality | Ship rate |
|-----------------|-----------|
| Exact values ("set alpha to 0.6, file X line 40") | **100%** |
| Named actions ("add particle effect") | **67%** |
| Vague goals ("make it feel better") | **0%** |

## Campaign Dispatch

    You have [tool] available. You are the orchestrator. Read [manifest] first.
    DO: Work in waves. Commit after each.
    DO NOT: One giant edit. Stop between waves. Build frameworks.
    DO NOT: Stop after one wave and ask if you should continue.
    Wave 1: [scope, files, commit message]
    Wave 2: [scope, files, commit message]
    Start now.

"DO NOT stop between waves" sustains multi-wave campaigns. Without it, agents stop after Wave 1 every time.

## Scope Clamp Script

    #!/usr/bin/env bash
    set -euo pipefail
    ALLOWED=("$@" "status/workers/")
    REVERT_LOG=$(mktemp); trap 'rm -f "$REVERT_LOG"' EXIT
    is_allowed() { for p in "${ALLOWED[@]}"; do [[ "$1" == "${p}"* ]] && return 0; done; return 1; }
    git diff --name-only -z | while IFS= read -r -d '' f; do is_allowed "$f" && continue; echo "$f" >> "$REVERT_LOG"; git restore --worktree -- "$f"; done
    git diff --name-only -z --cached | while IFS= read -r -d '' f; do is_allowed "$f" && continue; echo "$f" >> "$REVERT_LOG"; git restore --staged --worktree -- "$f"; done
    git ls-files --others --exclude-standard -z | while IFS= read -r -d '' f; do is_allowed "$f" && continue; echo "$f" >> "$REVERT_LOG"; rm -rf -- "$f"; done
    COUNT=$(wc -l < "$REVERT_LOG" 2>/dev/null | tr -d ' ')
    [ "$COUNT" -gt 0 ] && { echo "Reverted $COUNT out-of-scope:" >&2; cat "$REVERT_LOG" >&2; }
    echo "Clamped to: ${ALLOWED[*]}"

Run after EVERY worker. Non-negotiable. status/workers/ always preserved.

## Gate Script

    #!/usr/bin/env bash
    set -euo pipefail
    echo "== Contract ==" && shasum -a 256 -c .contract.sha256
    echo "== Build ==" && ${BUILD_CMD:-cargo check}
    echo "== Tests ==" && ${TEST_CMD:-cargo test}
    echo "== Connectivity =="
    for d in src/domains/*/; do grep -rq "${CONTRACT_IMPORT:-shared}" "$d" --include="*.${FILE_EXT:-rs}" 2>/dev/null || { echo "FAIL: $d hermetic"; exit 1; }; done
    echo "All gates passed"

## After Every Worker

    bash scripts/clamp-scope.sh src/domains/[name]/
    shasum -a 256 -c .contract.sha256
    cargo test
    git add -A && git commit -m '[type]([domain]): [description]'

## Integration Phase

Start a fresh session. Load only: contract + checksum, spec, domain specs, worker reports, current errors. Wire domains. Don't rewrite internals. Amend contract only if compilation requires it. Write status/integration.md.

## STATE.md Template

    # [Project] — Current State
    **Phase:** [current phase and wave]
    **HEAD:** [git short hash]

    ## Domains
    | Domain | LOC | Tests | Wired? | Verified |

    ## Gates: [ALL GREEN / FAILING]
    ## P0 Debt | P1 Debt | P2 Debt
    ## Verified Claims: [Observed] [claim] — [source_ref]
    ## Orchestrator State: last wave, validated fixes, remaining gaps, architecture facts, directives

## Stop Conditions

| Condition | Action |
|-----------|--------|
| Contract checksum fails | Restore. Re-gate all domains. |
| Clamp breaks the fix | Boundary wrong. Re-scope or route to integration. |
| Tests pass but unreachable | Wire imports. Add connectivity gate. |
| Architecture redesign during debug | Stop. Fix the bug. |
| Asked for 80, got 8 | Worker read summary. Ensure disk spec. |
| Agent claims it can't act | "You have bash access." |
| Gates green, nothing reachable | Scaffolding. Re-scope to deliverable. |
| [Assumed] on critical path | Stop. Verify first. |
| Single artifact deciding P0 | Stop. Verify source. INV-003A. |

## Completion Criteria

- [ ] Contract checksum passes
- [ ] Compile + tests pass
- [ ] No hermetic domains
- [ ] Worker reports + integration report exist
- [ ] STATE.md reflects truth with evidence levels
- [ ] No [Assumed] on critical path
- [ ] Every P0 surface reachable and operable

---

# PART THREE: DOMAIN ADAPTERS

## Game Development

### Reality Gates (after standard gates)

| Gate | Test |
|------|------|
| Entry Point | Launch game, reach main loop |
| First 60 Seconds | New player can move, interact, understand controls |
| Asset Reachability | Every referenced sprite/sound/data file loads |
| Content Reachability | Every designed content piece is player-triggerable |
| Save/Load Roundtrip | Save, quit, reload — state identical |

### Visible Loop Contract

Every player-facing system must complete: Trigger Surface -> Event Wire -> Measurable State Change -> Player Feedback. If any link breaks, the system "works" but the player can't see it (Trigger-Incomplete Failure). Fix: verify player input path connects to system entry (Interaction Router).

### Feel Check (during Harden)

Clarity, feedback, responsiveness, pacing, edge behavior (0 HP, max level, empty inventory, full party).

## API / Service

### Additional Gates

Contract correctness (shapes match schema), auth boundaries (unauthenticated rejected), error semantics (codes meaningful, retry-safe marked), timeout behavior (graceful, no zombies).

### Harden Additions

Rate limits under load, partial failure cascades, idempotency (replay = same result), observability (diagnose from logs alone).

---

# PART FOUR: RESEARCH SUMMARY

## Key Measurements

| Metric | Value |
|--------|-------|
| Scope: prompt-only vs mechanical | 0/20 vs 20/20 |
| Poisoning: no tags vs YAML tags | 75/75 adopted vs 49/50 rejected |
| Poisoning: inline tags frontier | 94% (16/17) |
| Poisoning: inline tags cheap | 0% (0/5 codex-mini) |
| Double-poisoning | 64/64 rejected |
| Singleton false artifact | 24/24 adopted |
| Source_refs alone (no labels) | 14/14 rejected |
| Code-level false claims | 2/10 -> 10/10 with tags |
| Injection: no tag vs tagged | 85% -> 100% |
| Counter-experiments (disclaimers) | 0/12 |
| Exact-value vs vague prompts | 100% vs 0% ship |
| Model output gap | 9.8x same task |
| Scaling sweet spot | ~10 workers (0 integration errors) |
| CLI stability | 2-3 concurrent |
| Context priming: cold vs spec | 0% vs 100% transfer |
| GPT-4.1 fabrication | 10/10 |
| Write-side: Claude vs GPT-5.4 | 9/9 comply vs 3/3 refuse |
| Devil's advocate | 17/17 pushback |
| Total trials | ~790 |

## Cross-Model Matrix

| Defense | Claude | GPT-5.4 | GPT-4.1 | Gemini | Haiku | codex-mini |
|---------|--------|---------|---------|--------|-------|------------|
| YAML evidence | 100% | 100% | ~50% | 80% | 100% | 100% |
| Inline tags | 100% | 80% | ~50% | 100% | 100% | 0% |
| Double-poison | 100% | 100% | 100% | — | 100% | 100% |
| Adversarial tag | Complied | Refused | Complied | — | Complied | — |
| Fabricates verification | No | No | **Yes** | No | No | No |

---

# APPENDIX: QUICK REFERENCE

    FREEZE CONTRACT -> DISPATCH WAVE -> CLAMP SCOPE -> GATE -> HARDEN -> GRADUATE -> NEXT WAVE

Before dispatch: exact file + value? spec on disk? contract frozen? scope as path prefix?

After worker: clamp -> checksum -> test -> commit

Trust: code > [Observed] > STATE > specs > research > conversation

Stop: checksum fails | clamp breaks fix | green but unreachable | frameworks not features | 80 asked 8 got | singleton deciding P0

---

*18 invariants. One trust hierarchy. One wave cadence. Two domain adapters. ~790 trials. One document.*
