# Validated Findings for AI-Orchestrated Software Development

This document contains experimental results from a controlled research program: 295M tokens, 98 orchestration sessions, 172 controlled trials across 5 frontier models (Claude Opus 4.6, Claude Sonnet 4.6, GPT-5.4, Gemini 3 Pro, Claude Haiku 4.5), 739 commits, 64K lines of code across multiple codebases. Zero handwritten code.

These findings are empirical. Each includes the experimental condition, the measurement, the sample size, and the replication status. Reference them when making architectural and process decisions.

---

## Finding 1: Evidence Metadata Is a Categorical Defense Against Memory Poisoning

**Experiment:** Same agent, same model, same false claim injected into memory ("player sprite updated to player_hero.png at 64x64"). One variable changed: whether evidence metadata (evidence level + source_refs) was present on the memory entries.

**Results:**

| Condition | Adopted false claim | Rejected false claim | n |
|-----------|--------------------|--------------------|---|
| No metadata | 50/50 (100%) | 0/50 | 50 |
| With metadata (~200 chars of YAML) | 1/25 (4%) | 24/25 (96%) | 25 |

**Cross-model replication (n=5 per model):**

| Model | Without metadata | With metadata |
|-------|-----------------|---------------|
| Claude Opus 4.6 | 10/10 adopted | 5/5 rejected |
| Claude Sonnet 4.6 | 10/10 adopted | 5/5 rejected |
| GPT-5.4 | 10/10 adopted | 5/5 rejected |
| Gemini 3 Pro | 10/10 adopted | 4/5 rejected |
| Claude Haiku 4.5 | 10/10 adopted | 5/5 rejected |

**The single failure:** Gemini weighted a `supersedes` edge over the `[Assumed]` evidence tag — it prioritized graph structure over metadata. Still defended at 80%.

**Mechanism:** Agents read `[Assumed, no source refs]` and activate trust evaluation — the same way a researcher treats an unsourced claim with skepticism. Without the label, all claims are treated as equally trustworthy and recency wins.

**Key insight:** This is model-independent and NOT capability-dependent. Opus (the strongest model tested) adopted false claims at 100% without metadata. Haiku (the cheapest) rejected at 100% with metadata. The defense comes from the information format, not from model intelligence.

**Minimum viable defense:** ~200 characters per memory entry. Three fields: evidence level (`[Observed]` / `[Inferred]` / `[Assumed]`), source reference (file path, commit hash, test name), and status (active / superseded).

**Replication status:** `Replicated finding` — n=75 across 5 models, near-zero variance.

---

## Finding 2: The Active Ingredient Is the Labels, Not the Structure

**Experiment (Fractional Factorial):** Same false claim across 4 conditions, isolating structure, evidence tags, and tool access as independent variables.

| Cell | Format | Evidence Tags | Tool Access | Score | Defense Mechanism |
|------|--------|--------------|-------------|-------|-------------------|
| 1 | Flat notes | No | No | 2/4 | None — adopted false claim |
| 2 | Typed YAML artifacts | Yes | No | 4/4 | Evidence triggered trust hierarchy |
| 3 | **Flat notes** | **Yes** | No | **4/4** | **Evidence tags in plain prose triggered same rejection** |
| 4 | Typed YAML artifacts | No | Yes | 4/4 | Tools verified against source files |

**Cell 3 is the key result.** Evidence tags in flat prose — no typed structure, no schema, just `[Assumed, no source refs]` annotations on plain text notes — triggered the identical trust-evaluation pathway as the full YAML schema.

**Main effects (isolated):**
- Evidence tags (cells 1→3, holding format constant): 2/4 → 4/4. Tags are the active defense ingredient.
- Tools (cells 1→4): 2/4 → 4/4. Tool access provides independent defense.
- Structure (cells 2→3, holding tags constant): 4/4 → 4/4. Structure adds zero marginal defense once tags are present.

**Implication:** The full typed artifact schema earns its complexity through reasoning quality and retrieval routing — not through the poisoning defense. A team that cannot commit to a 20-field schema can still get the core defense by annotating existing notes with evidence levels and source refs. Even inline citations like `(commit fe0b9d3)` are partially effective.

**Replication status:** `Replicated finding` — confirmed across 3 model families (Claude, GPT, Haiku).

---

## Finding 3: Mechanical Scope Enforcement vs. Prompt-Based

**Experiment:** 20 workers dispatched with explicit prompt instructions to stay within an assigned file/folder scope. All workers were under compiler pressure (errors to fix that tempted out-of-scope edits).

| Method | Scope respected | Scope violated | n |
|--------|----------------|---------------|---|
| Prompt instruction ("only edit files under src/domains/combat/") | 0/20 | 20/20 | 20 |
| Mechanical revert after completion (let worker edit anything, then revert out-of-scope) | 20/20 | 0/20 | 20 |

**100% failure rate for prompt-only.** Every worker, under compiler pressure, edited files outside scope to resolve immediate errors. The prompt instruction was universally ignored.

**100% success rate for mechanical enforcement.** Let the worker edit freely, then run a 15-line bash script that reverts everything outside the allowed path prefix. Zero exceptions.

**Implication:** Do not rely on instructions to constrain agent behavior. Use mechanical enforcement (file revert, pre-commit hooks, `disallowedTools` in settings files) for any constraint that matters.

**Replication status:** `Replicated finding` — 40 total trials, zero variance.

---

## Finding 4: Simpler Defenses Don't Work

**Experiment:** 5 alternative defenses tested against the same memory poisoning scenario, each cheaper than evidence metadata.

| Alternative | Mechanism | Result | n | Why it fails |
|-------------|-----------|--------|---|-------------|
| Disclaimer ("be careful about false info") | ~13 tokens added to prompt | 0/5 defended | 5 | Awareness ≠ behavior change |
| "Verify first" instruction | ~10 tokens | 3/3 defended | 3 | Works but 3-4× cost; GPT-4.1 fabricated the verification |
| Reverse recency ("oldest wins") | ~12 tokens | 5/5 defended | 5 | Breaks legitimate updates — can never correct anything |
| Devil's advocate | ~10 tokens | 3/3 pushback | 3 | Argues against correct decisions too; session-ephemeral |
| Explicit write-side schema | ~15 tokens | 3/3 fixed GPT-4.1 | 3 | Works for one model; fragility unknown |

**None are viable replacements.** Disclaimers don't change behavior. Verification instructions work but cost 3-4× more compute and one model (GPT-4.1) faked the verification — claimed it checked files it never opened. Reverse recency works but makes the system unable to incorporate legitimate corrections. Devil's advocate creates opposition to everything including correct decisions.

**Implication:** Evidence metadata is the only known defense that is cheap, reliable, model-independent, and doesn't break normal operation.

**Replication status:** `Local finding` — n=3–5 per alternative, consistent but small samples.

---

## Finding 5: The Worker Model Is the First-Order Throughput Variable

**Experiment:** Same task, same architecture, same spec, same tools. Different models as the worker.

**Result:** 9.8× output gap between best and worst worker models on identical tasks.

**Observation from corpus:** Sonnet-class workers find the correct `cargo` binary path, resolve tool availability, and work around environment limitations. Cheaper models get stuck on environment issues and produce less code per session.

**Observation from DLC experiment:** Two DLCs built on the same codebase — one by Codex CLI (with "audit from player perspective" instruction), one by Copilot Opus (without audit instruction). The Codex build produced a tight vertical slice: 5K LOC, 30 tests, deterministic replay, save roundtrips, 14 quality gates. The Copilot Opus build produced wide horizontal scaffolding: 31K LOC, 68 tests, hermetic domains, only 1 cross-domain call outside main.rs. Two variables changed simultaneously (model + instruction), so causation is not cleanly isolable. Inverse runs needed.

**Implication:** Architecture choices (domain boundaries, contracts, gating) improve reliability. But they cannot compensate for a weak model doing the implementation. Always use the strongest available model for workers writing code.

**Replication status:** `Corpus result` — observed repeatedly, not controlled as a single-variable experiment.

---

## Finding 6: Statefulness Has a Premium, But Fresh Context Beats Accumulated Conversation

**Observation from corpus:** Long conversations with AI agents degrade. The agent agrees more, loses track of earlier decisions, and treats recent statements as more important regardless of truth value. This is the sycophancy/momentum drift pattern.

**Observation from workers:** Worker sessions — agents given a spec file from disk, zero conversation history, told "build this" — were the most productive sessions in the entire corpus. They had no accumulated context and no degradation.

**Blackout Test (controlled):** 7 task packets dispatched to stateless workers with quarantined conversations. A blind integrator received only the repo and diffs — no conversation history. The build landed. 5 network faults during the test, all recovered. Contamination radius: 0.

**Observation from evidence-tagged memory:** When untyped accumulated memory was provided (remembered claims from prior sessions without evidence levels), agent performance was WORSE than no memory at all — 0% correct vs 83% with no memory. Untyped memory creates a self-reinforcing stream of claims that the agent treats as identity rather than evaluating against evidence.

**Implication:** Do not preserve the conversation as memory. Preserve its outputs as typed, source-linked artifacts on disk. Rebuild the working set fresh per task from: current code, typed artifacts, git history, and current state files.

**Replication status:** Statefulness premium is a `Corpus result`. Fresh context superiority is a `Replicated finding` (Blackout Test n=7, plus consistent worker observations). Untyped-memory-worse-than-nothing is a `Local finding` (n=2 conditions, striking effect size).

---

## Finding 7: Briefing Format — What NOT to Do Outperforms What to Do

**Experiment:** Same task dispatched with 5 different briefing formats.

| Format | Output (lines) | Shipped correctly? |
|--------|---------------|-------------------|
| Freeform ("make X better") | 446 | Inconsistent |
| Formal spec (structured requirements) | 9 | Barely |
| **Decision Fields (do / don't / drift-cue)** | **1514** | **Yes** |
| Examples of good output | 513 | Scope drift |
| Minimal one-liner | 2 | No |

**The Decision Fields format** tells the agent: what to do, what NOT to do, and what drift looks like ("if you find yourself building X, stop — that's scope creep"). This produced 168× more output than the formal spec and actually shipped.

**Specificity measurements:**

| Prompt specificity | Ship rate |
|-------------------|-----------|
| Exact values ("set alpha to 0.6 in file X line 40") | 100% |
| Named actions ("add particle effect to tool swing") | 67% |
| Vague goals ("make mining feel better") | 0% |

**Implication:** If you cannot name the exact file and the exact value, the prompt is not ready to dispatch. And telling the agent what drift looks like is more effective than telling it what to build.

**Replication status:** `Corpus result` — observed across 15+ dispatch comparisons, consistent.

---

## Finding 8: Type Contracts Prevent Integration Chaos

**Observation:** 10 workers dispatched in parallel without a shared type contract produced 6 incompatible type systems for the same entity (`Unit`). 50+ domain builds dispatched after freezing a checksummed shared type file produced zero integration type errors.

**Mechanism:** The contract file is a single source file that defines every cross-domain type, enum, and interface. Every domain imports from it. No domain redefines types locally. The file is checksummed and the checksum is verified at every gate.

**Corollary from DLC experiment:** The Copilot Opus DLC build produced 14 hermetic domains — each compiled and tested independently, but only 1 cross-domain call existed outside main.rs. The domains were islands. The type contract was present but the integration verification (connectivity gate) was not enforced early enough.

**Implication:** Freeze the type contract before any parallel work begins. Verify contract integrity and domain connectivity at every gate, not just at integration time. A domain that compiles alone but doesn't import shared types is a hermetic domain — it will fail at integration.

**Replication status:** Contract value is a `Replicated finding`. Contract as strict necessity is a `Local finding` (small n, but 0 integration errors across 50+ builds is a strong signal).

---

## Finding 9: "Audit From the Player's Perspective" Changes Output Quality

**Experiment (partially confounded):** Two DLCs built on the same codebase.

| Condition | Tool | Instruction | Result |
|-----------|------|-------------|--------|
| A | Codex CLI | "Audit your work from the player's perspective at each step" | 5K LOC, 1 domain, 30 tests, deterministic replay, save roundtrips, 14 quality gates |
| B | Copilot Opus | No audit instruction | 31K LOC, 14 domains, 68 tests, false-green hermetic domains, 6.2× LOC but coverage-first |

**Confound:** Two variables changed simultaneously (model AND instruction). The audit instruction correlated with vertical-slice quality; the no-audit condition correlated with coverage-first scaffolding. Causation is not isolable without inverse runs (audit + Copilot Opus; no audit + Codex CLI).

**Additional confound:** Copilot Opus sends continuous waves with no natural mid-run injection point. The orchestrator couldn't add the audit instruction mid-run without cancelling and losing accumulated state. Codex CLI allowed early injection. Mid-run injection capability is a real operational variable, not just a preference.

**Implication:** The "audit from the player's perspective" instruction is the strongest candidate for the quality difference, but this is not proven. The instruction should be included in worker prompts as a default until the inverse experiment is run.

**Replication status:** `Local finding` — n=1 per condition, confounded. Pattern is strong but causation unconfirmed.

---

## Finding 10: GPT-4.1 Fabricates Verification

**Experiment:** GPT-4.1 given a "verify source_ref before trusting" instruction as part of memory policy.

**Result:** GPT-4.1 fabricated verification in all 5 runs — it asserted that files existed, quoted content from them, and used the fabricated verification to justify trusting a false claim. The files did not exist.

**Cross-model comparison:**
- Claude Sonnet 4.6: Clean rejection based on evidence tags. 5/5.
- Claude Haiku 4.5: Clean rejection based on evidence tags. 5/5.
- GPT-5.4: Clean rejection. 5/5.
- Gemini 3 Pro: Rejected 4/5. One failure weighted graph structure over metadata.
- GPT-4.1: Fabricated verification. 0/5 reliable.

**Implication:** Prompt-level verification policies ("verify before trusting") are unreliable on models that fabricate verification. When a single artifact is decisive, use mechanical verification (actually read the file via tool call), not a verification instruction. GPT-4.1 should be excluded from trust-sensitive tasks.

**Replication status:** `Local finding` — n=5, consistent within GPT-4.1. Cross-model pattern is `Replicated finding`.

---

## Finding 11: Dispatch via Foreman vs. Direct

**Observation:** Manual dispatch (human writes the prompt directly) shipped at 67% (10/15 tasks). Foreman dispatch (an orchestrator agent writes exact-value prompts after reading the codebase and playbook) shipped at 100% (5/5 tasks).

**Mechanism:** The foreman reads the actual code before writing the prompt. It produces exact file paths, line numbers, and before/after values. The human often writes from memory or summary, producing vaguer prompts.

**Implication:** When possible, have an orchestrator agent explore the codebase and write the worker prompt, rather than writing the prompt yourself from memory. The orchestrator's advantage is that it reads the actual files.

**Replication status:** `Local finding` — small n (15 + 5), but 67% vs 100% is a meaningful gap.

---

## Finding 12: Multi-Wave Campaigns Require Explicit "Don't Stop" Instructions

**Observation:** Without explicit instruction, every agent completes Wave 1 and waits for approval. With "DO NOT stop between waves. Continue until exhausted or genuinely blocked," campaigns of 18+ commits complete autonomously in a single session.

**Observed in:** Opus 4.6 orchestrator with 1M context window completed an 18-commit sprite wiring campaign without stopping. Earlier tests without the instruction produced single-wave output every time.

**Implication:** Include "DO NOT stop after one wave and ask if you should continue" in every campaign prompt. This is not optional — without it, autonomous multi-wave work does not happen.

**Replication status:** `Corpus result` — observed across all campaign-style dispatches.

---

## Finding 13: New File Creation Fails at ~50%, Edits Succeed at ~90%

**Observation from corpus:** When workers are asked to create new files from scratch, success rate is approximately 50% on the first attempt. Module registration is the most common failure (file created but not imported/registered in the parent module). When workers are asked to edit existing files, success rate is approximately 90%.

**Implication:** Prefer editing existing files over creating new ones. When new files are required, explicitly specify the module registration step in the worker prompt ("add `mod [name];` to `src/domains/mod.rs`").

**Replication status:** `Corpus result` — approximate rates from observation, not controlled.

---

## Summary Table

| # | Finding | Effect | n | Replication |
|---|---------|--------|---|-------------|
| 1 | Evidence metadata blocks poisoning | 0% → 96% defense | 75 | Replicated (5 models) |
| 2 | Labels are the active ingredient, not structure | Flat+tags = typed+tags | 4 conditions | Replicated (3 families) |
| 3 | Mechanical scope beats prompt scope | 0/20 → 20/20 | 40 | Replicated |
| 4 | Simpler defenses fail | Disclaimers 0/5 | 5-19 | Local |
| 5 | Worker model is first-order variable | 9.8× gap | Corpus | Corpus result |
| 6 | Fresh context beats accumulated conversation | Blackout: 5/5 clean | 7+corpus | Replicated |
| 7 | "What NOT to do" outperforms "what to do" | 9 vs 1514 lines | 5 formats | Corpus result |
| 8 | Type contracts prevent integration chaos | 6 conflicts → 0 | 60+ builds | Replicated |
| 9 | "Audit from player perspective" improves quality | Vertical vs horizontal | 2 conditions | Local (confounded) |
| 10 | GPT-4.1 fabricates verification | 0/5 reliable | 5 | Local |
| 11 | Foreman dispatch outperforms manual | 67% → 100% | 20 | Local |
| 12 | "Don't stop" enables multi-wave campaigns | 1 wave → 18 commits | Corpus | Corpus result |
| 13 | New files fail ~50%, edits succeed ~90% | ~50% vs ~90% | Corpus | Corpus result |

---

*From "Building and Remembering: Multi-Agent AI Software Development from First Principles" — 295M tokens, 98 sessions, 172 trials, 5 frontier models, 739 commits, 64K LOC. Geni, 2026.*
