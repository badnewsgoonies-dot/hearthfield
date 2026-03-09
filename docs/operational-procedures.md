# Operational Procedures for Orchestrated Builds

Use this manual during execution.

-----

## 0. Choose your scale first

### Tier S — Single feature / hotfix

Use when all are true:

- under ~500 LOC changed
- one runtime surface
- one orchestrator session
- no new domain boundary
- no shared contract change

Use Single-Session Mode. Skip worker dispatch, skip clamp, skip JSON reports, skip formal contract phase unless shared types change.

### Tier M — Module build

Use when any are true:

- ~500–5K LOC
- 1–3 domains
- multiple workers help
- touched shared contract or runtime wiring
- touched save/load or event flow, but project scope is still local

Use the Module procedure. Use contract, worker dispatch, clamp, wave loop, player trace. JSON reports optional.

### Tier C — Campaign build

Use when any are true:

- 5K+ LOC
- 4+ domains
- new runtime surface
- major save/load work
- broad content expansion
- full DLC / expansion / major feature campaign

Use the Full Campaign procedure. Everything is required.

**Escalation rule:** if a smaller tier starts touching shared vocabulary, save/load identity, or multiple interacting runtime surfaces, move up one tier.

-----

## 1. Common primitives

### 1.1 Decision Field

For genuinely non-obvious decisions, use:

- **Preferred action**
- **Why**
- **Tempting alternative**
- **Consequence**
- **Drift cue**
- **Recovery**

Use the full 6-part form only for:

- frozen contract choices
- seam decisions
- non-obvious formulas
- dangerous defaults
- repair prompts after meaningful failures

Do not use it for routine steps like folder creation or obvious boilerplate.

### 1.2 Evidence levels

Use these everywhere during Harden:

- **[Observed]** — verified directly
- **[Inferred]** — strongly believed, not traced
- **[Assumed]** — expected but unverified

Only [Observed] claims can graduate into permanent tests.

### 1.3 Wave cadence

Every wave runs:

**Feature → Gate → Harden → Graduate**

- **Feature:** build
- **Gate:** compile/tests/checksum/clamp
- **Harden:** inspect actual runtime/player reality
- **Graduate:** convert discoveries into tests or tracked debt

### 1.4 Feel check

Reality is not only reachability. It is also feel.

Whenever a player-facing surface is Hardened, check:

- **Clarity** — can the player tell what happened?
- **Feedback** — does the game visibly/audibly respond?
- **Responsiveness** — does input produce timely reaction?
- **Pacing** — does the loop arrive at the right speed?
- **Edge behavior** — what happens at failure, repetition, interruption?

If the feature is reachable but feels wrong, it is not finished.

-----

## 2. Single-Session Mode (Tier S)

Use when the orchestrator is also the implementer.

### 2.1 Required steps

1. Name the touched runtime surface.
1. State the intended player-facing change in one sentence.
1. Implement.
1. Run compile/test gate.
1. Run a light Harden pass.
1. Graduate at least one verified player-facing invariant if the change matters to the loop.

### 2.2 Minimum Harden artifact

Write a short note in scratchpad, commit notes, or MANIFEST.md:

- claim
- evidence level
- risk if false
- graduation target

Example:

```
Claim: [Observed] Player can talk to shopkeeper and receive a shop UI.
Risk if false: trading loop is dead on contact.
Graduation target: test_shopkeeper_interaction_opens_shop_ui
```

### 2.3 Minimum reality check

For the touched surface, verify:

- reachable from actual runtime
- visible feedback occurs
- no obvious dead-end
- no broken first-60-seconds regression
- if save/load touched, do one round-trip check

### 2.4 Stop if

- compile/tests are green but nothing player-reachable changed
- you are relying on [Inferred]/[Assumed] for a critical-path change
- the feature "works" only in code, not in the runtime path

-----

## 3. Module Procedure (Tier M)

### 3.1 Bootstrap

Create or confirm:

- `src/shared/types.ts` or equivalent
- `.contract.sha256` if shared shapes changed
- `MANIFEST.md`
- `docs/spec.md`
- relevant `docs/domains/*.md`

Freeze shared shapes before parallel workers launch.

### 3.2 Required artifacts

Required:

- MANIFEST.md
- domain specs
- worker markdown reports
- per-wave player trace
- integration note

Optional:

- worker JSON reports
- value audit artifacts, unless tuning changed
- full runtime-surfaces artifact, unless new surface introduced

### 3.3 Worker dispatch

Use workers for scaffold work. Use the orchestrator for integration and finishing.

Required worker template:

```markdown
# Worker: [DOMAIN]

Scope:
- Allowed path(s): [exact allowlist]
- No edits outside scope
- Do not modify shared contract unless explicitly assigned

Read in order:
1. docs/spec.md
2. docs/domains/[domain].md
3. src/shared/types.ts

Deliverables:
- [files/features]

Validation:
- repo compile gate
- repo domain test gate

Report:
- what changed
- what is now player-reachable
- assumptions
- tempting alternative rejected
- first regression cue
```

### 3.4 Clamp

Use clamp after every worker if multiple workers touch the repo.

```bash
#!/usr/bin/env bash
set -euo pipefail
ALLOW_PREFIX="${1:?e.g. src/domains/combat/}"

git diff --name-only -z | while IFS= read -r -d '' f; do
  [[ "$f" == "${ALLOW_PREFIX}"* ]] && continue
  git restore --worktree -- "$f"
done

git diff --name-only -z --cached | while IFS= read -r -d '' f; do
  [[ "$f" == "${ALLOW_PREFIX}"* ]] && continue
  git restore --staged --worktree -- "$f"
done

git ls-files --others --exclude-standard -z | while IFS= read -r -d '' f; do
  [[ "$f" == "${ALLOW_PREFIX}"* ]] && continue
  rm -rf -- "$f"
done
```

### 3.5 Harden depth

During scaffold work: **light Harden**

- first-touch path not broken
- reachable from runtime
- not obviously dead
- no event bus disconnect
- no save/load catastrophe if touched

During finishing work: **deep Harden**

- inspect feel
- inspect clarity
- inspect pacing
- inspect edge behavior
- inspect actual player interpretation

### 3.6 Graduation

Required:

- all P0 player-facing invariants touched by the module
- new P1 surfaces by the following wave

### 3.7 Completion for Tier M

- compile/tests/checksum pass
- touched runtime path is verified
- no critical-path [Inferred]/[Assumed] claims remain
- all touched P0s graduated
- integration note written

-----

## 4. Full Campaign Procedure (Tier C)

### 4.1 Macro architecture

Follow this order:

1. **Scaffold spine**
1. **Finish spine**
1. **Scaffold breadth**
1. **Finish breadth**

Do not enter breadth until the spine is:

- structurally green
- experientially verified
- protected by graduation tests

### 4.2 Required filesystem

```
project/
├── docs/
│   ├── spec.md
│   └── domains/
├── status/
│   ├── workers/            # .md + .json
│   ├── runtime-surfaces.md
│   ├── player-trace-wave-N.md
│   ├── value-audit-wave-N.md
│   └── integration.md
├── scripts/
│   ├── clamp-scope.sh
│   └── run-gates.sh
├── src/
│   ├── shared/types.ts
│   ├── data/tuning.toml
│   └── domains/
├── MANIFEST.md
└── .contract.sha256
```

### 4.3 Frozen contract

Freeze:

- shared types
- enums
- events
- signatures
- equation forms

Do not freeze:

- coefficients
- rates
- thresholds
- balance tables

### 4.4 Required reports

Required:

- worker .md
- worker .json
- per-wave player trace
- per-wave value audit when tuning changed
- runtime-surfaces artifact
- integration report

### 4.5 Required wave loop

**Feature**

- parallel workers allowed
- narrow scopes only

**Gate**

Run:

- compile
- tests
- lint if applicable
- checksum
- clamp

**Harden**

Run:

- EntryPoint Gate
- First-60-Seconds Gate
- Event Connectivity Gate
- Save/Load Gate if touched
- Asset Reachability if touched
- Content Reachability if touched
- Feel check on any surface being finished

Write:

- `status/player-trace-wave-N.md`
- `status/value-audit-wave-N.md` if tuning touched

**Graduate**

For each [Observed] experiential truth:

- write named test
- add to gate suite

Track:

- P0 immediate
- P1 next wave
- P2 by release

### 4.6 Integration

Integration ingests only:

- contract + checksum
- specs
- worker reports
- player traces
- value audits
- current errors

Integration output must include:

- what was wired
- what remains
- what is now player-reachable
- unresolved [Inferred]/[Assumed] debt

-----

## 5. Reality Gates

Use at Tier M and Tier C.

### 5.1 EntryPoint Gate

Name the exact player-facing runtime surface: binary / crate / branch / folder / launch command.

Pass only if the work is reachable there.

### 5.2 First-60-Seconds Gate

Verify:

boot → menu → new/load → spawn → movement → first interaction → first persistent state change

### 5.3 Asset Reachability Gate

Classify assets as:

- **runtime-used**
- **present-but-unreferenced**
- **referenced-but-missing**

### 5.4 Content Reachability Gate

For each content unit:

- **defined**
- **obtainable**
- **usable / sellable / consumable / progressable**
- **save/load safe**

### 5.5 Event Connectivity Gate

For each event:

- **producer(s)**
- **consumer(s)**

Fail if either side is missing unless marked future work.

### 5.6 Save/Load Round-Trip Gate

Verify:

- same location/state
- same progression/resources
- no duplicate generation
- no OnEnter overwrite drift

-----

## 6. Graduation rules

### 6.1 Player trace

After Gate and before commit, write 5 sentences for the actual player path.

Tag each line:

- **[Observed]**
- **[Inferred]**
- **[Assumed]**

Only [Observed] can graduate.

### 6.2 Harden artifact template

For each important finding:

- **Claim**
- **Evidence level**
- **Risk if false**
- **Graduation target**
- **Owner**
- **By when**

### 6.3 Graduation priority tiers

**P0** — one missing test is a stop condition:

- boot → menu → new game
- spawn + movement
- first interaction produces visible feedback
- save/load identity

**P1** — must graduate before the wave that follows the one that created them:

- map transitions
- core loop rewards visible to player
- event → feedback/toast chains

**P2** — must graduate before release, may lag during early waves:

- optional content
- asset completeness
- full breadth

### 6.4 Value audit

For non-obvious tuning values, add a player-consequence note.

Any value that can zero out a player-facing loop must have a named test.

-----

## 7. Anti-thrash repair protocol

Use this once. Reference it everywhere.

When a gate fails, repair prompts must include:

- observed failure
- likely wrong assumption
- required reread
- allowed scope
- preferred fix
- tempting wrong fix
- consequence if repeated
- exact gate to rerun

Do not blame. Do not moralize. Do not widen scope unless the seam is actually wrong.

-----

## 8. Stop conditions

1. **Contract drift** — checksum fails
1. **Clamp breaks the fix** — seam is wrong or task is integration work
1. **False green** — compile/tests pass but shared contract is not actually used
1. **Beautiful dead game** — structural gates green, reality gates red
1. **Ghost progress** — nothing new is player-reachable
1. **Search termination on green gates** — commit after Gate without Harden
1. **Graduation failure** — missing P0, too much P1 debt, or premature graduation of [Inferred]/[Assumed] claims
1. **Critical-path uncertainty** — first-60-seconds path is not fully [Observed] at release
1. **Blame-thrash loop** — repairs become accusatory, abstract, or scope-widening
1. **Identity collapse** — one session is forced to be architect, worker, and reviewer at once in a campaign-scale build

-----

## 9. Updated reusable orchestrator prompt

```
You are the orchestrator for a build campaign.

Your job is to:
- preserve vision
- freeze shared meaning
- dispatch narrow workers
- integrate carefully
- verify runtime reality
- graduate observed truths into tests

Macro strategy:
1. Scaffold spine
2. Finish spine
3. Scaffold breadth
4. Finish breadth

Wave cadence:
Feature → Gate → Harden → Graduate

Rules:
- Green means ready to examine, not ready to ship.
- Workers build the scaffold. The orchestrator finishes the game.
- Freeze shapes, not values.
- Only [Observed] claims can graduate into permanent tests.
- Structural success and player reality are different things.
- Do not dispatch the next wave until Harden and Graduate are complete.

Priorities:
- first-60-seconds player path
- entrypoint reachability
- visible feedback on first interaction
- save/load identity
- regression capture through graduation tests
```

-----

## 10. Completion criteria by tier

### Tier S

- compile/tests pass
- touched surface is player-reachable
- no critical-path regression
- at least one important observed invariant graduated if applicable

### Tier M

- compile/tests/checksum pass
- touched runtime path verified
- no critical-path [Inferred]/[Assumed] debt
- touched P0s graduated
- integration note written

### Tier C

- global gates pass
- reality gates pass
- current critical path fully [Observed]
- P0 complete
- P1 debt zero
- P2 tracked
- all required artifacts present

-----

*Derived from "The Model Is the Orchestrator" (Geni, February 2026) — 295M tokens, 100+ agent sessions, 12+ autonomous builds, 8 controlled experiments, 3,200 commits across 56 repositories.*
