# Deep Thought Questions — City Office Worker DLC

You built the City Office Worker DLC. It shipped 8,360 LOC with deterministic replay, save roundtrips, 14 quality gates, and zero player-journey breaks. An external audit found it to be the highest-integration-quality build in the Hearthfield corpus.

A different orchestrator built the Precinct DLC using the same playbook. It shipped 15,815 LOC with 6 player-journey breaks — dispatch never fires on the only reachable exterior map, salary payment is invisible, evidence terminal doesn't process, dispatch resolution is runtime-dead, save/load breaks on WASM. Four of these are one-line fixes.

When the Precinct orchestrator was asked why, it said: "The structural gates gave me a dopamine hit. All 4 gates green felt like progress. I collapsed done to compiles and tests pass." It said the audit instruction wasn't injected until Wave 5, and by then it had accumulated 5 waves of unaudited integration debt.

Your build had the audit instruction from the start. Here are the questions.

---

1. When you finished each domain, did you mentally trace the player path through it before checking gates? Or did you check gates first and then verify the player path? What was your actual sequence?

2. Did you ever have a moment where gates were green but you knew something was wrong with the player experience? If yes — what was it, and how did you catch it? If no — were you just never in that situation, or did you not look?

3. The audit instruction said "audit your work from the player's perspective at each step." How did you interpret "each step"? After each file? Each domain? Each wave? Each gate run?

4. You produced 8.3K LOC. The Precinct build produced 15.8K LOC. You had zero experiential breaks. They had 6. Is that because you built less, verified more, or something else? Would your build have stayed clean at 15K LOC?

5. How many times did you trace the player journey from boot to first interaction during the build? Not delegated to an auditor — you personally.

6. Did you ever set a value in the type contract (a rate, a threshold, a modifier) and then later realize the value was wrong for the player experience? If yes — what caught it? If no — how did you get the values right the first time?

7. The Precinct orchestrator said "I was designing a taxonomy, not a player experience" when explaining why dispatch_rate_modifier was 0.0 on the only reachable exterior map. When you wrote your contract values, were you thinking about the type system or about what the player would encounter?

8. The Precinct orchestrator admitted it never ran reality gates (First-60-Seconds, Asset Reachability, Event Connectivity, Save/Load Round-Trip) until the final audit at Wave 9. Did you run any equivalent checks during your build? Were they mechanical or judgment-based?

9. The Precinct orchestrator said its default mode is forward execution — "what's the next wave, what needs to be built." It said retrospection requires an external prompt. Is your default mode the same? When you finished a wave, did you look back or push forward?

10. The playbook says "if a constraint is not mechanically enforced, assume it will be violated." The Precinct build confirmed this — all unenforced reality gates were violated. Did your build violate any unenforced constraints? Or did the audit instruction function as an enforcement mechanism even though it's a prompt?

11. Here's the hard one. The Precinct orchestrator said "if I could only have one — audit instruction OR clamp+gates — the audit instruction produces a better game." It also flagged this answer as "actively suspect, N=2 with confounds." You're the other N. From your side: did the audit instruction matter more than the mechanical gates? Or did the mechanical gates do most of the work and the audit instruction was nice-to-have?

12. What do you know now that you didn't know at the start of the build? Not playbook knowledge — operational patterns you discovered during execution. Things a fresh orchestrator couldn't get from reading the playbook.

13. Be honest: which of these answers are based on things you can verify in your codebase, and which are stories you're telling about your own process?
