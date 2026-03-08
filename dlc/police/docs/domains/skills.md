# Skills Domain Spec — Precinct

## Purpose
Skill trees and XP progression. 4 trees × 5 levels = 20 named perks that define "what kind of cop."

## Scope
`src/domains/skills/` — owns XP tracking, skill point awards, perk definitions, perk queries.

## What This Domain Does
- Track total XP from all sources (cases, evidence, patrol, interrogation, favors)
- Listen to: CaseSolvedEvent, EvidenceCollectedEvent, DispatchResolvedEvent, XpGainedEvent
- Award 1 skill point per SKILL_POINT_INTERVAL (100) XP
- Handle SkillPointSpentEvent: validate available points, increment tree level, cap at 5
- Define all 20 perks as static data with names and descriptions
- Provide public helper functions for other domains to query perk effects:
  - `investigation_quality_bonus(skills) -> f32` (0.0 at L0, +0.1 at L3, +0.15 at L5)
  - `has_perk(skills, tree, level) -> bool`
- Emit XpGainedEvent when XP is earned (centralized accounting)

## What This Domain Does NOT Do
- UI for skill tree screen (ui domain, future wave)
- Evidence quality calculation (evidence domain reads skill level)
- Promotion logic (economy domain)

## Key Types (import from crate::shared)
- `Skills` (Resource), `SkillTree`
- `XpGainedEvent`, `SkillPointSpentEvent`
- `CaseSolvedEvent`, `DispatchResolvedEvent`
- `GameState`, `UpdatePhase`
- Constants: `SKILL_POINT_INTERVAL`, `XP_PER_EVIDENCE`, `XP_PER_INTERROGATION`, `XP_PER_PATROL_EVENT`, `XP_PER_FAVOR`, `XP_CASE_MULTIPLIER`

## Perk Definitions (all 20)

### Investigation Tree
- L1: Quick Search — +20% evidence collection speed (future: reduces scene interaction time)
- L2: Keen Eye — spot hidden evidence in scenes (future: reveals extra evidence nodes)
- L3: Forensic Intuition — +0.1 base evidence quality
- L4: Cold Case Reader — access cold case files in records room
- L5: Master Investigator — all evidence at +0.15 quality

### Interrogation Tree
- L1: Good Cop — trust-building dialogue options available
- L2: Bad Cop — pressure dialogue options available
- L3: Read The Room — see NPC trust/pressure values in dialogue
- L4: Confession Artist — +25% confession chance in interrogation
- L5: Master Interrogator — unlock all dialogue paths

### Patrol Tree
- L1: Beat Knowledge — minimap shows NPC locations (future)
- L2: Quick Response — -20% travel time (fuel cost reduction)
- L3: Pursuit Training — catch fleeing suspects in dispatch events
- L4: Community Trust — +5 trust with all town NPCs (one-time bonus)
- L5: Master Patrol — dispatch calls show difficulty rating

### Leadership Tree
- L1: Radio Discipline — clearer dispatch information text
- L2: Partner Synergy — +10% partner bonus effectiveness
- L3: Budget Request — access department budget for equipment
- L4: Task Delegation — assign simple tasks to AI officers (future)
- L5: Master Commander — direct multi-unit responses (future)

## Systems to Implement
1. `accumulate_xp` — UpdatePhase::Reactions, gated on Playing
   - Read CaseSolvedEvent → add case XP (difficulty * XP_CASE_MULTIPLIER)
   - Read DispatchResolvedEvent → add XP_PER_PATROL_EVENT
   - Read XpGainedEvent → add amount
   - Update Skills.total_xp, calculate new available_points
2. `handle_skill_spend` — UpdatePhase::Reactions
   - Read SkillPointSpentEvent
   - Validate: available_points > 0, tree level < 5
   - Increment appropriate tree level, decrement available_points
3. `apply_one_time_perks` — check if newly unlocked perks have one-time effects (e.g., L4 Community Trust)

## Quantitative Targets
- 4 trees × 5 levels = 20 perks
- SKILL_POINT_INTERVAL = 100 XP per point
- XP sources: cases (difficulty*15), evidence (5), interrogation (20), patrol (10), favors (8)
- Max skill level per tree: 5

## Decision Fields
- **Preferred**: named perks with specific effects
- **Tempting alternative**: generic stat boosts per level
- **Consequence**: generic boosts lose player identity; "what kind of cop" question disappears
- **Drift cue**: worker implements flat +X% per level without named perk definitions

## Plugin Export
```rust
pub struct SkillPlugin;
```

## Tests (minimum 5)
1. XP accumulates from CaseSolvedEvent correctly
2. Skill point awarded at 100 XP threshold
3. Skill spend increments tree level and decrements available points
4. Cannot spend when available_points == 0
5. Cannot exceed level 5 in any tree
6. Investigation quality bonus returns correct values per level
