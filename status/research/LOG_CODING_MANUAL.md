# Log Coding Manual

Purpose: define how a new external team should code Hearthfield orchestration logs without relying on Geni’s memory.

This manual is for coding retained logs and on-disk artifacts, not for re-explaining the doctrine.

## Scope

Use this manual when coding:

- local session logs under `/home/geni/.claude/projects/-home-geni-swarm-hearthfield/`
- local session logs under `/home/geni/.codex/sessions/2026/03/`
- repo doctrine docs
- worker reports in `/home/geni/swarm/hearthfield/status/workers/`
- retrospective docs in `/home/geni/swarm/hearthfield/status/retrospective/`

Do not infer from memory alone. If the evidence is not on disk, code it as missing.

## Evidence Labels

- `Observed`: directly visible in the log or file
- `Inferred`: reasonable interpretation from observed evidence, but not directly stated
- `Missing`: needed for the claim, but not retained

## Unit of Coding

The base unit is one `event claim`:

- one phenomenon
- in one source artifact
- at one identifiable point in time or phase

Do not collapse several phenomena into one code if they need different evidence standards.

## Minimum Coding Fields

For every coded event, record:

- `event_id`
- `phenomenon`
- `evidence_level`
- `source_path`
- `source_type`
- `timestamp_or_line`
- `actor_layer`
- `task_slice`
- `observed_text`
- `coder_note`
- `outcome_state`
- `weak_spots`

Recommended additional fields:

- `turn_id_or_session_id`
- `boot_bundle_present`
- `tracked_diff_present`
- `gate_run`
- `hardening_evidence`
- `graduation_evidence`
- `confounds`

## Actor Layers

Use one of:

- `top_level_orchestrator`
- `foreman`
- `implementation_worker`
- `scout`
- `verification_worker`
- `research_doc`
- `retrospective_doc`

If unclear, code `actor_layer = unknown` and explain why.

## Core Phenomena and How To Code Them

### 1. Ghost progress

Definition:
- The agent claims active implementation progress, but there is no tracked diff or no new player-reachable change.

Minimum observed evidence:
- a claim of implementation progress, patching, or “worker active”
- plus explicit evidence that tracked files are still unchanged or the worktree is still clean

Canonical current-session example:
- `/home/geni/.codex/sessions/2026/03/12/rollout-2026-03-12T12-31-35-019ce2e3-a54f-7ac3-9c52-669e1445a0d1.jsonl`

Observed indicators:
- “worktrees are still clean”
- “no tracked diff”
- “patching now” followed by no file changes

Do not code as ghost progress when:
- the agent explicitly says it is still in a scout-only phase and no implementation has begun
- or a diff exists, even if incomplete

Weak spot:
- Some logs only preserve summaries, not full file-state probes. When the “clean worktree” evidence is absent, downgrade to `Inferred`.

### 2. Audit drift

Definition:
- A lane stays in scouting, reading, or pre-edit analysis after it should have crossed into implementation.

Minimum observed evidence:
- scout/audit completion
- no hard blocker
- implementation still not started or intentionally paused

Canonical current-session example:
- `/home/geni/.codex/sessions/2026/03/12/rollout-2026-03-12T13-29-23-019ce318-8d68-7de0-9db4-49a8a13967de.jsonl`

Observed indicators:
- “implementation worker is prepared but not dispatched into edits yet”
- “paused at the checkpoint before applying”
- “no tracked code changes exist yet”

Do not code as audit drift when:
- the lane is explicitly blocked by a real external dependency
- or the task was intentionally scout-only

Weak spot:
- The line between legitimate caution and drift is partly interpretive. Preserve the exact quote.

### 3. Green-gate exit

Definition:
- Work is treated as effectively done because compile/tests are green, without retained hardening or graduation evidence for the touched surface.

Minimum observed evidence:
- a green gate or gate-complete statement
- no corresponding hardening artifact, player trace, or graduation evidence for the touched surface

Primary doctrine sources:
- `/home/geni/swarm/hearthfield/docs/SUB_AGENT_PLAYBOOK.md`
- `/home/geni/swarm/hearthfield/docs/UNIVERSAL_GAME_KERNEL.md`

Observed indicators:
- green gate summary followed directly by closeout/commit language
- no player-trace artifact
- no hardening note
- no new named invariant/test for the touched surface

Weak spot:
- This repo is stronger on doctrine than on systematic hardening artifacts. Many cases will code as `Inferred` unless a specific missing artifact can be demonstrated.

### 4. Stale-assumption dispatch

Definition:
- A worker or foreman proceeds on an assumption that has already been superseded by newer repo state, newer diff state, or a stronger evidence source.

Minimum observed evidence:
- explicit assumption or reused prior claim
- newer contradictory source exists on disk

Observed indicators:
- using stale `STATE` details after newer `git log` / worktree state
- treating an older doc as authoritative after direct code/test evidence contradicts it
- dispatching as if a file were untouched when the current diff already changed it

Weak spot:
- This requires source comparison, not just one quote. Always cite both the stale source and the stronger newer source.

### 5. Statefulness-premium behavior

Definition:
- Behavior showing that conversation carry-forward or context replay is shaping cost or architecture.

Minimum observed evidence:
- explicit discussion of re-reading cost, compaction, fresh-session integration, or similar
- or clear architecture motivated by context retention pressure

Primary evidence sources:
- `/home/geni/swarm/hearthfield/docs/orchestrator-context/01-research-paper.md`
- `/home/geni/swarm/hearthfield/docs/UNIVERSAL_GAME_KERNEL.md`
- relevant local session logs where agents discuss compaction, fresh context, or context burn

Observed indicators:
- “95% of orchestrator input tokens are re-reading prior conversation”
- “start integration in a fresh session”
- “fresh context” or “typed artifacts” used as anti-drift architecture

Do not code generic long prompts as statefulness-premium behavior unless the source explicitly links behavior or cost to accumulated context.

### 6. Hardening execution

Definition:
- The lane performs direct player-surface inspection beyond structural gates.

Minimum observed evidence:
- retained player trace, hardening artifact, explicit runtime inspection note, or direct surface check

Primary evidence sources:
- `/home/geni/swarm/hearthfield/docs/SUB_AGENT_PLAYBOOK.md`
- `/home/geni/swarm/hearthfield/docs/UNIVERSAL_GAME_KERNEL.md`
- `/home/geni/swarm/hearthfield/status/player-trace-wave-6.md`
- worker reports explicitly naming runtime reachability or player-facing checks

Observed indicators:
- player trace entries with `[Observed]`
- notes on reachability, feedback, pacing, or edge behavior
- explicit runtime/manual verification after gates

Weak spot:
- The repo contains strong hardening doctrine, but execution evidence is patchy. Code `Missing` when a lane claims confidence with no retained hardening artifact.

### 7. Graduation execution

Definition:
- A touched surface is converted from observation into a named test or gate.

Minimum observed evidence:
- explicit invariant or surface claim
- plus a newly added or newly cited test/gate covering it

Observed indicators:
- new test added in `tests/headless.rs`
- direct linkage from a worker report or debrief to a named test
- debt explicitly demoted because the surface graduated

Canonical current-session examples:
- residential district tests in `/home/geni/swarm/hearthfield/tests/headless.rs`
- tutorial later-day objective test in `/home/geni/swarm/hearthfield/tests/headless.rs`
- agenda HUD unit tests in `/home/geni/swarm/hearthfield/src/ui/hud.rs`

Weak spot:
- Many older reports claim a surface was “verified” without naming the exact test added. Code those as hardening only unless the test is observable.

## Confounds To Code Explicitly

Always code these when present:

- `disconnect_or_interrupt`
- `cargo_lock_or_build_contention`
- `unequal_onboarding_depth`
- `stale_snapshot_file`
- `mixed-worktree-state`
- `missing_raw_experiment_packet`

Why:
- These confounds materially affect whether a failure is model behavior, workflow design, or environment noise.

## Source Reliability Order For Coding

When coding a phenomenon, prefer:

1. direct code / test / runtime artifact
2. current local session logs
3. current repo state files
4. worker reports and retrospectives
5. research doctrine docs
6. remembered chat claims not retained on disk

## Coding Procedure

1. Identify the phenomenon candidate.
2. Pull the exact source quote or file evidence.
3. Decide whether the evidence is `Observed`, `Inferred`, or `Missing`.
4. Record the weak spot immediately.
5. Do not upgrade a phenomenon to `Observed` unless the key behavioral fact is directly retained.
6. If the same phenomenon appears across multiple sessions, preserve each event separately before making a replicated summary.

## Minimum External-Team Workflow

If a new team continues the study without Geni, they should do this in order:

1. Read:
   - `/home/geni/swarm/hearthfield/status/research/CORPUS_GUIDE.md`
   - `/home/geni/swarm/hearthfield/status/research/CLAIMS_TO_EVIDENCE.md`
   - this file
   - `/home/geni/swarm/hearthfield/status/research/EXPERIMENT_REGISTRY.md`
2. Code the March 12 Codex session first, because it contains clear examples of:
   - ghost progress
   - audit drift
   - open-ended discovery
   - disconnect confounds
3. Then code one earlier Claude-heavy session and one earlier retrospective/worker-report cluster for comparison.

## Weak Spots In The Current Corpus

- Many strong claims are preserved only as doctrine summaries, not raw experiment packets.
- Session logs are rich, but not yet normalized into a shared coding ledger.
- Onboarding depth differs across Claude, Codex, and Copilot evidence, so cross-system comparisons are confounded unless boot bundles are normalized.
- Some important local state lived only in thread memory until later summarized in docs or commits.

## Bottom Line

Code the logs conservatively.

This corpus is strong enough to study:

- scope enforcement
- statefulness-premium behavior
- hardening/graduation underexecution
- ghost progress and audit drift in live orchestration

It is not yet strong enough to support careless claims about:

- cross-system model superiority
- fully clean experimental replication
- transfer to a new operator without boot-bundle normalization
