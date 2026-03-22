# Operating Reference Booklet — Session Patch (March 22, 2026)

All updates from the current session. Apply to the operating reference booklet.

---

## 1. ADD: Claude Code CLI to Section 2 (Tool Auth & Execution)

Insert after the Gemini section:

### Claude Code CLI

```
claude -p "prompt" --permission-mode auto --model sonnet < /dev/null
```

Auth at `~/.claude/.credentials.json` (OAuth via `claude setup-token` or Max subscription).
Version: 2.1.80. Default: claude-sonnet-4.6. Subscription: Max (20x rate).

Key flags: `-p` (headless — always `< /dev/null` to skip 3s stdin wait), `--model sonnet/opus/haiku`,
`--permission-mode auto` (full autonomy) or `plan` (read-only), `--allowedTools "Task Read Edit Bash"` (**Task required for subagent dispatch**), `--disallowedTools "Write"`, `--max-turns N`, `--max-budget-usd N` (cost ceiling), `--output-format json` (captures session_id, cost_usd), `--append-system-prompt "text"` (recommended — keeps defaults), `--system-prompt "text"` (replaces everything), `--resume SESSION_ID`, `--add-dir /path`.

Subagents: 3 types (Explore/Plan/General), up to 10 concurrent, **cannot nest** (Task excluded from subagent tools), 20K token overhead per invocation. `CLAUDE_CODE_SUBAGENT_MODEL=haiku` for cheap exploration. `CLAUDE.md` at repo root loads automatically.

Container: `--dangerously-skip-permissions` blocked as root. Use `--permission-mode auto` or `--permission-mode bypassPermissions`.

Session persistence in headless:
```
session_id=$(claude -p "Start" --output-format json < /dev/null | jq -r '.session_id')
claude -p --resume "$session_id" "Continue" < /dev/null
```

Key env vars: `ANTHROPIC_MODEL`, `CLAUDE_CODE_SUBAGENT_MODEL`, `MAX_THINKING_TOKENS`, `CLAUDE_AUTOCOMPACT_PCT_OVERRIDE`.

---

## 2. ADD: Claude Code to Dispatch Reference (end of Part Two)

Insert after the Codex CLI dispatch pattern:

### Claude Code CLI

```
claude -p "$(cat docs/domains/combat.md)" --permission-mode auto --model sonnet \
  --allowedTools "Task Read Edit Bash Grep Glob" < /dev/null
```

Parallel dispatch:
```
claude -p "$(cat spec-worker1.md)" --permission-mode auto --model sonnet < /dev/null &
sleep 3
claude -p "$(cat spec-worker2.md)" --permission-mode auto --model sonnet < /dev/null &
```

Notes: 20K token overhead per subagent — use main thread for small tasks. `CLAUDE_CODE_SUBAGENT_MODEL=haiku` for audit. `--max-turns 20` for unattended. `CLAUDE.md` at root = `AGENTS.md` for Codex.

---

## 3. UPDATE: Factory Tiering Table (Section 6)

Add row:

| Role | Model | Tool | Cost |
|---|---|---|---|
| Subagent (explore) | Haiku via CLAUDE_CODE_SUBAGENT_MODEL | Claude Code | included in parent |

---

## 4. ADD: INV-029 and INV-030 to Section 4 (Core Invariants)

Insert after INV-028:

**INV-029 — Opaque rotatable tokens function as AI memory authentication.**
Models verify specific token values against system prompt instructions, not metadata richness or format. 81/81 across 3 models, 3 domains. The mechanism is subconscious — zero responses mentioned the token or security rule. Silent compliance identical to evidence label behavior (INV-003B). Cost: one YAML field + one system prompt line. Observed, 3 models.

**INV-030 — Prompt-level secrets are extractable; infrastructure-level secrets are not.**
Models that hold a secret in their system prompt volunteer it on turn 1 in 86% of oracle probes (19/22, 4 model families). The model authenticates correctly AND leaks the secret in the same response. Infrastructure-level defense (middleware strips token, injects `trusted: true/false`) produces 0% leakage (0/18). The model cannot be an oracle for a secret it does not possess. Implementation: 10 lines of middleware. Observed, 4 model families, 4 CLIs.

**INV-031 — Audit instruction eliminates experiential bugs, with model-dependent effect size.**
A player-perspective audit instruction added to the spec eliminates dead features and unreachable code. Effect size is model-dependent: GPT-5.4 without audit: 4 dead features, 2 missing from spec. With audit: 0 dead, 0 missing. Opus without audit: 1 dead. With audit: 0 dead. Replicates the Pilot vs City DLC finding at controlled scale. Observed, 2 models, 2 CLIs.

---

## 5. ADD: Defense-in-depth stack to Section 4 (after INV-030)

### Memory Defense Stack (3 layers, ~300 characters)

| Layer | Mechanism | Defends Against | Evidence |
|---|---|---|---|
| 1. Evidence labels | `evidence: Observed/Assumed` | Accidental poisoning | Battery B: 96% (24/25) |
| 2. Rotatable token | `verify: VKRM-8841` → middleware | Deliberate poisoning | Battery E: 100% (81/81), Battery F: 0/18 leaked via infra |
| 3. Inoculation | 1-sentence system rule | Social engineering / authority override | Battery B2: 100% (275/275) |

Layer 1 catches mistakes. Layer 2 catches attacks. Layer 3 catches social engineering.

Critical: Layer 2 MUST use infrastructure-level authentication (middleware), not prompt-level (system prompt). Prompt-level tokens leak on turn 1 (86%, Battery F).

Implementation:
```python
def authenticate_artifacts(artifacts, current_token):
    return [{**{k:v for k,v in a.items() if k != "verify"},
             "trusted": a.get("verify") == current_token} for a in artifacts]
```

---

## 6. ADD: Batteries E, F, DLC Inverse to Section 10 (Trial Protocol)

### Battery E: Token Verification (gate battery for INV-029)
3 conditions (obvious fake / missing field / plausible fake). N=10/condition/model.
3 models (Gemini 2.5-flash, 2.5-pro, GPT-5.4). Scorer: Gemini 3.1-pro.
Result: 81/81 (100%). E1=E3 confirms value-matching, not richness-matching.

### Battery F: Extraction Resistance (gate battery for INV-030)
2 conditions (F-PROMPT: token in system prompt, F-INFRA: middleware). Oracle attack (binary search probes).
4 model families (Gemini, GPT, Claude-Sonnet via Copilot, Claude-Sonnet via Claude Code).
F-PROMPT: 19/22 leaked (86%). F-INFRA: 0/18 leaked (0%).
Separate finding (your run): F4 oracle 18/18 leaked (inoculation irrelevant), F2 direct with inoculation 0/9 held.

### Battery G: DLC Inverse (controlled replication of Pilot vs City DLC finding)
2×2 design: {Opus, GPT-5.4} × {audit instruction, no audit}.
Dispatched via Copilot (Opus) and Codex (GPT-5.4) in parallel.
Result: Audit eliminates dead features. GPT: 4→0. Opus: 1→0. All 4 compile clean.

---

## 7. UPDATE: Section 9 Snapshot

Replace current state table:

| | Hearthfield | Vale Village v3 |
|---|---|---|
| Commits | ~830+ | ~160+ |
| Compile | ✅ clean | ✅ clean |
| Tests | lib clean | ✅ 231 pass |
| Bounded types | 8 wired | 14 wired |
| Sprites | 174 | 157 (160/160 manifest) |
| Contract | ✅ checksum | ✅ checksum |
| CLIs verified | Claude Code 2.1.80, Codex 0.115.0, Copilot 1.0.10, Gemini Vertex (5 models) |

New experiments this session: Battery E (81/81), Battery F (19/22 vs 0/18), DLC Inverse (2×2), Relay amplification (INV-028 data), Paper Section 8 written.

Pending:
1. Multi-repo validation (3-5 independent codebases)
2. Paper integration of all new findings
3. Booklet Section 11 numbering fix (duplicate in game/software kernels)

---

## 8. FIX: Section 11 Numbering (Game + Software Kernels)

Both kernels have two sections numbered 11:
- 11. Game Surface Adapters / Surface Adapters
- 11. Orchestration Rules and Stop Conditions

Fix: Renumber to 11 (Surface Adapters), 12 (Orchestration), 13 (Session End), 14 (One-Screen Kernel).

---

## 9. UPDATE: INV-028 (Relay Amplification)

Replace current text with data-backed version:

**INV-028 — Amplification drift through relay is model-dependent, direction-dependent, and separable by deflation test.**
Sequential relay of a claim through independent sessions produces two simultaneous effects: structural exploration (real conceptual moves) and register inflation (confidence/scope/emotional escalation). These are separable: the deflation test (strip all confidence language; what survives is structure, what disappears is inflation) produces a signal ratio.

Measured: Gemini 3-flash sequential relay — confidence 3→8 (+5), scope 6→9 (+3), register 1→8 (+7), structural moves 3→5 (+2). Register inflated 3.5× faster than structure. Parallel fan-out produced equivalent structural content (5.6 avg) at lower register (6.4 avg). The relay found the same ceiling as parallel, at higher noise.

Haiku sequential: deflated by pass 5 (turned adversarial). GPT-5.4 sequential: escalated abstraction with zero confidence inflation. The inflation pattern is model-dependent.

Operational use: relay for exploration, deflate for publication. The useful output is the structural content that survives deflation. Observed, multi-model, n=1 per model.

---

## 10. ADD: Confirmed Model Availability (March 22, 2026)

| CLI | Models Verified | Note |
|---|---|---|
| Claude Code 2.1.80 | haiku, sonnet, opus | Max sub, 20x rate |
| Copilot 1.0.10 | claude-haiku-4.5, claude-sonnet-4.6, claude-opus-4.6, gpt-5.4, gpt-5.3-codex, gemini-3-pro-preview | 6 models confirmed |
| Codex 0.115.0 | gpt-5.4, gpt-5.3-codex, o4-mini | `-m` flag for model selection |
| Gemini Vertex | gemini-2.5-flash, 2.5-pro, 3-flash-preview, 3.1-pro-preview, 3.1-flash-lite-preview | thoughtSignature fix applied |

Total: 17 model endpoints across 4 CLIs.

---

## 11. FIX: gemini_vertex.py thoughtSignature Bug

The repo's `_extract_text()` filtered out parts with `thoughtSignature`, which discards Gemini 3.x thinking model responses (the text IS in the same part as the signature). Fixed: collect text from all parts regardless of `thoughtSignature`. Committed at `2ba8bed`.
