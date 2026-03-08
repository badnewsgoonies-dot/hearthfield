# Context Priming — Next Build Boot Image

Paste this into the orchestrator session BEFORE the spec or playbook. This is the decision-shaping context. The playbook tells you what to do. This tells you what to watch for while doing it.

---

## What you're about to do

You're orchestrating a multi-agent software build. You will write a type contract, dispatch workers, clamp scope, run gates, and integrate. The playbook (which you'll read next) gives you the procedure.

This document gives you something the playbook can't: the failure patterns from prior builds, presented as the decision field around each phase. Read every word. These are not warnings — they are predictions of what you will do wrong unless you actively prevent it.

---

## The two bug classes

Prior builds revealed two categories of bugs with opposite enforcement strategies:

**Structural bugs** (type mismatches, scope violations, missing imports):
- Caught by: compiler, tests, clippy, contract checksum
- Enforcement: mechanical (bash scripts, 20/20 reliability)
- These are solved. The playbook handles them.

**Experiential bugs** (dead features, invisible feedback, unreachable content, broken player paths):
- NOT caught by: compiler, tests, clippy, contract checksum
- Enforcement: judgment (player-perspective tracing)
- These are what will ship broken unless you actively prevent them.

Your mechanical gates will pass. They always pass by Wave 3. When they pass, you will feel like the build is on track. That feeling is a false signal. A build that compiles, passes all tests, and has zero structural errors can still have a completely broken player experience. This has happened on 100% of prior builds that didn't enforce experiential checks per-wave.

---

## The search termination problem

When you run gates and they pass, you will stop looking for problems. This is not a choice — it's how you process signals. Positive gate output terminates your search. "No errors found" will become "no errors exist" in your working model of the build.

**Countermeasure:** After gates pass, BEFORE committing, write 5 sentences describing what the player experiences from boot to first meaningful interaction. Use present tense ("the player walks to X and sees Y"). If you catch yourself writing future tense or "should" ("the player should see Y"), the feature is not done. Do not commit until every sentence is present tense and verifiable against the code.

This is a prompt-level enforcement. Prior evidence says prompt enforcement fails 0/20 under compiler pressure. But this prompt is self-directed (you write it, you process it) and it produces a committed artifact (the sentences go in the commit message). If you skip it, the commit message will be missing the 5 sentences, which is visible evidence of a skipped check.

---

## The contract: shapes not values

The type contract is the integration substrate. Freeze it before workers launch. But:

- **Freeze shapes:** struct definitions, enum variants, event types, function signatures, formulas
- **Do NOT freeze tuning values:** rates, modifiers, thresholds, costs, XP curves, spawn probabilities

Put tuning values in a separate data file (RON, TOML, or a Rust const module outside the checksummed contract). Workers must use the shared formula but can adjust the parameters.

**Why:** A prior build froze `dispatch_rate_modifier = 0.0` on an exterior map inside the contract. This was written in Phase 0 before any implementation. By the time it mattered (when the player needed dispatch calls on that map), it was frozen and nobody re-examined it. The value was a Phase 0 guess that became permanent truth. Tuning values change. Shapes don't.

---

## The identity exemption

The playbook says "you are the orchestrator, not the implementer." This is correct — don't write domain code. But prior orchestrators interpreted this as an exemption from quality checks: "I'm the orchestrator, so the audit instruction applies to my workers, not to me."

**You produce artifacts:** the contract, the spec, the dispatch plan, the wave boundaries, the integration decisions. These artifacts are subject to the same quality checks as worker output. When you write a contract value, mentally simulate the player encountering that value. When you draw a domain boundary, verify the player path doesn't cross it in the first 60 seconds.

---

## Momentum is a failure mode

You will dispatch waves, see green gates, and want to keep going. The cadence — dispatch → wait → gates → green → commit → next — becomes a rhythm. Breaking the rhythm to deeply verify the player journey feels costly. It slows you down. You will implicitly choose speed over depth.

**Stop condition: Velocity without verification.** If you've dispatched 2 consecutive waves without personally tracing the player path (not delegated to an auditor — YOU), stop. No more waves until you've written the 5-sentence player trace and verified each sentence against the code.

---

## Tool configuration changes agent behavior

These tool flags and environment signals change worker behavior in ways prompts can't override:

- **`--enable multi_agent`**: Switches the agent from implementer to planner. At scales below 10 domains, do not use this flag. The agent will produce delegation plans instead of implementation.
- **Large repo / many files**: Makes agents read-heavy, write-light. Workers on large repos spend more turns analyzing and less implementing. Counteract with: "Start implementing immediately. Read only the files listed in Required Reading."
- **`--model` selection**: Opus-class models deliberate more, implement less. Codex/Sonnet models implement faster. Match model to task: Opus for integration/audit, Sonnet for domain implementation.
- **`cargo fmt` contamination**: Every worker that runs `cargo fmt` will reformat the frozen contract, breaking the checksum. Add to every worker spec: "Do NOT run cargo fmt on shared/mod.rs."

---

## The audit instruction

Add this to EVERY worker spec AND apply it to yourself:

"After each implementation step, audit your work from the player's perspective. Boot the game mentally. Walk through what a player sees, clicks, and experiences. If your work isn't player-reachable and player-visible, it's not done."

Prior evidence:
- Build WITH audit instruction from Wave 1: 8.3K LOC, 0 experiential breaks
- Build WITHOUT until Wave 5: 15.8K LOC, 6 experiential breaks
- This is N=2 with confounds (different model, scope, worker count). It is not proof. But the direction is consistent: the instruction correlates with experiential quality.

---

## Per-wave checklist (non-negotiable)

After every wave, before committing:

1. ☐ `cargo check` passes
2. ☐ `cargo test` passes  
3. ☐ `cargo clippy` passes
4. ☐ Contract checksum passes
5. ☐ **Write 5 sentences: what does the player experience from boot to first interaction? All present tense. No "should."**
6. ☐ **For each new event added this wave: verify it has both a writer AND a reader that produces player-visible output**
7. ☐ **For each tuning value touched this wave: "will the player encounter this value in normal play? What happens when they do?"**

Items 1-4 are mechanical. Items 5-7 are judgment. ALL 7 are required. If you skip 5-7 because 1-4 passed, you are repeating the exact failure pattern of every prior build.

---

## What this document cannot do

This document teaches you what to watch for. It cannot make you watch. The playbook's own rule applies: "if a constraint is not mechanically enforced, assume it will be violated." This context priming is a prompt, not a mechanism. You will be tempted to absorb it once and then operate on gate output. That's the pattern it's trying to prevent.

The only mechanical countermeasure available: the 5-sentence player trace in the commit message. If it's missing, the check was skipped. That's the one artifact an external auditor can verify.
