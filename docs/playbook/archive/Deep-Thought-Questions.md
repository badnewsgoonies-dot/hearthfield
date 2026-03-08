# Deep Thought Questions — Session Analysis

These questions probe what this session actually proved, what it didn't, and what it means.

---

## 1. Did the playbook produce a game or a codebase?

The miracle audit found 12/18 steps WORKING and 6 BROKEN. The 6 breaks include dispatch never spawning (wrong constant), salary toast missing (one line), and WASM persistence (architectural gap). The code compiles, tests pass, events are wired — but the player journey has holes.

**The question:** Is "15,815 LOC, 120 tests, 0 event gaps" a game? Or is it the same "beautiful dead game" failure the playbook's stop condition #11 was designed to prevent? The orchestrator wrote that stop condition, articulated why it matters, then produced output that partially triggers it. What does this tell us about the gap between understanding a failure mode and preventing it?

---

## 2. The audit instruction experiment got contaminated.

The original experimental design: City Office Worker DLC had "audit your work from the player's perspective" → tight vertical slice. Pilot DLC didn't → broad shallow scaffolding. The conclusion: the audit instruction is the key variable.

But Precinct used the audit instruction starting from Wave 5 (after you injected it), not from Wave 1. Waves 1-4 produced the 7 event gaps. Waves 5-9 closed them. This looks like the audit instruction working — but it's also 4 more waves of implementation, which would close gaps anyway.

**The question:** Can we actually attribute the event gap closure to the audit instruction, or would any additional waves have closed them regardless? Is there a way to isolate this variable in future builds?

---

## 3. The research paper says solo execution outperforms multi-agent below ~10 domains.

Precinct has 12 domains. The bare-prompt ablation in the paper found that at 30K LOC, solo frontier execution beat coordinated builds. Precinct is 15.8K LOC — below the crossover point.

**The question:** Would a single Claude Opus 4.6 session, given the spec and type contract but no worker dispatch, have produced a better Precinct in less time and fewer tokens? The paper's own data suggests yes. Did the multi-agent architecture add value here, or did it add coordination overhead to a project that was below the complexity threshold?

---

## 4. Context priming worked perfectly. What does that mean?

The calendar worker hit every frozen constant (TIME_SCALE=2.0, 8hr shifts, weather 60/20/10/10, rank thresholds). The paper predicts this: 0% formula transfer without context, 100% with it. The spec was on disk. The worker read it. Every number transferred.

But this is the *easy* part. Numbers transfer when present. The hard part — the part the audit found broken — is emergent behavior: "does dispatch actually fire on this map?" "does this interaction path produce evidence the player can process?" These aren't numbers in a spec. They're consequences of how systems interact.

**The question:** Can contrastive-causal specs prevent emergent integration failures, or only prevent specification-level failures? Is there a category of bug that no spec can prevent because it only exists at the intersection of multiple correctly-implemented domains?

---

## 5. The 9.8x worker throughput gap is the elephant in the room.

The paper found a 9.8x gap between best and worst worker models at the same scale and architecture. This session used GPT-5.4 as orchestrator and both GPT-5.4 and Claude Opus 4.6 as workers.

**The question:** What would the output quality difference be if ALL workers were Opus 4.6 vs all GPT-5.4? The session didn't control for this. The orchestrator chose which model to use for which task ad hoc. Is model selection per-task an important orchestration variable that the playbook doesn't address?

---

## 6. Mechanical enforcement worked. Prompt enforcement didn't. But the reality gates weren't mechanical.

Scope clamping: 20/20 mechanical, 0/20 prompt. This held in Precinct — zero contract violations. But the reality gates (Phase 5A) — the gates that would have caught the 6 miracle path breaks — were run manually by the orchestrator as a final audit, not mechanically after each wave. The orchestrator ran compile/test/clippy/contract gates mechanically. It ran reality gates once, at the end.

**The question:** Can reality gates be made mechanical? What would a CI script look like that verifies "dispatch spawns on exterior maps" or "evidence terminal leads to processing"? Are these inherently human-judgment checks, or can they be encoded as headless integration tests?

---

## 7. The statefulness premium prediction held — but did we waste it?

The paper says ~95% of orchestrator cost is re-reading conversation history. The Opus orchestrator session used 288K/1M tokens (29%). It had 71% free space. It ran 9 waves without compaction.

**The question:** Did the orchestrator use its context efficiently? The Precinct session accumulated orchestration state, worker dispatch prompts, gate results, and audit reports across 9 waves. But it also accumulated the entire research paper, the Hearthfield audit, the sprite mapping docs, and the context transfer package. How much of that 288K was decision-relevant vs. carried baggage?

---

## 8. The Hearthfield base game sprite overhaul was done by Claude (this session), not by orchestrated workers.

I built 25 sprite replacements directly using Python/PIL — analyzing source sheets, extracting frames, building atlases, verifying dimensions. No workers were dispatched. No scope clamping. No gates. Just direct execution.

**The question:** Was this the right call? The playbook says "the orchestrator is not the implementer." But the sprite work was image processing with exact dimensional constraints — closer to a deterministic pipeline than creative implementation. Does the playbook's delegation rule apply to all work, or only to work where the decision field is wide enough that contrastive-causal specs add value?

---

## 9. What's the actual per-game cost?

Rough accounting for this session:
- Phase 0-2 (spec, contract, bootstrap): this Claude session (~1M context window, several hours)
- Waves 1-9: ~20 Codex GPT-5.4 invocations + Opus orchestrator at 288K tokens
- Sprite work: this Claude session (direct execution)
- Premium requests: ~60-100 across all copilot dispatches

**The question:** What's the total dollar cost of producing Precinct? Is it cheaper than a solo developer working for a week? Is the time-to-output (one evening session) the real value proposition, not the cost?

---

## 10. Is this reproducible?

This session had: a researcher who's done 11 prior builds, a battle-tested playbook refined across 98 agent sessions, a pre-existing codebase to reference, exact knowledge of Bevy's edge cases, and the full research paper in context.

**The question:** Could someone who hasn't done the prior 11 builds reproduce this result using only the v4 playbook? Or is the playbook necessary-but-not-sufficient, with the operator's accumulated pattern library (the human intuition we researched at the start of this session) being the actual differentiator? If so, what does that mean for the playbook as a shareable artifact?
