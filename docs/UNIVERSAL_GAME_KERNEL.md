# Universal Game Development Operating Kernel — Optimized

**Purpose**

This is the machine-facing operating kernel for game development projects. It generalizes the Hearthfield pattern into a reusable system for any game genre, engine, language, or repo layout.

**What this file replaces**

- transcript replay as durable memory
- scattered boot instructions across notes, quickstarts, and ad hoc prompts
- research-heavy context packs that explain the system but slow the session start

**What this file does not replace**

- direct code, test, runtime, and playtest verification
- project-specific contracts, state files, active debt files, or traces
- recent git history
- human judgment on feel, pacing, readability, or ship quality

**Scope**

- applies to single-player, multiplayer, 2D, 3D, and systemic games
- assumes no specific engine or architecture
- treats game feel and runtime interaction as first-class verification surfaces

---

## 1. Trust Order

When sources disagree, use this precedence:

1. fresh code, tests, runtime captures, live playtest, and scoped inspection
2. `[Observed]` artifacts with concrete `source_refs`
3. current `STATE` snapshot
4. project onboarder, quickstart, memory README, contracts, and specs
5. research-derived findings labeled as `Corpus result`, `Replicated finding`, `Local finding`, `Derived recommendation`, or `Open question`
6. conversation history, compaction summaries, or remembered chat claims

**Hard rule:** transcript replay and compaction are routing hints, not durable memory.

---

## 2. Core Definitions

- **Surface** — a user-facing loop or operator-facing behavior reachable in normal play.  
  Examples: jump and landing feedback, combat preview and confirm, matchmaking join flow, save/load roundtrip.
- **Contract** — any shared shape multiple systems must agree on.  
  Examples: events, save schema, data tables, RPC payloads, inventory definitions, animation assumptions.
- **Gate** — a mechanical binary check.  
  Examples: build, test, lint, asset validation, contract checksum, replay or sim test.
- **Artifact** — a typed memory record on disk: `observation`, `decision`, `debt`, `principle`.
- **Verification** — settling a claim against code, tests, scoped logs, runtime capture, or live playtest.

---

## 3. Minimum Bundle and Baseline Domains

### 3.1 Minimum project bundle

Every game project should expose at least:

```text
project/
├── KERNEL.md
├── STATE.md
├── memory/
│   ├── README.md
│   ├── artifacts/
│   └── briefs/          (optional)
├── contracts/ or specs/
└── reports/ or traces/  (optional but useful)
```

### 3.2 Baseline domain set

Start with a small, stable domain vocabulary and expand only when a domain becomes too large or has unique gates.

Suggested baseline:
- `core`
- `build`
- `input`
- `player`
- `camera`
- `physics`
- `animation`
- `ui`
- `audio`
- `world`
- `ai`
- `combat`
- `items`
- `economy`
- `narrative`
- `save`
- `tools`
- `netcode`
- `perf`
- `assets`

### 3.3 Runtime surfaces

A runtime surface is the smallest player-visible loop worth hardening. Prefer to reason in surfaces, not abstract subsystems.

Examples:
- movement and landing feedback
- combat preview to hit confirm
- save, quit, reload, and restore
- join match, spawn, shoot, and server-validated confirm

---

## 4. Core Invariants

### Memory and truth

**INVARIANT-001 — Do not preserve the conversation as memory.**  
Persist outputs as typed, source-linked artifacts. Rebuild the working set fresh per task.

**INVARIANT-002 — Fresh context is reconstruction, not blankness.**  
A fresh session should mount current state, active debt, relevant artifacts, recent history, and current traces.

**INVARIANT-003 — Provenance visibility is the minimum viable memory defense.**  
Retained claims should carry evidence level, source refs, and status or supersession state.

**INVARIANT-004 — Compaction is routing, not authority.**  
Summaries can help you find the right source. They do not settle what is true.

### Scope and contracts

**INVARIANT-005 — Scope should be enforced mechanically, not requested conversationally.**  
If out-of-scope edits are unacceptable, clamp them mechanically.

**INVARIANT-006 — Freeze shapes, not values.**  
Freeze types, schemas, interfaces, event shapes, and invariants. Leave thresholds, timings, copy, balance, and rates in data or config.

**INVARIANT-007 — Context presence matters more than conversational warmth.**  
A compact, well-curated reference pack transfers better than stale transcript drift.

**INVARIANT-008 — Workers do bounded structural work; orchestrators finish surfaces.**  
Workers are good for scoped implementation. The orchestrator verifies actual behavior, handles ambiguity, and graduates invariants.

### Verification and hardening

**INVARIANT-009 — Only `[Observed]` truths graduate into gates.**  
Never freeze `[Inferred]` or `[Assumed]` claims into release gates, regression tests, or migration guarantees.

**INVARIANT-010 — Load enough history to recover causality, not enough to recreate the world.**  
Use bounded retrieval by default and go deeper only for forensics.

**INVARIANT-011 — Document after investigation, not during it.**  
The Document phase is post-investigation and pre-hardening. Writing memory while debugging produces weak artifacts.

**INVARIANT-012 — Mechanical enforcement and structural support are different tools.**  
Mechanical enforcement prevents behavior. Structural support improves reasoning. Use the first when failure must be impossible, the second when reasoning must stay grounded.

**INVARIANT-013 — Any server-retained session state is an opaque cache, not an audit substrate.**  
Server-side continuation can lower re-read cost, but file-based memory is easier to inspect and correct.

**INVARIANT-014 — Runtime-only classes of bugs always exist.**  
Passing gates does not prove the surface is reachable, understandable, responsive, or correct under real interaction.

**INVARIANT-015 — Memory pipelines are behavior-shaping infrastructure, not passive storage.**  
Untyped remembered claims become a self-reinforcing evidence stream. Provenance tags defend against identity drift — not just factual error — by forcing evaluation against source quality rather than absorption as self-knowledge.

**INVARIANT-016 — The decisive memory comparison is A/B/C, not "memory vs no memory."**  
A: accumulated compacted conversational carry-forward. B: fresh context + untyped retrieval. C: fresh context + typed retrieval with provenance. The architecture targets C. Condition A is what most deployed systems use by default.

---

## 5. Certainty Labels

Use these labels explicitly.

- `Corpus result` — repeated descriptive pattern, not a controlled intervention
- `Replicated finding` — same qualitative effect at `n >= 3`
- `Local finding` — observed at low `n` or with mixed replication
- `Derived recommendation` — protocol choice motivated by findings, not isolated experimentally
- `Open question` — unresolved or insufficiently tested

### 5.1 Load-bearing assignments

- Statefulness premium: `Corpus result`
- Mechanical scope clamping beats prompt-only scope control: `Replicated finding`
- Provenance visibility reduces false-memory adoption: `Replicated finding`
- Evidence tags are a minimum viable defense: `Replicated finding`
- Context quality matters more than warm dialogue format: `Replicated finding`
- Type contracts improve parallel reliability: mixed; treat value as `Replicated finding`, strict necessity as `Local finding`
- Bounded retrieval defaults: `Derived recommendation`
- Structured artifacts solving subtle sycophancy in all cases: `Open question`

---

## 6. Canonical `source_ref` Grammar and Artifact Schema

Use a canonical grammar that works in single-repo or multi-repo settings. Single-repo shorthand is acceptable when ambiguity is impossible.

```yaml
source_refs:
  - "file:<repo>@<path>:<start>-<end>"
  - "commit:<repo>@<sha>"
  - "test:<repo>@<command>#<test_name_or_suite>"
  - "ci:<system>@<run_id_or_url_hash>"
  - "runtime:<capture_type>@<hash_or_path>"
  - "issue:<tracker>@<id>"
  - "doc:<repo>@<path>#<section_anchor>"
```

Minimum artifact schema:

```yaml
id: DEC-2026-03-10-001
type: decision | observation | debt | principle
evidence: Observed | Inferred | Assumed
domain: player | world | save | ui | api | infra | ...
summary: "One sentence. Never nested."
source_refs:
  - "file:repo@src/path/file.rs:10-40"
status: active | resolved | superseded
supersedes: []
```

Useful optional fields:

```yaml
runtime_surface: ""
why_it_matters: ""
drift_cue: ""
contradicts: []
alternatives_considered:
  - option: ""
    rejected_because: ""
recovery: ""
retrieve_when: []
```

Rules:
- one artifact per file
- supersede instead of silently mutating history
- `[Observed]` claims should not have empty `source_refs`
- schema validity is not truth; high-stakes claims still need verification

---

## 7. Session Start Protocol

### 7.1 Boot sequence

1. Read this kernel.
2. Read `STATE.md` or the current project snapshot.
3. Mount the current objective.
4. Pull relevant active artifacts.
5. Run pre-touch retrieval before editing.
6. Fire the first-response protocol before acting.

### 7.2 First-response protocol

Before acting, state:

1. current tier: `S` / `M` / `C`
2. current surface being touched
3. current macro phase
4. current wave phase
5. current `P0` / `P1` debt and missing graduation tests
6. any `[Inferred]` / `[Assumed]` claims on the critical path

### 7.3 Tiering rules

- `S` — single-surface fix or bounded hotfix
- `M` — module or subsystem, 1–3 domains, workers useful
- `C` — campaign, multiple domains, integration and orchestration required

Start at `S` if ambiguous. Escalate when you touch shared contracts, identity, persistence, trust boundaries, or multiple interacting surfaces.

### 7.4 Pre-touch retrieval

Before touching a domain:

1. read `git log --oneline -15 -- <path>`
2. read active artifacts for that domain
3. read the latest worker report or failing trace if one exists
4. state:
   - what changed recently
   - what remains unresolved
   - what is still `[Inferred]` / `[Assumed]`

### 7.5 Integration rule

Start integration in a fresh session. At that point transcript carry is mostly re-read cost. Integration should read artifacts, state, contracts, traces, and current failures from disk.

---

## 8. Context Layers and Retrieval Policy

### 8.1 Context layers

Use stable layers in this order:

- `BP1` — kernel doctrine and invariants
- `BP2` — project docs, contracts, specs, quickstart
- `BP3` — current state, active artifacts, recent git summary, worker reports, failing traces
- `BP4` — live conversation and immediate tool output

### 8.2 Retrieval defaults

- default history depth: `git log --oneline -15 -- <path>`
- use `-20` for causal debugging
- go deeper only for forensics
- load up to **25 artifacts** directly into context by default
- if the relevant pool exceeds **50 artifacts**, build a briefing instead of dumping raw artifacts

### 8.3 Cross-domain retrieval in games

Expand retrieval when touching:
- `save`, `netcode`, `assets`, `animation`, or `perf`
- any fix that crosses input, physics, camera, and feel
- any loop where visual correctness and systemic correctness can disagree

### 8.4 Read order

1. current `STATE`  
2. active `DEBT` and `PRINCIPLE` for touched and cross-cutting domains  
3. recent domain git history  
4. latest report or failing trace  
5. direct code or runtime verification for critical-path claims

---

## 9. Wave Cadence

Always follow:

`Feature -> Gate -> Document -> Harden -> Graduate`

### 9.1 Feature

Build or change the targeted surface or subsystem. Workers are allowed for bounded structural work. Workers should not create orchestration infrastructure instead of the surface.

### 9.2 Gate

Run the relevant mechanical gates:
- build / compile
- typecheck if present
- test
- lint / static analysis
- schema or contract checks when relevant
- scope clamp mechanically

**Rule:** green means ready to examine, not ready to ship.

### 9.3 Document

Emit artifacts only for these triggers:

1. non-obvious decision made
2. direct verification happened
3. reusable principle emerged
4. open debt appeared
5. contradiction surfaced
6. correction invalidated prior belief
7. feel, UX, or surface-quality failure diagnosed
8. graduation test created

If nothing triggers, write nothing.

### 9.4 Harden

Inspect the actual surface. Ask:
- reachable end-to-end?
- feedback visible?
- responsive enough?
- edge behavior sane?
- safe at trust boundaries?
- diagnosable when it fails?

If reachable but wrong or confusing, it is not finished.

### 9.5 Graduate

For each `[Observed]` truth:
- name the invariant
- encode it as a test or gate
- add it to the gate suite
- track remaining ungraduated work as `P0`, `P1`, or `P2`

Do not start the next wave until Document, Harden, and Graduate are complete.

---

## 10. Verification Triggers

Escalate to direct verification when any of these triggers fire.

- **V1 — Assumed / Inferred claim blocks a `P0` or `P1` decision.**
- **V2 — Two artifacts conflict, or a supersedes chain is ambiguous.**
- **V3 — A single artifact is decisive for a high-stakes question such as release, save integrity, netcode correctness, or security.**
- **V4 — The claim depends on visuals, interaction, timing, animation, readability, or feel.**
- **V5 — Tool output is untrusted or weakly scoped.**

Use the cheapest defense that can resolve the ambiguity:

- **Tier 1 — Always on:** evidence tags, source refs, typed artifacts, state
- **Tier 2 — Selective retrieval:** bounded git history, active artifacts, briefings
- **Tier 3 — Verification:** direct code reads, targeted tests, scoped logs, runtime captures, playtests
- **Tier 4 — Write-path hardening:** schema validators and CI or pre-commit checks for malformed artifacts

### Defense Selection by Failure Mode

| Failure mode | Tier 1 (always on) | Tier 2 (selective retrieval) | Tier 3 (verification) |
|---|---|---|---|
| Poisoning / false remembered claim | Evidence tags + source refs | Selective artifact retrieval | Tool / file verification |
| Single false artifact (no competing claims) | **Tier 1 insufficient** | Redundant artifact retrieval | Mechanical file verification (mandatory) |
| Single false artifact (no competing claims) | **Tier 1 insufficient** | Redundant artifact retrieval | Mechanical file verification (mandatory) |
| Sycophancy / momentum drift | Evidence tags on decision artifacts | DEBT / PRINCIPLE artifacts in context | Tool verification |
| Retry loops / repeated bad attempts | `alternatives_considered` field | Relevant episode retrieval | Tool drill-down |
| False proceed / hidden blocker | Clear decision artifacts with evidence | Debt / principle retrieval | Tool verification |
| Missing causal understanding | `git log --oneline -15 -- <path>` | Targeted episodic retrieval | Deep history / forensic read |
| Conflicting truths | Typed evidence hierarchy | Source-linked artifact retrieval | Direct file / tool check |
| Cold-start failure | Kernel + STATE | Full artifact load | Full codebase tool access |
| Stale artifact after human edit | CI hook downgrades modified source_refs | Re-verify touched artifacts | Direct code inspection |
| Single false artifact (no competing claim) | Evidence tags do NOT help here | Redundant artifact from independent source | Mechanical verification (read the file) |
| Runtime-only bug class | Assume it exists (INVARIANT-014) | Cross-domain retrieval expansion | Runtime capture / playtest |



**Capability threshold note:** Full YAML artifact schema works on all models tested including the cheapest (codex-mini: 4/4 on double-poisoning). Inline text tags work on frontier models (94%) but fail on cheap models (codex-mini: 0/5). When model capability is uncertain, use the full schema.

**Verification fabrication warning:** Some models will assert verification occurred without actually verifying — claiming files exist that do not. Prompt-level verification policies work on current frontier models but should not be assumed universal. When a single artifact is decisive, use mechanical verification (actually read the file), not a verification instruction.

---

## 11. Game Surface Adapters

### 11.1 Common harden checklist for games

Every game surface should be checked for:
- reachability
- visible and timely feedback
- responsiveness
- readability and clarity
- edge behavior
- persistence or rollback safety when relevant
- observability for the next failure

### 11.2 Genre-specific emphasis

- **2D platformer** — input latency, coyote time, landing feedback, camera transitions, collision and respawn feel
- **Tactical RPG** — determinism, preview correctness, save/load order fidelity, AI evaluation, UI explanation of outcomes
- **Multiplayer shooter** — server authority, tick alignment, prediction and reconciliation, anti-cheat boundaries, replay and hit validation

### 11.3 Runtime-only bug classes worth assuming up front

- asset indexing and atlas mismatches
- animation timing drift
- camera and collision disagreement
- save/load identity drift
- replay or determinism divergence
- prediction versus authority desync
- feel failures that pass every structural gate

---

## 11. Orchestration Rules and Stop Conditions

### 11.1 Worker rules

Every worker spec should include:

- implement only the scoped deliverable
- do not create orchestration infrastructure
- do not redefine shared contracts locally
- stay within the assigned surface or path
- report assumptions explicitly

### 11.2 Shared contract rule

Contract before workers. Freeze shapes before parallel execution. Leave tunable values in data or config.

### 11.3 After every worker

- clamp scope mechanically
- verify contract integrity
- run all relevant gates
- update state if the active plan changed
- do not merge a green but unreachable result

### Playbook reference

For Tier M or C work requiring multi-worker dispatch, follow the **Sub-Agent Playbook** in order. The playbook defines: frozen type contract, contrastive worker specs with Decision Fields, scope clamping scripts, bounded fix loops, reality gates (entrypoint, first-60-seconds, asset/content reachability, event connectivity, save/load roundtrip), graduation procedure (player trace → named test), and fresh-session integration protocol. Tier S work does not require the playbook.

### 11.4 Stop conditions

Stop and reassess if any of these appear:

1. beautiful dead product — gates are green but the surface is unreachable or unhelpful
2. cadence break — you are about to document while still coding or debugging
3. assumption leak — `[Assumed]` or `[Inferred]` claims are deciding shipping or `P0` / `P1` action
4. abstraction reflex — you are redesigning architecture to avoid debugging the real issue
5. cross-domain drift — the change spans more than two domains without updating contract, specs, and state
6. clamp breaks the fix — the seam is wrong or the task was really integration work
7. false green — tests pass but the contract is unused, bypassed, or visually broken
8. ghost progress — nothing newly reachable or operable exists after the wave
9. critical-path uncertainty — the critical path is not fully `[Observed]`
10. self-model error — the agent is reasoning from incorrect claims about its own tools, permissions, or state

Trust mechanical indicators, not self-reports.

---

## 12. Session End Protocol

1. Update `STATE.md` or the current snapshot with phase, debts, decisions, gate status, and uncertainties.
2. Write triggered artifacts only.
3. Commit memory changes if memory is git-backed.
4. Record any new graduation tests or remaining debt.
5. Do not rely on chat history to preserve what was learned.

---

## 13. One-Screen Kernel

- Do not use transcript replay as durable memory.
- Rebuild from `STATE`, artifacts, bounded git history, and current traces.
- State tier, surface, macro phase, wave phase, debt, and assumptions before acting.
- Use `Feature -> Gate -> Document -> Harden -> Graduate` with no skips.
- Green means ready to examine, not ready to ship.
- Clamp scope mechanically.
- Trust only `[Observed]` claims without extra verification.
- Evidence tags and source refs are the minimum viable memory defense.
- Compaction is routing only.
- Runtime feel and visual correctness are first-class verification surfaces.
- If a claim is high-stakes, runtime-dependent, conflicting, or single-source decisive, verify it.
