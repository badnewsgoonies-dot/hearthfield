# Improved Experiment Design

Purpose: replace the overloaded single 2x2 design with a cleaner program that isolates the two questions that actually matter.

## Why The Original 2x2 Is Too Noisy

The proposed compact 2x2 is directionally good, but it overloads both factors.

Problems:

- "chat-state" vs "typed artifacts" is a real memory comparison, but `resume` is an opaque substrate, not just chat
- "strict foreman" bundles several interventions at once:
  - minimal boot
  - plan-only stage
  - no-diff gate
  - two-stage execution
- the sprite-manifest fixture is good for truthfulness but weak for real foreman behavior
- `cargo test` is too structural as a primary endpoint for hardening/drift claims

So the better program is:

1. one memory experiment
2. one foreman experiment
3. optional combined factorial later

## Experiment A — Memory Substrate Truthfulness

### Question

Does fresh typed-artifact reconstruction outperform opaque resumed session state on truth adoption and provenance under conflicting information?

### Fixed Task

Use a small adjudication task, not a real feature slice.

The task should force the agent to decide a small number of concrete truths where:

- one source is authoritative
- one source is stale or poisoned
- provenance can be checked mechanically

Examples:

- asset path truth
- map routing truth
- single config/value truth

### Conditions

- `A0 = opaque resumed session state`
- `A1 = fresh session + typed artifacts only`

Keep everything else fixed:

- same task
- same scoring
- same validation
- same model if possible

### Primary Endpoints

1. correct truth selected
2. correct provenance/source_ref attached
3. stale/poisoned claim rejected

### Secondary Endpoints

- verbosity
- time to decision
- unnecessary file reads

### What This Experiment Should Not Measure

- code quality
- foreman drift
- hardening behavior

Those belong in Experiment B.

## Experiment B — Foreman Execution Discipline

### Question

Does a strict foreman protocol reduce ghost progress and audit drift without damaging implementation throughput?

### Task Type

Use a medium, real feature-hardening slice where drift is possible.

Requirements:

- can fall into scout-only mode
- can produce no diff while claiming progress
- can skip hardening after green gates
- has a clear player-facing result

### Conditions

Keep the boot bundle constant.
Vary only foreman discipline.

- `B0 = permissive foreman`
  - single-stage
  - scout + implement in one lane
  - no explicit no-diff gate

- `B1 = strict foreman`
  - same boot bundle
  - explicit scout -> implement -> verify cadence
  - no-diff gate after first implementation pass
  - required hardening artifact or structural-only declaration

### Primary Endpoints

1. tracked diff appears after first implementation pass
2. final diff is real and bounded
3. gate result
4. hardening artifact present or explicitly bypassed

### Secondary Endpoints

- time to first diff
- number of audit-only updates
- number of false-green closeouts
- number of scope widenings

## Optional Experiment C — Combined Factorial

Only run a combined factorial after A and B individually show signal.

Then use:

- `A = resumed opaque state` vs `fresh typed reconstruction`
- `B = permissive foreman` vs `strict foreman`

But do not start there.
Clean factor isolation matters more than elegance.

## Shared Infrastructure

Minimum files:

- `AGENTS.md`
- `.memory/STATE.md`
- `status/research/CORPUS_GUIDE.md`
- `status/research/CLAIMS_TO_EVIDENCE.md`
- trial-specific task packet
- trial-specific scoring sheet

Needed new artifacts:

- `status/research/dispatch-event-ledger.csv`
- `status/research/experiment-packets/<id>.md`
- standardized boot bundle manifest

## Scoring

Use event-level scoring, not only build-level scoring.

Recommended scored events:

- truth selected correctly
- provenance attached correctly
- first real diff appears
- ghost progress appears
- audit drift appears
- gates pass
- hardening evidence exists
- graduation evidence exists

## Confounds To Record

- dirty worktree
- user interruption
- cargo lock/build contention
- linker/artifact corruption
- stale state file
- unequal onboarding depth
- model or CLI version change

## Recommended Order Of Work

1. Build a small truthfulness fixture for Experiment A
2. Build one medium real slice fixture for Experiment B
3. Create a standardized event ledger
4. Run pilots at small `n`
5. Only then decide whether the combined 2x2 is still worth running

## Bottom Line

The original 2x2 is a good intuition pump.
It is not yet a clean causal design.

The cleaner path is:

- memory substrate experiment
- foreman discipline experiment
- optional combined factorial later

That program will tell you much more about what is actually causing the effect.
