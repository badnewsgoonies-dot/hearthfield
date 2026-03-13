# Build Methodology — Read and Internalize Before Working

Before you write any code, dispatch any workers, or plan any waves, internalize
these principles. They are not suggestions.

## First-Response Protocol

Before acting, state:

1. current tier: `S` / `M` / `C`
2. current runtime surface — what user-facing thing are you touching?
3. current macro phase: `scaffold spine` / `finish spine` / `scaffold breadth` /
   `finish breadth`
4. current wave phase: `Feature` / `Gate` / `Document` / `Harden` /
   `Graduate`
5. current `P0` / `P1` debt — what graduation tests are missing?
6. any `[Inferred]` / `[Assumed]` claims on the critical path

If tier is ambiguous, start at `S` and escalate if you touch shared vocabulary,
save/load identity, or multiple interacting runtime surfaces.

## Pre-Touch Retrieval

Before touching any domain:

1. run `git log --oneline -15 -- <path>`
2. read the latest worker report if one exists
3. check `.memory/` for active artifacts tagged to that domain
4. state:
   - what changed recently
   - what remains unresolved
   - what is still `[Inferred]` / `[Assumed]`

## Doctrine

Carry these in every decision:

- Green means ready to examine, not ready to ship.
- Workers build the scaffold. The orchestrator finishes the product.
- Judgment fires once per surface, then becomes a test.
- Freeze shapes, not values.
- Do not preserve the conversation as memory; preserve its outputs as typed,
  source-linked artifacts.

## Wave Cadence

Every wave:

`Feature -> Gate -> Document -> Harden -> Graduate`

- `Feature`: build it
- `Gate`: compile, test, lint, checksum, clamp
- `Document`: emit artifacts for what was learned
- `Harden`: inspect the actual runtime surface
- `Graduate`: turn `[Observed]` truths into named tests

Do not start the next wave until `Document`, `Harden`, and `Graduate` are
complete.

## Artifact Format

Write to `.memory/` as individual YAML files:

```yaml
id: DEC-2026-03-09-014
type: decision | observation | debt | principle
evidence: Observed | Inferred | Assumed
domain: farming | save | world | player | ...
summary: "One sentence. Never nested."
source_refs:
  - src/save/mod.rs:566
  - commit fe0b9d3
why_it_matters: "One sentence."
alternatives_considered:
  - option: "..."
    rejected_because: "..."
why_reverted: null
drift_cue: "Condition that means this artifact is wrong."
supersedes: []
status: active | resolved | superseded
```

Filename convention:

- `{type}-{domain}-{short-slug}.yaml`

If nothing triggers, write nothing. Most waves should produce `0–3` artifacts.

## Evidence Levels

Tag every claim about user-facing behavior:

- `[Observed]` — exact path traced and verified
- `[Inferred]` — strongly believed, not directly traced
- `[Assumed]` — design expectation, unverified

Only `[Observed]` claims graduate into tests.

## Macro Sequence

For anything larger than a hotfix:

1. scaffold spine
2. finish spine
3. scaffold breadth
4. finish breadth

Do not enter breadth until the spine is structurally green, experientially
verified, and protected by graduation tests.

## Feel Check

Every `Harden` pass on a user-facing surface must check:

- clarity
- feedback
- responsiveness
- pacing
- edge behavior

If reachable but feels wrong, it is not finished.

## Asset / Resource Quality

Classify runtime-used assets as:

- style-consistent
- placeholder

A surface with placeholder assets adjacent to production assets fails the quality
gate.

## Worker Rules

Every worker spec must include:

- “Do not create orchestration infrastructure. Implement only domain
  deliverables.”
- “Do not redefine shared types locally. Import from the contract.”

After every worker:

- clamp scope mechanically
- verify contract integrity
- run all gates

## Integration

Start integration in a fresh session. Integration should read only:

- contract
- specs
- worker reports
- traces
- current errors

## Stop Conditions

Stop and reassess if you observe any of these:

1. contract drift
2. clamp breaks the fix
3. false green
4. beautiful dead product
5. ghost progress
6. search termination on green gates
7. graduation failure
8. critical-path uncertainty
9. blame-thrash loop
10. identity collapse
11. abstraction reflex
12. self-model error

Trust mechanical indicators, not self-reports.
