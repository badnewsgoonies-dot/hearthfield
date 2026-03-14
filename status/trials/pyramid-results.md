# Pyramid Trial Results — 2026-03-13

## Topology Attempted

```
Claude Code (root/synthesizer)
├── Codex session 1: Foreman-Animals → 3 workers
├── Codex session 2: Foreman-Calendar → 3 workers
└── Codex session 3: Foreman-Farming → 3 workers
```

True 1→3→9 pyramid. 13 agents total. Each Codex session = fresh thread budget.

## Prior Attempts (same session)

| Attempt | Topology | Failure | Root tokens |
|---------|----------|---------|-------------|
| V1 | 1 Codex root → 3 foremen (parallel) → 9 workers | `agent thread limit reached (max 6)` — 13 agents in one session | 12,064 |
| V2 | 1 Codex root → 3 foremen (sequential) → 9 workers | Cumulative thread limit — completed agents still count toward cap | 20,358 |

## V3 Results (3 separate sessions)

| Foreman | Tokens | Wall time | Workers spawned | Workers completed work | Files changed |
|---------|--------|-----------|-----------------|----------------------|---------------|
| Animals | 22,540 | ~1m00s | 3 (all parallel) | 0 | 0 |
| Calendar | 21,800 | ~2m16s | 3 (all parallel) | 0 | 0 |
| Farming | 15,519 | ~3m50s | 3 (attempted, partial) | 0 | 0 |
| **Total** | **59,859** | **~3m50s** (parallel) | **9 attempted** | **0** | **0** |

## Failure Modes

### 1. LandlockRestrict sandbox on sub-agents
Workers spawned by foremen could not execute shell commands to read files. The Codex `danger-full-access` sandbox mode works for the root/foreman level but sub-agents at depth-2 inherit a more restrictive sandbox (`LandlockRestrict`).

**Impact:** Workers could not read source files → could not make informed edits → refused to make blind edits (correct behavior).

### 2. Session thread limit (max 6) is cumulative, not concurrent
Even after a spawned agent completes and returns, it still counts toward the per-session cap of 6 agent threads. This means:
- Foreman(1) + explorer(1) + 3 workers = 5 → works
- But if foreman retries or spawns any extra agents: cap hit

**Impact:** Farming foreman hit this after spawning an explorer + attempting workers.

### 3. Foreman noncompliance (V2 only)
First foreman in V2 tried to execute the entire pyramid itself instead of being a foreman. Consumed thread slots for agents that did the wrong work.

## Token Economics (what we CAN measure)

| Layer | Agent | Tokens | What it processed |
|-------|-------|--------|-------------------|
| Root (V1) | Codex root | 12,064 | 3 foreman error summaries |
| Root (V2) | Codex root | 20,358 | 1 noncompliant foreman + 1 retry |
| Root (V3) | Claude Code | ~0 (just wait + grep) | 3 foreman reports |
| Foreman | Animals | 22,540 | Spec read attempt + 3 worker error reports |
| Foreman | Calendar | 21,800 | Spec read attempt + 3 worker error reports |
| Foreman | Farming | 15,519 | Spec read attempt + partial worker dispatch |
| Workers | All 9 | Unknown (not reported) | Blocked before meaningful work |

### Compression theory status: PARTIALLY CONFIRMED

The V1 root (12,064 tokens) processed 3 foreman summaries — confirming that root cost drops when it only sees summaries, not source. But we cannot measure the full pyramid cost because workers were blocked before producing work output.

**Estimated full pyramid cost if workers had succeeded:**
- 9 workers × ~5k tokens each (single-file read + edit) ≈ 45k
- 3 foremen × ~15k each (spec read + 3 worker summaries) ≈ 45k
- 1 root × ~12k (3 foreman summaries) = 12k
- **Total estimate: ~102k tokens**
- Compare: flat single agent reading all 19 files = ~59k tokens

The pyramid would be MORE expensive total, but the root sees only 12k. The cost savings are an illusion at the system level — the compression saves root context but multiplies total tokens through overhead at every layer.

## Platform Limits Discovered

| Limit | Value | Source |
|-------|-------|--------|
| Session agent thread cap | 6 | `agent thread limit reached (max 6)` |
| Thread counting | Cumulative (not concurrent) | Completed agents still count |
| Sub-agent sandbox | More restrictive than parent | `Sandbox(LandlockRestrict)` at depth-2 |
| WebSocket transport | 403 Forbidden (falls back to HTTPS) | Every session, ~10s startup cost |

## Key Finding

**The pyramid topology does NOT scale cost DOWN in total.** It scales root-level cost down by pushing work to leaves, but total system cost is HIGHER due to:

1. Per-agent overhead (~3-5k tokens for prompt parsing, tool discovery, sandbox negotiation)
2. Error reporting overhead (each blocked worker still costs tokens to report its failure)
3. Foreman overhead (reading spec + managing wait loops)

**Where it WOULD help (if platform limits were removed):**
- Root context stays small → root can orchestrate more domains
- Leaf workers can run in parallel → wall time drops
- Each worker only loads 1 file → no cross-file context pollution

**But Codex v0.114.0 cannot execute this topology** because of the cumulative thread cap (6) and sub-agent sandbox restrictions.

## Comparison: What Claude Code CAN do

Claude Code has a structural depth-1 cap (Agent tool removed from sub-agents). But:
- Sub-agents CAN read files (no sandbox restriction)
- Sub-agents CAN modify files
- No agent thread limit observed in practice
- Worktree isolation available for parallel workers

The irony: Claude Code's depth-1 cap prevents the pyramid, but its agents actually work. Codex's configurable depth enables the pyramid, but its platform restrictions prevent execution.

## Bugs fixed: 0

The pyramid produced zero shipped code. All 9 intended fixes remain pending.
