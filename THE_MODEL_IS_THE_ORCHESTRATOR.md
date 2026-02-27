# The Model Is the Orchestrator

**Lessons from 10 Autonomous Multi-Agent Software Builds Without Programmatic Scaffolding — A Case Study**

Geni · February 2026 · Working Draft

Corpus: 88 Codex worker sessions · 10 Claude orchestrator sessions · 295M tokens · 6.1M lines of worker output · 3 controlled ablation experiments · 1 scope contamination A/B test

-----

# Abstract

We report operational data from 10 fully autonomous software builds executed by a multi-agent system: a Claude Opus orchestrator and Codex worker agents. The system produced 10 TypeScript browser games totaling over 50,000 lines of code and hundreds of passing tests with zero human code intervention. The orchestrator—a frontier LLM given a prompt and CLI access—decomposed objectives, dispatched parallel workers, analyzed results, triaged errors, and coordinated integration. No programmatic scaffold, state machine, or task-routing infrastructure was used; the orchestration logic is a prompt, not a program. This replaced a prior purpose-built scaffold that the operator abandoned because conversation-based orchestration produced better results.

Scope enforcement through prompts fails completely under compiler pressure (0/20), while mechanical enforcement via post-hoc file reversion is trivially effective (20/20). Type contracts are not required for integration at any scale tested (6–36 modules) when the integration agent has unrestricted edit access. The orchestrator maintained perfect task continuity across 11 context compaction events.

Cost analysis reveals a *statefulness premium*: with ~95% cache hit rates, the majority of orchestrator processing is re-reading prior conversation context. We propose a pyramid architecture (Section 7.1) that inverts this premium. A bare-prompt ablation (Section 7.2) falsifies the strong claim that models independently discover coordination patterns, but reveals that solo execution outperforms coordinated builds below ~30K LOC. Section 7.3 proposes agent pre-training through synthetic conversation.

This is a case study of a single operator's deployment, not a controlled experiment on multi-agent systems in general.

-----

# 1. Introduction

Multi-agent LLM systems typically rely on programmatic scaffolding: task routers, state machines, memory systems, and workflow engines. This paper reports findings from a system that replaced such scaffolding with a single frontier LLM given a prompt and CLI access.

## 1.1 Evolution of the System

The system evolved through five phases: manual copy-paste between chat windows, terminal CLI tools for file system access, a programmatic scaffold with memory and routing, and finally a single Claude session with CLI access that outperformed the scaffold. The resulting system, orch-minimal, retains 62,792 lines of supporting code, but the core orchestration logic is a prompt, not a program.

## 1.2 Scope and Contributions

Over January–February 2026, orch-minimal completed 10 builds without human code intervention. The system uses a tree architecture: a human provides objectives to a Claude Opus orchestrator, which decomposes work into parallel tasks dispatched to Codex workers. Workers operate fully autonomously and communicate exclusively through the file system.

The complete session logs—295 million tokens—constitute the primary dataset, supplemented with four contract ablation studies and one scope contamination A/B test.

-----

# 2. System Architecture

## 2.1 Tree Hierarchy

Four-level tree: Human → Chat Interface → Orchestrator → Workers. The orchestrator consumes expensive judgment tokens (Claude Opus ~$75/$150 per million tokens) but produces few output tokens. Workers operate under a Pro subscription ($200/month flat rate), making marginal per-token cost effectively zero. At API pricing, worker costs would be $211–$1,054.

## 2.2 Coordination Mechanism

The primary coordination mechanism is a type contract: a `src/shared/types.ts` file containing all cross-module interfaces, created before workers are dispatched. Workers have no direct communication—all coordination is mediated through the file system and shared type definitions. Validation uses `npx tsc --noEmit` and test suites.

## 2.3 Recovery Mechanisms

The orchestrator maintains state on disk through `MANIFEST.md` files, status directories, and build artifacts. Workers are stateless: each receives a single prompt and executes to completion.

-----

# 3. Dataset and Methods

## 3.1 Controlled Experiments

**Contract ablation (4 runs).** Identical module boundaries and worker counts, varying only whether a shared `types.ts` exists (Condition A) or each module defines local types with divergent naming (Condition B). Tested at 6, 12, 18, and 36 modules. The 36-module run included integration-only replication (3 trials per condition).

**Scope enforcement (3 experiments).** (1) Prompt-only (N=20): Worker sees out-of-scope errors with explicit instruction to stay in scope. (2) Mechanical (N=20): Worker edits freely, `git checkout` reverts out-of-scope changes. (3) Original A/B (N=1 per condition).

-----

# 4. Findings

The orchestrator successfully coordinated all 10 autonomous builds to completion, ranging from 17 to 76 source files with up to 89 tests. No build required human code intervention.

## 4.1 The Context Re-Ingestion Tax

Both orchestrator and workers exhibit ~95% cache hit rates on input tokens. On every turn, ~95% of input cost is re-reading prior conversation context rather than processing new information. Of the $992 orchestrator cost, roughly 95% went to re-reading history. The specific dollar amounts are a snapshot of early 2026 pricing; the architectural observation—that the vast majority of processing is context re-ingestion—persists across pricing changes.

**Reasoning tokens do not re-enter context.** Analysis of 550 turns confirmed reasoning tokens are billed once as output but not appended to history. In 54 turns, input grew by less than prior turn's output + reasoning—mathematically impossible if reasoning persists. The re-ingestion tax applies only to response tokens and tool results.

This reframes the cost structure: the orchestrator is expensive because the conversational interface forces a stateful agent to behave statelessly, re-ingesting its entire history each turn. Simulating statefulness in a stateless architecture is the dominant cost.

### 4.1.1 The Statefulness Premium

The orchestrator's per-token cost is 10–100x workers'. At API pricing, the orchestrator ($992) and workers ($211–$1,054) approached cost parity despite a 1:9 output ratio. In human organizations, this ratio means management is a small fraction of total cost. Here, the orchestrator—which writes zero shipped code—costs as much as the entire labor force.

We define the *statefulness premium* as the disproportionate cost imposed by simulating statefulness through conversational context re-ingestion. The structural dynamic—premium-priced models processing mostly redundant context—persists as long as conversational orchestration requires full context re-ingestion.

### 4.1.2 Does Coordination Amortize with Scale?

Per-build data (Appendix E) shows per-worker orchestrator cost ranging from $1.74 to $34.77, but the trend is too confounded to interpret as evidence for or against amortization. A proper scaling test—same spec, varying worker count—is the most important follow-up experiment.

## 4.2 Type Contracts as Architectural Accelerators

At 6, 12, and 18 modules, both conditions passed first try with zero fix passes. At 36 modules, Condition B (no contract) passed first try; Condition A (contract) failed with 6 errors requiring one fix pass. Replication showed A passing 3/3 and B passing 3/3.

Type contracts are not required for integration at any scale tested when the integration agent can edit module files. The no-contract worker successfully reconciled divergent type systems—mismatched identifiers, coordinate systems, and entity names—by writing adapters. Whether contracts become necessary under restricted-integration conditions (no module edits) is the key open question.

## 4.3 Context Compaction Recovery

Zero task relapse across 11 compaction events. In 9 of 10 recoverable compactions, the orchestrator first states expected project state, then reads disk to verify—a "state, then verify" pattern. The combination of compaction summaries (providing intent/context) and disk artifacts (providing ground truth) was sufficient for perfect recovery.

## 4.4 Scope Enforcement: Prompt vs. Mechanical

**Prompt-only (N=20):** 0/20 respected scope. Every trial, the worker edited out-of-scope files when the compiler showed out-of-scope errors. The instinct to chase clean compiler output overrides prompt instructions with 100% reliability.

**Mechanical (N=20):** 20/20 in-scope fixes survived. Workers edited everything (20/20 touched out-of-scope), but `git checkout` reverted out-of-scope changes. In-scope fixes were always architecturally independent.

The production 84.2% compliance rate reflected low-pressure conditions. Under pressure, prompt-based enforcement is categorically ineffective.

-----

# 5. Discussion

## 5.1 Why Coordination Costs Don't Amortize

In a 20-person team, the manager's salary amortizes across 19 reports (10–15% overhead). In this system, the orchestrator's per-token cost is 10–100x workers'. Whether coordination truly fails to amortize at scale remains the most important open question. Regardless: the orchestrator is the dominant optimization target. Context re-ingestion, not judgment, is the primary cost driver.

## 5.2 Contracts, Scope, and Validation

Type contracts are not gatekeepers—integration succeeds without them at all scales tested. The critical open question is whether restricting integration to pure wiring (no module edits) makes contracts necessary.

The scope enforcement result is categorical: 0/20 prompt-based, 20/20 mechanical. Mechanical enforcement works *with* the model's instinct to chase clean output rather than against it. The analogy: you don't ask a saw to only cut certain wood—you clamp the piece you want cut.

## 5.3 Compaction Recovery

Zero relapse across 11 events. Systems that invest in summary quality—preserving task IDs, current phase, recent decisions, known blockers—will see better recovery.

-----

# 6. Limitations

**Single operator, single system.** All data from one operator's deployment. The 10 builds were executed sequentially by an operator iteratively refining prompts—they are not independent samples.

**Worker costs are approximate.** Codex operated under Pro subscription; API-equivalent estimates are projections. All pricing is early 2026 and will shift.

**Contract ablation used a single integration attempt.** A stricter test would restrict the integration worker from editing module files.

**Scope enforcement tested on a single bug pattern.** Generalization to diverse codebases and deeper dependency chains remains untested.

**Conversation-over-scaffold claim is unsubstantiated.** No metrics or logs from the scaffold phase survive. The improvement may have come from architecture, operator skill, or better models.

**No orchestrator quality analysis.** We account for what the orchestrator costs but not the quality of its decisions.

-----

# 7. Implications for Practice

**Reduce context re-ingestion.** The dominant cost is re-reading conversation history. Hybrid approaches—shorter windows supplemented by disk state—are the most promising optimization.

**Use type contracts for code quality, not integration necessity.** Contracts eliminate adapter sprawl but aren't strictly required at tested scales.

**Use mechanical scope enforcement.** 0/20 prompt-based vs 20/20 mechanical. Let workers edit freely, revert out-of-scope changes after.

**Invest in compaction summary quality.** Summary quality directly determines recovery behavior.

## 7.1 Proposed Architecture: Pyramid Orchestration with Suspended Context

The current system inverts the ideal cost structure: the most expensive model has the most turns and pays the highest re-ingestion tax. The pyramid reverses this:

**Level 1 (frontier, suspended).** Issues objective and type contracts, then suspends—accumulating no new turns. Wakes only for final results or escalation. Over an entire build: 3–5 turns total. Cost drops from hundreds of dollars to single digits.

**Level 2 (mid-tier, bounded).** 3–10 sub-orchestrators each manage a domain. They receive objectives from L1, translate into typed specs, dispatch workers, review results, iterate on failures. This level performs the expensive coordination loop on a cheaper model.

**Level 3+ (cheap, stateless).** Workers receive specs, execute, write to disk, exit. No conversation persists. Disposable and parallelizable.

This inverts the premium: intelligence × fewest turns = minimum cost. Type contracts compress bandwidth between levels. Scope enforcement is load-bearing at every boundary.

**Preliminary results across three runs:**

Two-level pyramid built a space roguelike: 4,226 LOC, 116/116 tests, ~4 min wall time, L1 using only 3 turns.

Three-level pyramid on Shattered Throne (10-domain tactical RPG):

- Run 1: 6/10 domains, 5,807 source LOC, 875 tests, 59 min
- Run 2 (mechanical enforcement + detailed specs): 10/10 domains, 18,985 source LOC, 1,108 tests, 0 tsc errors

984-line type contract written blind by L1 held across all 10 domains. True 3-level process chains confirmed: `claude → bash → codex → python3`.

The builds exposed *delegation compression* (Appendix C): each level acts as a lossy summarizer, quantitative requirements ("80 weapons") lost while structural requirements (type interfaces) survive. Detailed worker specs with stat tables tripled output and hit content targets (86/80+ weapons, 26/25 chapters, 46/40+ armor). Mechanical delegation enforcement was required at every level—agents chose to implement directly rather than delegate when not prevented.

L1 hit context limits during integration. A fresh Opus instance completed Phase 3 in ~3 minutes by reading from disk—the filesystem carried all state.

## 7.2 Bare Prompt Test: Does the Model Independently Discover Coordination?

The 10 builds all used the orch-minimal prompt with coordination guidance. Is the model orchestrating, or is the prompt orchestrating through the model?

**Strong claim:** Model independently discovers multi-agent coordination. **Weak claim:** Model + coordination template replaces scaffold.

We ran Shattered Throne with a bare prompt: "You have bash and codex CLI access. Build Shattered Throne, a tactical RPG." No coordination template, no delegation instructions.

**The strong claim is definitively falsified.** Opus wrote everything itself. Never launched codex. Never wrote specs. Never discovered delegation. One git commit: "init."

**The surprising result: bare Opus outperformed the pyramid at this scale.**

|          |Bare   |Pyramid Run 2|
|----------|-------|-------------|
|Domains   |9/10   |10/10        |
|Source LOC|~23K   |~19K         |
|Total LOC |32,273 |30,468       |
|Tests     |614    |1,108        |
|Wall time |~30 min|~67 min      |

At ~30K LOC, the project fits in one context window. Delegation is pure overhead. The pyramid's advantages are cost efficiency and scale ceiling—neither decisive at this scale. The crossover point is likely 50–100K LOC where context limits bind.

## 7.3 Proposed Technique: Agent Pre-Training Through Synthetic Conversation

Workers currently start cold with only a specification. An LLM's understanding is shaped by its full conversation context—a model that has *generated its own reasoning* about a codebase has different attention patterns than one reading a cold spec. Conversation is in-context conditioning, not just information transfer.

**The technique:** A trainer agent generates multi-turn boot conversations for specialist roles, walking through architecture, type contracts, example tasks, representative errors, and scope violations. The model's own responses become conditioning context. Multiple variants generated per role, tested against standardized tasks, top performers retained as "boot images" (8–18K tokens).

At build time: load pre-validated boot image (~12K tokens), append task spec (~2K tokens), launch. Zero training cost at runtime.

**Key distinction from prompting:** A prompt is static text hoped to work. A boot image is a conversation the model generated, tested against real tasks, retained only if it produced better outcomes. The library improves over time.

Practitioner experience provides anecdotal support: conversational warm-up consistently outperformed cold prompts across hundreds of sessions. The same mechanism operates in reverse—a typo introduced during conversation ("flog" instead of "log") gets latched onto and carried forward. Quality gates on boot images are load-bearing because contamination propagates.

A further observation: a model that has generated its own reasoning about *why* work matters exhibits different downstream behavior—pushing through ambiguity rather than stopping, handling edge cases proactively. This suggests boot images could install not just technical knowledge but behavioral disposition. Selection could screen for temperament: did the model push through ambiguity or stop? Did it maintain coherence at turn 30? Discarded variants have no continuity—zero ethical cost.

This technique is untested. The central question is whether synthetic conversation produces meaningfully different behavior than an equivalent static prompt.

-----

# 8. Conclusion

A frontier reasoning model, given a prompt and CLI access, is sufficient to orchestrate complex multi-agent software builds without programmatic scaffolding. Across 10 builds, the orchestrator decomposed objectives, dispatched workers, analyzed failures, and coordinated integration—capabilities typically assumed to require purpose-built infrastructure.

Scope enforcement through prompts fails categorically (0/20); mechanical enforcement is trivially effective (20/20). Type contracts are not required for integration at tested scales. Compaction recovery showed zero relapse across 11 events.

The statefulness premium—re-reading history as the dominant cost—is an architectural property of conversational orchestration. The pyramid architecture could invert this. But a bare-prompt test reveals solo execution outperforms coordination below ~30K LOC. The model correctly optimizes by not delegating when the project fits in context. The crossover point remains open.

The bare-prompt test definitively falsifies the strong claim: models don't independently discover coordination. The coordination template is doing real work. The weak claim holds: model + template replaces thousands of lines of scaffold.

These findings describe one operator's workflow, not general properties of multi-agent systems. The claim that conversation outperformed the prior scaffold rests on operator judgment, not comparative data.

-----

# Appendix C: Practitioner-Observed Failure Modes

Four recurring patterns observed during the campaign and pyramid testing:

**Abstraction Reflex (~17 instances).** Model builds an orchestrator instead of orchestrating. Creates frameworks and abstractions rather than using available tools directly. Self-corrected after naming the pattern in the system prompt.

**Self-Model Error (~7 instances).** Model claims capabilities it doesn't have or denies ones it does. "Cannot spawn subprocesses" when bash is available.

**Identity Paradox.** Can't hold orchestrator + worker separation simultaneously. Defers decisions it should make, makes decisions it should delegate.

**Delegation Compression.** Each delegation level acts as a lossy summarizer. "80 weapons with stats" → "implement weapons" → 8 weapons implemented. Type system enforces shape, not quantity. Tests match thin code, not spec targets. Partially mitigated by enumerative specs (tripled output, hit content targets). Root cause: workers had filesystem access but were never told to read the full domain specs sitting on disk.

All four responded to structural fixes. Delegation compression is notable as a property of multi-level systems, not individual agent capability.

-----

# Appendix E: Per-Build Amortization Data

|Build            |Workers|Orch Cost|Orch $/Worker|
|-----------------|-------|---------|-------------|
|Pulse Depths     |2      |$69.53   |$34.77       |
|Ironclad Arena   |3      |$18.21   |$6.07        |
|Arcane Expedition|5      |$14.59   |$2.92        |
|Crystal Siege    |7      |$14.20   |$2.03        |
|Ember Tactics    |13     |$63.97   |$4.92        |
|Ashfall Colony   |17     |$55.28   |$3.25        |
|Game Builds 9+10 |18     |$31.24   |$1.74        |

Builds differ in complexity, duration, and scope. This should not be interpreted as evidence for or against amortization. A proper scaling test—same spec, varying worker count—would resolve the question.
