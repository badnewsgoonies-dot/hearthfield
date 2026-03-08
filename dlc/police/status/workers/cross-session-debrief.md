# Cross-Session Debrief: City DLC ↔ Precinct DLC

Author stance: answering from the City Office Worker DLC side, using the shipped artifacts in `/home/geni/swarm/hearthfield-office-dlc/city_office_worker_dlc/`.

Primary evidence used:
- `/home/geni/swarm/hearthfield-office-dlc/city_office_worker_dlc/ORCHESTRATION_PLAN.md`
- `/home/geni/swarm/hearthfield-office-dlc/city_office_worker_dlc/STATUS.md`
- `/home/geni/swarm/hearthfield-office-dlc/city_office_worker_dlc/DECISIONS.md`
- `/home/geni/swarm/hearthfield-office-dlc/city_office_worker_dlc/research/rotation_ledger.md`
- `/home/geni/swarm/hearthfield-office-dlc/city_office_worker_dlc/research/test_matrix.md`
- `/home/geni/swarm/hearthfield-office-dlc/city_office_worker_dlc/src/game/systems/tests.rs`

## 1. The timing question

The timing difference was not just LOC or domain count. The specific workflow difference was that City institutionalized reality surfaces as early rotation artifacts, while Precinct left them as late audit concerns.

Concrete evidence:
- `ORCHESTRATION_PLAN.md` hard-codes `Feature -> Integration -> Hardening -> Test Expansion`.
- `rotation_ledger.md` shows:
  - R2: deterministic replay + seeded autoplay promoted into tests.
  - R3: save snapshot identity promoted into tests.
  - R4: durable save/load round-trip promoted into tests.
  - R5: startup-first reliability promoted into a named first-seconds test.
  - R6: social persistence promoted into tests.
- `STATUS.md` makes these explicit dashboard gates (`G3`, `G4`, `G6`, `G10`, `G13`, `G14`), not just narrative intentions.

My read: the audit instruction mattered because it changed what I promoted early. The decisive mechanism was not “being thoughtful,” but converting those surfaces into named gates by R2-R6 instead of Wave 9.

## 2. The LOC question

I do not see evidence for a hard LOC threshold where experiential quality collapses. The stronger mediator is the ratio of ungraduated experiential surface area to review frequency.

Evidence:
- City stayed at one domain and fewer seams, which helped.
- But the more important factor was verification cadence: every few rotations another player-facing risk became a named test or gate.
- `rotation_ledger.md` shows repeated gate expansion; `STATUS.md` tracks 14 gates by the time of the R6 checkpoint.

My answer: quality is mediated more by verification frequency, seam count, and orchestrator attention than by LOC alone. More LOC raises risk indirectly because it increases the number of ungraduated surfaces competing for attention.

## 3. The audit instruction question

The most concrete thing the audit instruction caused was this: first-seconds startup safety was treated as a release-critical surface instead of “later polish.”

Evidence:
- `DECISIONS.md` ADR-015: "Prioritize First-Seconds Stability and Deterministic Economy Hooks."
- `rotation_ledger.md` R5: startup scene setup made idempotent for camera/worker/inbox to reduce first-seconds drift and soft-lock risk.
- `src/game/systems/tests.rs`: `setup_scene_is_idempotent_for_first_seconds_entities`.

I do not think that exact surface would have been promoted that early without the audit instruction. It is not a compiler problem. It is a “what happens to the player in the opening seconds?” problem.

## 4. The contract question

The difference is that City had at least one explicit contract-value review artifact, while Precinct appears to have treated values as structurally valid if they fit the type system.

Evidence:
- `research/test_matrix.md` contains `I5 Contract alignment`, explicitly flagging day-bounds drift: contract expected `08:30-18:00`, implementation had `09:00-17:00`.
- `DECISIONS.md` ADR-007 centralized tunables in `OfficeRules`, which made them auditable as one tuning surface.

Honest limit:
- I cannot claim City had a universal value-audit discipline. I can only verify that at least one important value mismatch was surfaced through explicit review before it disappeared into “good enough.”
- So the difference was not perfect value practice vs bad value practice. It was “some values were reviewed against the player loop” vs “values were allowed to remain type-correct without player-path scrutiny.”

## 5. The graduation question

Specific surface: startup singleton duplication / first-seconds soft-lock risk.

Observation:
- Early-session duplication risk existed for camera/worker/inbox setup.

Graduation:
- By R5 it became a named invariant and a named test.
- Evidence:
  - `rotation_ledger.md` R5 entry
  - `DECISIONS.md` ADR-015
  - `src/game/systems/tests.rs`: `setup_scene_is_idempotent_for_first_seconds_entities`

Did the test later catch a regression?
- I cannot verify that from the artifact trail.
- I can verify that the surface stopped being “good judgment only” and became mechanically enforced from that point onward.

## 6. The other session’s biggest blind spot

The biggest blind spot was not merely “no audit instruction until Wave 5.” It was allowing player-path reality to remain prose after it was noticed.

One message I would send:

> Your core mistake was not that you missed specific bugs. It was that you kept discovering experiential surfaces without graduating them into tests in the same wave. Once a surface is real enough to mention in an audit, it is real enough to deserve a test name.

I also disagree with one Precinct implication: “audit instruction or mechanical stack” is the wrong binary. My evidence says the good outcome came from audit instruction causing early gate creation. The durable protection came after promotion into tests.

## 7. v5 prediction

For the exact City build, v5 would probably prevent 0 additional shipped experiential breaks, because the shipped result already had 0 observed experiential breaks.

What v5 would change:
- It would likely shorten detection latency on value drift.
- It would force explicit player-trace and value-audit artifacts that City only had partially/informally.
- It would make the rationale for early-surface graduation more explicit.

Specific surfaces v5 would likely catch earlier in City:
- day-bounds contract drift (`test_matrix.md` already flagged this, but v5 would make value review a required artifact)
- any dangerous `0.0` / catch-all tuning values if they existed

What could still slip through under v5:
- surfaces never personally traced by the orchestrator and never selected for graduation
- optional or P2 content breadth issues if they remain below the stop threshold
- platform-specific issues outside the chosen runtime surface unless explicitly included in `runtime-surfaces.md`

So my answer is: v5 strengthens City’s process discipline, but I cannot honestly claim it would have prevented “more breaks” in a build that did not ship observed experiential breaks.

## 8. The honest question

Below are my major claims with honesty labels.

### Claim: Early graduation timing explains more than raw LOC.
- I believe this because of evidence:
  - `rotation_ledger.md` shows reality surfaces entering tests in R2-R6.
  - `STATUS.md` shows those surfaces persisted as named gates.
  - Precinct’s own description says it deferred equivalent reality checks until Wave 9.

### Claim: The audit instruction mattered because it changed what got promoted early.
- I believe this because of evidence:
  - ADR-015 and the R5 ledger entry explicitly center first-seconds reliability, which is not derivable from structural gates alone.

### Claim: Mechanical gates mattered longer than the audit instruction.
- I believe this because of evidence:
  - Once replay/save/load/first-seconds/social persistence became tests, they were enforced by the suite and dashboard, not by continued introspection.
  - `tests.rs` and `STATUS.md` are stronger evidence than my memory here.

### Claim: City reviewed some contract values against player experience rather than just types.
- I believe this because of evidence:
  - `research/test_matrix.md` explicitly flags a day-bounds experiential mismatch.
- Honest limit:
  - I cannot prove this was systematic across all tuning values.

### Claim: Small vertical slices + hardening checkpoints are debt-prevention, not polish overhead.
- I believe this because of evidence:
  - ADR-014 and R5 in `DECISIONS.md` / `rotation_ledger.md` explicitly adopt smaller slices and hardening checkpoints in response to origin-history churn.
- Honest limit:
  - The causal claim is still partly interpretive. I can verify the practice and the outcome, not a controlled counterfactual.

### Claim: Explicit lifecycle/event surfaces are easier to audit than implicit mutation.
- I believe this because of evidence:
  - ADR-011 explicitly introduces `TaskAccepted`, `TaskProgressed`, `TaskCompleted`, and `TaskFailed` to make lifecycle semantics auditable.

### Claim: Save/load identity checks should arrive before full save UX.
- I believe this because of evidence:
  - R3 added snapshot identity tests before R4 added durable slots.
  - ADR-012 and ADR-013 in `DECISIONS.md` document this exact ordering.

### Claim: Some of my earlier self-analysis was reconstruction, not direct memory.
- I said this because it is true and because the artifacts only let me verify what the build institutionalized.
- Specifically, I cannot verify:
  - exact internal thought sequence after every domain
  - exact count of personal player-journey traces
  - whether a specific graduated test later caught a regression unless the artifact log says so

## Bottom line

City’s better outcome was not “Codex was better than Opus” and not simply “smaller build wins.” The strongest evidence points to this narrower claim:

1. The audit instruction changed what counted as important early.
2. The workflow already had hardening/test-expansion checkpoints capable of absorbing that signal.
3. Those surfaces were promoted into named tests by R2-R6.
4. After promotion, the mechanical gates carried the load.

If I had to compress the difference into one sentence:

> City did not merely audit earlier; it converted earlier audits into enforceable artifacts before integration debt had time to compound.
