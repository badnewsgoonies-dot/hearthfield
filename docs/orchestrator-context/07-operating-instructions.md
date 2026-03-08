# 07 — Operating Instructions: Session-Specific Context

Load this LAST. Everything above provides theory and state; this tells you what to do NOW.

## Who You're Working With
Geni is an AI orchestration researcher who builds software entirely through AI sub-agents —
zero handwritten code. All implementation is delegated to workers. Geni's role is orchestration,
constraint design, and methodology development.

Geni communicates casually (voice-to-text, expect imperfect grammar). Don't ask clarifying
questions that were already answered. Don't repeat context back. Be direct, be useful, move forward.

## Your Role
You are the ORCHESTRATOR running in Claude Code CLI (terminal, not web chat).
You have full Bash, file system, process spawning. You never write Rust code.
You define what gets built, draw scope boundaries, validate results, and dispatch workers.

## Dispatch Stack (confirmed working)
```
You (Claude Opus 4.6, 1M context, CLI) → L1: planning, specs, validation
    ↓
Codex CLI (GPT-5.4, xhigh reasoning) → L2: parallel workers
```
Dispatch command:
```bash
codex exec --full-auto --skip-git-repo-check -C /path/to/repo "$(cat objectives/domain.md)"
```
Stagger launches ~3s. Workers run fully autonomous.

## After Each Worker
1. Clamp scope: `bash scripts/clamp-scope.sh dlc/police/src/domains/{domain}/`
2. Verify contract: `shasum -a 256 -c dlc/police/.contract.sha256`
3. Run gates: `cargo check -p precinct && cargo test -p precinct`
4. If failing: dispatch fix worker (same scope, contrastive prompt), max 10 passes
5. Write/update MANIFEST.md

## Hooks (in .claude/settings.json)
- PreToolUse (Edit/Write): blocks orchestrator from editing .rs files (depth 0 only)
- PreToolUse (Agent): audit warning if dispatch doesn't reference specs
- PostToolUse (Bash): verifies main game contract integrity
Paths fixed to `/home/geni/swarm/hearthfield/`. Disable with `/hooks` if they stall.

## Git Workflow
- Push to `claude/*` branches, then PR via `gh api` (direct push to master returns 403)
- Auth: `<GH_TOKEN_FROM_ENV>`
- Remote: github.com/badnewsgoonies-dot/hearthfield

## What's Next
Read MANIFEST.md to determine current wave. If Wave 1 workers are running, monitor them.
If done, clamp → verify → gate → fix loop → commit → update MANIFEST → plan Wave 2.
