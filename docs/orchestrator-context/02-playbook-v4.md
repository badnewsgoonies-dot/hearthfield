# 02 — Sub-Agent Playbook v4 FINAL (Contrastive-Causal + Reality Gates)

Load this SECOND. These are the operational procedures derived from the research.
Full version on disk: `docs/Sub-Agent-Playbook-v4-FINAL.md`

## Mission
Ship working builds via: (1) frozen type contract, (2) mechanical scope clamping,
(3) compiler+test gates, (4) contrastive-causal specs, (5) reality gates.

## Phase Sequence
0. **Bootstrap**: filesystem, contract (frozen+checksummed), MANIFEST.md
1. **Boundaries**: domain allowlists that survive clamping
2. **Specs on disk**: full quantitative specs with contrastive decision fields
3. **Dispatch**: workers read specs from disk, strongest model available
4. **Clamp**: mechanical scope enforcement after every worker
5. **Validate**: gate + fix loop (max 10 passes)
5A. **Reality gates**: 6 gates verifying player-reachable progress
6. **Integration**: fresh session, artifact-only, wire domains

## The Contrastive-Causal Layer (v4's key addition)
Every non-obvious instruction includes:
- **Preferred action** + why
- **Tempting alternative** + what breaks
- **Drift cue** (first signal of wrong interpretation)
- **Recovery** (smallest correction path)

Weighting: 35% causal explanation, 25% tempting alternatives, 20% consequence mapping, 10% recovery, 10% ownership.

Anti-thrash: blame-heavy prompts → apology loops, rigidity. Calm factual prompts → flexibility, narrow repair. Errors are diagnostic data, not moral events.

## Phase 5A Reality Gates
1. **EntryPoint**: work is reachable from player-facing runtime
2. **First-60-Seconds**: boot → menu → spawn → move → interact → persist
3. **Asset Reachability**: runtime-used / unreferenced / missing
4. **Content Reachability**: defined → obtainable → usable → save-safe
5. **Event Connectivity**: every event has producer AND consumer
6. **Save/Load Round-Trip**: save, reload, verify identical state

## 12 Stop Conditions
1-7: Contract drift, clamp breaks fix, false green, abstraction reflex, delegation compression, self-model error, identity paradox
8-12 (v4 additions): Blame-thrash loop, happy-path training, rule-without-rationale drift, beautiful dead game, ghost progress

## Worker Spec Required Fields
Scope, required reading, interpretation contract, required imports, deliverables, quantitative targets, failure patterns to avoid, validation commands, contrastive self-check, "what is now player-reachable"

## Key Rules
- Workers read full specs from disk (not summarized prompts)
- Worker model = first-order throughput variable (9.8x gap)
- Mechanical delegation enforcement at every level
- Never issue bare corrections — always preferred/wrong/consequence/scope/next-gate
- Machine-readable JSON reports alongside markdown
