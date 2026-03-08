# 03 — Tooling Landscape: Dispatch Mechanics

Load this THIRD. Understand what tools exist and their constraints before planning dispatch.

## Claude Code Sub-Agent Architecture

### Task Tool (Agent tool)
- Spawns subagents within a session, up to 10 concurrent
- **One-level nesting only** — Task tool excluded from subagent tool sets (intentional, confirmed GitHub #4182)
- Types: Explore (read-only, fresh context), Plan (inherits context), General-purpose (full tools)
- ~20K token overhead per invocation — reserve for multi-step work
- Cost: ~15x tokens vs chat, ~4x vs single agent

### `claude -p` (headless CLI)
- Completely independent process, starts with zero context
- Arbitrary nesting depth (via Bash tool in subagents)
- No visibility, no progress tracking, no context sharing
- Needs `CLAUDECODE=` unset when called from within Claude Code
- Session persistence via `--resume` with captured `session_id`

### Key flags: `--allowedTools`, `--max-turns`, `--output-format json`, `--append-system-prompt`, `--cwd`

## Codex CLI (GPT-5.4)
- `codex exec --full-auto --skip-git-repo-check "prompt"` — fully autonomous
- Defaults to GPT-5.4 model with xhigh reasoning
- Native multi-agent: `--enable multi_agent` → spawn_agent/wait/close_agent tools
- Depth limited to 1 by default (`agents.max_depth` configurable)
- **Bug**: parallel `codex exec` instances can interfere via shared `~/.codex/` session files (#11435) — use unique session dirs or stagger launches
- MCP server mode: `codex mcp-server` (experimental, self-referential pattern is buggy)

## Copilot CLI
- `copilot -p "prompt" --allow-all-tools --model claude-sonnet-4.6`
- Auth: `COPILOT_GITHUB_TOKEN=github_pat_...` (fine-grained PAT, NOT classic)
- Models: Claude Opus 4.6, Sonnet 4.6, GPT-5.4, Gemini 3 Pro
- Cost: 1 premium request (Sonnet), 3 (Opus)

## GPT-5.4 Capabilities
- 1,050,000-token context window, 128K max output
- Subsumes GPT-5.3-Codex (no separate -codex variant)
- Image generation via Responses API tool calls (gpt-image-1.5 renders pixels)
- Pricing: $2.50/$15.00 per 1M input/output, cached input $0.25/1M
- 47% fewer reasoning tokens than GPT-5.2

## CLAUDE.md Behavior
- Injected as user message after system prompt, NOT part of system prompt
- Wrapped in `<system-reminder>` tags with "may or may not be relevant" caveat
- Model CAN deprioritize instructions it deems irrelevant
- **Survives compaction** — re-read from disk after auto-compact
- 200-line recommendation (not enforced)
- This is WHY prompt-only scope enforcement fails — CLAUDE.md is context, not configuration

## Architecture Convergence
Both Anthropic (Claude Code) and OpenAI (Codex CLI) independently converged on:
- Depth-1 subagents by default
- Filesystem-mediated coordination
- Main agent as sole orchestrator
- Anti-recursion controls

## Recommended Dispatch Stack for This Project
```
Orchestrator (Claude Opus 4.6, 1M CLI) → L1: specs, planning, validation
    ↓
Codex CLI (GPT-5.4, xhigh) → L2: parallel workers
```
Stagger launches ~3s. Clamp scope after each. Verify contract. Run gates.
