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

### 5.1 Briefing format matters more than content volume

| Format | Output | Ship? |
|--------|--------|-------|
| Freeform ("make X better") | 446 lines | ✓ but inconsistent |
| Formal spec | 9 lines | Barely |
| **Decision Fields (do/don't/drift-cue)** | **1514 lines** | **✓** |
| Examples of good output | 513 lines | Scope drift |
| Minimal one-liner | 2 lines | ✗ |

Telling the agent what NOT to do and what drift looks like produces more output than telling it what TO do.

### 5.2 Specificity determines ship rate

| Specificity | Ship rate |
|------------|-----------|
| Exact values ("set alpha to 0.6, file X line 40") | **100%** |
| Named actions ("add particle effect to tool swing") | **67%** |
| Vague goals ("make mining feel better") | **0%** |

If you can't name the file and the value, the prompt isn't ready.

### 5.3 Model selection

| Model | Role | Critical note |
|-------|------|--------------|
| Opus 4.6 | Orchestrator | 1M context sustains 20+ wave campaigns |
| Sonnet 4.6 | Coding worker | Finds tool paths, resourceful |
| GPT-5.4 | Codex worker | Reliable, good scoped output |
| gpt-5-mini | Cheap tasks | Evidence tags work on simple claims |
| **gpt-4.1** | **Avoid** | **Fabricates verification — claims files exist that don't** |

The model is the first-order throughput variable. Same architecture, 9.8× output gap between best and worst worker models.

## 6. PROMPT TEMPLATES

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

The "DO NOT stop between waves" line is what sustains multi-wave campaigns. Without it, every agent completes Wave 1 and waits.

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

### Recovery (after session death)
```
Continuing [project]. Read before acting:
- MANIFEST.md, STATE.md, docs/spec.md, src/shared/mod.rs

Recent: [1 sentence]. Task: [what to do now].
State tier (S/M/C) and any [Assumed] claims on critical path before acting.
```

## 7. FINDINGS

### 7.1 Untyped memory is worse than no memory

| Condition | Correct on resolvable decisions |
|-----------|-------------------------------|
| **No memory** | **83%** |
| Untyped notes (same claims, plain text) | **0%** |
| Typed provenance (YAML + evidence levels) | 14% |

Untyped notes introduce conflicting claims that make agents second-guess correct decisions.

### 7.2 Typed provenance rescues calibration on ambiguous decisions

| Condition | Calibrated abstention ("I need to verify") |
|-----------|------------------------------------------|
| No memory | 18% |
| Untyped | 13% |
| **Typed provenance** | **54%** |

Same claims, different encoding. 4× improvement. The 2×2 interaction:

|  | Simple claims | Complex ambiguity |
|---|---|---|
| Frontier models | works | **✓ 13%→54%** |
| Cheap models | **✓ 33%→100%** | no effect |
| gpt-4.1 | no effect | no effect |

Evidence tags have both a model capability threshold and a task complexity threshold. Validated on non-game Rust API (multi-repo replication): 33%→100% on gpt-5-mini.

### 7.3 Conversations are scaffolding, not substrate

Blackout Test: 7 task packets, stateless workers, quarantined conversations. Blind integrator gets only repo + diffs. Build lands. Five network faults (reorder, drop, corrupt, duplicate, delay) all recovered. Contamination radius: 0.

Don't maintain long conversations. Put state in files. Start fresh sessions. The dispatch document is the cognitive substrate.

### 7.4 Contract freeze prevents parallel integration failure

Without frozen type contract: 10 workers → 6 incompatible interfaces.
With frozen contract: 50+ domain builds → zero integration type errors.

Freeze shapes (types, enums, events). Leave values in config. Contract changes are integration-phase only.

### 7.5 Wave-based dispatch, not one-shot

One-shot dispatch for large builds: explicit anti-pattern. Wave-based: 15+ sessions, 100% foreman ship rate. Commit after each wave. Clamp after each worker.

## 8. AGENT FAILURE MODES

**Defaults to solo execution.** Fix: "You have [tool] available. Dispatch workers."

**Stops after one wave.** Fix: "DO NOT stop between waves." If it still stops: type "continue."

**Builds frameworks instead of features.** Fix: "Do NOT create orchestration infrastructure. Implement deliverables only."

**Reads summaries, ignores source files.** Symptom: asked for 80 items, got 8. Fix: specs on disk, quantities in worker prompt, "read the file at [path]."

**Edits frozen contract during parallel build.** Fix: mechanical clamp. `disallowedTools` in settings.

**Makes better architectural decision than your spec.** Not a failure. The HashMap override instead of contract modification was better than the spec. Let the agent find the cleanest route; the gate/harden cycle catches bad decisions.

**Claims it can't do things it can.** "I don't have bash access" / "I can't read files." Fix: "You have bash access. You can read and write files."

**New file creation fails (~50%).** Fix: prefer edits. When new files required, specify module registration in prompt.

**Session dies mid-campaign.** Fix: manifest on disk is the state. New session: "Resume from Wave N."

## 9. MEASUREMENTS

| Metric | Value |
|--------|-------|
| Tokens consumed (full program) | 295M+ |
| Commits | 739 |
| LOC | 64K, 0 handwritten |
| Manual dispatch ship rate | 67% (10/15) |
| Foreman dispatch ship rate | 100% (5/5) |
| 4-layer stack ship rate | 100% (5/5) |
| Scope: prompt-only | 0/20 |
| Scope: mechanical clamp | 20/20 |
| Evidence tags on ambiguous decisions | 13%→54% |
| Poisoning defense (multi-repo) | 33%→100% |
| Blackout test fault recovery | 5/5, contamination 0 |
| Briefing winner | Decision Fields (1514 vs 9 lines) |
| Exact-value ship rate | 100% |
| Vague-goal ship rate | 0% |
| New file first-try success | ~50% |
| Edit first-try success | ~90% |
| Parallel workers stable | 2-3 |
| Parallel workers crash | 5+ |
| Model output gap (same task) | 9.8× |
| Opus 1M campaign | 18 commits, 45 sprites, 1 session |
| Campaign context utilization | 168K of 1M (17%) |
