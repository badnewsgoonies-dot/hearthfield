# Git Archaeology Report

Repository: `/home/user/hearthfield`  
Analysis date: 2026-03-06 (UTC)  
Scope: full Git history through `19289ef`.

## Commands Executed
- `git log --oneline --reverse`
- `git log --stat --reverse`
- `git shortlog -sn` (produced no rows in this repo context)
- `git shortlog -sn HEAD` (used for contributor counts)
- Supporting queries for dates, per-path churn, and file-specific history:
  - `git rev-list --count HEAD`
  - `git log --reverse --date=... --pretty=...`
  - `git log --reverse --numstat --pretty=...`
  - `git log --reverse -- ... src/shared/mod.rs`

## Topline Facts
- Total commits: **305** (`git rev-list --count HEAD`)
- Merge commits: **34** (`git rev-list --count --merges HEAD`)
- Non-merge commits: **271**
- Active commit dates: **2026-02-26 to 2026-03-06** (9 calendar days with commits)

Contributor counts (`git shortlog -sn HEAD`):
- 100 `Claude`
- 84 `jim jam`
- 57 `Geni (Orchestrator)`
- 28 `badnewsgoonies`
- 17 `Geni Xure`
- 17 `Hearthfield Orchestrator`
- 1 `Orchestrator`

## Build Phases (with commit/date evidence)

### 1) Scaffolding
- **2026-02-26** `4d61117`: initial scaffold, main plugin registration, and domain stubs; includes `src/shared/mod.rs` 925-line contract.
- **2026-02-26** asset bootstrap immediately follows:
  - `9d06546` sprites pack
  - `c7b0565` UI pack
  - `759650d` audio pack

Interpretation: project skeleton + production assets were laid down first, before iterative gameplay feature waves.

### 2) Domain Implementation
- Initial domain implementation wave starts same day:
  - **2026-02-26** `3ef6aa4` “implement 10 domain plugins ... (WIP)”
  - `1702100`, `a056b90`, `239b63d`, `887a314`, `320c374`, `d9f09df`, `588ce36` (wave progression)
- Feature phase progression continues:
  - **Phase 3**: `a5fb75e`, `f8806a6`, `81d43bf`, `3a6799e`, `32c756d`
  - **Phase 4**: `474903f`, `497bf67`, `14a0be0`, `aa0f8b1`
  - **Phase 5** (rendering/animation): `bb27c6e` (2026-02-27)
- Cross-cutting gameplay slice:
  - **2026-02-28** `1a60310` vertical slice (cutscene runner, intro, day transitions, tutorial)

Interpretation: implementation was executed as explicit phased waves, not one domain at a time.

### 3) Integration
- Regular PR merge cadence begins early (`73f04d1`, `85b311d`, `0758531`, `ed63aff`, etc.).
- Test harness integration checkpoint:
  - **2026-02-26** `cef42ae` headless ECS test harness (25 integration tests)
- Structural integration pivot:
  - **2026-03-03** `b1fbe67` unify into cargo workspace with all DLCs
- Post-workspace merge/conflict integration appears immediately after (`1967ce3`, multiple merge commits on 2026-03-03/04).

Interpretation: there is a clear second integration phase around multi-crate/DLC unification.

### 4) Polish
- Early polish begins alongside implementation:
  - `ab3dd1d` menu save/load polish (2026-02-26)
  - `c9d8e78` sprite wiring + warning suppression (2026-02-27)
- UX/content polish intensifies during 2026-03-04 orchestration waves:
  - `b0d4285` UX feedback layer
  - `8ab5516`, `dc7d09d`, `5876cc4` feedback/weather toasts
  - `f8a2d2e`, `d0550a8`, `6610909` city DLC content/UI depth
- Platform polish late:
  - **2026-03-05** `21d135d`, `249c284`, `2262953`, `3852c88` (gamepad/touch support and fixes)

### 5) Bugfix / Hardening
- Continuous bugfix stream after initial feature waves:
  - **2026-02-26** `d803c3f` 3 critical gameplay bugs
  - **2026-02-27** `3711bd3` audit fixes (9 bugs)
  - **2026-03-03/04** dedicated wave bug sweeps: `8ffea1d`, `c66bbfa`, `0af8bb0`
  - **2026-03-06** second-zero audit fixes: `dd4fcec`, `4750de9`

Interpretation: stabilization was iterative, with explicit audit-driven fix campaigns.

## Timeline Mapping: What Was Built When
- **Feb 26**: project created, assets imported, first domain implementation waves, Phase 3/4 kickoff, initial merges.
- **Feb 27**: deeper systems/features (quests, automation, contracts), input abstraction pivot (`6868082`), rendering phase, audit fixes.
- **Feb 28**: spatial truth fixes, event dispatcher wiring, vertical slice, more safety fixes.
- **Mar 1**: Tier 0 loop wiring (`00ac666`) and broad gameplay/UX fixes.
- **Mar 2**: largest rework day in shared contracts and gameplay correctness (many orchestrator commits); heavy cleanup/testing.
- **Mar 3**: major architecture pivot to multi-crate workspace with DLCs (`b1fbe67`) plus bug sweep waves.
- **Mar 4**: highest velocity day, substantial DLC depth + UX/polish + contract fixes + wasm fix.
- **Mar 5**: platform-input expansion (gamepad/touch), then broader polish.
- **Mar 6**: second-zero audit plan + 8 bug fixes + retrospective setup docs.

## `src/shared/mod.rs` Forensics

### How many times modified
- Modified in **39 commits** (`git log -- ... src/shared/mod.rs`)

### By whom
- 15 `Geni (Orchestrator)`
- 15 `Claude`
- 4 `badnewsgoonies`
- 3 `jim jam`
- 2 `Geni Xure`
- 1 `Hearthfield Orchestrator`

### When (date concentration)
- 2026-02-26: 3 commits
- 2026-02-27: 6 commits
- 2026-02-28: 5 commits
- 2026-03-01: 2 commits
- 2026-03-02: 16 commits (largest burst)
- 2026-03-03: 3 commits
- 2026-03-04: 4 commits
- 2026-03-05: 1 commit

### Notable rework evidence in `src/shared/mod.rs`
- Contract expansion waves: `a5fb75e`, `474903f`, `dbd8fff`, `096fcf8`
- Abstraction refactor pivot: `6868082` (input/menu abstraction)
- Cleanup/dead-code removal and test-linked contract edits around Mar 2: `d51374e`, `52005c4`, `ddcaf12`, `457b7cc`, `bcdc2f2`, `8e7faf0`
- Late keybinding contract correction: `ecfa262`

Interpretation: `src/shared/mod.rs` served as the integration seam and was reworked repeatedly as systems converged.

## Which Domains Were Built First vs Reworked Most

### Built first
- In scaffold commit `4d61117`, stubs were created for almost all domains simultaneously (`animals`, `calendar`, `crafting`, `data`, `economy`, `farming`, `fishing`, `mining`, `npcs`, `player`, `save`, `shared`, `ui`, `world`).
- `input` appears as a later explicit domain path first touched on **2026-02-27** (`6868082`), indicating it became a distinct concern after initial scaffold.

### Most rework (by commit-touch count on `src/<domain>/`)
- `ui`: 72 commits touching domain files
- `world`: 59
- `npcs`: 56
- `player`: 44
- `shared`: 39

### Most churn (added+deleted lines in `src/<domain>/`)
- `ui`: 12,981
- `npcs`: 9,925
- `world`: 8,608
- `data`: 6,326

### Highest deletion pressure (deletions/additions ratio; proxy for rework intensity)
- `npcs`: 0.294
- `player`: 0.259
- `economy`: 0.243
- `world`: 0.218

Conclusion: `ui`, `world`, and `npcs` absorbed the largest sustained rework; `npcs` and `player` show especially high rewrite pressure relative to net growth.

## Commit Velocity, Stalls, and Pivots

### Daily commit velocity
- 2026-02-26: 35
- 2026-02-27: 30
- 2026-02-28: 22
- 2026-03-01: 34
- 2026-03-02: 64
- 2026-03-03: 42
- 2026-03-04: 66 (peak)
- 2026-03-05: 7
- 2026-03-06: 5

Interpretation: two high-throughput peaks (Mar 2 and Mar 4), then sharp deceleration into final audit/fix/documentation.

### Longest stalls (inter-commit gaps)
- **19.2h** between `1b2771f` (2026-03-05 08:39Z) and `adfa982` (2026-03-05 22:51 -0500)
- **16.2h** between `adfa982` and `3d22f53` (Mar 6 docs/audit planning)
- **12.0h** between `95a710d` and `e87ff59` on Mar 2

Interpretation: pauses align with handoffs between campaign phases (platform/polish -> audit-driven fixes).

### Pivots
- **Scaffold -> wave implementation**: `4d61117` -> `3ef6aa4` (same day)
- **Feature waves -> testing/vertical slice**: `cef42ae`, `1a60310`
- **Single-crate -> workspace+DLC**: `b1fbe67` (major architecture pivot)
- **Feature growth -> bug sweep campaigns**: `8ffea1d`, `c66bbfa`, `0af8bb0`
- **Build/polish -> second-zero audit fixes**: `3d22f53`, `dd4fcec`, `4750de9`

## Rework Summary
- Early development prioritized breadth (many domains activated quickly).
- Middle period introduced heavy integration complexity (shared contracts, input abstraction, event wiring, workspace migration).
- Late period concentrated on rework and hardening, especially in `ui`, `npcs`, `world`, and `player`, with repeated audit-triggered fixes.
