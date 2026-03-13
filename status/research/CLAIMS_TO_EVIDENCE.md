# Claims To Evidence

Purpose: list the orchestration claims that are actually supported by files/commits in this repo and adjacent session worktrees, and identify where the evidence is weak.

## Supported Claims

### 1. `Observed` — The session used a contract-first content-expansion branch stack.
- Evidence:
  - `/home/geni/swarm/hearthfield/docs/CONTENT_EXPANSION_DISPATCH.md`
  - `/home/geni/swarm/hearthfield/CLAUDE.md`
  - `/home/geni/swarm/hearthfield/.memory/STATE.md`
  - `b0e0e9ed` in `/home/geni/swarm/hearthfield-dispatch-base`
- Why supported:
  - The dispatch doc explicitly requires contract amendment before launch.
  - A real base-branch commit exists: `b0e0e9ed contract: extend MapId for content expansion`.
- Gap:
  - No single branch ledger records the moment-by-moment branch launch order.

### 2. `Observed` — The campaign actually ran as multiple sibling worktrees, not just as a plan.
- Evidence:
  - `git worktree list` in `/home/geni/swarm/hearthfield`
  - Worktrees present:
    - `/home/geni/swarm/hearthfield-dispatch-base`
    - `/home/geni/swarm/hearthfield-interiors`
    - `/home/geni/swarm/hearthfield-lake-plaza`
    - `/home/geni/swarm/hearthfield-mountain-range`
    - `/home/geni/swarm/hearthfield-collision-pass`
    - `/home/geni/swarm/hearthfield-session-audit`
    - `/home/geni/swarm/hearthfield-integration`
- Why supported:
  - These worktrees and branch heads exist on disk.
- Gap:
  - No durable artifact records which worker owned which branch at each moment.

### 3. `Observed` — The interiors slice was completed as a real branch output.
- Evidence:
  - `/home/geni/swarm/hearthfield-interiors/status/workers/interiors-complete.md`
  - `6295b2ed feat(world): add town interiors slice`
- Why supported:
  - Report and branch tip align on files delivered: `player_house.ron`, `town_house_*.ron`, `tavern.ron`, `library.ron`, `town.ron`, `src/world/objects.rs`, `src/world/mod.rs`.
- Gap:
  - Full gate completion was not achieved in that worktree because of cargo contention.

### 4. `Observed` — The lake-plaza slice was completed as a real branch output.
- Evidence:
  - `/home/geni/swarm/hearthfield-lake-plaza/status/workers/lake-plaza-complete.md`
  - `c2237231 feat(world): add lake plaza slice`
- Why supported:
  - Report and branch tip align on `lake_plaza.ron`, `lakeside_bar.ron`, `town.ron`, `src/world/*`, `src/npcs/schedules.rs`.
- Gap:
  - Full gate completion was not achieved in that worktree because of shared-target cargo contention.

### 5. `Observed` — The mountain slice was completed as a real branch output.
- Evidence:
  - `/home/geni/swarm/hearthfield-mountain-range/status/workers/mountain-range-complete.md`
  - `458a3b51 feat(world): add mountain range slice`
  - `c678018c docs(status): record mountain range completion`
- Why supported:
  - Branch contains the mountain RON maps, `mountain_decor.rs`, and the supporting world/transition/weather files.
- Gap:
  - Validation was partial and timing-limited.

### 6. `Observed` — Collision hardening was elevated into a central rule instead of staying as map-specific door hacks.
- Evidence:
  - `/home/geni/swarm/hearthfield-collision-pass/status/workers/collision-pass-complete.md`
  - `82b161e9 fix(world): derive outdoor collision from building footprints`
  - Integrated commit `3ae4ee4b fix(world): derive outdoor collision from building footprints`
  - File: `/home/geni/swarm/hearthfield-integration/src/world/mod.rs`
- Why supported:
  - The report explicitly states the shift to registry-driven door exemptions and the removal of stale farmhouse carve-outs.
- Gap:
  - Runtime/playback evidence for every entrance pair is still limited.

### 7. `Observed` — A session-audit lane repaired clear visual miswirings from prior work.
- Evidence:
  - `/home/geni/swarm/hearthfield-session-audit/status/workers/session-audit-complete.md`
  - `9eac5b42 fix(world): recover visual mapping miswires`
  - Integrated commit `4bd8e4d8 fix(world): recover visual mapping miswires`
  - File: `/home/geni/swarm/hearthfield-integration/src/world/objects.rs`
- Why supported:
  - The report and commit both name the same repairs: dock atlas, toned-down town shell tinting, and residential furniture atlas corrections.
- Gap:
  - The report explicitly says some art problems remain broader than this narrow repair.

### 8. `Observed` — An integration branch was actually assembled from the session outputs.
- Evidence:
  - `/home/geni/swarm/hearthfield-integration`
  - `git log --oneline -10` there
  - Commit chain:
    - `4f3cb85d` interiors
    - `101ac6ec` lake plaza
    - `9b6cd3a5` mountain range
    - `3ae4ee4b` collision
    - `4bd8e4d8` session audit
    - `9538b4d3` tool feel
    - `89f7b140` eased impact feedback
    - `10c0115e` atlas prewarm
- Why supported:
  - The integrated branch exists on disk with an ordered merge/cherry-pick history.
- Gap:
  - There is no single integrated narrative report on disk that explains every conflict resolution choice.

### 9. `Observed` — Cargo contention and environment termination were a real operational constraint in this session.
- Evidence:
  - `/home/geni/swarm/hearthfield-interiors/status/workers/interiors-complete.md`
  - `/home/geni/swarm/hearthfield-lake-plaza/status/workers/lake-plaza-complete.md`
  - `/home/geni/swarm/hearthfield-mountain-range/status/workers/mountain-range-complete.md`
  - `/home/geni/swarm/hearthfield-collision-pass/status/workers/collision-pass-complete.md`
  - `/home/geni/swarm/hearthfield-session-audit/status/workers/session-audit-complete.md`
- Why supported:
  - Multiple independent reports describe lock contention, terminated cargo builds, or incomplete validation.
- Gap:
  - No retained machine log cleanly quantifies how often or why the jobs were killed.

### 10. `Observed` — Atlas preload remained a live debt and then received a partial implementation on the integration branch.
- Evidence:
  - `/home/geni/swarm/hearthfield/.memory/STATE.md` (`Atlas pre-loading incomplete`)
  - Integrated commit `10c0115e fix(loading): prewarm gameplay visual atlases`
  - Files:
    - `/home/geni/swarm/hearthfield-integration/src/player/spawn.rs`
    - `/home/geni/swarm/hearthfield-integration/src/player/mod.rs`
    - `/home/geni/swarm/hearthfield-integration/src/world/mod.rs`
    - `/home/geni/swarm/hearthfield-integration/src/farming/mod.rs`
- Why supported:
  - The debt is explicitly recorded, and the later integration commit adds earlier preload hooks.
- Gap:
  - This is a preload/prewarm step, not a fully verified “asset-ready barrier” with runtime proof.

## Supported But Still Weak / Inferred

### 11. `Inferred` — The session demonstrated the need for a serialized foreman dispatch ledger.
- Evidence:
  - There is no file in the repo or sibling worktrees that records: active branches, last known worker state, merge order, or next action.
  - Work had to be reconstructed from `git worktree list`, branch tips, and worker reports.
- Why only inferred:
  - The absence is directly observable; the causal claim (“this would have prevented drift”) is not experimentally proven here.

### 12. `Inferred` — The orchestration quality depends on more than the long docs alone.
- Evidence:
  - Canonical docs exist (`CLAUDE.md`, kernel, playbook, dispatch plan), but the live session outputs still had to be reconstructed from worktrees, commits, and operator intervention.
  - `status/integration.md` was not maintained as a live artifact.
- Why only inferred:
  - The gap is visible, but the exact transfer failure mode is not fully serialized in the repo.

## Claims Not Adequately Supported Here

### A. “The primitive transfers cleanly to a new operator with no Geni involvement.”
- Status: Not supported here.
- Why:
  - No multi-operator transfer trial artifact exists on disk.
  - No A/B comparison exists between Geni and an external foreman using only the repo package.

### B. “The docs alone force the intended scout -> implement -> integrate rhythm.”
- Status: Not supported here.
- Why:
  - The docs are present, but there is no on-disk session artifact proving that they mechanically enforced the rhythm without operator correction.

### C. “All branch outputs were fully validated before integration.”
- Status: Not supported here.
- Why:
  - Multiple branch reports explicitly record incomplete cargo/test/clippy validation due contention.

## Highest-Value Gaps For Future Study

1. Missing durable branch/worker ledger (`dispatch-state.yaml` equivalent)
2. Missing integrated-session report explaining actual conflict resolutions
3. Missing retained local build logs / session logs as evidence
4. Missing external-operator transfer experiment
