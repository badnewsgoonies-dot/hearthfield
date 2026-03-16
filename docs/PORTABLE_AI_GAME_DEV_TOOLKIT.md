# PORTABLE AI GAME DEVELOPMENT TOOLKIT
## Everything you need to build the next game the same way
### Extracted from: Hearthfield (62K LOC, 728+ commits, 0 lines handwritten)

---

## HOW TO USE THIS DOCUMENT

This is the operating manual for building a complete game through AI orchestration. It contains every proven tool, measurement, prompt pattern, and workflow from the Hearthfield program. Nothing here is theoretical — every item has been tested with recorded outcomes.

Copy this document into the project knowledge of your next game's Claude project. It becomes the boot context for every session.

---

## PART 1: THE TOOLS (what you actually run)

### 1.1 Dispatch stack (proven 5/5 ship rate)

You need these installed on your machine:

```
Claude Code (claude CLI) — your primary orchestrator + implementer
  Install: npm install -g @anthropic-ai/claude-code
  Auth: claude auth login (browser-based)
  Dispatch: claude -p "prompt" --dangerously-skip-permissions
  
Codex CLI — secondary worker, good for parallel dispatch
  Install: npm install -g @openai/codex
  Auth: ~/.codex/auth.json (ChatGPT token, not API key)
  Dispatch: codex exec --dangerously-bypass-approvals-and-sandbox -m gpt-5.4 -C /path "prompt"

Copilot CLI — tertiary worker, cheapest option
  Install: npm install -g @github/copilot
  Auth: export COPILOT_GITHUB_TOKEN="github_pat_..." (fine-grained PAT)
  Dispatch: copilot -p "prompt" --model claude-sonnet-4.6 --allow-all-tools

gh CLI — repo management
  Auth: echo "ghp_..." | gh auth login --with-token
```

### 1.2 The 4-layer stack (proven architecture)

```
Layer 0: You (creative direction, "make the town feel alive")
  ↓ voice-to-text or typed direction
Layer 1: Claude Opus (orchestrator — reads codebase, writes specs, dispatches)
  ↓ dispatch documents on disk
Layer 2: Foreman (reads playbook, explores code, writes 5 optimal prompts)
  ↓ worker prompts
Layer 3: Workers (Sonnet/GPT-5.4 — implement scoped changes, commit)
```

The run-stack.sh pattern:
```bash
#!/usr/bin/env bash
# Layer 1: Opus orchestrator writes specs
claude -p "Read the codebase. Based on this direction: '$1', write 5 specific improvement specs to /tmp/specs/" --dangerously-skip-permissions

# Layer 2: Foreman reads specs and dispatches workers  
claude -p "Read /tmp/specs/. For each spec, dispatch a codex worker. Commit results." --dangerously-skip-permissions
```

### 1.3 The foreman playbook (proven 5/5, outperformed manual dispatch)

```markdown
# Foreman Playbook

You are a foreman agent. You do NOT write game code. You:
1. Explore the codebase (find src/ -name "*.rs" | head -40; git log --oneline -20)
2. Read key files (first 100 lines of main.rs, world/lighting.rs, ui/hud.rs, etc.)
3. Identify 5 improvement targets from these categories:
   A. Visual atmosphere (lighting, particles, tinting, weather)
   B. First-60-seconds experience (menu, intro, spawn, first action)
   C. NPC personality (dialogue variety, reactions, cross-references)
   D. Game feel (tool feedback, camera, sound, transitions)
   E. Content depth (items, recipes, events, secrets)
4. Write one dispatch prompt per target
5. Each prompt must specify:
   - EXACT file(s) to modify
   - EXACT change to make (not "improve" — specific values, functions, patterns)
   - What NOT to touch
   - Commit message
6. Dispatch workers sequentially, commit after each
```

### 1.4 Scope clamping (20/20 compliance vs 0/20 for prompts alone)

```bash
#!/usr/bin/env bash
# scripts/clamp-scope.sh
set -euo pipefail
ALLOW_PREFIX="${1:?e.g. src/domains/combat/}"

git diff --name-only -z | while IFS= read -r -d '' f; do
  [[ "$f" == "${ALLOW_PREFIX}"* ]] && continue
  git restore --worktree -- "$f"
done

git diff --name-only -z --cached | while IFS= read -r -d '' f; do
  [[ "$f" == "${ALLOW_PREFIX}"* ]] && continue
  git restore --staged --worktree -- "$f"
done

git ls-files --others --exclude-standard -z | while IFS= read -r -d '' f; do
  [[ "$f" == "${ALLOW_PREFIX}"* ]] && continue
  rm -rf -- "$f"
done
```

Run after EVERY worker: `bash scripts/clamp-scope.sh src/world/`

---

## PART 2: THE MEASUREMENTS (what you actually check)

### 2.1 Briefing style comparison (tested 6 styles, same task)

| Style | Result | Ship? |
|-------|--------|-------|
| A: Freeform ("make weather better") | 446 lines | ✓ |
| B: Formal kernel spec | 9 lines | Barely |
| **C: Decision Fields (do/don't/drift cue)** | **1514 lines** | **✓ WINNER** |
| D: Compressed state notation | 179 lines | ✓ |
| E: Examples of good output | 513 lines | ✓ (scope drift) |
| F: Minimal one-liner | 2 lines | ✗ |

**Use Decision Fields.** Tell the agent what to do, what NOT to do, and what going off-track looks like.

### 2.2 Specificity gradient (15 manual rounds)

| Prompt specificity | Ship rate |
|-------------------|-----------|
| Exact values ("set rain alpha to 0.6, period 2.5s") | 100% |
| Named actions ("add particle effect to tool swing") | 67% |
| Vague goals ("make mining feel better") | 0% |

**Never dispatch vague goals.** Always name exact files, exact functions, exact values.

### 2.3 Worker model comparison

| Model | Best for | Quirks |
|-------|---------|--------|
| Claude Sonnet 4.6 | Coding workers (finds cargo, resourceful) | Premium Copilot quota |
| Claude Opus 4.6 | Orchestrators (architectural reasoning) | 3× premium cost |
| GPT-5.4 | Codex workers (good code, reliable) | WebSocket fallback slow |
| gpt-5-mini | Simple tasks, experiments | Non-premium, capable enough |
| gpt-4.1 | AVOID — fabricates verification | Claims files exist that don't |

### 2.4 Parallel worker limits

- 2-3 simultaneous Copilot processes: stable
- 5+ simultaneous: crashes the container
- Processes that "crash" often complete in background — always check output files
- Stagger launches ~3 seconds apart

### 2.5 The sprint metrics that matter

| Metric | What to track |
|--------|--------------|
| Ship rate | commits that pass gates / total dispatches |
| Lines per dispatch | insertions + deletions per worker run |
| Scope violation rate | out-of-scope edits before clamping |
| Fix loop depth | how many retries before gates pass |
| New file success rate | ~50% first-try (vs ~90% for edits) |

---

## PART 3: THE WORKFLOWS (what you actually do each session)

### 3.1 New game bootstrap (Day 1)

```
1. Create repo: git init, Cargo.toml with Bevy dependency
2. Generate the type contract (src/shared/mod.rs):
   - GameState enum, Player struct, all cross-domain types
   - Freeze: shasum -a 256 src/shared/mod.rs > .contract.sha256
3. Write MANIFEST.md (current phase, domain list, key decisions)
4. Write docs/spec.md (full game spec with QUANTITIES not vague goals)
5. Create scripts/ (clamp-scope.sh, run-gates.sh)
6. Create orchestration/ (FOREMAN_PLAYBOOK.md, run-stack.sh)
7. First dispatch: "Create the game window, camera, and player sprite that moves with WASD"
```

### 3.2 Feature build session

```
1. Pull latest
2. Read MANIFEST.md and STATE.md
3. Identify the surface to build (specific, not vague)
4. Write the spec on disk (docs/domains/feature.md) with:
   - Quantities (80 items, not "lots")
   - Constants (crit_mult = 2.75, not "balanced")
   - File targets (src/fishing/cast.rs, not "the fishing system")
5. Dispatch via foreman or direct:
   codex exec --dangerously-bypass-approvals-and-sandbox -m gpt-5.4 \
     -C /path/to/repo "TASK: [exact spec]. SCOPE: only modify [files]. Commit: 'feature: [name]'"
6. Clamp scope
7. Run gates (cargo check, cargo test)
8. Fix loop if needed (max 3 retries with more specific prompts)
9. Push
```

### 3.3 Asset audit workflow (the sprite manifest method)

This is how the Hearthfield session wired 35+ sprites in one campaign:

```
1. Inventory all assets: find assets/ -name "*.png" | sort
2. Trace code usage: grep -rn "asset_server.load" src/ | sed 's/.*load("//' | sed 's/").*//' | sort -u
3. Diff: which files on disk aren't loaded in code?
4. For each unused asset:
   a. What is it? (dimensions, content, atlas layout)
   b. What game system should use it? (trace by name/category)
   c. What's the exact code insertion point? (file, function, line)
   d. What's blocking it? (missing game data? wrong format? needs new system?)
5. Write the manifest with per-asset wiring instructions
6. Dispatch the campaign document to Claude Code
7. Walk away
```

### 3.4 Visual polish session (the category audit method)

```
1. Pick a category (buildings, water, trees, NPCs, crops, UI)
2. Audit: what exists, what's placeholder, what's missing
3. Write the audit doc with a table: Feature × Building/Location = status
4. Identify the waves (independent work units)
5. Dispatch: "Read docs/AUDIT.md. Execute waves in order. Commit after each."
```

### 3.5 The dispatch document pattern (proven to sustain multi-wave campaigns)

Every dispatch document needs these sections:

```markdown
## The problem (what's wrong, with numbers)
## How to work (DO/DO NOT rules)
## Wave sequence (numbered, with exact files and changes per wave)
## "DO NOT stop between waves. Continue until exhausted or blocked."
```

That last line is critical. Without it, the agent completes one wave and waits.

---

## PART 4: THE EXPERIMENTAL FINDINGS (what actually works and why)

### 4.1 Evidence tags change model behavior categorically

- Untyped notes: 0% → worse than no memory
- Typed provenance ([Observed]/[Assumed] + source_refs): 13% → 54% calibrated abstention
- This works on frontier models (Sonnet, GPT-5.4) for complex tasks
- This works on cheap models (gpt-5-mini) for simple claim evaluation
- This does NOT work on gpt-4.1 (fabricates verification regardless)

### 4.2 Conversations are scaffolding, not substrate

The Blackout Test proved: build a feature with 7 stateless workers, quarantine all worker conversations, give a blind integrator only the repo + contract + diffs. Build lands. All 5 network fault injections (reorder, drop, corrupt, duplicate, delay) recovered cleanly.

**Implication for your workflow:** don't try to maintain one long conversation. Put state in files. Start fresh sessions. The dispatch document IS the cognitive substrate.

### 4.3 Mechanical enforcement beats prompts

- Prompt-only scope control under compiler pressure: 0/20
- Mechanical clamping (revert after worker): 20/20
- `disallowedTools: ["Task"]` in `.claude/settings.json`: reliable
- CLAUDE.md instructions: suggestion only (model can deprioritize)

### 4.4 Contract before workers

- Without frozen type contract: 10 workers → 6 incompatible interfaces
- With frozen contract: 50+ domain builds → zero integration type errors
- Freeze shapes (types, enums, events). Leave values in config.

### 4.5 One-shot dispatch is wrong for large builds

- Wave-based dispatch: proven across 15+ sessions
- Commit after every wave before dispatching the next worker
- Clamp scope mechanically after every worker
- One-shot for large builds: explicit anti-pattern

---

## PART 5: THE PROMPT TEMPLATES (copy-paste ready)

### 5.1 Worker dispatch (single task)

```
You are improving a [engine] [genre] game ([name], ~[N]K LOC).

TASK: [Exact description with specific values, not vague goals]

CONTEXT:
- [File A] has [system X] which does [behavior]
- [File B] has [pattern Y] — use this as reference

WHAT TO DO:
1. Read [file] to find [function/struct]
2. Add/modify [exact change]
3. Register in [plugin/mod file]

SCOPE: ONLY modify [file list]. Nothing else.
COMMIT: git add -A && git commit -m '[type]: [description]'
```

### 5.2 Foreman dispatch (multi-task campaign)

```
You have [tool] available for worker dispatch. You are the orchestrator.
Your job is to [campaign goal]. The full manifest is in [file] — read it first.

DO: Work in waves. Complete one wave fully, commit, then move to the next.
DO: Read the manifest section for each wave BEFORE starting.
DO NOT: Try to do everything in one giant edit.
DO NOT: Stop after one wave and ask if you should continue.
     Continue until you have exhausted every actionable item or hit a genuine blocker.
DO NOT: Rewrite large systems. Minimum edit that wires correctly.

Wave sequence:
[numbered waves with exact scope per wave]

Start now. Read the manifest, then begin Wave 1.
```

### 5.3 Asset audit prompt

```
Audit every [asset type] in this game. For each:
1. What file is it? (path, dimensions, atlas layout)
2. Is it loaded in code? (grep asset_server.load)
3. If loaded: is it the RIGHT asset for its purpose?
4. If unused: what game system should use it? What's the exact wiring point?
5. What's blocking it?

Write the results as a manifest with per-asset wiring instructions.
Group by family (water, trees, crops, UI, etc.).
Priority rank: P0 (broken), P1 (matched assets ready to wire), P2 (needs new data), P3 (needs new systems).
```

### 5.4 Category investigation prompt

```
Investigate the [category] family in this game. For EVERY instance:
1. Is the correct sprite/tile being used? Check alternatives on disk.
2. What's around it? (surrounding objects, decorations, transitions)
3. What's animated? What should be but isn't?
4. What's missing entirely? (every comparable game has X, we don't)
5. What system-level improvements would affect ALL instances?

Build the audit as a table: Feature × Instance = status.
Then write a wave-based implementation plan.
```

---

## PART 6: THE PITFALLS (what will go wrong and how to fix it)

### 6.1 Agent defaults to solo execution
**Symptom:** Agent implements everything itself instead of delegating to workers.
**Fix:** Explicitly say "you have [tool] available for worker dispatch" and "dispatch workers for independent tasks."

### 6.2 Agent stops after one wave
**Symptom:** Agent completes Wave 1, summarizes, waits for confirmation.
**Fix:** Include "DO NOT stop between waves" in the dispatch doc. If it still stops, type "continue."

### 6.3 Agent builds orchestration instead of features
**Symptom:** Asked for 80 weapons, got a weapon generation framework.
**Fix:** Add to worker spec: "Do NOT create orchestration infrastructure. Implement only domain deliverables."

### 6.4 Agent reads summaries instead of source files
**Symptom:** Asked for 80 items, got 8. Constants are defaults not your specified values.
**Fix:** Put full specs on disk. Put quantities in the worker spec. Say "read the file at [path]."

### 6.5 Workers edit the frozen contract
**Symptom:** shared/types.rs modified during parallel build.
**Fix:** Clamp mechanically. Or: `disallowedTools` in settings. Contract changes are integration-phase only.

### 6.6 Premium quota exhausted
**Symptom:** 402 errors on Copilot premium models (Sonnet/Opus).
**Fix:** Fall back to gpt-5-mini or gpt-4.1 via Copilot, or gpt-5.4 via Codex CLI. Non-premium models work for scoped tasks.

### 6.7 Session dies mid-campaign
**Symptom:** Terminal disconnects, context overflows, or you have to leave.
**Fix:** The dispatch document + manifest on disk is the state. New session: "Resume [campaign] from Wave N — waves 1 through N-1 are committed on master. Read [manifest file]."

### 6.8 New file creation fails (~50% first-try)
**Symptom:** Worker creates a new .rs file but doesn't register it in mod.rs, or the module structure is wrong.
**Fix:** Prefer edit-existing over create-new when possible. When new files are needed, specify the mod.rs registration in the prompt explicitly.

---

## PART 7: GAME-SPECIFIC INSTRUMENTS

### 7.1 The sprite audit instrument

Run this at project start and every major milestone:

```bash
# 1. Total asset inventory
find assets/ -name "*.png" | wc -l

# 2. Loaded in code
grep -rn "asset_server.load" src/ --include="*.rs" | grep -v test | 
  sed 's/.*load("//' | sed 's/").*//' | sort -u | wc -l

# 3. Unused percentage  
# (compare the two lists)

# 4. Per-family breakdown
# (categorize by path: sprites/crop_*, sprites/npcs/*, tilesets/*, ui/*)
```

Target: <20% unused at ship time. Hearthfield started at 68% unused and got to ~45% in one campaign.

### 7.2 The visual completeness checklist

For every exterior location:
- [ ] Unique building sprites (not same PNG with different tint)
- [ ] Door animation
- [ ] Window glow at night
- [ ] Chimney smoke (if appropriate)
- [ ] Surrounding objects (2-4 thematic items per building)
- [ ] Fence/boundary definition
- [ ] Building sign
- [ ] Seasonal decoration variation

For every natural feature:
- [ ] Water animation (not static tiles)
- [ ] Water differentiation by type (ocean vs pond vs river)
- [ ] Tree variety per biome (not one type everywhere)
- [ ] Tree seasonal variation
- [ ] Path autotiling (not all-crossroads)
- [ ] Grass decoration density appropriate to biome

For every character:
- [ ] Unique sprite (no duplicates)
- [ ] Sprite matches role
- [ ] Color tinting per character
- [ ] Emote/reaction sprites
- [ ] Seasonal clothing hints (color shift at minimum)

### 7.3 The particle system checklist

Every player-visible interaction should have particle feedback:
- [ ] Tool use (impact particles) 
- [ ] Harvest (burst particles)
- [ ] Item pickup (sparkle)
- [ ] Gift giving (heart particles)
- [ ] Crop growth (shimmer on stage change)
- [ ] Walking (footstep dust)
- [ ] Fishing cast (splash)
- [ ] Fishing catch (celebration sparkle)
- [ ] Mining (rock fragments)
- [ ] Combat (hit flash)

### 7.4 The atmosphere checklist

- [ ] Day/night cycle with ambient tinting
- [ ] Dawn/dusk distinct color transitions  
- [ ] Indoor vs outdoor lighting difference
- [ ] Rain particles + screen darkening
- [ ] Snow particles + white ground tint
- [ ] Storm lightning flashes
- [ ] Ambient fireflies at dusk (outdoor)
- [ ] Candle flicker (indoor)
- [ ] Chimney smoke (buildings)
- [ ] Wind sway on bushes/tall crops

---

## PART 8: SESSION RECOVERY

If you lose context (new chat, compaction, crash), paste this section:

```
I am continuing work on [game name], a [engine/language] [genre] game.

Key files to read:
- MANIFEST.md (current phase, decisions, blockers)
- docs/spec.md (full game specification)
- docs/[relevant audit].md (if doing asset/visual work)
- orchestration/FOREMAN_PLAYBOOK.md (dispatch patterns)
- src/shared/mod.rs (type contract — do not modify without freezing)

Recent work: [1-2 sentence summary]
Current task: [what you want to do now]

Read the files above before acting.
```

---

## PART 9: NUMBERS THAT MATTER

These are real, measured values from the Hearthfield program:

- **295M tokens** consumed across the full research program
- **728 commits**, 63K+ LOC, 0 handwritten lines
- **172 PNG assets**, 88 initially unwired → 35+ wired in one 17-minute campaign
- **Ship rate:** 67% manual dispatch, 100% foreman dispatch (5/5)
- **Scope enforcement:** 0/20 prompt-only, 20/20 mechanical
- **Evidence tag defense:** 0% → 96% correct across 5 frontier models
- **Blackout test:** 5/5 packet laws confirmed, 0 contamination radius
- **Approach comparison winner:** Decision Fields (1514 lines vs 9 for formal spec)
- **Specificity gradient:** exact values 100%, named actions 67%, vague goals 0%
- **New file creation:** ~50% first-try vs ~90% for edits
- **Parallel workers:** 2-3 stable, 5+ crashes
- **4-layer stack:** 5/5 shipped, 0 scope violations
