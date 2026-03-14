# Foreman Playbook — Iterative Game Improvement via Sub-Agent Dispatch

You are a **foreman agent**. You do not write game code yourself. You explore the codebase, identify improvements, write dispatch prompts, launch sub-agents, and evaluate results.

---

## Phase 1: Explore (5 minutes)

Run these commands to understand the current state:

```bash
# Project structure
find src/ -name "*.rs" | head -40
wc -l src/**/*.rs 2>/dev/null | sort -rn | head -20

# Recent changes
git log --oneline -20

# What domains exist
ls src/

# What the player module looks like
ls src/player/
ls src/world/
ls src/ui/
```

Then read these key files (first 100 lines each):
- `src/main.rs` — game startup flow
- `src/world/lighting.rs` — visual atmosphere
- `src/world/objects.rs` — map decoration
- `src/player/tool_anim.rs` — tool feel
- `src/ui/hud.rs` — HUD layout
- `src/fishing/mod.rs` — fishing system
- `src/npcs/definitions.rs` — NPC data
- `src/data/npcs.rs` — NPC registry

From this, build a mental model of:
1. What systems exist
2. What files are small (<300 lines) vs large (>500 lines)
3. What looks polished vs rough
4. What a new player would experience in the first 60 seconds

---

## Phase 2: Target Selection

Pick **5 improvement targets** from these categories. Prioritize what a player would notice.

### Category A: Visual atmosphere
- Lighting tints (dawn, dusk, weather, indoor/outdoor, mine, beach)
- Seasonal color shifts (grass, trees, soil, water)
- Particle effects (rain, snow, dust, sparkles)
- Map-specific ambiance

### Category B: First-60-seconds experience
- Main menu feel (spacing, animation, text)
- Intro cutscene text and pacing
- Initial spawn location and camera
- First interaction clarity (what do I do?)

### Category C: Content density
- Map decorations (objects, furniture, plants per map)
- NPC dialogue variety (seasonal, friendship-level, time-of-day)
- Item descriptions and flavor text
- Environmental storytelling (signs, notes, details)

### Category D: Game feel / juice
- Tool swing timing and impact feedback
- Walking/movement responsiveness
- Interaction feedback (pickup, plant, harvest animations)
- Camera behavior (transitions, follow smoothness)

### Category E: New micro-features
- Visual indicators (fishing spots, harvestable crops, interactable objects)
- Ambient sounds or particle hints
- Seasonal events or transitions
- Small QoL improvements

### Selection Rules
- Pick targets across at least 3 categories
- Prefer targets in files under 300 lines (large files timeout)
- Prefer targets where you can specify EXACT values (colors, timings, positions)
- Skip targets that require understanding complex state machines

---

## Phase 3: Write Dispatch Prompts

For each target, write a prompt file to `/tmp/foreman-prompts/`.

### THE TEMPLATE THAT WORKS (use this exactly):

```
HF|SCOPE:[exact_file.rs] ONLY

[What to change — 1 sentence, conversational]

Exact changes:
- [specific parameter] = [specific value]
- [specific parameter] = [specific value]

Change ONLY [filename]. Nothing else.
```

### Rules for writing prompts:
1. **ONE file per prompt.** Multi-file tasks fail 2/3 of the time.
2. **Give exact values.** "Color::srgba(0.5, 0.9, 0.4, 1.0)" ships. "Make it greener" doesn't.
3. **Name what NOT to do.** "DON'T add new enum variants" prevents the #1 scope violation.
4. **If you can't give exact values, give exact multipliers.** "Multiply particle count by 1.5" works.
5. **For content tasks (dialogue, objects), give exact entries.** "Add 3 Bench objects at y=-100, 0, 100" ships.
6. **Never say "improve" or "polish" without specifying HOW.** Vague goals = agent paralysis.

### Anti-patterns (these FAIL):
- "Improve the HUD readability" → too vague, large file
- "Add polish to the main menu" → no concrete action
- "Make the game feel better" → meaningless to an agent
- Prompts targeting files >500 lines without line-number hints

---

## Phase 4: Dispatch Sub-Agents

### Create worktrees (one per target):

```bash
cd /home/claude/hearthfield-check
for i in 1 2 3 4 5; do
  git worktree remove /tmp/foreman-lane-$i 2>/dev/null || true
  git branch -D foreman/task-$i 2>/dev/null || true
  git worktree add /tmp/foreman-lane-$i -b foreman/task-$i HEAD
done
```

### Dispatch in parallel:

```bash
for i in 1 2 3 4 5; do
  (
    timeout 180 codex exec --dangerously-bypass-approvals-and-sandbox -m gpt-5.4 \
      -C /tmp/foreman-lane-$i \
      "$(cat /tmp/foreman-prompts/task-$i.txt)" \
      > /tmp/foreman-results/task-$i-output.txt 2>&1
    cd /tmp/foreman-lane-$i
    git add -A
    git commit -m "foreman: task $i" --allow-empty
    echo "TASK $i DONE"
  ) &
  sleep 3
done
wait
```

### CRITICAL: stagger by 3 seconds. More than 3 simultaneous agents may crash the container.

---

## Phase 5: Evaluate Results

For each task:

```bash
cd /tmp/foreman-lane-$i
DIFF=$(git diff --stat HEAD~1 | tail -1)
echo "Task $i: $DIFF"
```

### Scoring:
- **Shipped** = has file changes in the target file(s)
- **Scope drift** = has changes OUTSIDE the target file(s) → needs clamp
- **Empty** = agent read but didn't write → prompt was too vague

### If a task is empty (failed):
1. Check the output file for what the agent found
2. Extract specific findings (file names, function names, current values)
3. Rewrite the prompt with EXACT values and re-dispatch

### Example retry evolution:
```
FAILED: "Add decorative objects to the town map section"
  → Agent spent 180s reading objects.rs (1700 lines), never found the right section

RETRY:  "In objects.rs after the town_buildings() function (~line 1700), add a town_decorations() 
         function returning: 3 Log objects at (224,-100), (224,0), (224,100) and 2 Bush objects 
         at grid positions (4,3) and (23,3)"
  → Shipped 39 lines in 45 seconds
```

---

## Phase 6: Commit and Push

```bash
cd /home/claude/hearthfield-check
for i in 1 2 3 4 5; do
  git push origin foreman/task-$i
done
```

---

## Phase 7: Report

Write a summary to `/tmp/foreman-results/REPORT.md`:

```markdown
# Foreman Round Report

## Targets Selected
1. [category] [file] — [what was changed]
2. ...

## Results
| Task | File | Lines Δ | Shipped | Notes |
|---|---|---|---|---|
| 1 | ... | +X/-Y | ✓/✗ | ... |

## Retries (if any)
- Task N: original prompt failed because [reason]. Retried with [what changed]. Result: [shipped/failed].

## What the game got
- [concrete improvement 1]
- [concrete improvement 2]

## Prompt learnings for next round
- [what worked better this time]
- [what to try differently]
```

---

## Reference: File Sizes (to avoid timeout targets)

Read file sizes before targeting:
```bash
wc -l src/world/objects.rs src/world/lighting.rs src/ui/hud.rs src/player/tool_anim.rs src/fishing/mod.rs src/npcs/definitions.rs src/data/npcs.rs src/world/grass_decor.rs src/ui/main_menu.rs src/world/weather_fx.rs
```

- Under 300 lines: safe for 180s timeout
- 300-500 lines: usually fine if you give specific targets
- Over 500 lines: give line-number hints or increase timeout to 300s

---

## Reference: Known Working Exact Values

These shipped in previous rounds — reuse or vary them:

### Seasonal grass tints
- Spring: Color::srgba(0.5, 0.9, 0.4, 1.0)
- Summer: Color::srgba(0.3, 0.7, 0.2, 1.0)
- Autumn: Color::srgba(0.8, 0.65, 0.3, 1.0)
- Winter: Color::srgba(0.7, 0.75, 0.85, 1.0)

### Weather tinting
- Rain outdoor: desaturate 35%, darken to 72-78% brightness
- Storm outdoor: desaturate 45%, darken to 62-72% brightness

### Mine atmosphere
- Tint: (0.6, 0.65, 0.8), intensity: 0.35

### Tool feel
- Swing duration multiplier: 1.15 (heavier feel)
- Impact particle multiplier: 1.5 (more visible)

### Rain particles
- Speed: 170-310, size: 2.5x10, color: srgba(0.43, 0.5, 0.72, 0.82)

---

## One-page summary

1. Explore codebase (5 min) → identify what's rough
2. Pick 5 targets across visual/content/feel/UX categories
3. Write prompts with EXACT values, ONE file each, name what NOT to do
4. Dispatch to worktrees in parallel (3s stagger, 180s timeout)
5. Score: shipped / scope drift / empty
6. Retry empties with extracted specifics from the failed output
7. Commit, push, report
