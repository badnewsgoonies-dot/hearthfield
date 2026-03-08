# Orchestrator Context Transfer — Loading Order

Hand these files to a new orchestrator session in this exact order.
Each file builds on the previous. Skipping or reordering degrades priming quality.

## Order & Rationale

| # | File | Why This Order | ~Tokens |
|---|------|---------------|---------|
| 01 | `01-research-paper.md` | Foundational theory — all constraints derive from this evidence | ~8,000 |
| 02 | `02-playbook-v4.md` | Operational procedures — the paper's findings distilled into protocol | ~4,000 |
| 03 | `03-tooling-landscape.md` | Dispatch mechanics — how the orchestrator actually executes | ~5,000 |
| 04 | `04-base-game-reference.md` | Hearthfield architecture + audit — the reference implementation | ~4,000 |
| 05 | `05-build-plan-template.md` | The Perfect Build Plan — battle-tested orchestration template | ~3,000 |
| 06 | `06-precinct-project-state.md` | Current project: spec, contract, what's done, what's next | ~5,000 |
| 07 | `07-operating-instructions.md` | Session-specific: who Geni is, working style, immediate task | ~1,000 |

**Total**: ~30,000 tokens — fits comfortably in any model's context alongside system prompt.

## Why This Order

1. **Theory first** (01-02): The orchestrator needs to understand WHY constraints exist
   before being told WHAT the constraints are. Without the paper, the playbook is
   arbitrary rules. With it, every rule maps to a measured failure mode.

2. **Tools second** (03): Before seeing the project, understand what dispatch
   mechanisms exist and their tradeoffs. This prevents the orchestrator from
   planning around tools that don't work.

3. **Reference third** (04-05): The base game's architecture, audit results, and
   the build plan template provide patterns to follow and anti-patterns to avoid.
   The orchestrator sees what "done" looks like before seeing the current project.

4. **Project last** (06-07): With theory, tools, and reference loaded, the
   orchestrator can evaluate the current project state against known patterns
   rather than discovering them from scratch.

## For Claude Code Sessions

Paste these into chat in order, or use @-imports in a CLAUDE.md:
```
@docs/orchestrator-context/01-research-paper.md
@docs/orchestrator-context/02-playbook-v4.md
...etc
```

Or load via `--append-system-prompt-file` for headless sessions.
