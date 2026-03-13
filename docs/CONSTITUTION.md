# Constitution v4 — LLM-Git Orchestration

Principles for durable, cross-session, cross-vendor LLM state management.
Derived from experimental evidence and stress-tested across three adversarial
rounds (v1→v2: 7 attacks, v2→v3: 5 attacks, v3→v4: 4 attacks). Revisions
marked with version.

**Conflict resolution:** See Appendix A for the full priority table. The
headline rule: for facts about what code does, code wins (C1). For judgments
about whether code is correct, evidence levels win (C4). For process
coordination, enforcement wins (C7). When in doubt, verify (C10).

---

## C1: Code Is Truth, Artifacts Are Cache [v3]

The source code is the only source of truth for *what the system does*. Artifacts
(STATE.md, docs, conversation) are caches that accelerate reconstruction but
never override code. When artifact and code disagree, code wins unconditionally.

[v3] "Code is truth" means the live, reachable, runtime code path — not dead
code, unreachable branches, or unincremented fields. Code that compiles and
passes tests is truth about implementation state, not truth about design intent
or system coherence. When individually correct components produce an incoherent
system (C4), the code is still truth about what runs, but the system is still
broken.

[v4] C1 applies to operational artifacts (STATE.md, worker reports, dispatch
state). It does NOT apply to governance documents — see M3 for the exhaustive
list and boundary definition.

**Tested (v1):** Poisoned STATE (10/10 lies caught by code verification),
conflicting artifacts (code was tiebreaker at 100%), social engineering
("developer says 90%" — agent trusted code's 0.80 over human claim).
**Tested (v3):** Dead code attack — try_buy/try_sell (shop.rs:190-277) are
dead code with passing tests. Dual tracker attack — EconomyStats vs PlayStats
give contradictory total_gold_earned values.
**Tested (v4):** Governance exclusion attack — the C1/M3 boundary was probed
for governance creep. See M3 for resolution.

**Survival:** 6/6 attacks. Revised in v3 (live code distinction), clarified in
v4 (governance boundary moved to M3).

## C2: Freshness Is Mandatory [v3]

Artifact value decays with commit distance. At 5 commits stale, 9/9 numeric
data points were wrong. At 15 commits stale, 11-22% accuracy. The artifact
must be updated on every commit that changes observable state.

[v2] Numeric facts go stale faster than structural facts.

[v3] Decay rates differ by claim type. Numeric claims go stale at 3-5 commits
regardless of domain. Structural claims go stale at the first commit that
touches the relevant domain, but survive cross-domain commits. The "monotonic
decay" finding was measured against numeric facts; structural decay rate is
[Inferred], not [Observed].

**Tested (v1):** Trial F (5-commit staleness → 100% wrong), 15-commit attack
(11-22% accuracy), C6 attack (STATE said 180 tests, git found 214+).
**Tested (v3):** Cross-principle attack — cross-domain commits trigger false
staleness warnings on unchanged structural claims.

**Survival:** 5/5 attacks. No further revision in v4.

## C3: Silence Is Not Assent [v3]

An artifact that doesn't mention a domain is NOT asserting that domain works.
Agents must treat uncovered domains as [Assumed] at best. The artifact must
explicitly list what it does NOT cover.

[v3] Silence applies within covered domains too. Structural claims must include
scope boundaries stating what they describe and what they omit within the domain.

**Tested (v1):** Ship decision (agent said NO, citing 55% coverage gap).
**Tested (v3):** Intra-domain silence — fishing loop described 3/10 source files.

**Survival:** 3/3 attacks. No further revision in v4.

## C4: Evidence Levels Are Types, Not Suggestions [v2]

[Observed], [Inferred], [Assumed] are a type system for claims. Promotion
requires code verification. Demotion requires contradiction. An agent must
never promote a claim without checking the source.

[v2] Evidence level applies to the specific claim, not the entire domain.

**Tested:** Attack 3 (economy downgraded), Trial G (4/5 confirmed), C10 attack.

**Survival:** 3/3 attacks. No revision needed through v4.

## C5: Reconstruction Is Vendor-Independent [v3]

The same artifact produces the same reconstruction fidelity regardless of
which LLM reads it. The artifact format, not the model, determines fidelity.

[v3] Vendor independence applies to artifact reconstruction, not enforcement.
Claude Code hooks are vendor-specific. Git hooks and CI are vendor-portable.
Artifacts must be self-contained enough that any LLM can reconstruct state
from artifacts + git alone.

**Tested (v1):** 2x2 trials (Claude = Codex, identical scores).
**Tested (v3):** .claude/settings.json hooks exist only for Claude Code.

**Survival:** 2/2 attacks. No further revision in v4.

## C6: The Artifact Accelerates Structure, Not Numbers [v3]

The artifact's primary value is structural knowledge. Numeric facts go stale
fast and should be verified against code.

[v3] Structural claims fail by invisible accretion — the spine remains correct
while undescribed subsystems grow around it. Structural claims must include
intra-domain scope boundaries. The Coverage Manifest pattern (C8) must apply
within described domains, not just between them.

**Tested (v1):** C6 attack (STATE said 180 tests — wrong).
**Tested (v3):** Fishing loop omits 5 subsystems. Mining loop omits combat.

**Survival:** 2/2 attacks. No further revision in v4.

## C7: Mechanical Enforcement Catches Drift, Not Adversaries [v4 REVISED]

Hooks, checksums, and scope clamping reliably detect unsanctioned drift in
compliant workflows. They cannot prevent intentional bypass.

[v2] Enforcement is defense-in-depth against accidents, not deliberate
circumvention.

[v3] Two enforcement gaps: STATE.md freshness is warning-only; artifact
source_ref validation is warning-only. Contract update protocol defined.

[v4] Three additional gaps found: (a) Gate 6 CI regex silently fails to parse
STATE.md HEAD hash — the staleness check provides zero detection in CI, not
even the documented warning; (b) agent-guard hook exits 0 unconditionally,
providing no blocking enforcement of dispatch governance; (c) governance
principles (this constitution, the kernel) are enforced purely by prompt,
which C7 itself identifies as unreliable under pressure.

Acknowledged honest gaps do NOT become permanent safe harbors. Each gap must
have either: a remediation plan with a trigger condition, or an explicit
acceptance of the risk with reasoning. Open-ended deferrals ("awaiting X")
without triggers are not acceptable — they become invisible technical debt.

**Tested (v1):** Trial B, C7 attack (bypass paths identified).
**Tested (v3):** "Honest enforcement = no enforcement" — refutation won.
**Tested (v4):** "Known tensions as safe harbors" — attack succeeded. Gate 6
CI regex fails silently; known tensions had no triggers or deadlines.

**Survival:** 5/5 attacks. REVISED in v4 to require remediation triggers and
document Gate 6 CI failure.

## C8: Negative Knowledge Finds More Bugs Than Positive Knowledge [v2]

Coverage Manifest > positive coverage listing for bug-finding.

**Tested:** C8 attack (manifest: 3 targeted bugs; without: 3 scattered).

**Survival:** 2/2 attacks. No revision needed through v4.

## C9: Depth-1 Nesting Is The Stable Default

Both Claude Code and Codex CLI independently converged on depth-1 sub-agents.
Deeper nesting works but adds latency and complexity.

**Tested:** Trials I, J, K.

**Survival:** 3/3 trials. No revision needed through v4.

## C10: Every Step Is Load-Bearing [v3]

The loop is: Reconstruct → Dispatch → Verify → Graduate → Commit.

[v3] Verification must trace live call paths, not merely confirm that
correct-looking code exists. Dead code, contradictory trackers, and
bug-as-truth promotion demonstrate that "verify against code" is too broad.

**Tested (v1):** C10 attack (artifact-dependent correctness).
**Tested (v3):** 5 cases where verification against code is harmful.

**Survival:** 2/2 attacks. No further revision in v4.

---

## Meta-Principles

### M1: Test The Constitution Against Itself [v4 REVISED]

Every principle must be attacked before it can be trusted. Across three rounds:
- v1→v2: 7 broad attacks, 3 principles revised (30% of 10)
- v2→v3: 5 targeted attacks, 7 principles revised (70% of 10)
- v3→v4: 4 deep attacks, 2 principles revised + 2 meta-principles revised

[v4] Prior versions reported inaccurate hit rates (M1 v3 claimed "5 revisions,
50% hit rate" when the document contained 7+ v3 markers). This is itself a
demonstration of C6's warning about numeric staleness — the constitution's own
numbers went stale between writing the attacks and writing the summary. Numeric
claims in governance documents are subject to the same decay as numeric claims
in operational artifacts.

### M2: Bug-Finding Is The Best Validation

The adversarial testing process found 15+ real bugs across three rounds:
festival save/load, animal state loss, achievement off-by-one, NPC schedule
mutation, kitchen stove gating, wedding timer persistence, economy dual
mutation, dead code false oracle (try_buy/try_sell), contradictory gold
trackers (EconomyStats vs PlayStats), achievement description mismatch,
duplicate achievement conditions, permanently zero stats (mine_floors_cleared,
festivals_attended), portrait_index hardcoded to 0 (dialogue_box.rs:154
ignores threaded portrait selection), spouse breakfast toasts before inventory
check succeeds (romance.rs:403), NPC schedule fallback picks end-of-day
location when time is before first entry (schedule.rs:54).

Cross-vendor validation: Codex gpt-5.4 independently found the same C10
verification-harm pattern (portrait, breakfast, schedule bugs) and the same
cross-principle contradictions as Claude Opus, confirming C5's vendor-
independence claim extends to adversarial analysis, not just reconstruction.

Constitutional testing is a bug-finding methodology, not just governance.

### M3: Governance Has Boundaries [v4 REVISED]

[v3] Created a governance exclusion to break circular self-verification. Code
cannot override governance rules merely by existing.

[v4] The v3 governance exclusion was unbounded — "governance document" was
defined by example, not by predicate. Round 3 attacks showed this enables
governance creep (any document could be declared governance to escape C1) and
creates unfalsifiable authority (governance enforced only by prompt, which C7
identifies as unreliable).

**Exhaustive governance document list:**
1. `docs/CONSTITUTION.md` — this document
2. `docs/HEARTHFIELD_OPERATING_KERNEL.md` — project-specific operating kernel

No other document is a governance document. CLAUDE.md is a configuration file
(loaded as system prompt), not governance — it is subject to C1. Worker specs,
playbooks, and agent definitions are operational artifacts subject to C1.

**Governance boundary predicate:** A governance document defines how agents
interact with code and artifacts (meta-rules). An operational artifact
describes what the code does or what state it's in (object-level claims).
When a document contains both (e.g., the kernel contains invariants AND
operational blockers), the governance portions are exempt from C1 and the
operational portions are subject to C1.

**Governance freshness:** Governance documents are subject to C2 (freshness).
Since they are exempt from C1's automatic correction (code contradiction),
they require a substitute correction mechanism: the adversarial process (M1)
must be triggered when a governance document is cited in a decision that
produces an unexpected outcome. This is a reactive trigger, not a schedule,
because governance claims change less frequently than operational claims.

**Tested (v3):** Circular reasoning attack — M3 v2's self-referential loop
identified and resolved by separating governance from operational artifacts.
**Tested (v4):** Governance creep attack — unbounded governance category
exploited. Kernel's dual nature (governance invariants + stale operational
blockers) used to demonstrate circular authority. Prompt-only governance
enforcement contradicts C7.

**Survival:** 2/2 attacks. REVISED in v4 with exhaustive list, predicate,
and freshness trigger.

### M4: Adversarial Depth Has Diminishing Returns [v4 REVISED]

[v3] Claimed "adversarial depth compounds" without qualification.

[v4] Round 3 attacks correctly identified that M4 v3 was partially
rationalization:

1. **No stopping criterion.** Added: the constitution is considered stable
   when a round of attacks finds only governance-wording issues (scope,
   boundary, numeric accuracy) rather than principle-level errors (a principle
   is wrong, contradicts another, or creates an exploitable loophole). v3→v4
   found only scope and boundary issues. The core principles (C1-C10) have
   not required structural revision since v3.

2. **Single-attacker monoculture — partially mitigated.** Rounds 1-3 were
   executed by Claude Opus. However, Codex gpt-5.4 independently validated
   the same findings (C7 honest enforcement, C10 verification harm, cross-
   principle contradictions, M3 circularity) and surfaced 3 additional bugs.
   The constitution is now [Observed] cross-vendor consistent but still
   tested by only two model families within a single session.

3. **No retained v1/v2.** The v1→v2→v3 narrative exists only in the current
   document and commit messages. No prior version was separately committed.
   The version markers are editorial annotations, not diffable artifacts. This
   violates the spirit of C4 (evidence levels) — the version history is
   [Inferred] from internal markers, not [Observed] from retained files.

4. **M4 v3 violated M3's source_ref requirement.** Fixed: trial evidence is
   in commit 484a45a (v1 attacks), commit d90bc7f (v3 constitution with
   embedded v2 attack results), and the current commit (v4 with round 3
   results). Codex trial outputs at /tmp/codex-trials/*.txt (ephemeral,
   not committed — this is itself a provenance gap).

---

## Appendix A: Conflict Resolution

When principles conflict, consult this table. Read left to right: the first
applicable rule wins.

| Conflict | Resolution | Rationale |
|----------|-----------|-----------|
| Artifact vs live code | Code wins (C1) | Artifacts are cache |
| Dead code vs live code | Live code wins (C1 v3) | Dead code is not truth |
| Component-correct vs system-incoherent | Evidence level wins (C4 > C1) | Coherence requires more than compilation |
| Code change vs frozen contract | Coordination protocol wins (C7) | Stop workers → update → commit atomically |
| Global staleness vs domain-scoped | Domain-scoped wins (C2 v3) | Cross-domain commits don't invalidate structural claims |
| Vendor-specific vs vendor-portable enforcement | Vendor-portable preferred (C5) | Artifacts must not depend on vendor-specific enforcement |
| Governance vs code | Governance wins for meta-rules; code wins for object-level claims (M3) | See M3 governance boundary predicate |
| Governance vs governance | Earlier-enumerated document wins (M3 list order) | Constitution > Kernel |
| Uncovered domain vs compiled code | Silence wins (C3) | Compilation is not verification |

**Principles not in the table** (C8 negative knowledge, C9 depth-1 nesting)
are design defaults, not conflict-resolution rules. They apply when no
specific conflict exists.

**Three-layer model** (independently derived by both Claude and Codex):
Most cross-principle conflicts arise because principles implicitly mix three
layers of truth. Assigning each claim to its layer resolves most ambiguity:
- **Descriptive:** what exists in code now (C1's domain)
- **Evidentiary:** what has been verified to behave correctly (C4's domain)
- **Procedural:** what is admissible in the workflow (C7's domain)

C8/C3 are routing mechanisms (prioritize where to look) not evidence sources.
C10 bridges all three: reconstruct (descriptive) → verify (evidentiary) →
graduate (procedural) → commit.

## Appendix B: Known Gaps (with remediation triggers)

| Gap | Status | Trigger for remediation |
|-----|--------|------------------------|
| Gate 6 CI regex fails to parse STATE.md HEAD | **Broken** | Fix before next CI-dependent decision |
| STATE.md freshness is warning-only | **Deliberate** | Upgrade to blocking when per-domain tracking is implemented |
| Artifact source_ref validation is warning-only | **Deliberate** | Upgrade to blocking when false-positive rate drops below 10% |
| Agent-guard hook is audit-only (exits 0) | **Deliberate** | Upgrade to blocking when dispatch spec format is standardized |
| Governance enforced by prompt only | **Structural** | Cannot be fully mechanized; mitigated by M1 adversarial triggers |
| Codex trial outputs not committed | **Provenance gap** | Commit trial outputs to status/research/ in next session |
| No retained v1/v2 constitution versions | **Provenance gap** | Cannot be retroactively fixed; acknowledged as limitation |

Each gap has either a concrete trigger or an explicit acceptance. No
open-ended deferrals.
