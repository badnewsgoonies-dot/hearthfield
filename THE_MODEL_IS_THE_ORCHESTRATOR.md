# Claude Code sub-agents and nested agent calls: the complete guide

**Claude Code offers three distinct mechanisms for multi-agent orchestration**: the built-in Task tool (which spawns subagents within a session), custom named subagents (a management layer atop the Task tool), and the `-p` flag for headless CLI invocation as an external process. The critical constraint is that **subagents cannot spawn their own subagents** — the Task tool is intentionally excluded from subagent tool sets,  limiting built-in nesting to one level.   Workarounds exist via `claude -p` through the Bash tool, but with significant tradeoffs in visibility, error handling, and resource coordination.  This guide covers every flag, pattern, and gotcha for building multi-agent systems with Claude Code.

-----

## The `-p` flag: non-interactive headless execution

The `-p` (or `--print`) flag is Claude Code's primary interface for programmatic, non-interactive invocation.   It processes a prompt, outputs the response to stdout, and exits  — making it the foundation for CI/CD pipelines, shell scripts, and external orchestration.

**Basic syntax** follows a straightforward pattern:

```bash
claude -p "Explain this project"                    # Simple query
cat error.log | claude -p "Summarize errors"         # Piped input
git diff main | claude -p "Review for security"      # Pipeline integration
claude -p "Fix bugs" --output-format json            # Structured output
```

Three output formats are available. **`text`** (default) returns plain text. **`json`** returns a complete JSON object containing `result`, `session_id`, `cost_usd`, `num_turns`, and token usage — essential for scripting. **`stream-json`** emits each message as a separate newline-delimited JSON object, enabling real-time streaming pipelines.  A `--json-schema` flag can further constrain JSON output to conform to a specific schema.

**Session persistence** across non-interactive calls works by capturing the `session_id` from JSON output:

```bash
session_id=$(claude -p "Start code review" --output-format json | jq -r '.session_id')
claude -p --resume "$session_id" "Now check for GDPR compliance"
claude -p --resume "$session_id" "Generate executive summary"
```

In headless mode, **permission resolution** follows a strict order: (1) check `--allowedTools` / `--disallowedTools` flags and settings.json files, (2) if unresolved, call the MCP tool specified by `--permission-prompt-tool`, (3) if no permission-prompt-tool exists, deny the tool call. The `PermissionRequest` hook does **not** fire in `-p` mode — use `PreToolUse` hooks instead.

-----

## The Task tool and built-in subagent types

The Task tool is Claude Code's native mechanism for spawning subagents within a session.  When Claude encounters a task suitable for delegation, it invokes the Task tool with three parameters: a short **description** (3–5 words), a detailed **prompt**, and a **subagent_type** specifying which specialized agent to use.   The tool returns a structured result containing the subagent's output, token usage, cost, and execution duration.

**Three built-in subagent types** serve different purposes:

- **Explore** — A fast, read-only agent optimized for codebase search.   It starts with a **fresh context** (no parent conversation inheritance),  has access only to read-only tools (Bash, Glob, Grep, LS, Read, WebFetch, WebSearch), and can run on Haiku for cost optimization.  Ideal for file discovery and code analysis.
- **Plan** — A software architect agent for implementation planning. It inherits the full parent conversation context but cannot modify files.
- **General-purpose** — A fully capable agent with access to all tools including Bash, Edit, Write, and file operations.  It inherits parent context and permissions.

For the Task tool to function, it **must be included in `allowedTools`**  — e.g., `--allowedTools "Task Read Edit Bash"`. Even without custom subagents defined, Claude can invoke the general-purpose built-in subagent when Task is in the allowed tools list.   Up to **10 concurrent tasks** can run simultaneously, with intelligent queuing for additional requests.

-----

## Custom subagents: three definition methods

Custom subagents extend the Task tool with persistent, named, specialized agents. They can be defined in three ways, each suited to different workflows.

**Markdown files** with YAML frontmatter are the most common approach.  Project-level agents live in `.claude/agents/` (checked into version control), while user-level agents in `~/.claude/agents/` work across all projects:

```markdown
---
name: code-reviewer
description: Expert code review specialist. Use proactively after code changes.
tools: Read, Grep, Glob, Bash
model: sonnet
color: orange
---
You are a senior code reviewer ensuring high standards of code quality and security.
```

The frontmatter supports rich configuration: `tools` (comma-separated or array), `model` (`sonnet`, `opus`, `haiku`, or `inherit`), `permissionMode`, `memory` scope (`user`, `project`, `local`), `background: true` for async execution, `isolation: "worktree"` for git worktree isolation, and visibility controls like `disable-model-invocation` and `user-invocable`.

**The CLI `--agents` flag** defines agents inline as JSON for a single session:

```bash
claude --agents '{
  "debugger": {
    "description": "Debugging specialist for errors and test failures.",
    "prompt": "You are an expert debugger. Analyze errors and provide fixes.",
    "tools": ["Read", "Grep", "Glob", "Bash"],
    "model": "sonnet"
  }
}'
```

**The SDK programmatic API** (TypeScript/Python) defines agents in code:

```typescript
import { query } from "@anthropic-ai/claude-agent-sdk";

for await (const message of query({
  prompt: "Review auth module for security issues",
  options: {
    allowedTools: ["Read", "Grep", "Glob", "Task"],
    agents: {
      "security-reviewer": {
        description: "Security specialist for vulnerability analysis.",
        prompt: "Analyze code for OWASP Top 10 vulnerabilities...",
        tools: ["Read", "Grep", "Glob"],
        model: "opus",
      },
    },
  },
})) {
  if (message.type === "result") console.log(message.result);
}
```

Programmatically defined agents take precedence over filesystem-based agents with the same name.  The `/agents` slash command provides an interactive interface for managing agents,  and `claude agents` lists all configured agents from the CLI.

-----

## Why subagents cannot nest — and the `-p` workaround

**Subagents cannot spawn their own subagents.** The Task tool is explicitly excluded from every subagent's available tool set.  This is an intentional design decision,  confirmed in GitHub Issue #4182,  to prevent recursive task decomposition and resource management chaos.

The **workaround** is invoking `claude -p` through the Bash tool inside a subagent. Since subagents have Bash access, they can spawn entirely separate Claude Code processes.  However, Anthropic's documentation and community consensus strongly discourage this pattern because of several serious tradeoffs:

|Aspect                 |Task tool (built-in)                              |`claude -p` via Bash             |
|-----------------------|--------------------------------------------------|---------------------------------|
|**Visibility**         |Full progress tracking, structured output         |Opaque — no progress indicators  |
|**Error handling**     |Properly propagated through task hierarchy        |Buried in bash stdout/stderr     |
|**Resource accounting**|Coordinated token tracking and cost               |Separate process, own rate limits|
|**Context sharing**    |General-purpose/Plan agents inherit parent context|Starts with zero context         |
|**Observability**      |Hook events (SubagentStart/SubagentStop)          |None                             |
|**Concurrency**        |Up to 10 coordinated tasks                        |Unlimited but uncoordinated      |
|**Nesting**            |One level only                                    |Arbitrary depth                  |

The fundamental difference: the Task tool operates **within** a session with structured communication, while `-p` creates a **completely independent process** communicating only through stdout, files, or environment variables. For large-scale batch operations where independence is a feature (not a bug), `-p` scripting can be more appropriate.  For tightly coordinated work, the Task tool is superior.

-----

## Complete CLI flag reference for sub-agent patterns

Beyond `-p`, several flags are essential for sub-agent orchestration:

**Permission and tool control** flags determine what agents can do. `--allowedTools` pre-approves specific tools using prefix matching  (e.g., `"Bash(git diff *)"` allows any git diff command).  `--disallowedTools` blocks specific tools.  `--permission-mode` sets the overall mode: `default`, `acceptEdits` (auto-accept file operations), `plan` (read-only analysis), or `bypassPermissions` (auto-approve everything — all subagents inherit this).   `--dangerously-skip-permissions` skips all safety checks and should only be used in fully isolated containers.

**System prompt flags** control agent behavior. `--append-system-prompt` adds instructions while keeping Claude Code's defaults intact — **recommended for most use cases**. `--system-prompt` replaces the entire system prompt (blank slate — removes all default Claude Code behavior).  Both have `-file` variants for loading from disk.

**Execution control** flags manage scope. `--max-turns` limits agentic turns (critical for CI/CD to prevent runaway execution).   `--max-budget-usd` sets a cost ceiling. `--fallback-model` enables automatic fallback when the primary model is overloaded (only works with `--print`).

**Session and directory flags** manage state. `--continue` / `--resume` enable multi-turn sessions.  `--cwd` sets working directory. `--add-dir` grants access to additional directories.  `--worktree` creates an isolated git worktree. `--from-pr` links to GitHub PR sessions.

**The `--agents` flag** accepts a JSON object defining subagents inline,   while `--mcp-config` loads MCP server configurations for custom tool integrations.

-----

## Context passing and state management across agents

Context flow between parent and child agents varies by agent type. **General-purpose and Plan agents inherit the full parent conversation context**, meaning they can reference earlier discussion, understand project state, and build on prior analysis. **Explore agents start fresh** with no conversation inheritance — appropriate since search tasks are typically independent.

Each subagent runs in its **own context window**  (up to 200K tokens, or 1M with the beta flag for Opus/Sonnet 4.6). Subagent transcripts are stored in separate files that persist independently of the main conversation. When the main conversation auto-compacts (triggered near the context limit), subagent transcripts remain unaffected. Transcripts are cleaned up based on the `cleanupPeriodDays` setting (default: 30 days).

For `-p` flag orchestration, context passing is entirely manual. Common patterns include piping content via stdin, passing file paths as arguments, using `--resume` with captured session IDs, and writing intermediate results to files. The `CLAUDE.md` file hierarchy (user → project → local) provides persistent project context that loads automatically at session start,  including in headless mode  if `settingSources` is configured.

**Key environment variables** for model and context control include `ANTHROPIC_MODEL` (default model override), `CLAUDE_CODE_SUBAGENT_MODEL` (subagent-specific model),  `MAX_THINKING_TOKENS` (extended thinking budget),  and `CLAUDE_AUTOCOMPACT_PCT_OVERRIDE` (auto-compaction threshold).

-----

## CI/CD integration patterns

Claude Code's headless mode integrates directly into CI/CD pipelines.  The official **GitHub Action** (`anthropics/claude-code-action@v1`) supports `@claude` mentions in PR comments, automated PR reviews on open/sync events, and issue-to-PR automation:

```yaml
- uses: anthropics/claude-code-action@v1
  with:
    anthropic-api-key: ${{ secrets.ANTHROPIC_API_KEY }}
    prompt: "Review this PR for security issues"
    claude_args: "--allowedTools 'Read Grep' --max-turns 5"
```

For custom CI pipelines, the standard pattern combines `-p` with strict tool control:

```bash
claude -p "Analyze system performance" \
  --append-system-prompt "You are a performance engineer" \
  --allowedTools "Bash(npm test),Read,Grep" \
  --max-turns 3 \
  --output-format json \
  --cwd /path/to/project
```

Anthropic provides a **reference DevContainer configuration** for teams needing consistent, secure environments — the container's isolation and firewall rules make `--dangerously-skip-permissions` safe for unattended operation.   Exit codes follow convention: **0** for success, **1** for general error, **2** for authentication error.

-----

## Agent Teams: the next level beyond subagents

For sustained multi-agent parallelism beyond the Task tool's one-level limit, Claude Code supports **Agent Teams** (launched with Opus 4.6).  Enabled via the `CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS=1` environment variable, agent teams create separate Claude Code sessions that coordinate through shared inbox files and a task list. Unlike subagents, **teammates can message each other directly** — not just through the coordinator.

Agent teams consume approximately **5–7× the tokens** of a single session, as each teammate maintains its own context window.  They are best suited for complex, multi-faceted projects where agents need sustained independence and peer-to-peer coordination — a significant step beyond subagents' focused, short-lived task delegation.

-----

## Limits, gotchas, and hard-won best practices

**The 20K token overhead** is the most underappreciated cost. Every subagent invocation carries approximately 20,000 tokens of overhead regardless of task size. For small, focused tasks, staying in the main thread is roughly **10× cheaper** than delegating to a subagent.  Reserve subagents for genuinely complex, multi-step work.

**Cost scales linearly with parallelism.** Anthropic's own data shows multi-agent systems use approximately **15× more tokens** than chat and **4× more** than single agents.   One developer session with 49 subagents consumed 887,000 tokens per minute. A financial services company burned $47,000 in three days on a code quality project with 23 subagents.  Budget accordingly.

**Anthropic's multi-agent research system** (Opus 4 orchestrator + Sonnet 4 subagents) outperformed single-agent Opus 4 by **90.2%** on internal evals — but token usage alone explained 80% of the performance variance.   More tokens, better results, higher cost.

**Practical guidelines from Anthropic's engineering team and community:**

- **Write subagent outputs to the filesystem** rather than passing through the coordinator to avoid a "game of telephone" where context degrades through relay.
- **Scale effort to query complexity**: 1 agent for simple tasks, 2–4 for comparisons, 10+ only for genuinely complex research.
- **Teach the orchestrator how to delegate** with detailed task descriptions including objective, output format, tools, and boundaries.
- **Context window pollution** is the silent killer — test harnesses printing thousands of bytes of output consume precious context. Log to files, use grep-friendly formats.
- **Include `Task` in `allowedTools`** or subagents will never be invoked, even if defined.
- **Set `--max-turns` and cost limits** for any automated or CI/CD usage to prevent runaway execution.
- **Use `CLAUDE_CODE_SUBAGENT_MODEL`** to route subagents to cheaper/faster models (e.g., Haiku for exploration, Sonnet for focused work) while keeping the main session on Opus for complex reasoning.

## Conclusion

Claude Code's multi-agent architecture is deliberately layered: the Task tool provides structured, observable single-level delegation; custom subagents add persistent configuration and team-shareable specialization;  `-p` flag scripting enables arbitrary nesting and batch automation at the cost of coordination; and Agent Teams push into sustained multi-session collaboration. The intentional prohibition on nested subagents  reflects a design philosophy that favors predictable, debuggable systems over unlimited recursive decomposition.   For most developers, the optimal approach is starting with the main thread for simple tasks, graduating to subagents for recurring specialized workflows, and reaching for `-p` scripting or Agent Teams only when the problem genuinely demands independent, parallel execution at scale.
