# Experiment Registry

Purpose: give a new external team a concrete registry of the orchestration and memory experiments that are actually evidenced on disk, distinguish what is observational versus controlled, and mark what is still missing.

This file is not a theory essay. It is a source map.

## Evidence Rules

- `Observed` means directly supported by retained repo files, commits, worktrees, or local session logs.
- `Inferred` means the interpretation is plausible from retained evidence, but the exact experimental trace is not fully preserved.
- `Missing` means the write-up makes a claim, but the raw run table, prompt, or outcome artifact is not currently retained in a way this repo can verify.

## Design Classes

- `Observational` = corpus pattern or historical read across prior sessions/builds
- `Controlled` = explicit intervention or ablation described as an experiment
- `Replicated` = same qualitative result reported across multiple runs / conditions
- `Derived` = synthesis or recommendation built on prior findings, not a standalone experiment

## Registry

| ID | Experiment / claim | Design class | Status | What is directly observed | Primary evidence | What is missing / weak |
|---|---|---|---|---|---|---|
| E01 | Statefulness premium: orchestrator cost is dominated by re-reading prior conversation (~95%) | Observational | `Observed` | The claim, numbers, and interpretation are retained in the research foundation docs | `/home/geni/swarm/hearthfield/docs/orchestrator-context/01-research-paper.md`; `/home/geni/swarm/hearthfield/docs/UNIVERSAL_GAME_KERNEL.md` | Raw token/accounting tables are not retained in this repo |
| E02 | Prompt-only scope control fails; mechanical clamp succeeds (`0/20` vs `20/20`) | Controlled + replicated | `Observed` | The numerical result and operational prescription are retained in multiple docs | `/home/geni/swarm/hearthfield/docs/orchestrator-context/01-research-paper.md`; `/home/geni/swarm/hearthfield/docs/SUB_AGENT_PLAYBOOK.md`; `/home/geni/swarm/hearthfield/docs/UNIVERSAL_GAME_KERNEL.md` | Trial-by-trial raw logs are not retained here |
| E03 | Type contracts reduce false-green parallel divergence; no-contract ablation produced incompatible interfaces | Controlled | `Observed` | The ablation result and interpretation are retained in the research foundation doc | `/home/geni/swarm/hearthfield/docs/orchestrator-context/01-research-paper.md` | Raw interface snapshots and scoring sheet are missing |
| E04 | Context priming: cold `0/10`, static doc `10/10`, dialogue `10/10` | Controlled | `Observed` | The condition structure and outcomes are retained on disk | `/home/geni/swarm/hearthfield/docs/orchestrator-context/01-research-paper.md` | Raw prompt/output corpus for the 30 implementations is not retained here |
| E05 | Bare-prompt delegation ablation: strong model does not spontaneously discover coordination | Controlled | `Observed` | The claim and its implications are retained in the research foundation doc | `/home/geni/swarm/hearthfield/docs/orchestrator-context/01-research-paper.md` | The exact bare prompt and resulting build transcript are not retained locally |
| E06 | Worker model capability is first-order throughput variable (`9.8x` gap) | Observational / benchmark | `Observed` | The finding and number are retained on disk | `/home/geni/swarm/hearthfield/docs/orchestrator-context/01-research-paper.md` | Per-model run sheet is missing |
| E07 | Scaling sweet spot: around 10 workers, `2.05x` speedup | Controlled / benchmark | `Observed` | The number and interpretation are retained | `/home/geni/swarm/hearthfield/docs/orchestrator-context/01-research-paper.md` | The detailed benchmark matrix is not in this repo |
| E08 | Compaction recovery: zero relapse across 11 events | Observational | `Observed` | The finding is retained in the research foundation doc | `/home/geni/swarm/hearthfield/docs/orchestrator-context/01-research-paper.md` | No per-event recovery ledger is retained here |
| E09 | A/B/C memory framing: `A` compacted carry-forward, `B` fresh untyped retrieval, `C` fresh typed retrieval with provenance | Derived from controlled work | `Observed` as framing, `Missing` as a fully retained full matrix | The mapping of existing experiments into A/B/C is retained on disk | `/home/geni/swarm/hearthfield/docs/orchestrator-context/01-research-paper.md`; `/home/geni/swarm/hearthfield/docs/UNIVERSAL_GAME_KERNEL.md` | A single retained table with prompts, conditions, and outcomes for all cells is missing |
| E10 | Hardening / graduation timing explains downstream quality better than raw LOC alone | Observational comparative claim | `Observed` for the case evidence, `Inferred` for broad generalization | The City vs Precinct comparative argument is retained in a worker debrief | `/home/geni/swarm/hearthfield/status/workers/cross-session-debrief.md` | External replication across more than one pair of builds is missing |
| E11 | Process overhead is large and rework-heavy; fix-loop frequency is a major cost center | Observational | `Observed` | Report counts, token estimates, and fix ratios are retained | `/home/geni/swarm/hearthfield/status/retrospective/build-process.md`; `/home/geni/swarm/hearthfield/status/retrospective/EXECUTIVE_SUMMARY.md` | Raw cost ledger is missing; token estimate is inferred from artifact volume |
| E12 | Ghost progress in current Codex orchestration session: repeated “patching now” claims with clean worktrees | Observational local-session case | `Observed` | Direct log lines show clean worktrees despite implementation claims | `/home/geni/.codex/sessions/2026/03/12/rollout-2026-03-12T12-31-35-019ce2e3-a54f-7ac3-9c52-669e1445a0d1.jsonl` | Single-session case only; not yet cross-session coded |
| E13 | Audit drift / pre-edit pause in current world-map lane | Observational local-session case | `Observed` | Direct log lines show scout completion, no diff, and pause before edit | `/home/geni/.codex/sessions/2026/03/12/rollout-2026-03-12T13-29-23-019ce318-8d68-7de0-9db4-49a8a13967de.jsonl` | Single-session case only; not yet compared against successful lanes |
| E14 | Open-ended discovery can surface non-obvious, locally grounded opportunities (agenda layer) | Observational local-session case | `Observed` | Direct scout output plus resulting code in the repo | `/home/geni/.codex/sessions/2026/03/12/rollout-2026-03-12T14-15-02-019ce342-5c51-7a02-941b-1c0f5dc79c6f.jsonl`; `/home/geni/swarm/hearthfield/src/ui/hud.rs`; `/home/geni/swarm/hearthfield/src/ui/tutorial.rs` | `n=1`; open-ended execution reliability is not proven by this alone |
| E15 | Dirty-tree linker failure looked like build-artifact corruption rather than source-level break | Observational local-session case | `Observed` for the sequence, `Inferred` for root cause | Logs show `cargo check` green, lib-only agenda tests green, generic test link failure, then full clean headless rebuild green | `/home/geni/.codex/sessions/2026/03/12/rollout-2026-03-12T16-16-47-019ce3b1-d284-79e1-a480-23fcf0219c82.jsonl` | Still not a controlled artifact-corruption experiment |

## What the Registry Supports Today

These are defensible from retained evidence:

- Mechanical scope enforcement is the best-supported controlled result.
- Statefulness-premium behavior is well-supported as a corpus observation in the retained doctrine.
- Context presence matters more than dialogue format in the retained context-priming experiment.
- Hardening/graduation underexecution is visible both in doctrine docs and in retrospective files.
- Current-session ghost progress and audit drift are directly visible in the March 12 Codex logs.
- A single open-ended discovery success exists on disk: the agenda-layer proposal and its proof slice.

## What Is Still Missing

### Missing raw experimental packets

The repo currently lacks a single durable place containing, for each experiment:

- exact prompt
- model / tool configuration
- run count
- scoring rubric
- raw outputs or reduced-coded outputs
- final interpretation

That weakens every “controlled” claim, because the result is often present only as a later summary in doctrine docs.

### Missing operator-normalized session coding

The repo has:

- doctrine docs
- worker reports
- retrospectives
- current session logs

But it does not yet have a coded cross-session event ledger for:

- ghost progress
- audit drift
- green-gate exit
- stale-assumption dispatch
- hardening/graduation execution
- disconnect/confound periods

### Missing fairness controls

The retained evidence does not support a strong clean comparison across Claude, Codex, and Copilot because onboarding depth was not equalized. The external team should treat cross-system comparisons as partially confounded until the boot bundle, task slice, and evaluation rubric are standardized.

## Recommended Next Additions

If an external team continues the study, the next missing artifacts to create are:

1. `status/research/dispatch-event-ledger.csv` or `.yaml`
2. `status/research/experiment-packets/<experiment-id>.md`
3. `status/research/source-index.md` mapping local logs, repo docs, and retained worker reports
4. A standardized trial template recording:
   - model
   - tool substrate
   - boot bundle
   - task slice
   - allowed scope
   - success criteria
   - observed failure modes

## Bottom Line

This repo is strong on doctrine and post-hoc interpretation. It is weaker on retaining raw experiment packets.

An external team can continue the study from here, but they should treat:

- the doctrine docs as the best retained synthesis,
- the local March 12 Codex logs as high-value observational material,
- and the missing raw run tables as the main evidence gap to close next.
