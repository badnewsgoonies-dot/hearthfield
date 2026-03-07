# Worker Objective: R7 Social Domain Expansion

## Context
You are working on the City Office Worker DLC at `/home/user/hearthfield/dlc/city`.
This is a Bevy 0.15 Rust game simulating office work life.

## Required Reading (read these files before writing code)
1. `/home/user/hearthfield/dlc/city/CONTRACT.md` - the type contract
2. `/home/user/hearthfield/dlc/city/src/game/resources.rs` - existing SocialGraphState, CoworkerProfile, CoworkerRole
3. `/home/user/hearthfield/dlc/city/src/game/systems/interruptions.rs` - existing social scenarios
4. `/home/user/hearthfield/dlc/city/src/game/systems/tests.rs` - existing test patterns

## Deliverables

### 1. Expand Coworker Archetypes (in resources.rs)
Add personality traits and backstory to CoworkerProfile:
```rust
pub struct CoworkerProfile {
    pub id: u8,
    pub codename: String,
    pub role: CoworkerRole,
    pub affinity: i32,
    pub trust: i32,
    // NEW fields:
    pub personality: CoworkerPersonality,
    pub backstory: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CoworkerPersonality {
    #[default]
    Neutral,
    Ambitious,     // competitive, seeks promotions
    Nurturing,     // helpful, mentoring
    Skeptical,     // suspicious, needs proof
    Enthusiastic,  // energetic, sometimes overwhelming
    Reserved,      // quiet, observant
}
```

Update the default SocialGraphState to assign distinct personalities:
- Marta (Manager): Ambitious
- Leo (Clerk): Enthusiastic  
- Sana (Analyst): Nurturing
- Ira (Coordinator): Skeptical
- Noah (Intern): Reserved

Add 3 more coworkers:
- id=6, "Derek", Specialist role, Reserved personality
- id=7, "Jun", Analyst role, Ambitious personality
- id=8, "Priya", Coordinator role, Nurturing personality

### 2. Manager Relationship Arc (in resources.rs and systems/interruptions.rs)
Add manager arc tracking:
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ManagerArcStage {
    #[default]
    Stranger,      // trust < 10
    Acquaintance,  // trust 10-29
    Mentor,        // trust 30-59, affinity >= 20
    Evaluator,     // trust 30-59, affinity < 20
    Ally,          // trust >= 60, affinity >= 40
    Antagonist,    // trust >= 60, affinity < 0
}
```

Add method to SocialGraphState:
```rust
pub fn manager_arc_stage(&self) -> ManagerArcStage {
    // Compute based on manager's trust/affinity thresholds
}
```

### 3. Affinity Milestone Events (in events.rs and systems/interruptions.rs)
Add new event types:
```rust
#[derive(Event, Debug, Clone)]
pub struct AffinityMilestoneEvent {
    pub coworker_id: u8,
    pub milestone: AffinityMilestone,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AffinityMilestone {
    CoffeeInvite,      // affinity >= 15
    LunchTogether,     // affinity >= 30
    ProjectCollab,     // affinity >= 50, trust >= 30
    WorkBuddy,         // affinity >= 70, trust >= 50
}
```

Add milestone tracking to SocialGraphState:
```rust
pub reached_milestones: HashSet<(u8, AffinityMilestone)>,
```

Add system to check and emit milestones after affinity changes.

### 4. Personality-Based Dialogue Variations (in resources.rs)
Extend CoworkerDialogue to include personality-specific lines:
```rust
impl CoworkerDialogue {
    pub fn line_for_personality(&self, role: CoworkerRole, personality: CoworkerPersonality, seed: u64) -> String
}
```

### 5. Social Scenario Variations Based on Personality (in systems/interruptions.rs)
Modify `INTERRUPTION_SCENARIOS` or add scenario selection logic that considers the involved coworker's personality for different stress/focus modifiers.

### 6. Save/Load Updates (in save/mod.rs)
Update OfficeSaveSnapshot to include:
- New CoworkerProfile fields (personality, backstory)
- ManagerArcStage
- reached_milestones HashSet

Ensure round-trip persistence tests pass.

## Tests to Add (in systems/tests.rs)
Add at least 25 new tests:

1. `test_coworker_personality_defaults_are_distinct` - each coworker has unique personality
2. `test_manager_arc_stranger_below_threshold` - trust < 10 → Stranger
3. `test_manager_arc_acquaintance_threshold` - trust 10-29 → Acquaintance
4. `test_manager_arc_mentor_with_high_affinity` - trust 30-59, affinity >= 20 → Mentor
5. `test_manager_arc_evaluator_with_low_affinity` - trust 30-59, affinity < 20 → Evaluator
6. `test_manager_arc_ally_high_trust_high_affinity` - trust >= 60, affinity >= 40 → Ally
7. `test_manager_arc_antagonist_high_trust_negative_affinity` - trust >= 60, affinity < 0 → Antagonist
8. `test_affinity_milestone_coffee_invite` - milestone fires at affinity >= 15
9. `test_affinity_milestone_lunch_together` - milestone fires at affinity >= 30
10. `test_affinity_milestone_project_collab` - requires trust and affinity thresholds
11. `test_affinity_milestone_work_buddy` - highest tier milestone
12. `test_milestone_does_not_refire` - once reached, milestone is not re-emitted
13. `test_personality_affects_dialogue_selection` - different personalities get different lines
14. `test_eight_coworkers_in_default_graph` - verify expanded roster
15. `test_social_graph_normalization_with_new_fields` - normalize clamps all values
16. `test_save_load_preserves_personality` - personality survives round-trip
17. `test_save_load_preserves_milestones` - reached_milestones survives round-trip
18. `test_save_load_preserves_manager_arc` - arc stage is derivable after load
19. `test_manager_arc_transitions_on_trust_change` - arc changes as trust changes
20. `test_specialist_role_exists` - new CoworkerRole::Specialist
21. `test_personality_scenario_modifier` - personality affects interruption outcomes
22. `test_nurturing_personality_reduces_stress` - specific personality bonus
23. `test_ambitious_personality_increases_competition_stress` - specific personality effect
24. `test_deterministic_social_milestone_sequence` - milestones fire in deterministic order
25. `test_all_coworkers_have_backstory` - no empty backstory strings

## Validation (run before reporting done)
```bash
cd /home/user/hearthfield/dlc/city
cargo fmt --all
cargo check
cargo test
cargo clippy --all-targets -- -D warnings
```

All four commands must pass with zero errors and zero warnings.

## Constraints
- Do NOT modify CONTRACT.md (the orchestrator handles contract updates)
- Maintain backward compatibility with existing save format (use schema migration if needed)
- Keep all new code in the existing file structure
- Use #[allow(dead_code)] for new types until they're wired into gameplay
