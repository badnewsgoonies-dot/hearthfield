# Orchestration Primitive Synthesis

Date: 2026-03-12

Purpose: capture the current operational idea behind the orchestration system in a compact form, without mixing it with the longer research papers or repo-specific implementation details.

## Core Idea

The winning shape is probably not:

- one giant agent doing everything
- or a large swarm where every agent carries the whole project
- or a heavy programmatic orchestrator full of custom machinery

The winning shape is more minimal:

- one high-context control-plane chat
- strong disk state
- bounded foremen
- narrow workers
- continuous integration
- mechanical scope enforcement

In this model, global coherence exists once.
Local execution is pushed downward.

## Correct Role Split

### Main chat / top-level orchestrator

Owns:

- product direction
- trust order
- current global state
- tranche selection
- integration truth
- what counts as verified

Does not:

- read every worker log by default
- hold lane-local state only in conversation
- collapse into routine implementation

### Foreman

Owns:

- one bounded surface
- 3–10 internal waves at most
- local dispatch
- local lane state
- local reports upward

Does not:

- become a second top-level orchestrator
- drift into endless audit mode
- own multiple unrelated surfaces

### Worker

Owns:

- one narrow implementation slice
- local validation
- short report

Does not:

- carry the global plan
- infer merge policy
- redesign outside scope

## Continuous Integration, Not Final Integration

The important correction is:

- integration must happen continuously

Correct:

```text
plan tranche
-> foreman executes bounded local waves
-> checkpoint integrate
-> update global state
-> next tranche
```

Wrong:

```text
plan 100 waves
-> everyone runs in isolation
-> integrate at the very end
```

The “100 waves” idea only works if those waves are a ledger, not an execution isolation plan.

## What The System Must Freeze

Not the whole conversation.
Not every lane.

Freeze the operating primitive:

- boot bundle
- trust order
- lane ledger schema
- foreman prompt shape
- post-run clamp/validate/review flow
- integration cadence
- stop conditions
- model split by role

That is the reusable unit.

## Why This Is Distinct

The core idea is simple:

- one context chat is enough to preserve the core plan

But only if:

- disk state carries lane-local truth
- foremen are bounded
- workers stay narrow
- integration truth is updated continuously

Without strong disk state, the top-level chat still decays.
With strong disk state, it becomes a control plane instead of a memory sink.

## Current Practical Reading

The current target should not be “perfect replica.”

The right target is:

- no loss in gameplay/mechanical quality
- no loss in visual quality
- no missing runtime surfaces
- no asset omission
- no wrong visual-role substitution
- no new bugs
- structural and code-quality improvements are allowed if they do not lower the floor

That is stricter than “inspired by” and more realistic than “bitwise replica.”

## Current Search Goal

The immediate task is to trial and tinker with orchestration shapes until one operating form wins disproportionately well.

That means varying things like:

- boot bundle size
- foreman strictness
- lane size
- when to clamp
- integration cadence
- what gets serialized

Once one form clearly wins, freeze it.

## Bottom Line

The primitive is:

- one high-context control plane
- bounded foremen
- narrow workers
- disk-mediated local truth
- continuous integration
- mechanical scope enforcement

That is probably the smallest version of the idea that still preserves quality.
