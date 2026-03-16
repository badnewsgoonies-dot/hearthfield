# AI ORCHESTRATION TOOLKIT
## Proven methods for building software through AI agent dispatch

---

## 1. DISPATCH STACK

Two independent dispatch paths minimum — any single tool's auth or quota fails mid-session.

```
Claude Code: npm install -g @anthropic-ai/claude-code
  Auth: claude auth login | ANTHROPIC_API_KEY | claude setup-token
  Run: claude -p "prompt" --dangerously-skip-permissions

Codex CLI: npm install -g @openai/codex
  Auth: ~/.codex/auth.json (ChatGPT session token — NOT API key)
  Run: timeout 180 codex exec --dangerously-bypass-approvals-and-sandbox -m gpt-5.4 -C /path "prompt"

Copilot CLI: npm install -g @github/copilot
  Auth: export COPILOT_GITHUB_TOKEN="github_pat_..." (fine-grained PAT ONLY)
  ⚠ Classic PATs (ghp_) REJECTED. GITHUB_TOKEN / GH_TOKEN env vars REJECTED.
  Run: timeout 120 copilot -p "prompt" --model claude-sonnet-4.6 --allow-all-tools
  ⚠ Without --allow-all-tools, some models spend entire timeout reading files instead of executing.
```

**Quota fallback:** Premium exhausted → gpt-5.4 via Codex (different pool) → gpt-5-mini via Copilot → gpt-4.1 via Copilot (last resort — fabricates verification).

## 2. AGENT ARCHITECTURE

```
Layer 0: You (direction — voice or text)
Layer 1: Orchestrator (reads codebase, writes specs on disk, dispatches)
Layer 2: Foreman (reads playbook, explores code, writes exact-value prompts)
Layer 3: Workers (implement scoped changes, commit)
```

The foreman outperformed manual dispatch (100% vs 67% ship rate) because the playbook crystallized 15 rounds of failure into reusable patterns. The foreman doesn't relay — it reads the code and writes more specific prompts than a human would.

Each additional layer adds handoff loss. Depth-1 (orchestrator → workers) is the stable default. Depth-2 only when domain count exceeds ~20. 4-deep nesting confirmed working but is the practical limit.

**Mid-run injection:** Claude Code allows redirection via `/btw` and natural pauses. Copilot Opus sends continuous waves — no injection point without cancelling. This determined whether the sprite wiring campaign could switch from solo to parallel dispatch mode mid-session.

## 3. SCOPE ENFORCEMENT

Prompt-only scope control under compiler pressure: **0/20.**
Mechanical clamping after worker completes: **20/20.**

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

Run after EVERY worker. Let the worker edit anything. Revert everything outside scope afterward.

`.claude/settings.json` with `disallowedTools` is reliable enforcement. CLAUDE.md instructions are suggestions — the model can deprioritize them.

## 4. DISPATCH DISCIPLINE

- **Wrap every dispatch in `timeout`.** Workers hang indefinitely without it.
- **Commit after every wave BEFORE dispatching the next worker.** Prevents scope wipe.
- **Clamp scope after every worker.** Not optional.
- **Stagger parallel launches ~3 seconds.** 2-3 simultaneous: stable. 5+: crashes.
- **"Crashed" processes often complete in background.** Check output files before assuming failure.
- **Prefer editing existing files over creating new ones.** New file success ~50% first-try vs ~90% for edits. When new files needed, specify module registration explicitly in prompt.

## 5. WHAT TO TELL THE AGENT

### Briefing format matters more than content volume

| Format | Output | Ship? |
|--------|--------|-------|
| Freeform ("make X better") | 446 lines | ✓ but inconsistent |
| Formal spec | 9 lines | Barely |
| **Decision Fields (do/don't/drift-cue)** | **1514 lines** | **✓** |
| Examples of good output | 513 lines | Scope drift |
| Minimal one-liner | 2 lines | ✗ |

Telling the agent what NOT to do and what drift looks like produces more output than telling it what TO do.

### Specificity determines ship rate

| Specificity | Ship rate |
|------------|-----------|
| Exact values ("set alpha to 0.6, file X line 40") | **100%** |
| Named actions ("add particle effect to tool swing") | **67%** |
| Vague goals ("make mining feel better") | **0%** |

If you can't name the file and the value, the prompt isn't ready.

### Model selection

| Model | Role | Critical note |
|-------|------|--------------|
| Opus 4.6 | Orchestrator | 1M context sustains 20+ wave campaigns |
| Sonnet 4.6 | Coding worker | Finds tool paths, resourceful |
| GPT-5.4 | Codex worker | Reliable, good scoped output |
| gpt-5-mini | Cheap tasks | Evidence tags work on simple claims |
| **gpt-4.1** | **Avoid** | **Fabricates verification — claims files exist that don't** |

The model is the first-order throughput variable. Same architecture, 9.8× output gap between best and worst worker models.

## 6. TRUST ORDER

When the agent encounters conflicting information, this precedence governs what it believes:

1. **Fresh code, tests, runtime output** — what the files actually say right now
2. **[Observed] artifacts** with concrete source_refs — verified claims on disk
3. **Current STATE.md** — the last committed snapshot of project truth
4. **Project docs, contracts, specs** — design intent
5. **Research findings** with certainty labels — methodology guidance
6. **Conversation history** — the LOWEST trust tier

An agent that trusts its own prior conversation over the current code will propagate stale claims as truth. The trust order prevents this.

## 7. EVIDENCE LEVELS

Every claim an agent persists should carry one of:

- **[Observed]** — directly verified against code, tests, or runtime. Can be frozen into gates.
- **[Inferred]** — logically derived but not directly verified. Cannot be frozen into gates.
- **[Assumed]** — stated without verification. Must be verified before any critical decision depends on it.

This is the mechanism behind the core finding: typed provenance changes model behavior from 13% → 54% calibrated abstention on ambiguous decisions. Without evidence levels, untyped memory is **worse than no memory** (0% vs 83% correct on resolvable decisions).

### Artifact schema for persistent memory

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

One artifact per file. Supersede, don't silently mutate. [Observed] claims must have source_refs.

## 8. WAVE CADENCE

Every wave follows this exact sequence. No skips.

```
Feature → Gate → Document → Harden → Graduate
```

- **Feature**: Build or change the targeted surface.
- **Gate**: Mechanical checks (compile, test, lint, scope clamp). Green = ready to examine, NOT ready to ship.
- **Document**: Emit artifacts ONLY when triggered: non-obvious decision, direct verification, reusable principle, open debt, contradiction, correction, or graduation test. If nothing triggers, write nothing.
- **Harden**: Inspect the actual result. Reachable? Visible feedback? Responsive? Edge behavior sane? Diagnosable when it fails?
- **Graduate**: For each [Observed] truth, encode it as a test or gate. Track remaining work as P0/P1/P2.

Do not start the next wave until Document, Harden, and Graduate are complete.

## 9. SESSION PROTOCOL

### Start (every session)

1. Read this toolkit + STATE.md
2. Mount the current objective
3. Pre-touch retrieval: `git log --oneline -15 -- <path>`, read active debt
4. State BEFORE acting: tier (S/M/C), surface being touched, current phase, any [Assumed] claims on the critical path

### Tiering

- **S** — single-surface fix or bounded hotfix
- **M** — module or subsystem, 1-3 domains, workers useful
- **C** — campaign, multiple domains, orchestration required

Start at S if ambiguous. Escalate when touching shared contracts, persistence, trust boundaries, or multiple interacting surfaces.

### End (every session)

1. Update STATE.md — phase, debts, decisions, gate status, uncertainties
2. Write triggered artifacts only
3. Commit memory changes
4. Do NOT rely on conversation to preserve what was learned

## 10. STOP CONDITIONS

Cease work and reassess when any of these appear:

1. **Contract drift** — checksum fails → restore contract, re-validate
2. **Clamp breaks the fix** — the domain boundary is wrong or the task is integration work → re-scope
3. **False green** — tests pass but the contract is unused, bypassed, or visually broken
4. **Abstraction reflex** — redesigning architecture to avoid debugging the real issue
5. **Delegation compression** — asked for 80 items, got 8 (worker read a summary, not the spec)
6. **Self-model error** — agent claims it cannot do things it can
7. **Identity paradox** — one agent playing both architect and worker loses role separation
8. **Beautiful dead product** — gates green but the surface is unreachable or unhelpful
9. **Ghost progress** — nothing newly reachable exists after the wave
10. **Cadence break** — documenting while still coding, or coding while still diagnosing

When a stop condition fires: stop. Don't push through. Diagnose. Restart the wave.

## 11. DISCOVERY TAXONOMY

When a worker finds something unexpected during a fix:

| Discovery | Action |
|-----------|--------|
| Reproducible bug (in scope) | Fix it |
| Fragile seam (missing coverage) | Write regression test FIRST, then fix |
| Fidelity gap (cross-domain) | Escalate to orchestrator — don't widen scope |
| Out of scope | Record as debt, don't touch |

## 12. STATE RECOVERY

### dispatch-state.yaml (the process table)

```yaml
lanes:
  - id: farming-fix
    status: in-progress  # pending | in-progress | gated | merged | failed
    goal: "Fix crop save/load roundtrip"
    owned_paths:
      - src/farming/
    next_action: "Run cargo test after worker completes"
```

Every active work lane tracked. Survives session death. Any new session reads it and knows what's in flight.

### Three core transactions

**Checkpoint** — preserves conversation + filesystem + ledger state. All three required.

**Restore** — recovers from a named checkpoint when a wave goes wrong.

**Launch** — creates an isolated work lane: new worktree + branch, copies contract, registers in dispatch-state.yaml.

### Mechanical verification

**verify-state-claims** — checks STATE.md claims against actual repo.

**hook-contract-integrity** — pre-commit hook rejecting contract modifications without override.

**hook-agent-guard** — prevents orchestrator from directly editing source files.

### Recovery prompt

```
Continuing [project]. Read before acting:
- STATE.md, MANIFEST.md, docs/spec.md, src/shared/mod.rs

Recent: [1 sentence]. Task: [what to do now].
State tier (S/M/C) and [Assumed] claims on critical path before acting.
```

## 13. PROMPT TEMPLATES

### Worker (single task)
```
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

### Campaign (multi-wave autonomous)
```
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

### Audit
```
Audit every [asset/system type]. For each:
1. What exists? (path, dimensions, contents)
2. Is it used in code? Where?
3. If unused: what should use it? Exact code insertion point?
4. What blocks it?

Write as manifest with per-item wiring instructions.
Priority: P0 broken → P1 ready to wire → P2 needs data → P3 needs systems.
```

## 14. FINDINGS

### Untyped memory is worse than no memory

| Condition | Correct on resolvable decisions |
|-----------|-------------------------------|
| **No memory** | **83%** |
| Untyped notes | **0%** |
| Typed provenance | 14% |

### Typed provenance rescues calibration on ambiguous decisions

| Condition | Calibrated abstention |
|-----------|-----------------------|
| No memory | 18% |
| Untyped | 13% |
| **Typed provenance** | **54%** |

The interaction with model capability:

|  | Simple claims | Complex ambiguity |
|---|---|---|
| Frontier models | works | **✓ 13%→54%** |
| Cheap models | **✓ 33%→100%** | no effect |
| gpt-4.1 | no effect | no effect |

### Conversations are scaffolding, not substrate

Blackout Test: 7 packets, stateless workers, blind integrator. Build lands. Five network faults all recovered. Contamination radius: 0.

### Contract freeze prevents parallel integration failure

Without: 10 workers → 6 incompatible interfaces. With: 50+ builds → zero type errors.

### Wave-based dispatch, not one-shot

One-shot for large builds: anti-pattern. Wave-based: 100% foreman ship rate across 15+ sessions.

## 15. AGENT FAILURE MODES

**Defaults to solo execution.** Fix: "You have [tool] available. Dispatch workers."

**Stops after one wave.** Fix: "DO NOT stop between waves." Still stops: "continue."

**Builds frameworks instead of features.** Fix: "Implement deliverables only."

**Reads summaries, ignores files.** Fix: specs on disk, quantities in prompt, "read [path]."

**Edits frozen contract.** Fix: mechanical clamp. `disallowedTools`.

**Makes better decision than spec.** Not a failure. Gate/harden catches bad decisions.

**Claims it can't do things it can.** Fix: "You have bash access."

**New file creation fails (~50%).** Fix: prefer edits. Specify registration.

**Session dies mid-campaign.** Fix: "Resume from Wave N."

## 16. MEASUREMENTS

| Metric | Value |
|--------|-------|
| Manual dispatch ship rate | 67% (10/15) |
| Foreman dispatch ship rate | 100% (5/5) |
| Scope: prompt-only | 0/20 |
| Scope: mechanical clamp | 20/20 |
| Evidence tags on ambiguous decisions | 13%→54% |
| Poisoning defense (multi-repo) | 33%→100% |
| Blackout fault recovery | 5/5, contamination 0 |
| Briefing winner | Decision Fields (1514 vs 9 lines) |
| Exact-value ship rate | 100% |
| Vague-goal ship rate | 0% |
| New file success | ~50% (edits ~90%) |
| Parallel workers stable | 2-3 (5+ crashes) |
| Model output gap | 9.8× same task |
| Opus 1M campaign | 18 commits, 45 sprites, 1 session |
