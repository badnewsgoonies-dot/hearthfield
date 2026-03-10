# Hearthfield Operating Kernel

**Purpose**

This is the canonical machine-facing transfer artifact for `/home/geni/swarm/hearthfield`.
Keep this file at the repo root and update it in place as the game state changes.

This kernel is grounded in:
- current code and working tree
- current repo docs and status files
- recent commit history on `master`

It is not a publication draft.

---

## 1. Trust Order

Use this precedence when sources disagree:

1. Fresh code / tests / runtime inspection
2. Current working tree state
3. `[Observed]` repo artifacts with concrete source refs
4. Current project-state docs (`ORCHESTRATOR_STATE.md`, `MANIFEST.md`, `ROADMAP_TO_SHIP.md`, `ACCEPTANCE.md`)
5. Research-derived doctrine and findings
6. Conversation history or compaction summaries

**Hard rule:** do not treat transcript replay or compaction output as durable memory.
Use them only as routing hints.

---

## 2. Source Set Merged Here

This kernel is based on the sources actually present in the repo now:

- [ORCHESTRATOR_STATE.md](/home/geni/swarm/hearthfield/ORCHESTRATOR_STATE.md)
- [MANIFEST.md](/home/geni/swarm/hearthfield/MANIFEST.md)
- [ROADMAP_TO_SHIP.md](/home/geni/swarm/hearthfield/ROADMAP_TO_SHIP.md)
- [ACCEPTANCE.md](/home/geni/swarm/hearthfield/ACCEPTANCE.md)
- [status/AI_STATUS_OVERVIEW.md](/home/geni/swarm/hearthfield/status/AI_STATUS_OVERVIEW.md)
- user-provided cognitive-science feedback on memory representation, identity drift,
  and the A/B/C ablation framing
- current `git log`
- current `git status`

**Important difference from the older kernel:** there is no `.memory/` directory or
active YAML debt/principle artifact set present in this repo snapshot, so the prior
references to uploaded `DEBT-*` / `PRINCIPLE-*` YAML files should be treated as
historical context, not live on-disk memory.

---

## 3. Current Project Snapshot

### 3.1 Branch / working tree

- Branch: `master`
- Remote relation: `master...origin/master`
- Working tree is **dirty**

Directly observed dirty paths:
- [src/farming/render.rs](/home/geni/swarm/hearthfield/src/farming/render.rs)
- [src/world/mod.rs](/home/geni/swarm/hearthfield/src/world/mod.rs)
- untracked `output/`
- untracked `screens/`

Interpretation:
- committed branch state and current local experimental state are not identical
- treat the last committed green-gate snapshot as baseline, and the current dirt/soil
  visual pass as in-progress local work

### 3.2 Current macro phase

Best current read:
- **post-Wave-10 / post-Wave-10+ feature-polish state**
- no longer a Tier-S “finish spine / Harden” hotfix project
- current repo state is a **late-stage polish, verification, and content-integration** phase

Why:
- `ORCHESTRATOR_STATE.md` says Wave 10 is complete and lists late-stage polish/deploy gaps
  rather than foundational build gaps
- `master` contains multiple commits newer than that document, including player tool
  feedback, sprite integration, content unlock fixes, scenic town houses, and tutorial work

### 3.3 Default tier assumption

Default to **Tier M** unless the incoming task is truly isolated.

Reason:
- current open work tends to span multiple interacting runtime surfaces:
  player, world, farming, UI, save/WASM, and DLC verification

### 3.4 Best documented gate baseline

From [ORCHESTRATOR_STATE.md](/home/geni/swarm/hearthfield/ORCHESTRATOR_STATE.md):
- `cargo check --workspace`: PASS
- `cargo test --test headless`: PASS
- `cargo test -p skywarden --tests`: PASS
- `cargo test -p city_office_worker_dlc --tests`: PASS

From direct inspection today:
- `cargo check`: PASS on the current dirty working tree

From [MANIFEST.md](/home/geni/swarm/hearthfield/MANIFEST.md):
- last documented headless count: **109** passing tests

From [ACCEPTANCE.md](/home/geni/swarm/hearthfield/ACCEPTANCE.md):
- project is **not yet at documented release acceptance**
- acceptance still requires full loop verification, DLC checks, and final gate suite

### 3.5 Current user-facing game state

The current branch includes all of the following:

- Deep Forest runtime surface
- Coral Island and sailing / docking flow
- two enterable scenic town houses: `TownHouseWest`, `TownHouseEast`
- visible held-tool sprites, swing arcs, impact particles, and till dust
- intro tool tutorial driven by Mayor Rex
- richer NPC/world feel: schedules, emotes, smoke, weather particles, tree/grass sway,
  furniture-theme work, and broader object visuals
- content reachability fixes for seeds / shops / recipes / blocked items
- repaired bridge / roof atlas issues that were still open in the old kernel

Primary evidence:
- recent commits:
  - `10b8b888 feat(ui): NPC-driven tool tutorial with visual overlay during intro`
  - `5a02a202 feat(tools): impact feedback for all 6 tools`
  - `8bf943e3 feat(player): visible held-tool sprites + impact particles + till dust poof`
  - `8d1f24c3 feat(world): add TownHouseWest/East interiors, player scale fix, tool anim cleanup`
  - `196fb88d feat(sailing): Wave 3 — boat boarding, sailing movement, dock interaction`
  - `31551153 feat(sailing): Wave 2A — CoralIsland map, object types, full wiring`
  - `c281c773 feat(world): add Deep Forest map`

### 3.6 Claims from the old kernel that are now obsolete

These older claims should be retired:

- “Player uses `npc_farmer.png` placeholder”
  - obsolete; player currently loads from [src/player/spawn.rs](/home/geni/swarm/hearthfield/src/player/spawn.rs)
    using `sprites/character_spritesheet.png`

- “Starter items missing hoe”
  - obsolete; fixed earlier and no longer a live debt on `master`

- “Tool animation uses walk sprite bob; no dedicated art” as a primary debt
  - stale / partial
  - body animation may still intentionally fall back to walk-sheet motion in some local
    iterations, but the current user-facing tool presentation includes held-tool sprites,
    impact particles, swing arcs, and target-side feedback

- `wood_bridge.png` row/index debt
  - obsolete; fixed

- `house_roof.png` empty-row debt
  - obsolete; fixed

### 3.7 Current P0-like / critical-path open work

The repo no longer tracks explicit `P0` / `P1` labels consistently, but the closest
current **critical-path / P0-like** items are:

- Building collision still appears in the shipping roadmap as unfinished
- Tutorial flow / first-week guidance is not yet fully closed
- Visual readability still has active polish work
- Shop auto-entry still requires verification / completion
- Atlas pre-loading is still called out as incomplete
- Crafting bench interaction still needs end-to-end verification

Primary source:
- [ROADMAP_TO_SHIP.md](/home/geni/swarm/hearthfield/ROADMAP_TO_SHIP.md)

### 3.8 Current P1-like / next-wave debts

Closest “next-wave” items:

- fishing loop verification
- mining loop verification
- crafting loop verification
- social loop verification
- economy loop verification
- WASM/browser verification
- performance / endurance
- full-year playthrough
- pilot DLC end-to-end playability verification

Primary sources:
- [ROADMAP_TO_SHIP.md](/home/geni/swarm/hearthfield/ROADMAP_TO_SHIP.md)
- [status/AI_STATUS_OVERVIEW.md](/home/geni/swarm/hearthfield/status/AI_STATUS_OVERVIEW.md)

### 3.9 Current local in-progress debt

As of this snapshot, there is active local visual iteration on terrain / soil:

- [src/world/mod.rs](/home/geni/swarm/hearthfield/src/world/mod.rs)
- [src/farming/render.rs](/home/geni/swarm/hearthfield/src/farming/render.rs)
- screenshot evidence in `screens/`
- ad hoc analysis outputs in `output/`

Treat those as **live local work**, not accepted baseline.

---

## 4. Core Findings and Invariants

These doctrine-level rules from the older kernel still hold and should remain active.

### 4.1 Memory / context

**INVARIANT-001 — Do not preserve the conversation as memory.**

- Preserve outputs as typed, source-linked artifacts when such artifacts exist
- Rebuild the working set fresh per task
- Treat transcript replay and compaction as routing only

**INVARIANT-002 — Fresh context is not blankness.**

- Fresh sessions should mount current state, recent git history, relevant docs,
  and direct code/test inspection

**INVARIANT-003 — Provenance beats assertion.**

- prefer claims with explicit source refs
- only trust `[Observed]` claims without extra verification
- treat memory representation as behavior-shaping, not neutral storage
- if memory changes what the agent believes about the project, it also changes
  what the agent believes about itself and what kind of action is appropriate

**INVARIANT-004 — Compaction is routing, not authority.**

- summaries point
- code and artifacts decide

**INVARIANT-005 — Memory pipelines should be treated like behavior-shaping training data.**

- untyped remembered claims are not just retrieval risk; they can become a
  self-reinforcing evidence stream
- provenance tags do not merely defend factual accuracy; they also defend
  against identity drift by forcing evaluation against source quality

**INVARIANT-006 — The important memory ablation is A/B/C, not just “memory vs no memory.”**

- `A`: accumulated / compacted conversational carry-forward
- `B`: fresh session + untyped retrieval
- `C`: fresh session + typed retrieval with provenance
- the historical experiments in this project already partially instantiate this:
  compacted false-summary adoption corresponds to `A`, sequential untyped notes to `B`,
  and typed evidence-linked artifacts to `C`

### 4.2 Orchestration / scope

**INVARIANT-007 — Scope should be enforced mechanically, not requested conversationally.**

**INVARIANT-008 — Workers build scaffold; orchestrator verifies user-facing reality.**

**INVARIANT-009 — Type contracts and map/runtime contracts matter most when parallel work is active.**

### 4.3 Verification

**INVARIANT-010 — Graduate only `[Observed]` truths.**

**INVARIANT-011 — Green means ready to examine, not ready to ship.**

**INVARIANT-012 — Atlas / sprite index bugs can bypass structural gates.**

- visual verification remains mandatory for atlas index changes
- this is still directly relevant to the current terrain/soil polish work

---

## 5. Session Start Protocol (Current)

For a fresh session on this repo:

1. Read this kernel
2. Read [ORCHESTRATOR_STATE.md](/home/geni/swarm/hearthfield/ORCHESTRATOR_STATE.md)
3. Read [ROADMAP_TO_SHIP.md](/home/geni/swarm/hearthfield/ROADMAP_TO_SHIP.md) for open loop-verification debt
4. Run `git status -sb`
5. Run `git log --oneline -15`
6. Run `git log --oneline -15 -- src/{domain}/` for the touched domain
7. State:
   - tier
   - touched runtime surface
   - whether you are operating on committed baseline or dirty local state
   - any `[Inferred]` / `[Assumed]` claims on the critical path

---

## 6. Wave Cadence

Still use:

`Feature -> Gate -> Document -> Harden -> Graduate`

Operational interpretation for this repo now:

- `Feature`: land the smallest coherent surface change
- `Gate`: compile / tests / lint / checksum as appropriate
- `Document`: update state artifacts only after the investigation is coherent
- `Harden`: run the actual path (especially first-60-seconds and runtime visuals)
- `Graduate`: encode observed truths into tests where practical

---

## 7. Current Read Order for Future Sessions

Use this sequence:

1. [docs/hearthfield_operating_kernel_current.md](/home/geni/swarm/hearthfield/docs/hearthfield_operating_kernel_current.md)
2. [ORCHESTRATOR_STATE.md](/home/geni/swarm/hearthfield/ORCHESTRATOR_STATE.md)
3. [ROADMAP_TO_SHIP.md](/home/geni/swarm/hearthfield/ROADMAP_TO_SHIP.md)
4. `git status -sb`
5. `git log --oneline -15`
6. relevant domain logs via `git log --oneline -15 -- src/{domain}/`
7. direct code / tests / runtime inspection

If the task is shipping / release readiness, also read:
- [ACCEPTANCE.md](/home/geni/swarm/hearthfield/ACCEPTANCE.md)

If the task is historical debugging, also read:
- [status/AI_STATUS_OVERVIEW.md](/home/geni/swarm/hearthfield/status/AI_STATUS_OVERVIEW.md)

---

## 8. One-Screen Kernel

- Do not use transcript replay as durable memory.
- Rebuild from current docs, git status, git `-15`, and direct code/tests.
- Default current work to Tier M unless the task is truly isolated.
- The old “missing hoe / placeholder player art / Tier-S spine finish” snapshot is obsolete.
- Current branch state includes sailing, Coral Island, Deep Forest, town-house interiors,
  tool feedback, and intro tool tutorial work.
- Current project state is late-stage polish / verification, not early spine construction.
- Current critical-path docs still flag building collision, tutorial flow, atlas preloading,
  shop auto-entry, crafting-bench verification, and multiple loop verifications as open.
- Acceptance is stricter than the optimistic state docs; not yet release-complete.
- Visual atlas changes still require runtime verification.
- Working tree is currently dirty with local terrain/soil experiments.

---

## 9. Source Notes

This file supersedes the earlier pasted kernel specifically on the **current game state**
sections. Its doctrine sections are intentionally conservative: keep the proven orchestration
rules unless direct repo evidence says otherwise.

The current version also absorbs the user-provided cognitive-science feedback that:
- the decisive experiment is the three-condition A/B/C memory ablation
- typed retrieval should be evaluated against both compaction and untyped recall
- memory pipelines should be treated as behavior-shaping infrastructure, not as
  passive storage
