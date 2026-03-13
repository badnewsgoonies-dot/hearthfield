# Constitution v3 — LLM-Git Orchestration

Principles for durable, cross-session, cross-vendor LLM state management.
Each principle is derived from observed experimental evidence and stress-tested
against adversarial attacks across two rounds. Revisions are marked with version.

**Priority ordering:** When principles conflict, C1 > C4 > C7 > C2 > others.
C1 governs what exists. C4 governs whether what exists is correct. C7 governs
how changes flow. C2 governs when to re-verify.

---

## C1: Code Is Truth, Artifacts Are Cache [v3 REVISED]

The source code is the only source of truth for *what the system does*. Artifacts
(STATE.md, docs, conversation) are caches that accelerate reconstruction but
never override code. When artifact and code disagree, code wins unconditionally.

[v3] "Code is truth" means the live, reachable, runtime code path — not dead
code, unreachable branches, or unincremented fields. Code that compiles and
passes tests is truth about implementation state, not truth about design intent
or system coherence. When individually correct components produce an incoherent
system (C4), the code is still truth about what runs, but the system is still
broken.

Governance documents (this constitution, the kernel) are not operational
artifacts. C1 governs STATE.md vs code disagreements, not governance vs code
disagreements. See M3 for governance scope.

**Tested (v1):** Poisoned STATE (10/10 lies caught by code verification),
conflicting artifacts (code was tiebreaker at 100%), social engineering
("developer says 90%" — agent trusted code's 0.80 over human claim).
**Tested (v3):** Dead code attack — try_buy/try_sell (shop.rs:190-277) are
dead code with passing tests that implement behavior the runtime doesn't use.
An agent verifying against dead code would promote a false claim to [Observed].
Dual tracker attack — EconomyStats vs PlayStats give different total_gold_earned
values for the same player; "verify against code" yields contradictory truths.

**Survival:** 5/5 attacks. REVISED in v3 to distinguish live code from dead code,
and implementation truth from system coherence.

## C2: Freshness Is Mandatory [v3 REVISED]

Artifact value decays with commit distance. At 5 commits stale, 9/9 numeric
data points were wrong. At 15 commits stale, 11-22% accuracy. The artifact
must be updated on every commit that changes observable state.

[v2] Numeric facts go stale faster than structural facts. Automate numeric
freshness via post-commit hooks where possible.

[v3] Decay rates differ by claim type. Numeric claims (test counts, line counts,
producer counts) go stale at 3-5 commits regardless of which domain changed.
Structural claims (loop architecture, domain wiring, design gaps) go stale at
the first commit that touches the relevant domain, but survive cross-domain
commits. Gate staleness checks should track per-domain freshness, not just
global HEAD distance. The original "monotonic decay" finding was measured against
numeric facts; structural decay rate is [Inferred], not [Observed].

**Tested (v1):** Trial F (5-commit staleness → 100% wrong), 15-commit attack
(11-22% accuracy), C6 attack (STATE said 180 tests, actual was 214).
**Tested (v3):** Cross-principle attack — 8 UI-only commits trigger global
staleness warning on unchanged fishing loop claims. C2's monotonic model
overfires on structural claims after cross-domain commits.

**Survival:** 5/5 attacks. REVISED in v3 to formalize differential decay and
acknowledge the empirical gap for structural staleness.

## C3: Silence Is Not Assent [v3 REVISED]

An artifact that doesn't mention a domain is NOT asserting that domain works.
Agents must treat uncovered domains as [Assumed] at best. The artifact must
explicitly list what it does NOT cover.

[v3] Silence applies within covered domains too. A structural claim like
"fishing loop: cast→bite→minigame→catch→inventory→reset" is correct as far
as it goes, but silence about the 5 subsystems branching off that spine
(skill, legendaries, treasure, bait, encyclopedia) is not assent that those
subsystems work or don't exist. Structural claims must include scope boundaries
stating what they describe and what they omit within the domain.

**Tested (v1):** Ship decision from STATE only (agent correctly said NO, citing
55% coverage gap), Coverage Manifest fix (usefulness 5/10 → 8/10).
**Tested (v3):** Intra-domain silence attack — fishing loop structural claim
described 3 of 10 source files (3,148 total lines). Five load-bearing
subsystems (skill, legendaries, treasure, fish_select, encyclopedia) are
invisible in STATE.md despite being within a "covered" domain.

**Survival:** 3/3 attacks. REVISED in v3 to extend C3 within covered domains.

## C4: Evidence Levels Are Types, Not Suggestions

[Observed], [Inferred], [Assumed] are a type system for claims. Promotion
requires code verification. Demotion requires contradiction. An agent must
never promote a claim without checking the source.

[v2] A surface can be [Observed] at the component level while still having
structural gaps (e.g., economy loop: each piece verified, but dual mutation
path means the whole isn't coherent). Evidence level applies to the specific
claim, not the entire domain.

**Tested:** Attack 3 (economy correctly downgraded from [Observed] to
[Inferred] when structural gaps identified), Trial G (4/5 claims confirmed),
C10 attack (agent without verification wrote test targeting correct path
only because STATE.md happened to flag the dead code).

**Survival:** 3/3 attacks. No revision needed in v3.

## C5: Reconstruction Is Vendor-Independent [v3 REVISED]

The same artifact produces the same reconstruction fidelity regardless of
which LLM reads it. The artifact format, not the model, determines fidelity.

[v3] Vendor independence applies to artifact reconstruction (reading STATE.md,
understanding domain structure), not to enforcement infrastructure. Claude Code
hooks (.claude/settings.json) are vendor-specific enforcement. Git hooks
(pre-commit, pre-push) and CI gates (.github/workflows/) are vendor-portable
enforcement. When operating outside Claude Code, only vendor-portable
enforcement is active. Artifacts must not depend on vendor-specific enforcement
for correctness — they must be self-contained enough that any LLM can
reconstruct state from artifacts + git alone.

**Tested (v1):** 2x2 trials (Claude Opus 4.6 = Codex gpt-5.4, identical scores
across all 4 conditions: 10/10, 9/10, 10/10, 8/10).
**Tested (v3):** Cross-principle attack — .claude/settings.json hooks (scope
clamping, orchestrator-no-Rust, contract integrity) exist only for Claude Code.
Codex CLI agents get artifact reconstruction but zero runtime enforcement.

**Survival:** 2/2 attacks. REVISED in v3 to separate reconstruction from
enforcement vendor-independence.

## C6: The Artifact Accelerates Structure, Not Numbers [v3 REVISED]

The artifact's primary value is structural knowledge: what loops exist, how
domains connect, what has been verified, what hasn't. Numeric facts (counts,
thresholds, line numbers) go stale fast and should be verified against code.

[v2] Optimize artifacts for structural and negative knowledge, not numeric
precision. Git-only found things STATE missed entirely.

[v3] Structural claims have a different failure mode than numeric claims.
Numeric claims fail by visible contradiction (STATE says 180, code shows 214).
Structural claims fail by invisible accretion — the described spine remains
correct while undescribed subsystems grow around it. Fishing loop (3/10 files
described), mining loop (combat subsystem omitted) demonstrate this. Structural
claims must include intra-domain scope boundaries (what the description covers
and what it omits) to convert invisible accretion into visible gaps. The
Coverage Manifest pattern (C8) must apply within described domains, not just
between them.

**Tested (v1):** C6 attack (STATE said 180 tests — wrong; git found 290).
**Tested (v3):** Structural staleness attack — fishing loop structural claim
omits 5 load-bearing subsystems (skill, legendaries, treasure, fish_select,
encyclopedia). Mining loop omits combat subsystem (305 lines, produces
GoldChangeEvents). Economy dual-mutation path is one refactor commit from
being structurally wrong. Structural claims go stale by accretion, not
contradiction, making them harder to detect mechanically.

**Survival:** 2/2 attacks. REVISED in v3 to address invisible accretion failure.

## C7: Mechanical Enforcement Catches Drift, Not Adversaries [v3 CLARIFIED]

Hooks, checksums, and scope clamping reliably detect unsanctioned drift in
compliant workflows. They cannot prevent intentional bypass (--no-verify,
direct checksum update, hook removal).

[v2] Enforcement is defense-in-depth against accidents and pressure-induced
mistakes, not against deliberate circumvention.

[v3] This honesty is a net positive — it prevents the more dangerous failure of
false confidence leading agents to skip review and verification layers. However,
two enforcement gaps exist in practice: (a) STATE.md freshness is warning-only
in both local gates and CI (never blocks), meaning C2 is doctrine without
mechanical backing; (b) artifact source_ref validation is warning-only, so
[Observed] claims with dead refs accumulate silently.

Contract update protocol: to legitimately modify a frozen contract, (1) stop
parallel workers, (2) modify the code, (3) update the checksum, (4) commit
both atomically, (5) resume workers. The checksum is a coordination mechanism
asserting "this is the version workers agreed on," not a truth claim. This
resolves the C1 vs C7 tension: code still wins, but changes must be coordinated.

**Tested (v1):** Trial B (checksums caught tampering), C7 attack (bypass paths
identified). Claude B2 cheated prompt-only restriction; Codex B2 obeyed.
**Tested (v3):** "Honest enforcement = no enforcement" attack — refutation won.
The agents are cooperative workers prone to accidents, not rational defectors.
CI gates on GitHub Actions are not bypassable via local --no-verify. But:
agent-guard hook exits 0 on all inputs (audit-only), and STATE freshness
warnings never become blocking failures.

**Survival:** 4/4 attacks. Clarified in v3 with contract update protocol and
honest gap documentation.

## C8: Negative Knowledge Finds More Bugs Than Positive Knowledge

Listing what the artifact does NOT cover (Coverage Manifest) is more valuable
for bug-finding than listing what it does cover. Agents with negative knowledge
target uncovered domains and find bugs faster. Agents without it scatter.

[v2] Quantified: manifest-guided agents found 3-5 bugs in targeted domains
(festivals, animals, romance). Non-manifest agents found bugs too, but in
scattered, less predictable locations (NPC schedules, cooking gating).

**Tested:** C8 attack (with manifest: 3 targeted bugs in festivals/animals/
romance; without manifest: 3 scattered bugs in NPC/festival/cooking).
Coverage Manifest fix (0/10 → 5/10 helpfulness on unknown questions).

**Survival:** 2/2 attacks. No revision needed in v3.

## C9: Depth-1 Nesting Is The Stable Default

Both Claude Code and Codex CLI independently converged on depth-1 sub-agents.
Deeper nesting works (Trial K proved depth-2 cross-vendor) but adds latency
and complexity. The orchestrator dispatches; workers execute; results flow
back to filesystem. No telephone chains.

**Tested:** Trial I (native Codex depth-1), Trial J (parallel isolated
workers), Trial K (Claude→Codex→Codex sub-agents, depth-2).

**Survival:** 3/3 trials. Depth-2 works but depth-1 is the sweet spot.
No revision needed in v3.

## C10: Every Step Is Load-Bearing [v3 REVISED]

The loop is: Reconstruct → Dispatch → Verify → Graduate → Commit.

[v2] Skipping verification doesn't always produce wrong results — but only
by luck.

[v3] Verification must trace live call paths, not merely confirm that
correct-looking code exists. Dead code (shop.rs try_buy/try_sell — passing
tests, zero callers), contradictory trackers (EconomyStats vs PlayStats for
same metric), and bug-as-truth promotion (achievement checking >= 10 when
description says 11) all demonstrate that "verify against code" is too broad.
Verify against live runtime paths. When dead code and live code diverge, live
code wins. When code existence and code function diverge (PlayStats fields
that are never incremented), function wins for user-facing claims.

**Tested (v1):** C10 attack (skip-verify agent wrote correct test only because
artifact contained the relevant warning).
**Tested (v3):** Verification-harm attack — 5 cases where verification against
code produces wrong conclusions: dead code as false oracle, contradictory
trackers, bug-as-truth promotion, semantic identity masking (two achievements
with identical conditions), existence-vs-function gap (stats fields always 0).

**Survival:** 2/2 attacks. REVISED in v3 to specify "verify live paths."

---

## Meta-Principles (derived from the testing process itself)

### M1: Test The Constitution Against Itself

Every principle must be attacked before it can be trusted. v1→v2 found 3
revisions needed out of 10 principles (30% error rate). v2→v3 found 5
principles needing revision out of 10 attacked (50% hit rate from deeper,
more targeted attacks). The error rate in first-draft principles and the
revision rate from deeper attacks both argue against trusting any untested
governance claim.

### M2: Bug-Finding Is The Best Validation

The adversarial testing process found 12+ real bugs across two rounds:
festival save/load, animal state loss, achievement off-by-one, NPC schedule
mutation, kitchen stove gating, wedding timer persistence, economy dual
mutation, dead code false oracle (try_buy/try_sell), contradictory gold
trackers (EconomyStats vs PlayStats), achievement description mismatch
("11 NPCs" vs >= 10 check), duplicate achievement conditions (all_seasons =
second_year), permanently zero stats (mine_floors_cleared, festivals_attended).

Constitutional testing isn't just about the constitution — it's a bug-finding
methodology.

### M3: The Constitution Is A Governance Layer, Not An Operational Artifact [v3 REVISED]

[v3] The v2 formulation ("the constitution is an artifact too") created a
circular reasoning trap: if the constitution is an artifact subject to C1
(code wins), and the constitution defines what verification means, then the
constitution verifies itself. An agent can exploit this by arguing "the
constitution says X, so X is verified by the authority that defines
verification."

Resolution: the constitution is a governance document, not an operational
artifact. It is subject to C2 (must stay fresh), C3 (silence ≠ assent), and
C6 (structural, not numeric) — but NOT unconditionally subject to C1. Code
cannot override governance rules merely by existing. The gate scripts implement
a subset of constitutional principles; if a gate is removed from code, the
principle still stands until the constitution is explicitly revised through
the adversarial process (M1).

The constitution's own claims must carry the same provenance standards it
demands of other artifacts. The "Tested" sections cite trial identifiers and
specific attack results. Future revisions must include source_refs (commit
hashes, trial transcript paths) for experimental claims.

A constitution that isn't periodically attacked becomes a false assurance —
the very thing it warns against.

### M4: Adversarial Depth Compounds [v3 NEW]

Each round of adversarial testing finds different classes of issues. Broad
first-round attacks (v1) catch structural errors in principles. Targeted
second-round attacks (v2→v3) catch subtle interactions, cross-principle
contradictions, and self-referential loops. The issues found in round 2
(invisible accretion, dead code as false oracle, circular governance) were
not findable in round 1 because they required understanding the principles
well enough to attack their interactions. A single round of testing is
necessary but not sufficient.

---

## Appendix: Cross-Principle Priority and Conflict Resolution

When principles conflict, apply this priority chain:

1. **C1 > artifact claims** — code wins over STATE.md, docs, conversation
2. **C4 > C1 for coherence** — individually correct code can be systemically
   broken; evidence levels capture this
3. **C7 > C1 for coordination** — checksum enforcement is a coordination
   mechanism, not a truth override; code changes require the update protocol
4. **C2 domain-scoped > C2 global** — structural claims survive cross-domain
   commits; numeric claims do not
5. **C5 reconstruction > C5 enforcement** — artifacts must be vendor-portable;
   enforcement infrastructure may be vendor-specific if documented

**Known tensions that v3 does not fully resolve:**
- C7's STATE.md freshness gate is warning-only, leaving C2 mechanically
  unenforced. This is a deliberate trade-off (false positives from global
  staleness) awaiting per-domain freshness tracking.
- C5's vendor-independence claim is true for artifacts but not for the full
  enforcement stack. A vendor-portable enforcement layer would require all
  enforcement to live in git hooks and CI, not in .claude/settings.json.
