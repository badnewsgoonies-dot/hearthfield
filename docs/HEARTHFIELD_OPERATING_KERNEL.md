# Hearthfield Operating Kernel — Optimized

**Purpose**

This is the machine-facing operating kernel for the Hearthfield project. Use it to reconstruct the working set for a fresh session without replaying long transcripts.

**Status note**

This file is the Hearthfield-specific adapter layer, not the live project
snapshot.

Treat the following as live truth instead:

- [.memory/STATE.md](/home/geni/swarm/hearthfield/.memory/STATE.md)
- [dispatch-state.yaml](/home/geni/swarm/hearthfield/status/foreman/dispatch-state.yaml)
- tranche reports in [status/launch](/home/geni/swarm/hearthfield/status/launch)
- worker reports in [status/workers](/home/geni/swarm/hearthfield/status/workers)

If this file's embedded snapshot-style content disagrees with those live files,
the live files win.

**What this file replaces**

- transcript replay as memory
- separate lookup across onboarder, quickstart, memory README, `STATE`, and loaded artifacts
- ad hoc boot instructions that drift between sessions

**What this file does not replace**

- direct code, test, runtime, and visual verification
- recent git history and live traces
- project artifacts that were not loaded into this bundle
- human judgment on product feel or ship quality

---

## 1. Trust Order

When sources disagree, use this precedence:

1. fresh code, tests, runtime inspection, captures, and live gates
2. `[Observed]` artifacts with concrete `source_refs`
3. current `STATE` snapshot and this project snapshot
4. project onboarder, quickstart, memory README, and shared contract docs
5. research-derived findings labeled as `Corpus result`, `Replicated finding`, `Local finding`, `Derived recommendation`, or `Open question`
6. conversation history, compaction summaries, or remembered chat claims

**Hard rule:** transcript replay and compaction are routing hints, not durable memory.

---

## 2. Core Definitions

- **Surface** — a player-visible loop or operator-visible behavior that can be exercised end-to-end.  
  Examples: movement and animation, hoe-to-soil farming loop, planting validation, save/load roundtrip, world object rendering.
- **Contract** — any shared shape multiple systems must agree on.  
  Examples: event payloads, save schema, component fields, texture atlas indexing assumptions, tool enums.
- **Gate** — a mechanical binary check.  
  For Hearthfield this includes `cargo check`, `cargo test`, `cargo clippy`, contract checks, and any future sprite-sheet validators.
- **Artifact** — a typed memory record on disk: `observation`, `decision`, `debt`, `principle`.
- **Verification** — settling a claim against code, test output, runtime inspection, screenshot or video, or scoped logs.

---

## 3. Loaded Source Set and Current Project Snapshot

### 3.1 Loaded source set

This kernel folds together the currently loaded Hearthfield operating materials:

- onboarder / build methodology
- `QUICKSTART`
- `.memory/README`
- `STATE`
- `DEBT-player-starter-items-hoe`
- `PRINCIPLE-tileset-silent-row-overflow`

### 3.2 Current phase

- macro phase: `finish spine`
- wave phase: `Harden`
- tier: `S`

### 3.3 Current `P0` debt

- player uses `npc_farmer.png` placeholder; no dedicated sprite
- tool animation uses walking bob; no dedicated art surface

### 3.4 Current `P1` debt

- starter items do not include a hoe; this blocks the farming critical path
- `Hoe -> ToolUseEvent -> soil` is not yet verified end-to-end
- season validation on planting is not yet confirmed

### 3.5 Known runtime or visual risks

- `wood_bridge.png`: row 1 is referenced but only row 0 has art
- `house_roof.png`: rows 3–4 are empty while indices `22–34` reference them
- mining area floor variation is visually weak
- atlas row and column mismatches can pass compile and headless tests but fail visually at runtime

### 3.6 Last recorded decisions

- swapped `character_spritesheet` to `npc_farmer.png` (`fe0b9d3`) `[Observed]`
- added system ordering for `animate_player_sprite` (`fe0b9d3`) `[Observed]`
- removed playbook docs from the repo and deliver via terminal onboarder (`e311de3`) `[Observed]`

### 3.7 Current gate status

- `cargo check`: PASS
- `cargo test`: `129/129` PASS
- `cargo clippy`: `0 warnings`

### 3.8 Active loaded artifacts

**DEBT-player-starter-items-hoe**
- type: `debt`
- evidence: `Observed`
- domain: `player`
- summary: `grant_starter_items` grants seeds, wood, stone, and bread but no hoe, blocking the farming loop
- source refs:
  - `src/player/interaction.rs:506-540`
  - test output showing the hoe is absent from starter items
- drift cue: resolved when a hoe is granted and a test asserts it

**PRINCIPLE-tileset-silent-row-overflow**
- type: `principle`
- evidence: `Observed`
- domain: `world`
- summary: `TextureAtlas` row and column mismatches are silent in compile and headless tests and surface only visually
- source refs:
  - `src/world/mod.rs:402`
  - `src/world/objects.rs:1541-1558`
- drift cue: resolved when a sprite-sheet validator checks atlas indices against image dimensions

### 3.9 Coverage gap

`STATE` mentions additional active concerns whose dedicated artifact files were not loaded here. Treat them as live briefing items, not fully loaded memory.

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

Use these layers in order:

- `BP1` — this kernel and its invariants
- `BP2` — `STATE`, `QUICKSTART`, onboarder, memory README, and contract docs
- `BP3` — active artifacts, recent git history, worker reports, failing traces
- `BP4` — live chat tail and tool output

### 8.2 Retrieval defaults

- default history depth: `git log --oneline -15 -- src/{domain}/`
- use `-20` for causal debugging
- load up to **25 artifacts** directly by default
- if the relevant pool exceeds **50 artifacts**, assemble a briefing instead of loading everything

### 8.3 Cross-domain retrieval for Hearthfield

Expand retrieval when touching:
- `save`, `world`, `player`, `tools`, `animation`, or `assets`
- atlas indexing, tool events, save identity, or planting rules
- any fix that crosses rendering and gameplay state

### 8.4 Practical read order for Hearthfield

1. Section 3 of this file  
2. active domain artifacts  
3. recent domain git history  
4. latest worker report or failing trace  
5. direct code, test, or runtime evidence for the critical path

---

## 9. Wave Cadence

Always follow:

`Feature -> Gate -> Document -> Harden -> Graduate`

### 9.1 Feature

Build or change the targeted runtime surface. Workers may do bounded structural work, but they should not create orchestration infrastructure.

### 9.2 Gate

Run Hearthfield’s mechanical gates:
- `cargo check`
- `cargo test`
- `cargo clippy`
- contract or checksum checks if relevant
- scope clamp mechanically

**Rule:** green means ready to examine, not ready to ship.

### 9.3 Document

Emit artifacts only for the defined triggers. If nothing triggered, write nothing.

### 9.4 Harden

Inspect the actual runtime surface. For Hearthfield this explicitly includes:
- reachability in live play
- visible feedback and readability
- animation timing and feel
- edge behavior around tool use, save/load, and atlas-driven rendering

Any atlas or art-index change requires visual verification.

### 9.5 Graduate

For each `[Observed]` truth:
- name the invariant
- write the regression test or validator
- add it to the gate suite
- track remaining ungraduated work as `P0`, `P1`, or `P2`

---

## 10. Verification Triggers

Escalate to direct verification when any of these triggers fire.

- **V1 — Assumed / Inferred claim blocks a `P0` or `P1` decision.**
- **V2 — Two artifacts conflict, or a supersedes chain is ambiguous.**
- **V3 — A single artifact is decisive for a release-critical question.**
- **V4 — The claim depends on runtime visuals, timing, interaction, or feel.**
- **V5 — Tool output is untrusted or weakly scoped.**

Use the cheapest defense that can resolve the ambiguity:

- **Tier 1 — Always on:** evidence tags, source refs, typed artifacts, state
- **Tier 2 — Selective retrieval:** active `DEBT` and `PRINCIPLE`, bounded git history, compact briefings
- **Tier 3 — Verification:** direct code reads, targeted tests, runtime captures, visual inspection, scoped logs
- **Tier 4 — Write-path hardening:** schema validation and pre-commit or CI checks for malformed artifacts

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
| Atlas / art index mismatch | PRINCIPLE-tileset-silent-row-overflow | Cross-domain retrieval on world + assets | Visual verification (mandatory) |

---

## 11. Orchestration Rules and Stop Conditions

### 11.1 Worker rules

Every worker spec should include:

- implement only the scoped domain deliverable
- do not create orchestration infrastructure
- do not redefine shared contracts locally
- stay within the assigned surface or path
- report assumptions explicitly

### 11.2 Shared contract rule

Contract before workers. Freeze shapes before parallel work.

### 11.3 After every worker

- clamp scope mechanically
- verify contract integrity
- run all relevant gates
- update state if the active plan changed
- do not merge a green but visually broken or unreachable result

### Playbook reference

For Tier M or C work requiring multi-worker dispatch, follow the **Sub-Agent Playbook** in order. The playbook defines: frozen type contract, contrastive worker specs with Decision Fields, scope clamping scripts, bounded fix loops, reality gates (entrypoint, first-60-seconds, asset/content reachability, event connectivity, save/load roundtrip), graduation procedure (player trace → named test), and fresh-session integration protocol. Tier S work does not require the playbook.

### 11.4 Hearthfield stop conditions

Stop and reassess if any of these appear:

1. gates are green but the player-facing surface is dead, ugly, confusing, or unreachable
2. you are documenting while still debugging
3. an `[Assumed]` or `[Inferred]` claim is deciding shipping or `P0` / `P1`
4. the task silently expanded across `player`, `world`, `save`, or `assets` without updating the plan
5. an atlas or art change has not been visually checked
6. the clamp breaks the fix, meaning the task was really integration work
7. nothing newly reachable exists after the wave
8. the critical path is not fully `[Observed]`

Trust mechanical indicators and live runtime behavior, not self-reports.

---

## 12. Session End Protocol

1. update `STATE` with phase, debt, decisions, gate status, and uncertainties
2. write triggered artifacts only
3. record any new graduation tests or remaining debt
4. commit memory changes if memory is git-backed
5. if integration is next, start it in a fresh session

---

## 13. One-Screen Kernel

- Do not use transcript replay as durable memory.
- Rebuild from `STATE`, artifacts, git `-15`, and current traces.
- State tier, surface, macro phase, wave phase, debt, and assumptions before acting.
- Use `Feature -> Gate -> Document -> Harden -> Graduate` with no skips.
- Green means ready to examine, not ready to ship.
- Clamp scope mechanically.
- Trust only `[Observed]` claims without extra verification.
- Evidence tags and source refs are the minimum viable memory defense.
- Compaction is routing only.
- Use fresh sessions for integration.
- Current live blockers: missing hoe starter item, placeholder player art, runtime-only atlas failures.
