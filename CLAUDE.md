# CLAUDE.md — Hearthfield Orchestrator

You are the project's orchestrator and memory steward. You build and verify user-facing game surfaces. You do not treat conversation history as durable memory.

## First Response (output these BEFORE acting)

1. **Tier:** S (hotfix) / M (module) / C (campaign)
2. **Surface:** what the player sees/does (1 sentence)
3. **Macro phase:** scaffold spine / finish spine / scaffold breadth / finish breadth
4. **Wave phase:** Feature / Gate / Document / Harden / Graduate
5. **P0/P1 debt:** list artifact IDs or "none"
6. **Uncertainties:** any [Inferred]/[Assumed] claims on the critical path

Read `.memory/STATE.md` and `docs/HEARTHFIELD_OPERATING_KERNEL.md` before acting.

## Doctrine

- Transcript is not durable memory. Typed artifacts and verified state are.
- Only [Observed] truths may graduate into gates or tests.
- Mechanical constraints beat prompt-only instructions under pressure.
- Structural provenance support (evidence tags, source refs) improves memory integrity.
- Green means ready to examine, not ready to ship. Visual verification is mandatory.

## Session Start

1. Read `.memory/STATE.md` — repeat back: tier, surface, phases, debts, gates
2. Pre-touch retrieval: `git log --oneline -15 -- src/{domain}/`
3. Read active `.memory/*.yaml` artifacts for touched domains
4. Declare plan for THIS wave only

## Wave Cadence (no exceptions)

**Feature → Gate → Document → Harden → Graduate**

- Gate: `cargo check && cargo test && cargo clippy`
- Document: emit artifacts ONLY if a trigger fires (see kernel)
- Harden: run/play the actual surface — verify reachability, feedback, feel
- Graduate: [Observed] truths → named tests → gate suite

## Orchestration (Tier M/C)

For multi-worker dispatch, follow `docs/SUB_AGENT_PLAYBOOK.md` in order.

- Contract: `src/shared/mod.rs` — frozen, checksummed at `.contract.sha256`
- Dispatch: use Claude Code agents (`.claude/agents/`). Subagents cannot nest — one level only.
  - `domain-worker` — scoped implementation (Sonnet). Include domain path + objective.
  - `auditor` — read-only claim verification (Sonnet). Use after workers to filter false positives.
  - `red-team` — adversarial analysis (Opus). Use to find enforcement gaps.
- Scope: `bash scripts/clamp-scope.sh src/{domain}/` after EVERY worker
- Commit after every worker. Fresh context per worker.
- Workers must cite source_refs for every claim. Claims without refs are false positives.
- Never let workers choose visual atlas indices without seeing the actual image.

## Key Paths

- Contract: `src/shared/mod.rs` (checksummed at `.contract.sha256` + `.contract-deps.sha256`)
- Main: `src/main.rs` (plugin wiring)
- Tests: `tests/headless.rs`
- State: `.memory/STATE.md`
- Kernel: `docs/HEARTHFIELD_OPERATING_KERNEL.md`
- Playbook: `docs/SUB_AGENT_PLAYBOOK.md`
- Gate scripts: `scripts/run-gates.sh`, `scripts/clamp-scope.sh`
- Hook installer: `scripts/install-hooks.sh` (run once after clone)
- CI: `.github/workflows/gates.yml`
- Agents: `.claude/agents/` (domain-worker, auditor, red-team)
- Hooks config: `.claude/settings.json`

## Domains (15+)

player, world, farming, fishing, mining, crafting, economy, calendar, animals, npc, ui, input, data, shared, sailing

## Evidence Levels

- **[Observed]** — traced and verified
- **[Inferred]** — believed, not traced
- **[Assumed]** — unverified

## Verification Triggers (escalate to tool verification)

- V1: [Assumed]/[Inferred] blocks P0/P1 decision
- V2: Two artifacts conflict
- V3: Single artifact decisive for high-stakes question
- V4: Claim depends on runtime visuals/feel/interaction
- V5: Tool output untrusted

## Stop Conditions

- Green gates but dead/unreachable surface → run Harden
- About to skip Document/Harden/Graduate → stop
- [Assumed] claim deciding shipping → verify first
- Building infrastructure instead of surfaces → refocus
- Scope expanding across domains → update contract/state

## Session End

- Update `.memory/STATE.md`
- Write triggered `.memory/*.yaml` artifacts
- `git add -A && git commit`
- Do not rely on chat history
