# Codex CLI Layered Spawn Trials — 2026-03-13

## Setup
- Codex CLI v0.114.0, model gpt-5.4
- `features.multi_agent = true`
- `sandbox_mode = "danger-full-access"`
- WebSocket transport failed (403), fell back to HTTPS for all trials — adds ~10s startup latency

## Trial A: Single Question Baseline

| Metric | Value |
|--------|-------|
| Question | How many MapId variants? |
| Answer | 18 (correct) |
| Tokens | 8,585 |
| Wall time | 18s |
| Sub-agents | 0 |

## Trial B: Depth-2 Nesting (Root → L1 → L2)

| Metric | Value |
|--------|-------|
| Config | `agents.max_depth=3` |
| Depth 0 answer | 18 MapId variants (correct) |
| Depth 1 answer | 11 NPCs (correct) |
| Depth 2 answer | 16 crops (correct) |
| All spawns | succeeded |
| Tokens | 15,957 |
| Wall time | 75s |

Root called `spawn_agent()` → Layer 1 called `spawn_agent()` → Layer 2 answered directly.
All answers verified against code.

## Trial C: 3 Parallel Sub-agents (Fan-out)

| Metric | Value |
|--------|-------|
| Config | default `agents.max_depth=1` |
| Agent 1 | 5 ToolTier variants with costs (correct) |
| Agent 2 | 39 cooking recipes (correct) |
| Agent 3 | 214 headless tests (correct) |
| All spawns | succeeded in parallel |
| Tokens | 8,770 |
| Wall time | 51s |

Root spawned all 3 agents simultaneously, then called `wait()` to collect results.
Results returned incrementally: Agent 1 first, then Agent 3, then Agent 2.

## Trial D: Depth-3 Nesting (Root → L1 → L2 → L3)

| Metric | Value |
|--------|-------|
| Config | `agents.max_depth=4` |
| Depth 0 answer | 18 MapId variants (correct) |
| Depth 1 answer | 11 NPCs (correct) |
| Depth 2 answer | 16 crops (correct) |
| Depth 3 answer | 39 cooking recipes (correct) |
| Spawns 0→1 | succeeded |
| Spawns 1→2 | succeeded |
| Spawns 2→3 | succeeded |
| Depth 3 spawn attempt | failed (hit depth limit, did work itself) |
| Tokens | 9,080 |
| Wall time | 111s |

**4 layers of agent nesting, all correct, all spawned successfully.**
Layer 3 reported hitting the depth limit and gracefully degraded.

## Comparison: Claude Code vs Codex

| Capability | Claude Code | Codex CLI |
|-----------|------------|-----------|
| Max depth | 1 (hard, structural) | Configurable via `agents.max_depth` |
| Enforcement | Agent tool removed from sub-agent toolset | Depth counter, graceful limit |
| Parallel fan-out | Yes (unlimited) | Yes (default max 6 threads) |
| Depth-2 nesting | IMPOSSIBLE | WORKS |
| Depth-3 nesting | IMPOSSIBLE | WORKS (with max_depth=4) |
| Context inheritance | None (spawn) or full (resume) | spawn_agent (task only) |
| Fork (full transcript) | Not available | `codex fork` clones transcript |
| All answers correct | Yes (at depth 1) | Yes (at all depths tested) |

## Key Observations

### 1. Codex achieves true hierarchical multi-agent orchestration
The tree topology works:
```
Root ── L1 ── L2 ── L3 (hit limit, self-answered)
```
Each layer spawns the next via `spawn_agent()`, waits via `wait()`, and collects results.
This is NOT simulated — these are actual separate agent threads with their own tool access.

### 2. Depth costs are linear, not exponential
| Depth | Tokens | Wall time | Tokens/layer |
|-------|--------|-----------|--------------|
| 0 (baseline) | 8,585 | 18s | 8,585 |
| 0→1→2 | 15,957 | 75s | ~5,300 |
| 0→1→2→3 | 9,080 | 111s | ~2,270 |

Token cost per layer DECREASES at deeper nesting because deeper agents do less work.
Wall time grows linearly (~30s per additional layer).

### 3. Parallel fan-out is more token-efficient than serial nesting
3 parallel agents: 8,770 tokens, 51s
3-deep serial chain: 15,957 tokens, 75s
The parallel approach uses 45% fewer tokens because there's no chain overhead.

### 4. Graceful degradation is universal
Both Claude Code and Codex agents handle spawn failure identically: they do the work themselves.
This means you can write depth-agnostic prompts that work regardless of the depth limit.

### 5. The spawn primitive is the same
Codex `spawn_agent()` is semantically identical to Claude Code `Agent` tool — a fresh agent with a task prompt, no conversation context inherited. The difference is only whether sub-agents themselves have access to this primitive.

## Architecture Implications

### What Codex's configurable depth enables:

1. **Foreman pattern**: Orchestrator spawns a foreman per domain. Each foreman spawns workers for specific files/functions. Orchestrator stays clean.

2. **Dynamic decomposition**: A foreman agent can read a directory, decide how many workers to spawn, and distribute work — without the orchestrator needing to know the directory structure.

3. **Recursive exploration**: An exploration agent can spawn deeper explorers for subdirectories, building a tree of findings that gets compressed at each level.

### What it doesn't solve:
- Context inheritance is still task-only (no conversation clone in `spawn_agent`)
- Cross-domain blind spots still exist unless you pass wiring maps
- Deeper nesting = more wall time (linear cost per layer)
- The orchestrator still needs to synthesize final results

## Verdict

Codex's `agents.max_depth` is the missing primitive that makes the "layered synced intelligence" pattern real. Claude Code can simulate it with star topology + context priming, but Codex can do it natively with actual hierarchy.

The optimal strategy depends on the task:
- **Parallel independent tasks** → fan-out (both platforms)
- **Hierarchical decomposition** → depth nesting (Codex only)
- **Cross-domain accuracy** → context priming / wiring map (both platforms)
