# Corpus Guide

Purpose: map the orchestration evidence that actually exists on disk so a new external team can continue the study without relying on thread memory.

## Evidence Rules

- `Observed` = directly present in files, commits, worktrees, or retained logs.
- `Inferred` = supported by repo structure but not fully serialized as an explicit artifact.
- Conversation memory is not canonical. If it is not on disk, treat it as missing.

## Major Sources

| Source | Exact path / pattern | What it contains | Canonical? | Notes / weak spots |
|---|---|---|---|---|
| Foreman instructions | `/home/geni/swarm/hearthfield/CLAUDE.md` | Foreman boot order, wave cadence, verification triggers, key paths | Yes | Highest-signal operator boot file in this repo |
| Current state snapshot | `/home/geni/swarm/hearthfield/.memory/STATE.md` | Current tier, phase, debts, gate state, last decisions | Yes | Best single snapshot of what the project thinks is true now |
| Domain memory artifacts | `/home/geni/swarm/hearthfield/.memory/*.yaml` | Active principles / observations with evidence | Yes | Small set only; coverage is partial |
| Operating doctrine | `/home/geni/swarm/hearthfield/docs/HEARTHFIELD_OPERATING_KERNEL.md` | Trust order, evidence policy, invariants, artifact schema | Yes | Canonical doctrine, but some snapshot sections are stale compared to current `STATE.md` |
| Dispatch procedure | `/home/geni/swarm/hearthfield/docs/SUB_AGENT_PLAYBOOK.md` | Worker/spec/clamp/gate procedure | Yes | Strong on process; weak on enforcing behavior in practice |
| Session-specific campaign plan | `/home/geni/swarm/hearthfield/docs/CONTENT_EXPANSION_DISPATCH.md` | The content-expansion branch/worktree plan and branch names | Yes, for this campaign | Canonical for the branch split used in this session |
| Repo-wide AI overview | `/home/geni/swarm/hearthfield/status/AI_STATUS_OVERVIEW.md` | Earlier orchestrator snapshot of repo state and risks | Partial | Useful background; not current truth once newer `STATE.md` exists |
| Earlier orchestrator plan | `/home/geni/swarm/hearthfield/status/copilot-orchestrator-plan.md` | Earlier wave plan / sequencing ideas | Partial | Historic plan, not current source of truth |
| Integration template | `/home/geni/swarm/hearthfield/status/integration.md` | What integration reports should contain | Partial | Template only; not a live record of this session |
| Historic worker archive | `/home/geni/swarm/hearthfield/status/workers/*` | Prior worker reports from earlier waves/sessions | Partial | Broad context; mixed freshness and mixed evidence quality |
| Session branch report: interiors | `/home/geni/swarm/hearthfield-interiors/status/workers/interiors-complete.md` | Scope, atlas audit, delivered files, partial validation | Yes, for this branch output | Strong evidence of what the `content/interiors` branch produced |
| Session branch report: lake plaza | `/home/geni/swarm/hearthfield-lake-plaza/status/workers/lake-plaza-complete.md` | Delivered area, files, validation limits | Yes, for this branch output | Strong evidence of what the `content/lake-plaza` branch produced |
| Session branch report: mountain range | `/home/geni/swarm/hearthfield-mountain-range/status/workers/mountain-range-complete.md` | Delivered area, validation limits | Yes, for this branch output | Strong evidence of what the `content/mountain-range` branch produced |
| Session branch report: collision | `/home/geni/swarm/hearthfield-collision-pass/status/workers/collision-pass-complete.md` | Central collision fix and limits | Yes, for this branch output | Good evidence for one focused hardening pass |
| Session branch report: audit recovery | `/home/geni/swarm/hearthfield-session-audit/status/workers/session-audit-complete.md` | Atlas/visual miswire repair scope | Yes, for this branch output | Good evidence for the “recover omitted/miswired” lane |
| Session integration branch | `/home/geni/swarm/hearthfield-integration` + `git log --oneline` there | The actual integrated branch and ordered commits | Yes | Best source for what finally landed together |
| Worktree topology | `git worktree list` in `/home/geni/swarm/hearthfield` | Which branch lived in which worktree | Yes | Critical for reconstructing the session graph |
| Branch list | `git branch --list 'content/*'` in `/home/geni/swarm/hearthfield` | The campaign branches that were created | Yes | Good cross-check against worktree list |
| Local logs | `/tmp/*`, `~/.copilot/logs`, ad hoc build logs | Ephemeral command traces, if retained | Weak | This audit did not find durable `/tmp` research logs worth treating as canonical |

## Canonical Read Order For A New Researcher

1. `/home/geni/swarm/hearthfield/CLAUDE.md`
2. `/home/geni/swarm/hearthfield/.memory/STATE.md`
3. `/home/geni/swarm/hearthfield/.memory/principle-world-tileset-silent-overflow.yaml`
4. `/home/geni/swarm/hearthfield/.memory/principle-world-visual-mapping-blindness.yaml`
5. `/home/geni/swarm/hearthfield/docs/HEARTHFIELD_OPERATING_KERNEL.md`
6. `/home/geni/swarm/hearthfield/docs/SUB_AGENT_PLAYBOOK.md`
7. `/home/geni/swarm/hearthfield/docs/CONTENT_EXPANSION_DISPATCH.md` if studying this campaign
8. `git worktree list` and `git branch --list 'content/*'` in `/home/geni/swarm/hearthfield`
9. Session branch reports in the sibling worktrees:
   - `/home/geni/swarm/hearthfield-interiors/status/workers/interiors-complete.md`
   - `/home/geni/swarm/hearthfield-lake-plaza/status/workers/lake-plaza-complete.md`
   - `/home/geni/swarm/hearthfield-mountain-range/status/workers/mountain-range-complete.md`
   - `/home/geni/swarm/hearthfield-collision-pass/status/workers/collision-pass-complete.md`
   - `/home/geni/swarm/hearthfield-session-audit/status/workers/session-audit-complete.md`
10. `git log --oneline -10` in `/home/geni/swarm/hearthfield-integration`
11. Only then read the broader archive under `/home/geni/swarm/hearthfield/status/workers/`

## What Is Actually Canonical Here

- Current foreman behavior: `CLAUDE.md`
- Current project state/debt: `.memory/STATE.md`
- Process doctrine: `docs/HEARTHFIELD_OPERATING_KERNEL.md`
- Multi-worker procedure: `docs/SUB_AGENT_PLAYBOOK.md`
- This session’s branch topology and outputs: sibling worktrees + their reports + the `content/integration` commit chain

## Weak Spots / Gaps

- There is no single serialized `dispatch-state.yaml` or branch ledger recording active/completed worktrees, merge order, or validation state.
- `status/integration.md` is a template, not a maintained session artifact.
- Some session evidence exists only as commits in sibling worktrees, not copied back into the root repo.
- Local `/tmp` logs were not retained as durable evidence in a way worth treating as canonical.
- The most important operator corrections in this session were not written to disk at the moment they happened; they had to be inferred later from resulting commits and reports.

## Practical Use

- If two sources disagree, prefer: integrated branch code/commits -> session branch reports -> `STATE.md` -> doctrine docs -> older status archives.
- Treat “plan” files as intent evidence, not completion evidence.
- Treat worker completion files as branch-local truth only; always confirm against the branch tip in that worktree.
