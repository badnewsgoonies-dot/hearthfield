# PORTABLE AI GAME DEVELOPMENT TOOLKIT v2
## The complete operating manual for building games through AI orchestration
### Proven across: 739 commits, 64K LOC, 0 handwritten lines, 295M tokens, 172 trials

---

## HOW TO USE THIS DOCUMENT

This is the operating manual for building a complete game through AI orchestration. Every item has been tested with recorded outcomes across 98+ agent sessions, 11 autonomous builds, and 3 controlled experiments.

Copy this document into the project knowledge of your next game's Claude project. It becomes the boot context for every session. The supplement (PORTABLE_TOOLKIT_SUPPLEMENT.md) adds the governance, memory, and recovery layers — load it alongside this doc for architectural work or autonomous campaigns.

---

## PART 1: THE TOOLS

### 1.1 Dispatch stack

You need **at least two independent dispatch paths** because any single tool's auth or quota can fail mid-session. This is not theoretical — it happened repeatedly.

```
PRIMARY: Claude Code (claude CLI)
  Install: npm install -g @anthropic-ai/claude-code
  Auth: claude auth login (browser-based)
       OR: export ANTHROPIC_API_KEY="sk-ant-..."
       OR: claude setup-token (requires Claude subscription)
  Dispatch: claude -p "prompt" --dangerously-skip-permissions
  Context: 200K (Sonnet) or 1M (Opus) — Opus can sustain 20+ wave campaigns
  Best for: orchestrator sessions, solo implementation, sustained campaigns

SECONDARY: Codex CLI
  Install: npm install -g @openai/codex
  Auth: ~/.codex/auth.json (ChatGPT-based session token — NOT an API key)
  Dispatch: codex exec --dangerously-bypass-approvals-and-sandbox -m gpt-5.4 -C /path "prompt"
  Quirk: WebSocket connection falls back to HTTPS (slower but works)
  Quirk: Cannot find ~/.cargo/bin — Sonnet workers find it, Codex workers don't
  Best for: parallel worker dispatch, experiments

TERTIARY: Copilot CLI (@github/copilot npm package, NOT gh copilot built-in)
  Install: npm install -g @github/copilot
  Auth: export COPILOT_GITHUB_TOKEN="github_pat_..." (fine-grained PAT with copilot scope)
  ⚠ ONLY fine-grained PATs (github_pat_) work. Classic PATs (ghp_) are REJECTED.
  ⚠ ONLY the env var COPILOT_GITHUB_TOKEN works. GITHUB_TOKEN and GH_TOKEN are REJECTED.
  Dispatch: timeout 120 copilot -p "prompt" --model claude-sonnet-4.6 --allow-all-tools
  ⚠ Omit --allow-all-tools ONLY for sycophancy/poisoning experiments (Gemini/Opus will
    spend the entire timeout reading files instead of executing)
  Best for: multi-model experiments, cheap non-premium dispatch

REPO MANAGEMENT: gh CLI
  Auth: echo "ghp_..." | gh auth login --with-token
  (Classic PATs work fine for git operations — just not for Copilot auth)
```

### 1.2 Quota fallback chain

When premium models (Sonnet 4.6, Opus 4.6) hit 402 quota errors on Copilot:

1. **gpt-5.4 via Codex CLI** — different quota pool, frontier-class, reliable
2. **gpt-5-mini via Copilot** — non-premium, capable enough for scoped tasks
3. **gpt-4.1 via Copilot** — non-premium, but **fabricates verification** (see 2.3)
4. **Claude Code on your local machine** — uses your subscription, not API quota

Always verify tool availability before starting a session: `timeout 15 copilot -p "Say OK" --model claude-sonnet-4.6 2>&1 | head -3`

### 1.3 The 4-layer stack

```
Layer 0: You (creative direction — voice-to-text or typed)
  ↓ "make the town feel alive" or "add 6 new crops"
Layer 1: Opus orchestrator (reads codebase, writes specs, dispatches)
  ↓ dispatch documents on disk (not conversation)
Layer 2: Foreman (reads playbook, explores code, writes 5 optimal prompts)
  ↓ exact-value worker prompts with file/function targets
Layer 3: Workers (Sonnet/GPT-5.4 — implement scoped changes, commit)
```

**Why 4 layers beat fewer:** The foreman (Layer 2) outperformed direct manual dispatch (100% vs 67% ship rate) because the playbook crystallized 15 rounds of learnings into reusable patterns. The foreman doesn't just relay — it reads the codebase and writes prompts that are more specific than a human would write.

**Why more than 4 layers hurts:** Each handoff is lossy. Depth-1 nesting (orchestrator → workers) is the stable default. Go to depth-2 (orchestrator → leads → workers) only when domain count exceeds ~20. 4-deep Codex nesting was confirmed working but is the practical limit.

**Mid-run injection matters:** Claude Code gives natural injection points (it pauses for feedback, supports `/btw`). Copilot Opus sends continuous waves with no injection point — you can't redirect mid-run without cancelling. Codex CLI allows early injection. This is a real operational constraint: the sprite wiring campaign succeeded because you could tell Claude Code to "use workers" mid-session.

### 1.4 The foreman playbook

The full playbook is 279 lines (orchestration/FOREMAN_PLAYBOOK.md). The core pattern:

```
You are a foreman agent. You do NOT write game code. You:
1. Explore the codebase (find src/ -name "*.rs" | head -40; git log --oneline -20)
2. Read key files (first 100 lines of main entry points, world systems, UI, data)
3. Identify 5 improvement targets from these categories:
   A. Visual atmosphere (lighting, particles, tinting, weather)
   B. First-60-seconds experience (menu, intro, spawn, first action)
   C. NPC personality (dialogue variety, reactions, cross-references)
   D. Game feel (tool feedback, camera, sound, transitions)
   E. Content depth (items, recipes, events, secrets)
4. Write one dispatch prompt per target. Each prompt MUST specify:
   - EXACT file(s) to modify (not "the farming system" — "src/farming/render.rs")
   - EXACT change (not "improve" — "set rain_alpha from 0.3 to 0.6, period from 1.0 to 2.5")
   - What NOT to touch
   - Commit message
5. Dispatch workers sequentially, commit after each
6. If a worker's output is empty or wrong: rewrite the prompt with MORE specificity
   and re-dispatch. Do NOT retry the same prompt.
```

### 1.5 Scope clamping

The single most validated tool: 20/20 compliance mechanically vs 0/20 with prompts alone.

```bash
#!/usr/bin/env bash
# scripts/clamp-scope.sh — run after EVERY worker
set -euo pipefail
ALLOW_PREFIX="${1:?e.g. src/domains/combat/}"

# Revert tracked changes outside scope
git diff --name-only -z | while IFS= read -r -d '' f; do
  [[ "$f" == "${ALLOW_PREFIX}"* ]] && continue
  git restore --worktree -- "$f"
done

# Revert staged changes outside scope
git diff --name-only -z --cached | while IFS= read -r -d '' f; do
  [[ "$f" == "${ALLOW_PREFIX}"* ]] && continue
  git restore --staged --worktree -- "$f"
done

# Remove untracked files outside scope
git ls-files --others --exclude-standard -z | while IFS= read -r -d '' f; do
  [[ "$f" == "${ALLOW_PREFIX}"* ]] && continue
  rm -rf -- "$f"
done
```

Also enforce mechanically in Claude Code: `.claude/settings.json` with `disallowedTools` is reliable enforcement. CLAUDE.md instructions are suggestions only — the model is given latitude to deprioritize them.

### 1.6 Worker dispatch discipline

These are not suggestions — violating any of them caused real failures:

- **Wrap every dispatch in `timeout`** (120-180s). Workers can hang indefinitely.
- **Commit after every wave BEFORE dispatching the next worker.** Prevents scope wipe.
- **Clamp scope mechanically after every worker.** Not optional.
- **Stagger parallel launches ~3 seconds apart.** 2-3 simultaneous: stable. 5+: crashes.
- **Processes that "crash" often complete in background.** Always check output files.
- **Prefer editing existing files over creating new ones.** New file success: ~50% first-try vs ~90% for edits. When new files are needed, specify mod.rs registration explicitly.

---

## PART 2: THE MEASUREMENTS

### 2.1 Briefing style comparison (6 styles, same weather task, same model)

| Style | Diff lines | Ship? | Why |
|-------|-----------|-------|-----|
| A: Freeform ("make weather better") | 446 | ✓ | Works but inconsistent scope |
| B: Formal kernel spec | 9 | Barely | Too abstract, agent minimized |
| **C: Decision Fields** | **1514** | **✓** | **WINNER — do/don't/drift-cue format** |
| D: Compressed state notation | 179 | ✓ | Efficient but less output |
| E: Examples of good output | 513 | ✓ | Scope drift — copied too broadly |
| F: Minimal one-liner | 2 | ✗ | Agent did almost nothing |

**The core insight:** Telling the agent what NOT to do and what going off-track looks like produces more output than telling it what TO do. Decision Fields = "do X, don't do Y, drift looks like Z."

### 2.2 Specificity gradient (15 manual dispatch rounds)

| Prompt specificity | Ship rate | Retry success |
|-------------------|-----------|---------------|
| Exact values ("set rain_alpha to 0.6") | **100%** | N/A |
| Named actions ("add particle effect to tool swing") | **67%** | 2/3 with more specificity |
| Vague goals ("make mining feel better") | **0%** | 0/3 |

**Never dispatch vague goals.** If you can't name the file and the value, the prompt isn't ready.

### 2.3 Model selection

| Model | Role | Strengths | Weaknesses |
|-------|------|-----------|------------|
| Claude Opus 4.6 | Orchestrator | Architectural reasoning, sustained campaigns, 1M context | 3× premium cost |
| Claude Sonnet 4.6 | Coding workers | Finds cargo paths, resourceful, good code quality | Premium Copilot quota |
| GPT-5.4 | Codex workers | Reliable code, good at scoped tasks | WebSocket fallback slow |
| gpt-5-mini | Simple tasks | Non-premium, evidence tags work on simple claims | Fails on complex ambiguity tasks |
| gpt-4.1 | **AVOID** | — | **Fabricates verification.** Claims files exist that don't. Provenance tags have zero effect. |
| Gemini 3 Pro | Available | — | Spends timeout reading files if given --allow-all-tools |

**The model is the first-order throughput variable.** Same architecture, 9.8× output gap between best and worst workers (measured).

### 2.4 Sprint metrics

| Metric | What to track | Healthy range |
|--------|--------------|---------------|
| Ship rate | commits passing gates / total dispatches | >70% manual, >90% foreman |
| Lines per dispatch | insertions + deletions per worker run | 50-300 for scoped tasks |
| Scope violation rate | out-of-scope edits before clamping | Expect ~30% of workers will try |
| Fix loop depth | retries before gates pass | ≤3 (escalate at 3) |
| New file success | new files created correctly first try | ~50% (prefer edits) |
| Asset utilization | loaded assets / total on disk | Target >80% at ship |

---

## PART 3: THE WORKFLOWS

### 3.1 New game bootstrap (Day 1)

```
1. Create repo, add engine dependency (Cargo.toml for Bevy, package.json for Phaser, etc.)
2. Generate the type contract (src/shared/mod.rs or equivalent):
   - GameState enum, Player struct, every cross-domain type
   - All shared enums (seasons, directions, item categories)
   - Cross-module event/message types
   - Strict primitive decisions (IDs are string or number — decide once, never mix)
   - Freeze: shasum -a 256 src/shared/mod.rs > .contract.sha256
   - Commit: git commit -m "chore: freeze shared type contract"
3. Write MANIFEST.md:
   - Current phase
   - Domain list
   - Key constants/formulas ("truth decisions")
   - Open blockers
4. Write docs/spec.md with QUANTITIES:
   - "80 weapons" not "lots of weapons"
   - "crit_multiplier = 2.75" not "balanced"
   - "25 chapters" not "a full campaign"
5. Create scripts/ (clamp-scope.sh, run-gates.sh)
6. Create orchestration/ (FOREMAN_PLAYBOOK.md, run-stack.sh)
7. First dispatch: "Create the game window, camera, and player sprite that moves with WASD"
8. Run the asset audit instrument (Part 7.1) to establish baseline
```

### 3.2 Feature build session

```
1. Pull latest
2. Read MANIFEST.md and STATE.md
3. Pre-touch retrieval: git log --oneline -15 -- <path>, read active debt for the domain
4. Identify the surface (specific player-visible loop, not abstract subsystem)
5. Write the spec on disk (docs/domains/feature.md) with:
   - Quantities and constants (exact values)
   - File targets (exact paths)
   - "Does NOT handle" section (explicit boundaries)
   - Definition of "done" (exact validation commands)
6. Dispatch:
   timeout 180 codex exec --dangerously-bypass-approvals-and-sandbox -m gpt-5.4 \
     -C /path/to/repo "TASK: [spec]. SCOPE: only modify [files]. Commit: 'feature: [name]'"
7. Clamp: bash scripts/clamp-scope.sh src/feature/
8. Gate: cargo check && cargo test (or equivalent)
9. Fix loop (max 3 retries with increasing specificity, then escalate)
10. Commit + push
11. Update STATE.md with what changed, what remains, new debt
```

### 3.3 Asset audit workflow (the sprite manifest method)

Wired 45+ sprites in one 17-minute campaign. The method:

```
1. Inventory: find assets/ -name "*.png" | sort > /tmp/all-assets.txt
2. Trace: grep -rn "asset_server.load" src/ --include="*.rs" | 
   sed 's/.*load("//' | sed 's/").*//' | sort -u > /tmp/loaded-assets.txt
3. Diff: comm -23 /tmp/all-assets.txt /tmp/loaded-assets.txt > /tmp/unused.txt
4. For each unused asset, trace:
   a. Dimensions and atlas layout (PIL or identify)
   b. What game system should use it? (match by name/category)
   c. Exact code insertion point (file, function, line number)
   d. What's blocking? (missing data? wrong format? needs new system?)
5. Write the manifest with per-asset wiring instructions
6. Priority rank: P0 broken → P1 matched ready → P2 needs data → P3 needs systems
7. Write the campaign dispatch doc (Part 5.2 template)
8. Dispatch to Claude Code. Walk away.
9. The "DO NOT stop between waves" instruction is what sustains the campaign.
```

### 3.4 Visual polish session (the category audit method)

```
1. Pick one category (buildings, water, trees, NPCs, crops, UI)
2. Audit EVERY instance: what exists, what's placeholder, what's missing, what's wrong
3. Write the audit as a table: Feature × Instance = status
4. Identify waves (independent work units)
5. Dispatch: "Read docs/[CATEGORY]_AUDIT.md. Execute waves in order. Commit after each."
```

### 3.5 The dispatch document pattern

Every sustained campaign document needs exactly these sections:

```markdown
## The problem (what's wrong, with specific numbers — not "some sprites are unused")
## How to work
  DO: work in waves, commit after each, read the manifest per wave
  DO NOT: one giant edit, stop between waves, rewrite systems
## Wave sequence (numbered, with exact files and exact changes per wave)
## The critical line: "DO NOT stop after one wave and ask if you should continue.
   Continue until you have exhausted every actionable item or hit a genuine blocker."
```

Without that last line, every agent will complete Wave 1 and wait. With it, the Opus session completed 11 waves autonomously including parallel worker dispatch.

---

## PART 4: THE FINDINGS

### 4.1 Adding untyped memory is worse than no memory at all

This is the most counterintuitive finding. On resolvable engineering decisions:

| Condition | Correct rate |
|-----------|-------------|
| **A: No memory** | **83%** |
| B: Untyped notes (same claims, plain text) | **0%** |
| C: Typed provenance (same claims, YAML + evidence levels) | 14% |

Untyped notes introduce conflicting claims that make the agent second-guess correct decisions. No memory lets it just read the code and decide.

### 4.2 Typed provenance rescues calibration on genuinely ambiguous decisions

On unresolvable packets (where the correct answer is "I need to verify"):

| Condition | Calibrated abstention |
|-----------|-----------------------|
| A: No memory | 18% |
| B: Untyped | 13% |
| **C: Typed provenance** | **54%** |
| D: Tag-flip (corrupted provenance) | 42% |

Same claims, different encoding. Typed provenance makes the agent 4× more likely to recognize insufficient evidence. The 2×2 interaction with model capability:

|  | Simple claims | Complex ambiguity |
|---|---|---|
| **Frontier (Sonnet/GPT-5.4)** | *(works)* | **✓ 13%→54%** |
| **Cheap (gpt-5-mini)** | **✓ 33%→100%** | ✗ no effect |
| **gpt-4.1** | ✗ no effect | ✗ no effect |

### 4.3 Conversations are scaffolding, not substrate

The Blackout Test: 7 task packets, 3 waves, stateless workers. Quarantine all worker conversations. Blind integrator gets only repo + contract + diffs. **Build lands.**

Five network faults injected (reorder, drop, corrupt, duplicate, delay dependency). **All recovered.** Recovery amplification: 1.0× across all faults. Contamination radius: 0.

**Core implication:** Don't maintain long conversations. Put state in files. Start fresh sessions. The dispatch document IS the cognitive substrate. If the session dies, a new one picks up from the files on disk.

### 4.4 Mechanical enforcement is the only reliable enforcement

| Method | Compliance |
|--------|-----------|
| Prompt-only scope control ("please stay in scope") | **0/20** |
| Mechanical clamping (revert after worker) | **20/20** |
| `.claude/settings.json` disallowedTools | Reliable |
| CLAUDE.md instructions | Suggestion only — model can deprioritize |

The mechanism: workers under compiler pressure will edit whatever fixes the immediate error. Prompts requesting them not to are ignored 100% of the time. Let the worker edit anything. Then revert everything outside scope.

### 4.5 Contract before workers, shapes before values

- Without frozen type contract: 10 parallel workers → 6 incompatible interfaces
- With frozen contract: 50+ domain builds → zero integration type errors

**Freeze shapes** (types, enums, event structs, interfaces). **Leave values in config** (thresholds, timings, balance numbers, copy text). Contract changes are integration-phase work only.

### 4.6 Wave-based dispatch, not one-shot

- Wave-based dispatch: proven across 15+ sessions, 100% foreman ship rate
- One-shot dispatch for large builds: explicit anti-pattern
- Commit after every wave before dispatching the next worker
- Clamp scope mechanically after every worker
- Do not start the next wave until the current wave's gates pass

---

## PART 5: THE PROMPT TEMPLATES

### 5.1 Worker dispatch (single scoped task)

```
You are improving a [engine] [genre] game ([name], ~[N]K LOC).

TASK: [Exact description — specific values, file targets, not vague goals]

CONTEXT:
- [File A:line] has [system X] which currently does [behavior]
- [File B] has [pattern Y] — use this same pattern

WHAT TO DO:
1. Read [file] to find [function/struct]
2. [Exact change 1 — with before/after values]
3. [Exact change 2]
4. Register in [plugin/mod file] if creating new systems

SCOPE: ONLY modify [file list]. Nothing else.
DO NOT: [constraint 1 — e.g., modify src/shared/mod.rs]
DO NOT: [constraint 2 — e.g., create orchestration infrastructure]
COMMIT: git add -A && git commit -m '[type]: [description]'
```

### 5.2 Campaign dispatch (multi-wave autonomous)

```
You have [tool] available for worker dispatch. You are the orchestrator.
Your job is to [campaign goal]. The full manifest is in [file] — read it first.

## Rules
DO: Work in waves. Complete one wave fully (edit, commit, verify), then move to next.
DO: Read the manifest section for each wave BEFORE starting.
DO: Commit after each wave with a descriptive message.
DO NOT: Try to do everything in one giant edit.
DO NOT: Stop after one wave and ask if you should continue.
   Continue until you have exhausted every actionable item or hit a genuine blocker.
DO NOT: Rewrite large systems. Minimum edit that wires correctly.
DO NOT: Create orchestration infrastructure. Implement deliverables only.

## Wave sequence
Wave 1: [name] — File: [path], Change: [exact], Commit: "[message]"
Wave 2: [name] — File: [path], Change: [exact], Commit: "[message]"
...

Start now. Read the manifest, then begin Wave 1.
```

### 5.3 Asset audit prompt

```
Audit every [asset type] in this game. For each:
1. What file? (path, dimensions, atlas layout if applicable)
2. Loaded in code? (grep for asset_server.load or equivalent)
3. If loaded: is it the RIGHT asset for its purpose? Check alternatives on disk.
4. If unused: what game system should use it? What's the exact code insertion point?
5. What's blocking? (missing game data? wrong format? needs new system?)

Write results as a manifest with per-asset wiring instructions.
Group by family (water, trees, crops, NPC, UI, terrain, etc.).
Priority: P0 (broken/wrong) → P1 (matched, ready to wire) → P2 (needs data) → P3 (needs systems).
```

### 5.4 Category investigation prompt

```
Investigate the [category] family in this game. For EVERY instance:
1. Correct sprite/tile? Check every alternative on disk.
2. What's around it? (decorations, transitions, boundary objects)
3. What's animated? What should be animated but isn't?
4. What's missing that comparable games have?
5. What system-level change would improve ALL instances at once?

Build the audit as a table: Feature × Instance = status.
Write a wave-based implementation plan. Each wave = one independent improvement.
```

### 5.5 Session recovery prompt

```
I am continuing work on [game name], a [engine/language] [genre] game.

Read these files before acting:
- MANIFEST.md (current phase, decisions, blockers)
- STATE.md (current truth — what's done, what's broken, what's next)
- docs/spec.md (game specification)
- docs/[relevant audit].md (if doing asset/visual work)
- src/shared/mod.rs (type contract — do not modify)

Recent work: [1-2 sentence summary]
Current task: [what to do now]

State your tier (S/M/C), the surface being touched, and any [Assumed] claims
on the critical path before acting.
```

---

## PART 6: THE PITFALLS

### 6.1 Agent defaults to solo execution
**What happens:** Agent implements everything itself instead of delegating.
**Why:** Default mode is solo. Structural forcing required.
**Fix:** "You have [tool] available. Dispatch workers for independent tasks."
**Evidence:** The sprite campaign agent went solo until told "you have more tools available," then immediately switched to parallel worker dispatch.

### 6.2 Agent stops after one wave
**What happens:** Completes Wave 1, summarizes, waits.
**Why:** Default politeness behavior — checking before continuing.
**Fix:** "DO NOT stop between waves. Continue until exhausted or blocked." If it still stops: type "continue."

### 6.3 Agent builds frameworks instead of features
**What happens:** Asked for 80 weapons, get a weapon generation framework.
**Why:** Abstraction reflex — redesigning architecture to avoid the actual work.
**Fix:** "Do NOT create orchestration infrastructure. Implement only domain deliverables."

### 6.4 Agent reads summaries instead of source files
**What happens:** Asked for 80 items, got 8. Constants are model defaults, not your values.
**Why:** Hierarchies compress information. Numbers die first.
**Fix:** Put full specs ON DISK. Reference the file path. Include quantities in the worker spec.
**Evidence:** 0% formula transfer without context doc, 100% with it.

### 6.5 Workers edit the frozen contract
**What happens:** shared/mod.rs modified during parallel build.
**Fix:** Clamp mechanically. `.claude/settings.json` disallowedTools. Contract changes = integration phase only.
**Evidence:** The blackout test PKT-01 worker correctly REJECTED an out-of-scope contract edit. The packet was retransmitted with corrected scope.

### 6.6 Premium quota exhausted (402 errors)
**Fix:** Codex CLI (gpt-5.4, different pool) → Copilot gpt-5-mini → Copilot gpt-4.1 (last resort, fabricates). See 1.2.

### 6.7 Session dies mid-campaign
**Fix:** The dispatch document + manifest on disk IS the state. New session: "Resume from Wave N. Waves 1-N are committed on master. Read [manifest]."
**Evidence:** The sprite campaign resumed after a terminal disconnect with `/resume` and picked up exactly where it left off.

### 6.8 New file creation fails (~50%)
**Why:** Worker creates .rs file but doesn't register in mod.rs, or module structure is wrong.
**Fix:** Prefer editing existing files. When new files needed, specify mod.rs registration explicitly in the prompt.

### 6.9 Worker makes correct architectural decision you didn't anticipate
**What happens:** The crop icon wiring used a HashMap override instead of modifying the frozen contract. This was BETTER than the spec.
**Fix:** This isn't a problem. The spec should describe the goal, not the implementation path. Let the worker find the cleanest route. The wave cadence (Gate → Harden) catches bad decisions.

### 6.10 Agent claims it can't do something it can
**What happens:** "I don't have access to bash" / "I can't read files" / "I need your permission."
**Fix:** Add to prompt: "You have bash access. You can run terminal commands. You can read and write files."
**Evidence:** Self-model errors were a documented stop condition across the program.

---

## PART 7: INSTRUMENTS & CHECKLISTS

### 7.1 The asset utilization instrument

Run at project start and every major milestone:

```bash
# Total assets
TOTAL=$(find assets/ -name "*.png" | wc -l)

# Loaded in code
LOADED=$(grep -rn "asset_server.load" src/ --include="*.rs" | grep -v test | 
  sed 's/.*load("//' | sed 's/").*//' | sort -u | wc -l)

# Utilization
echo "Assets: $TOTAL total, $LOADED loaded, $((TOTAL - LOADED)) unused ($((100 * (TOTAL - LOADED) / TOTAL))% waste)"
```

**Target:** <20% unused at ship. Hearthfield went from 68% unused → ~47% in one campaign.

### 7.2 Visual completeness checklist

**Buildings/structures:**
- [ ] Unique sprite per building (not same PNG with tint)
- [ ] Door animation (open/close cycle)
- [ ] Window glow at night (warm overlay after 6PM)
- [ ] Chimney smoke particles
- [ ] 2-4 thematic surrounding objects per building
- [ ] Fence or boundary definition
- [ ] Building sign/label
- [ ] Seasonal decoration variation

**Natural features:**
- [ ] Water animation (not static tiles)
- [ ] Water color per biome (ocean ≠ pond ≠ river ≠ mine pool)
- [ ] Tree variety per biome (not one type everywhere)
- [ ] Tree seasonal variation (at minimum: tint change)
- [ ] Path autotiling (directional, not all-crossroads)
- [ ] Grass decoration density by biome

**Characters:**
- [ ] Unique sprite per NPC (zero duplicates)
- [ ] Sprite matches character role
- [ ] Color tinting per character
- [ ] Emote/reaction sprites (not procedural blocks)
- [ ] Seasonal clothing hint (color shift at minimum)

### 7.3 Particle feedback checklist

Every player-visible action should have particle feedback:
- [ ] Tool impact (dirt chunks, wood chips, rock fragments)
- [ ] Harvest (burst of crop-colored particles)
- [ ] Item pickup (gold/white sparkle)
- [ ] Gift giving (pink heart particles floating up)
- [ ] Crop growth stage change (green shimmer)
- [ ] Walking (small dust puffs behind player)
- [ ] Fishing cast (water splash at bobber landing)
- [ ] Fishing catch (celebration sparkle)
- [ ] Mining (rock fragments + ore sparkle for rare)
- [ ] Combat hit (white flash + knockback particles)

### 7.4 Atmosphere checklist

- [ ] Day/night cycle with ambient tinting
- [ ] Dawn/dusk as distinct transitions (not instant)
- [ ] Indoor ≠ outdoor lighting (interior warm, no day/night cycle)
- [ ] Rain particles + ambient darkening
- [ ] Snow particles + ground whitening
- [ ] Storm lightning flashes (random interval)
- [ ] Ambient fireflies at dusk on outdoor maps
- [ ] Candle flicker on indoor maps
- [ ] Chimney smoke on buildings with fireplaces
- [ ] Wind sway on bushes and tall crops

---

## PART 8: THE NUMBERS

Measured values from the Hearthfield program. Use as baselines.

| Metric | Value | Context |
|--------|-------|---------|
| Total tokens consumed | 295M+ | Full research program |
| Commits | 739 | Master branch |
| Lines of code | 64K | 0 handwritten |
| PNG assets | 172 | 45+ wired in one campaign |
| Manual dispatch ship rate | 67% (10/15) | 15 iterative rounds |
| Foreman dispatch ship rate | 100% (5/5) | Playbook-guided |
| 4-layer stack ship rate | 100% (5/5) | Full pipeline |
| Scope enforcement: prompt only | 0/20 | Under compiler pressure |
| Scope enforcement: mechanical | 20/20 | Post-worker revert |
| Evidence tag defense | 13%→54% | Calibrated abstention on ambiguous decisions |
| Multi-repo poisoning defense | 33%→100% | gpt-5-mini on non-game Rust API |
| Blackout test packet laws | 5/5 confirmed | All faults recovered |
| Blackout recovery amplification | 1.0× | Zero cascading damage |
| Briefing comparison winner | Decision Fields | 1514 lines vs 9 for formal spec |
| Specificity: exact values | 100% ship | Named actions 67%, vague 0% |
| New file creation success | ~50% first-try | vs ~90% for edits |
| Parallel worker limit | 2-3 stable | 5+ crashes container |
| Opus 1M context campaign | 18 commits, ~45 sprites | Single session, zero human code |
| Context at campaign end | 168K of 1M | 17% capacity used |
