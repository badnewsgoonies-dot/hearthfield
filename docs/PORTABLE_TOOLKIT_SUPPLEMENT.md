# PORTABLE TOOLKIT — SUPPLEMENT
## Everything the cross-check found missing from the first pass
### 5,813 lines of source → 508 lines of toolkit → this supplement fills the gaps

The original toolkit captured the practical dispatch layer well. This supplement adds four layers it missed entirely: governance, session discipline, the git-native orchestration OS, and the discovery/memory system.

---

## LAYER A: GOVERNANCE (The Constitution)

These are the 10 principles that survived 3 rounds of adversarial testing (16 attacks, 15+ bugs found as a side effect). They govern how the AI agent should reason, not just what it should build.

### The 10 Principles

**C1: Code Is Truth, Artifacts Are Cache**
When memory/artifacts and code disagree, code wins. Always. Artifacts accelerate reasoning but never override what the files actually say.

**C2: Freshness Is Mandatory**
Stale context is worse than no context. Every session must reconstruct from current state, not replay old conversations.

**C3: Silence Is Not Assent**
If a domain hasn't been touched or tested, assume it's broken. No news is not good news — it's unmeasured.

**C4: Evidence Levels Are Types, Not Suggestions**
[Observed], [Inferred], [Assumed] are not decorative labels. They change how claims should be weighted. An [Assumed] claim blocking a P0 decision is a stop condition.

**C5: Reconstruction Is Vendor-Independent**
The state files (STATE.md, artifacts, dispatch ledger) must be readable and recoverable by any model — Claude, GPT, Gemini, or a human. No vendor lock-in in the memory layer.

**C6: The Artifact Accelerates Structure, Not Numbers**
Artifacts help organize reasoning. They don't produce numbers, measurements, or proofs by existing. A well-structured artifact with wrong data is still wrong.

**C7: Mechanical Enforcement Catches Drift, Not Adversaries**
Scope clamps, checksums, and gates catch accidental drift (which is constant). They don't stop a determined adversary. Don't confuse the two.

**C8: Negative Knowledge Finds More Bugs Than Positive Knowledge**
Knowing what NOT to do, what WAS tried and failed, and what contradictions exist is more operationally valuable than knowing what works. DEBT and contradiction artifacts are first-class.

**C9: Depth-1 Nesting Is The Stable Default**
Orchestrator → workers (flat) is the stable default. Orchestrator → leads → workers adds handoff loss. Go deeper than depth-1 only when domain count forces it (20+).

**C10: Every Step Is Load-Bearing**
No ceremonial steps. If a step in the workflow doesn't change a decision, catch an error, or produce a needed output, delete it. Bureaucratic overhead kills agent productivity.

### When to apply
Load these into the system prompt or project knowledge for any session where the agent is making architectural decisions, evaluating evidence, or operating autonomously for extended periods. Don't load them for simple dispatch tasks — C10 says don't add overhead that doesn't earn its keep.

---

## LAYER B: SESSION DISCIPLINE

### B1. Trust order (when sources disagree)

1. Fresh code, tests, runtime captures, live playtest
2. [Observed] artifacts with concrete source_refs
3. Current STATE.md snapshot
4. Project docs, contracts, specs, quickstart
5. Research findings (with their certainty labels)
6. Conversation history, compaction summaries, remembered chat claims

**Conversation is the LOWEST trust tier.** This is the single most important operational principle.

### B2. Wave cadence (never skip steps)

Every wave follows this exact sequence:

```
Feature → Gate → Document → Harden → Graduate
```

- **Feature**: Build or change the targeted surface
- **Gate**: Run mechanical checks (compile, test, lint, scope clamp). Green = ready to examine, NOT ready to ship.
- **Document**: Emit artifacts ONLY for: non-obvious decisions, direct verifications, reusable principles, open debt, contradictions, corrections, feel/UX failures, graduation tests. If nothing triggers, write nothing.
- **Harden**: Inspect the actual surface. Reachable? Visible feedback? Responsive? Edge behavior sane? Diagnosable when it fails?
- **Graduate**: For each [Observed] truth, encode it as a test or gate. Track remaining ungraduated work as P0/P1/P2.

**Do not start the next wave until Document, Harden, and Graduate are complete.**

### B3. Session start protocol

Every session begins:
1. Read the kernel/toolkit
2. Read STATE.md or current project snapshot
3. Mount the current objective
4. Pull relevant active artifacts
5. Run pre-touch retrieval before editing
6. State BEFORE acting: current tier (S/M/C), current surface, current phase, current wave phase, P0/P1 debt, any [Inferred]/[Assumed] claims on the critical path

### B4. Tiering

- **S** — single-surface fix or bounded hotfix
- **M** — module or subsystem, 1-3 domains, workers useful
- **C** — campaign, multiple domains, integration and orchestration required

Start at S if ambiguous. Escalate when you touch shared contracts, persistence, trust boundaries, or multiple interacting surfaces.

### B5. Pre-touch retrieval

Before touching any domain:
1. `git log --oneline -15 -- <path>`
2. Read active artifacts for that domain
3. Read latest worker report or failing trace
4. State: what changed recently, what remains unresolved, what is still [Inferred]/[Assumed]

### B6. Session end protocol

1. Update STATE.md with phase, debts, decisions, gate status, uncertainties
2. Write triggered artifacts only
3. Commit memory changes
4. Record new graduation tests or remaining debt
5. Do NOT rely on chat history to preserve what was learned

---

## LAYER C: GIT-NATIVE ORCHESTRATION OS

The toolkit covered clamp-scope.sh. These are the other 8 scripts that form the complete orchestration operating system.

### C1. The three core transactions

**Checkpoint** (scripts/checkpoint-state.sh — 473 lines)
Preserves: forked conversation/session + filesystem state (worktree, branch, commit, dirty diff) + ledger state (dispatch-state.yaml, STATE.md, worker reports). All three required — partial recovery from one alone fails.

```bash
bash scripts/checkpoint-state.sh \
  --label tranche1-clean \
  --session <session_id> \
  --worktree /tmp/project-lane \
  --allow-prefix src/farming
```

**Restore** (scripts/restore-checkpoint.sh — 283 lines)
Recovers from a named checkpoint: restores branch, worktree state, ledger files. Used when a wave goes wrong and you need to roll back to last known good state.

```bash
bash scripts/restore-checkpoint.sh --label tranche1-clean
```

**Launch** (scripts/launch-lane.sh — 164 lines)
Creates an isolated work lane: new git worktree + branch, copies the type contract, sets up the allowed path prefix, registers in dispatch-state.yaml.

```bash
bash scripts/launch-lane.sh \
  --lane farming-fix \
  --from HEAD \
  --allow src/farming/
```

### C2. Validation pipeline

**run-gates.sh** (182 lines)
Runs in sequence: contract integrity check (shasum), typecheck/compile, test suite, connectivity check (no hermetic domains — every domain must import from the shared contract).

```bash
bash scripts/run-gates.sh
```

### C3. State verification

**verify-state-claims.sh** (207 lines)
Reads STATE.md and mechanically verifies its claims against the actual repo: do the files it references exist? Do the line numbers match? Are the stated test results current?

### C4. Git hooks

**hook-contract-integrity.sh** (33 lines) — pre-commit hook that rejects commits modifying the frozen contract without explicit override.

**hook-agent-guard.sh** (31 lines) — prevents the orchestrator session from directly editing .rs files (forces delegation to workers).

**hook-session-freshness.sh** (81 lines) — warns if the session hasn't read STATE.md recently.

**install-hooks.sh** (95 lines) — wires all hooks into .git/hooks/.

### C5. Coverage tracking

**check-coverage.sh** (53 lines) — reports which domains have tests, which don't, which have recent activity, which are stale.

### C6. dispatch-state.yaml (the process table)

```yaml
lanes:
  - id: farming-fix
    tranche: 2
    status: in-progress  # pending | in-progress | gated | merged | failed
    goal: "Fix crop save/load roundtrip"
    owned_paths:
      - src/farming/
    validation:
      compile: pending
      tests: pending
      clamp: pending
    next_action: "Run cargo test after worker completes"
```

This is the AI equivalent of a process table. Every active work lane is tracked with its scope, status, and next action. Survives session death. Any new session can read it and know what's in flight.

---

## LAYER D: DISCOVERY & MEMORY SYSTEM

### D1. Evidence levels (use everywhere)

- **[Observed]** — directly verified against code, tests, or runtime. Can be frozen into gates.
- **[Inferred]** — logically derived from observed evidence but not directly verified. Cannot be frozen.
- **[Assumed]** — stated without verification. Must be verified before any P0/P1 decision depends on it.

### D2. Artifact schema (for persistent memory)

```yaml
id: DEC-2026-03-10-001
type: decision | observation | debt | principle
evidence: Observed | Inferred | Assumed
domain: player | world | save | ui | api | infra
summary: "One sentence. Never nested."
source_refs:
  - "file:repo@src/path/file.rs:10-40"
  - "commit:repo@sha"
  - "test:repo@command#test_name"
status: active | resolved | superseded
supersedes: []
```

Rules:
- One artifact per file
- Supersede instead of silently mutating history
- [Observed] claims must not have empty source_refs
- Schema validity is not truth — high-stakes claims still need verification

### D3. Certainty labels (for research claims)

- **Corpus result** — repeated descriptive pattern, not controlled
- **Replicated finding** — same effect at n≥3
- **Local finding** — observed at low n or mixed replication
- **Derived recommendation** — motivated by findings, not isolated experimentally
- **Open question** — unresolved or insufficiently tested

### D4. Discovery taxonomy (when you find something during a fix)

| What you found | Action |
|---------------|--------|
| Reproducible bug | Fix it (in scope) |
| Fragile seam | Write a regression test FIRST, then fix |
| Fidelity gap | Escalate to orchestrator — don't widen scope |
| Out of scope | Record as debt, don't touch |

This prevents scope creep from discovered issues. A worker finds a bug in another domain? Record it and keep going. Don't chase it.

### D5. Stop conditions (cease work and reassess)

1. **Contract drift** — checksum fails → restore contract, re-run from validation
2. **Clamp breaks the fix** — the seam is wrong or the task is integration work → re-scope
3. **False green** — tests pass but the contract is unused, bypassed, or visually broken
4. **Abstraction reflex** — redesigning architecture to avoid debugging the real issue
5. **Delegation compression** — asked for 80 items, got 8 (worker reading summary not spec)
6. **Self-model error** — agent claims it cannot do things it can (no bash, can't read files, etc.)
7. **Identity paradox** — one agent playing both architect and worker loses role separation
8. **Beautiful dead product** — gates are green but the surface is unreachable or unhelpful
9. **Ghost progress** — nothing newly reachable exists after the wave
10. **Cadence break** — documenting while still coding, or coding while still diagnosing

**When a stop condition fires: stop. Don't push through. Diagnose. Then restart the wave.**

### D6. Verification triggers (escalate to direct verification)

- V1: [Assumed]/[Inferred] claim blocks a P0/P1 decision
- V2: Two artifacts conflict or a supersedes chain is ambiguous
- V3: Single artifact is decisive for high-stakes question (release, save integrity, security)
- V4: Claim depends on visuals, interaction, timing, animation, or feel
- V5: Tool output is untrusted or weakly scoped

Use the cheapest defense that resolves the ambiguity:
- Tier 1 (always on): evidence tags, source refs, typed artifacts
- Tier 2 (selective): bounded git history, active artifacts, briefings
- Tier 3 (verification): direct code reads, targeted tests, runtime captures
- Tier 4 (hardening): schema validators, CI/pre-commit checks

---

## LAYER E: COMPLETION CRITERIA

You are done ONLY when:

- [ ] Contract checksum passes
- [ ] Global typecheck/compile passes
- [ ] Global test suite passes
- [ ] Connectivity gate passes (no hermetic domains)
- [ ] Each worker report exists
- [ ] Integration report exists
- [ ] MANIFEST.md updated with final status
- [ ] STATE.md reflects current truth
- [ ] No [Assumed] claims on the critical path
- [ ] Every P0 surface is reachable and operable

---

## HOW TO USE THIS SUPPLEMENT

**For simple dispatch tasks (visual polish, asset wiring, content additions):**
The original toolkit is sufficient. Don't load this supplement — it adds overhead.

**For architectural work (new systems, cross-domain features, integration):**
Load both the toolkit and this supplement. The governance layer and session discipline prevent the most expensive mistakes.

**For autonomous campaigns (agent runs multiple waves unsupervised):**
Load both, plus put the stop conditions and discovery taxonomy in the dispatch document itself. These are what prevent the agent from silently going off-track during a long run.

**For the next game's Day 1:**
1. Copy the toolkit + this supplement into project knowledge
2. Run bootstrap from toolkit Part 3.1
3. Create the type contract, freeze it
4. Write MANIFEST.md
5. Write the spec with quantities
6. First dispatch
