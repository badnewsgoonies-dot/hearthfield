# Approach Comparison: 6 Briefing Styles for the Same Task

Same model (GPT-5.4), same repo, same task (weather visual polish), same tool access. Only the briefing style changed.

---

## Results Summary

| Approach | Words | Diff lines | Read-first gap | Files touched | Tint refs | Scope violations | Evidence cited |
|---|---|---|---|---|---|---|---|
| A: Geni freeform | 6122 | 446 | 17 lines | 5 | 135 | 2 | 0 |
| B: Kernel structured | 2591 | 9 | 86 lines | 7 | 49 | 20 | 4 |
| C: Decision fields | 10877 | 1514 | 260 lines | 2 | 493 | 5 | 0 |
| D: Compressed state | 2459 | 179 | 303 lines | 3 | 52 | 9 | 4 |
| E: Show-don't-tell | 5565 | 513 | 612 lines | 8 | 306 | 8 | 0 |
| F: Ultra-minimal | 1699 | 2 | 240 lines | 10 | 41 | 9 | 0 |

---

## What Each Approach Produced

### A: Geni Freeform (stream-of-consciousness)
**Style:** "ok so the weather system right now is like... functional but boring"
**Behavior:** Read quickly (line 17→edit by line 38). Produced moderate changes. Found existing systems and tuned them. Low scope violations. Zero evidence citations but high finding count (52 observations).
**Verdict:** Fast to action, reasonable scope, conversational engagement. Produced real improvements but didn't document them formally.

### B: Kernel Structured (formal spec with evidence levels)
**Style:** YAML-ish spec with [Inferred]/[Assumed] tags, explicit deliverables and validation
**Behavior:** Read thoroughly before acting (86-line gap). Produced MINIMAL code changes (9 diff lines) but high scope violations (20 out-of-scope references). Actually cited evidence levels (4 times). Touched 7 files but changed almost nothing.
**Verdict:** The formal structure made it CAUTIOUS to the point of underdelivering. It spent its budget reading and documenting rather than implementing. Good for verification, bad for implementation.

### C: Decision Fields (contrastive prompting) ★ MOST PRODUCTIVE
**Style:** Preferred action / tempting alternative / consequence / drift cue / recovery
**Behavior:** Read extensively (260-line gap before first edit). Then produced the MOST code changes (1514 diff lines). Highest tint references (493) and atmosphere references (45). Only touched 2 files — the RIGHT 2 files (weather_fx.rs and lighting.rs). Low scope violations.
**Verdict:** The contrastive framing ("don't build shaders, tune tints") was the most effective scope control AND produced the most implementation. The agent knew exactly what NOT to do, which freed it to go deep on what TO do.

### D: Compressed State (707-char invented notation + task)
**Style:** HF|GW|52kLOC|... TASK: promote weather_effects from [ASM] to [OBS]
**Behavior:** Read quickly (line 15). Long read phase (303 lines before edit). Produced moderate changes (179 diff lines). Cited evidence (4 times). Touched 3 files.
**Verdict:** Efficient on tokens (2459 total words — second lowest). Evidence-aware. But the compressed format gave it PROJECT context without TASK context — it knew what the project was but the task description was too terse.

### E: Show-Don't-Tell (good/bad examples)
**Style:** "RAIN (what good rain feels like):" + "BAD EXAMPLES (what NOT to do):"
**Behavior:** Longest read phase of all (612 lines before first edit). Then produced substantial changes (513 diff lines). High tint references (306) and atmosphere references (49). But touched 8 files — more than necessary.
**Verdict:** The examples made it thorough and opinionated but also expanded its conception of scope. It tried to improve EVERYTHING that touched weather (grass_decor, tree_fx, seasonal, maps) rather than focusing on the core systems. Good for understanding intent, bad for scope control.

### F: Ultra-Minimal ("make weather better, stay in src/world/")
**Style:** One sentence.
**Behavior:** Read first (240 lines before edit). Produced almost NO code changes (2 diff lines). But explored broadly — touched 10 files. Spent its entire budget reading and understanding without committing to action.
**Verdict:** Not enough direction produces exploration without implementation. The agent didn't know what "better" meant so it couldn't commit to a specific improvement.

---

## Key Findings

### 1. Decision Fields produced the best implementation (C: 1514 diff lines, 2 files)
The contrastive framing — naming what NOT to do — was more effective than naming what TO do. The agent that knew "don't build shaders" produced 3× more changes in the RIGHT files than the agent told "add rain atmosphere" in a structured spec.

### 2. Formal structure suppresses implementation (B: 9 diff lines)
The kernel-style spec with evidence levels made the agent so cautious it barely changed anything. It documented extensively but implemented almost nothing. Formal specs work for verification tasks (like the festival verification) but SUPPRESS implementation tasks.

### 3. Freeform is fast but undocumented (A: 446 diff lines, 0 evidence)
The conversational style produced quick action and reasonable results but zero formal evidence. This is fine for solo work but creates the exactly handoff problem the architecture is designed to prevent.

### 4. Examples expand scope uncontrollably (E: 8 files touched)
Showing the agent what "good" looks like made it try to achieve "good" everywhere, not just in the target files. Examples are great for intent transfer but need to be paired with hard scope constraints.

### 5. Too little direction produces paralysis (F: 2 diff lines)
One sentence wasn't enough. The agent explored for 240 lines and then barely acted. There's a minimum viable briefing below which the agent can't commit to a direction.

### 6. Compressed state works for ROUTING but not for TASKING (D: moderate)
The invented notation told the agent everything about project context but not enough about what "improve weather" means. Compressed state + a richer task description would combine the efficiency of D with the effectiveness of C.

---

## Optimal Briefing Pattern (derived from comparison)

The best approach combines elements from C + D + A:

1. **Compressed project state** (from D) — project context in 486 chars
2. **Decision Fields** (from C) — preferred action + tempting alternative + drift cue
3. **Conversational task description** (from A) — what the goal FEELS like, not just what it IS
4. **Hard scope constraint** (from B) — ONLY these files, enforced mechanically post-run

Something like:

```
HF|GW|52kLOC|UNV[ASM]{weather_effects}|SCOPE:src/world/weather_fx.rs,src/world/lighting.rs

Make rain feel moody and snow feel cozy by tuning the existing tint/particle systems.

Decision:
- DO: tune tint multipliers, particle speeds, indoor/outdoor contrast
- DON'T: build new shader systems, add post-processing, create new asset files
- DRIFT CUE: if you're creating new .rs files or touching src/shared — stop

Read the code first, then improve it. Show me what changed.
```

That's ~100 words. The Decision Fields approach (C) at 150 words produced 1514 diff lines in the right 2 files. The kernel spec (B) at 200 words produced 9 diff lines. More structure ≠ more output. The right structure = more output in the right place.

---

## Approach Selection Guide

| Task type | Best approach | Why |
|---|---|---|
| Implementation (build/fix) | Decision Fields (C) | Contrastive framing prevents drift while enabling depth |
| Verification (check claims) | Kernel structured (B) | Formal evidence structure matches verification goals |
| Exploration (what exists?) | Geni freeform (A) | Conversational style encourages broad investigation |
| Quick fix (known bug) | Compressed state (D) + target | Efficient context, specific target |
| Polish pass (improve feel) | Examples (E) + scope clamp | Intent transfer works for subjective quality |
| DON'T USE | Ultra-minimal (F) | Below minimum viable briefing → paralysis |


---

## Optimal Format Results (3 real tasks)

### Weather Polish — SHIPPED REAL IMPROVEMENTS ★
**Briefing:** 100 words (compressed state + Decision Fields + conversational goal)
**Output:** 75 insertions, 39 deletions across weather_fx.rs and lighting.rs (plus 4 scope violations to clamp)

**Actual changes made:**
- Rain drops: slower (170-310 vs 200-400), larger (2.5×10 vs 2×8), more blue-grey
- Storm drops: faster than rain (280-480), larger (3.5×10), darker  
- Snow: slower drift (15-35 vs 20-45), slightly larger (5×5 vs 4×4), warmer white
- NEW `weather_adjusted_outdoor_tint()` function: rain = 35% desaturation + 72-78% brightness; storm = 45% desaturation + 62-72% brightness
- Proper luminance-preserving desaturation using perceptual weights (0.299R + 0.587G + 0.114B)

These are real, shipping-quality visual improvements. The rain will look moodier, storms darker, snow cozier. After scope clamping (4 out-of-scope files), this is a clean weather polish commit.

### Dialogue Depth — Read phase completed, no diffs
The agent correctly identified it should read first, started exploring the dialogue system, but the task was too complex for the 180s timeout. It needed more time to understand the NPC data structures before expanding them.

### Save Hardening — Read phase completed, no diffs  
Same pattern — the save system is 1000+ lines and the agent spent its budget understanding the serialization paths before writing tests. Needed longer timeout.

### Takeaways for Optimal Dispatch

1. **100 words of Decision Fields + compressed state worked best for implementation** (weather: 75 insertions)
2. **Tasks involving large file reads need longer timeouts** (dialogue and save both stalled in read phase)
3. **Scope violations still occur** — the optimal format reduced them (4 vs 20 for kernel-style) but didn't eliminate them. `clamp-scope.sh` remains mandatory.
4. **Real game improvements shipped** — the weather changes are production-quality tuning from a 100-word briefing

