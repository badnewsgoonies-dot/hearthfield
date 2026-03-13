# Orchestration Topology

Purpose: define the control-plane structure for large multi-wave builds without forcing every agent to hold the full project in context.

## Core Primitive

One high-context orchestrator holds:

- product direction
- trust order
- current global state
- integration truth
- stop conditions

Everything else is pushed downward into bounded lanes.

This is not "one chat does everything."
It is:

- one chat for global coherence
- foremen for bounded surfaces
- workers for narrow implementation slices

## Layers

### 1. Top-Level Orchestrator

Responsibilities:

- own the global plan
- own `STATE`
- own the active lane ledger
- choose the next tranche
- decide what counts as true
- review integrated outputs, not every worker log

Reads first:

1. `AGENTS.md`
2. `.memory/STATE.md`
3. `status/foreman/dispatch-state.yaml`
4. `git status --short`
5. `git diff --name-only`
6. `git log --oneline -15 -- <touched paths>`

Must not:

- become a routine implementation worker
- read every worker log by default
- hold lane-local state only in thread memory
- defer integration until the end of a 100-wave campaign

### 2. Foreman

A foreman owns one bounded surface, not an abstract wave number range.

Examples:

- town housing + residential routing
- tutorial / first-week guidance
- map parity + overlay correctness
- economy loop hardening

Responsibilities:

- run 3-10 internal waves at most
- dispatch scouts/workers within that surface
- write lane state to disk
- return only high-signal results upward

Reads first:

1. `AGENTS.md`
2. `.memory/STATE.md`
3. `status/foreman/dispatch-state.yaml`
4. active lane artifact or worker report
5. `git status --short`
6. `git diff --name-only`
7. `git log --oneline -15 -- <owned paths>`

Must not:

- stay in audit-only mode
- create orchestration infrastructure
- take ownership of more than one bounded surface
- leave branch/worktree state only in thread memory

### 3. Worker

A worker owns one narrow slice.

Responsibilities:

- implement one bounded change
- validate locally
- write a short report

Reads first:

1. objective file
2. owned-path rules
3. required contract imports
4. local validation command

Must not:

- hold global product direction
- infer cross-lane merge policy
- redefine shared types locally
- read long theory docs unless the task explicitly needs them

Post-run rule:

- every bounded worker result must be clamped before review or acceptance
- validation must be rerun on the clamped result
- unclamped bounded-worker output is not an acceptable review surface

## State Model

The topology only works if disk state is stronger than chat memory.

Minimum files:

- `AGENTS.md`
- `.memory/STATE.md`
- `status/foreman/dispatch-state.yaml`
- `status/integration.md`
- `objectives/*.md`

### `dispatch-state.yaml`

This is the missing lane ledger.

It should contain:

- active lanes
- lane owner
- lane goal
- owned paths
- worktree / branch
- current phase
- next action
- last output
- validation state
- merge target
- resume command

## Cadence

Use rolling integration, not end-loaded integration.

Correct:

```text
Top-level plans tranche
-> foreman executes bounded waves
-> integration pass
-> top-level updates global state
-> next tranche
```

Incorrect:

```text
Top-level plans 100 waves
-> everyone runs independently
-> integrate only at the end
```

Why:

- stale assumptions compound
- false green accumulates
- local truth diverges from global truth
- the top-level eventually has to rediscover the whole project

## Read-Depth Policy

Top-level:

- read state, ledgers, integration outputs, and only high-signal reports

Foreman:

- read one bounded lane plus the immediate repo state around it

Worker:

- read only the task packet and owned files

This is an information-budgeting system.
Global information should exist once.
Local information should stay local.

## Stop Conditions

Top-level stop conditions:

- integration truth is stale
- lane ledger is stale
- critical-path uncertainty is still `[Inferred]`
- no integrated diff after a tranche

Foreman stop conditions:

- ghost progress
- audit drift
- no tracked diff after first implementation pass
- blocked validation with no recorded confound
- unclamped result being treated as review-ready

Worker stop conditions:

- owned scope breaks
- contract drift
- no local validation path

## Resume Policy

Resume is continuation, not identical-state replay.

Use:

- `resume` to continue evolving work
- `fork` to preserve a baseline and branch from it
- fresh session + disk state when exact reconstruction matters more than conversational continuity

## Bottom Line

The topology is:

- one high-context chat for global coherence
- many low-context lanes for execution
- disk state as the bridge between them

The system fails when:

- too much state stays only in chat
- foremen become mini top-levels
- workers are given global context they do not need
- integration is postponed too long
