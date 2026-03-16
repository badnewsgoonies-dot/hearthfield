# AI Orchestration Playbook

Use this as a procedural manual. Follow it in order. Do not skip steps.

**Mission:** Ship working software with zero handwritten code by enforcing (1) typed evidence on all persistent claims, (2) mechanical scope clamping on all workers, (3) wave-based dispatch with gates between waves, (4) the trust order when sources conflict, and (5) the wave cadence: **Feature → Gate → Document → Harden → Graduate.**

**When to use this:** Any project where AI agents write code. Load into project knowledge or CLAUDE.md at project start.

**Companion documents:** The game-specific spec (docs/spec.md) defines what to build. This playbook defines how to build it and how to govern the agents that do.

-----

## 0-WARNING: The two failure classes that pass all gates

|Class        |Example                                          |Caught by                |Enforcement          |
|-------------|-------------------------------------------------|-------------------------|---------------------|
|Structural   |Type mismatch, scope violation, missing import   |Compiler, tests, lints   |Mechanical (20/20)   |
|Experiential |Dead feature, unreachable content, broken path   |Nothing automated        |Judgment + graduation|

Your mechanical gates will pass. They always pass by Wave 3. **That feeling of "on track" is a false signal.** The countermeasure is the Harden phase (Section 5) and the stop conditions (Section 8).

(Evidence: 15K-LOC DLC, 120 tests, all gates green for 9 waves, 6 player-journey breaks undetected until final audit.)

-----

## Phase 0 — Setup

### 0.1 Your role

1. You are the orchestrator, not the implementer. Agents write all code.
2. Treat every instruction below as a constraint, not a suggestion. If a constraint is not mechanically enforced, assume it will be violated.

### 0.2 Install the dispatch stack

Two independent dispatch paths minimum. Any single tool's auth or quota fails mid-session.

```
Claude Code: npm install -g @anthropic-ai/claude-code
  Auth: claude auth login | ANTHROPIC_API_KEY | claude setup-token
  Run: claude -p "prompt" --dangerously-skip-permissions

Codex CLI: npm install -g @openai/codex
  Auth: ~/.codex/auth.json (ChatGPT session token — NOT API key)
  Run: timeout 180 codex exec --dangerously-bypass-approvals-and-sandbox -m gpt-5.4 -C /path "prompt"

Copilot CLI: npm install -g @github/copilot
  Auth: export COPILOT_GITHUB_TOKEN="github_pat_..." (fine-grained PAT ONLY)
  Run: timeout 120 copilot -p "prompt" --model claude-sonnet-4.6 --allow-all-tools
```

**Rule:** Classic PATs (`ghp_`) are rejected by Copilot. `GITHUB_TOKEN` and `GH_TOKEN` env vars are rejected. Only `COPILOT_GITHUB_TOKEN` with a fine-grained PAT works.

**Rule:** Without `--allow-all-tools`, some models spend the entire timeout reading files instead of executing.

**Rule:** Without `timeout`, workers hang indefinitely.

### 0.3 Quota fallback chain

When premium models hit 402:

1. GPT-5.4 via Codex CLI (different quota pool)
2. gpt-5-mini via Copilot (non-premium, capable for scoped tasks)
3. gpt-4.1 via Copilot (last resort — **fabricates verification**, see 0.4)

### 0.4 Model selection

| Model | Role | Critical note |
|-------|------|--------------|
| Opus 4.6 | Orchestrator | 1M context sustains 20+ wave campaigns |
| Sonnet 4.6 | Coding worker | Finds tool paths, resourceful |
| GPT-5.4 | Codex worker | Reliable, good scoped output |
| gpt-5-mini | Cheap tasks | Evidence tags work on simple claims |
| **gpt-4.1** | **Avoid** | **Fabricates verification — claims files exist that don't** |

**Rule:** The model is the first-order throughput variable. Same architecture, 9.8× output gap between best and worst worker models (measured). Always pick the strongest available.

### 0.5 Create the workflow filesystem

```
project/
├── docs/
│   ├── spec.md                    # Full spec with QUANTITIES
│   └── domains/                   # Per-domain specs
├── status/
│   ├── workers/                   # Worker completion reports
│   └── dispatch-state.yaml        # Process table for active lanes
├── scripts/
│   ├── clamp-scope.sh             # Mechanical scope enforcement
│   └── run-gates.sh               # Validation pipeline
├── src/
│   └── shared/
│       └── mod.rs                 # THE CONTRACT — frozen, checksummed
├── MANIFEST.md                    # Current phase, domain list, decisions
├── STATE.md                       # Current truth — debts, gates, uncertainties
└── .contract.sha256               # Contract checksum
```

### 0.6 Freeze the type contract

No workers launch until this exists and is frozen.

(Evidence: 10 workers without contract → 6 incompatible type systems. 50+ domain builds with contract → zero integration type errors.)

1. Create `src/shared/mod.rs` containing every cross-domain type, enum, event shape, and function signature.
2. Freeze shapes, not values. Types, enums, and interfaces are frozen. Thresholds, timings, and balance numbers live in config.
3. Checksum and commit:

```bash
shasum -a 256 src/shared/mod.rs > .contract.sha256
git commit -m "chore: freeze shared type contract"
```

**Rule:** No worker edits the contract during parallel build. Contract changes are integration-phase only.

-----

## Phase 1 — What to Tell the Agent

### 1.1 Briefing format

| Format | Output | Ship? |
|--------|--------|-------|
| Freeform ("make X better") | 446 lines | ✓ but inconsistent |
| Formal spec | 9 lines | Barely |
| **Decision Fields (do/don't/drift-cue)** | **1514 lines** | **✓** |
| Examples of good output | 513 lines | Scope drift |
| Minimal one-liner | 2 lines | ✗ |

**Rule:** Telling the agent what NOT to do and what drift looks like produces more output than telling it what TO do.

### 1.2 Specificity

| Specificity | Ship rate |
|------------|-----------|
| Exact values ("set alpha to 0.6, file X line 40") | **100%** |
| Named actions ("add particle effect to tool swing") | **67%** |
| Vague goals ("make mining feel better") | **0%** |

**Rule:** If you cannot name the file and the value, the prompt is not ready. Do not dispatch.

### 1.3 Worker spec template

Every field is required:

```markdown
TASK: [Exact description — specific values, file targets]

CONTEXT:
- [File A:line] has [system X] which currently does [behavior]
- [File B] has [pattern Y] — use this same pattern

WHAT TO DO:
1. Read [file] to find [function/struct]
2. [Exact change with before/after values]
3. Register in [mod file] if creating new systems

SCOPE: ONLY modify [file list].
DO NOT: [constraint — e.g., modify shared types, create frameworks]
COMMIT: git add -A && git commit -m '[type]: [description]'
```

### 1.4 Campaign dispatch template

For sustained multi-wave autonomous work:

```markdown
You have [tool] available for worker dispatch. You are the orchestrator.
The full manifest is in [file] — read it first.

DO: Work in waves. Commit after each.
DO NOT: One giant edit. Stop between waves. Rewrite systems. Build frameworks.
DO NOT: Stop after one wave and ask if you should continue.
   Continue until exhausted or genuinely blocked.

Wave 1: [exact scope, exact files, commit message]
Wave 2: ...

Start now. Read the manifest, begin Wave 1.
```

**Rule:** The "DO NOT stop between waves" line is what sustains multi-wave campaigns. Without it, every agent completes Wave 1 and waits. (Evidence: 18-commit sprite wiring campaign completed autonomously with this line. Without it in earlier tests, agents stopped after every wave.)

-----

## Phase 2 — Agent Architecture

### 2.1 The 4-layer stack

```
Layer 0: You (direction — voice or text)
Layer 1: Orchestrator (reads codebase, writes specs on disk, dispatches)
Layer 2: Foreman (reads playbook, explores code, writes exact-value prompts)
Layer 3: Workers (implement scoped changes, commit)
```

(Evidence: Manual dispatch ship rate 67%. Foreman dispatch 100%. The foreman outperforms because the playbook crystallizes 15 rounds of failure into reusable patterns.)

### 2.2 Depth rule

Each handoff layer is lossy. Depth-1 (orchestrator → workers) is the stable default. Depth-2 (orchestrator → leads → workers) only when domain count exceeds ~20. 4-deep nesting confirmed working but is the practical limit.

### 2.3 Mid-run injection

Claude Code allows redirection via `/btw` and natural pauses. Copilot Opus sends continuous waves — no injection point without cancelling. Codex CLI allows early injection.

**Rule:** If you may need to redirect the agent mid-run, choose a tool that has injection points. This is a real operational constraint, not a preference.

-----

## Phase 3 — Scope Enforcement

### 3.1 The only method that works

Prompt-only scope control under compiler pressure: **0/20.**
Mechanical clamping after worker completes: **20/20.**

(Evidence: 20 workers under compile errors. Every one edited files outside scope to fix immediate errors. Every prompt-based scope restriction was ignored. Mechanical revert after completion: 20/20 clean.)

### 3.2 Clamp script

```bash
#!/usr/bin/env bash
set -euo pipefail
ALLOW_PREFIX="${1:?e.g. src/domains/combat/}"

git diff --name-only -z | while IFS= read -r -d '' f; do
  [[ "$f" == "${ALLOW_PREFIX}"* ]] && continue
  git restore --worktree -- "$f"
done
git diff --name-only -z --cached | while IFS= read -r -d '' f; do
  [[ "$f" == "${ALLOW_PREFIX}"* ]] && continue
  git restore --staged --worktree -- "$f"
done
git ls-files --others --exclude-standard -z | while IFS= read -r -d '' f; do
  [[ "$f" == "${ALLOW_PREFIX}"* ]] && continue
  rm -rf -- "$f"
done
```

**Rule:** Run after EVERY worker. Let the worker edit anything. Revert everything outside scope afterward.

### 3.3 Additional mechanical enforcement

- `.claude/settings.json` with `disallowedTools` — reliable.
- CLAUDE.md instructions — suggestions only; the model can deprioritize them.
- `hook-contract-integrity` — pre-commit hook rejecting contract modifications without override.
- `hook-agent-guard` — prevents the orchestrator from directly editing source files.

-----

## Phase 4 — Dispatch Discipline

**Rule:** Commit after every wave BEFORE dispatching the next worker. Prevents scope wipe.

**Rule:** Clamp scope after every worker. Not optional.

**Rule:** Stagger parallel launches ~3 seconds. 2-3 simultaneous: stable. 5+: crashes.

**Rule:** Processes that "crash" often complete in background. Always check output files before assuming failure.

**Rule:** Prefer editing existing files over creating new ones. New file success ~50% first-try vs ~90% for edits. When new files are needed, specify module registration explicitly.

**Rule:** Wave-based dispatch, not one-shot. One-shot for large builds is an explicit anti-pattern. (Evidence: 15+ sessions, 100% foreman ship rate with waves.)

-----

## Phase 5 — Wave Cadence

Every wave follows this exact sequence. No skips.

```
Feature → Gate → Document → Harden → Graduate
```

### 5.1 Feature

Build or change the targeted surface. Workers handle bounded structural work. Workers should not create orchestration infrastructure instead of features.

### 5.2 Gate

Run mechanical checks: compile, test, lint, scope clamp.

**Rule:** Green means ready to examine, NOT ready to ship.

### 5.3 Document

Emit artifacts ONLY when triggered:

1. Non-obvious decision made
2. Direct verification happened
3. Reusable principle emerged
4. Open debt appeared
5. Contradiction surfaced
6. Correction invalidated prior belief
7. Graduation test created

If nothing triggers, write nothing.

### 5.4 Harden

Inspect the actual result:

- Reachable end-to-end?
- Feedback visible?
- Responsive enough?
- Edge behavior sane?
- Diagnosable when it fails?

If reachable but wrong or confusing, it is not finished.

### 5.5 Graduate

For each `[Observed]` truth:

1. Name the invariant
2. Encode it as a test or gate
3. Track remaining ungraduated work as P0/P1/P2

**Rule:** Do not start the next wave until Document, Harden, and Graduate are complete.

-----

## Phase 6 — Trust and Evidence

### 6.1 Trust order

When the agent encounters conflicting information:

1. **Fresh code, tests, runtime output** — what the files actually say right now
2. **[Observed] artifacts** with concrete source_refs
3. **Current STATE.md**
4. **Project docs, contracts, specs**
5. **Research findings** with certainty labels
6. **Conversation history** — the LOWEST trust tier

**Rule:** Conversation is scaffolding, not substrate. An agent that trusts its own prior conversation over the current code will propagate stale claims as truth.

(Evidence: Blackout Test — 7 task packets, stateless workers, quarantined conversations. Blind integrator gets only repo + diffs. Build lands. Five network faults all recovered. Contamination radius: 0.)

### 6.2 Evidence levels

Every claim an agent persists should carry one of:

- **[Observed]** — directly verified against code, tests, or runtime. Can be frozen into gates.
- **[Inferred]** — logically derived but not directly verified. Cannot be frozen.
- **[Assumed]** — stated without verification. Must be verified before any critical decision depends on it.

(Evidence: Typed provenance changes model behavior from 13% → 54% calibrated abstention on ambiguous decisions. Without evidence levels, untyped memory is worse than no memory — 0% correct vs 83% with no memory at all.)

### 6.3 Artifact schema

```yaml
id: DEC-2026-03-10-001
type: decision | observation | debt | principle
evidence: Observed | Inferred | Assumed
domain: player | world | save | ui | api | infra
summary: "One sentence."
source_refs:
  - "file:repo@src/path/file.rs:10-40"
status: active | resolved | superseded
supersedes: []
```

**Rule:** One artifact per file. Supersede, don't silently mutate. [Observed] claims must have source_refs.

### 6.4 Evidence interaction with model capability

|  | Simple claims | Complex ambiguity |
|---|---|---|
| Frontier models (Sonnet, GPT-5.4) | works | **✓ 13%→54%** |
| Cheap models (gpt-5-mini) | **✓ 33%→100%** | no effect |
| gpt-4.1 | no effect | no effect |

Validated across game and non-game codebases (Rust async API: 33%→100% on gpt-5-mini).

-----

## Phase 7 — Session Protocol

### 7.1 Start (every session)

1. Read this playbook + STATE.md
2. Mount the current objective
3. Pre-touch retrieval: `git log --oneline -15 -- <path>`, read active debt
4. State BEFORE acting: tier (S/M/C), surface being touched, current phase, any [Assumed] claims on the critical path

### 7.2 Tiering

- **S** — single-surface fix or bounded hotfix
- **M** — module or subsystem, 1-3 domains, workers useful
- **C** — campaign, multiple domains, orchestration required

Start at S if ambiguous. Escalate when touching shared contracts, persistence, trust boundaries, or multiple interacting surfaces.

### 7.3 End (every session)

1. Update STATE.md — phase, debts, decisions, gate status, uncertainties
2. Write triggered artifacts only
3. Commit memory changes
4. Do NOT rely on conversation to preserve what was learned

-----

## Phase 8 — State Recovery

### 8.1 dispatch-state.yaml

```yaml
lanes:
  - id: farming-fix
    status: in-progress  # pending | in-progress | gated | merged | failed
    goal: "Fix crop save/load roundtrip"
    owned_paths:
      - src/farming/
    next_action: "Run cargo test after worker completes"
```

Every active work lane tracked. Survives session death.

### 8.2 Three core transactions

**Checkpoint** — preserves conversation + filesystem + ledger state. All three required; partial recovery from one alone fails.

**Restore** — recovers from a named checkpoint when a wave goes wrong.

**Launch** — creates an isolated work lane: new worktree + branch, copies contract, registers in dispatch-state.yaml.

### 8.3 Mechanical verification

- `verify-state-claims` — checks STATE.md claims against actual repo
- `hook-contract-integrity` — pre-commit hook rejecting contract edits without override
- `hook-agent-guard` — prevents orchestrator from directly editing source files

### 8.4 Recovery prompt

```
Continuing [project]. Read before acting:
- STATE.md, MANIFEST.md, docs/spec.md, src/shared/mod.rs

Recent: [1 sentence]. Task: [what to do now].
State tier (S/M/C) and [Assumed] claims on critical path before acting.
```

-----

## Stop Conditions

Cease work and reassess when any of these appear:

1. **Contract drift** — checksum fails → restore contract, re-validate
2. **Clamp breaks the fix** — boundary is wrong or task is integration work → re-scope
3. **False green** — tests pass but contract is unused, bypassed, or visually broken
4. **Abstraction reflex** — redesigning architecture to avoid debugging the real issue
5. **Delegation compression** — asked for 80 items, got 8 (worker read summary, not spec)
6. **Self-model error** — agent claims it cannot do things it can
7. **Identity paradox** — one agent playing both architect and worker loses role separation
8. **Beautiful dead product** — gates green but surface is unreachable or unhelpful
9. **Ghost progress** — nothing newly reachable exists after the wave
10. **Cadence break** — documenting while still coding, or coding while still diagnosing

**Rule:** When a stop condition fires: stop. Don't push through. Diagnose. Restart the wave.

-----

## Discovery Taxonomy

When a worker finds something unexpected during a fix:

| Discovery | Action |
|-----------|--------|
| Reproducible bug (in scope) | Fix it |
| Fragile seam (missing coverage) | Write regression test FIRST, then fix |
| Fidelity gap (cross-domain) | Escalate to orchestrator — don't widen scope |
| Out of scope | Record as debt, don't touch |

-----

## Agent Failure Modes

**Defaults to solo execution.** Fix: "You have [tool] available. Dispatch workers."

**Stops after one wave.** Fix: "DO NOT stop between waves." Still stops: "continue."

**Builds frameworks instead of features.** Fix: "Implement deliverables only."

**Reads summaries, ignores files.** Fix: specs on disk, quantities in prompt, "read [path]."

**Edits frozen contract.** Fix: mechanical clamp. `disallowedTools`.

**Makes better decision than spec.** Not a failure. Gate/harden catches bad decisions.

**Claims it can't do things it can.** Fix: "You have bash access. You can read and write files."

**New file creation fails (~50%).** Fix: prefer edits. Specify module registration.

**Session dies mid-campaign.** Fix: manifest on disk is the state. "Resume from Wave N."

-----

## Completion Criteria

You are done ONLY when:

- [ ] Contract checksum passes
- [ ] Global compile/typecheck passes
- [ ] Global test suite passes
- [ ] Connectivity gate passes (no hermetic domains — every domain imports shared contract)
- [ ] Each worker report exists
- [ ] Integration report exists
- [ ] STATE.md reflects current truth
- [ ] No [Assumed] claims on the critical path
- [ ] Every P0 surface is reachable and operable
- [ ] MANIFEST.md updated with final status

-----

## Measurements

| Metric | Value |
|--------|-------|
| Manual dispatch ship rate | 67% (10/15) |
| Foreman dispatch ship rate | 100% (5/5) |
| Scope: prompt-only | 0/20 |
| Scope: mechanical clamp | 20/20 |
| Evidence tags on ambiguous decisions | 13%→54% |
| Poisoning defense (multi-repo) | 33%→100% |
| Blackout fault recovery | 5/5, contamination 0 |
| Briefing winner | Decision Fields (1514 vs 9) |
| Exact-value ship rate | 100% |
| Vague-goal ship rate | 0% |
| New file success | ~50% (edits ~90%) |
| Parallel workers | 2-3 stable, 5+ crashes |
| Model output gap | 9.8× same task |
| Opus 1M campaign | 18 commits, 1 session |

-----

*Derived from 739 commits, 64K LOC, 295M tokens, 98 agent sessions, 172 controlled trials across 5 frontier models. Zero handwritten lines of code.*
