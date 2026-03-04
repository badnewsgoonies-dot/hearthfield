# Worker: City DLC R7 — Content Depth Slice B (HUD Expansion + Relationship Milestones)

## Goal

Add career/progression display, coworker relationship panel, unlock catalog display,
and relationship milestone events to the City Office Worker DLC. This makes the social
and progression systems visible to the player during gameplay.

## Scope

You may modify these files:
- `dlc/city/src/game/resources.rs`
- `dlc/city/src/game/events.rs`
- `dlc/city/src/game/systems/interruptions.rs`
- `dlc/city/src/game/systems/day_cycle.rs`
- `dlc/city/src/game/systems/mod.rs`
- `dlc/city/src/game/ui/hud.rs`
- `dlc/city/src/game/ui/mod.rs`
- `dlc/city/src/game/mod.rs`
- `dlc/city/src/game/systems/tests.rs`

Do NOT modify files outside the `dlc/city/` directory.

## Required reading (read these files FIRST)

1. `dlc/city/CONTRACT.md`
2. `dlc/city/src/game/resources.rs` — CareerProgression, SocialGraphState, UnlockCatalogState, CoworkerProfile
3. `dlc/city/src/game/events.rs`
4. `dlc/city/src/game/ui/hud.rs` — current HUD layout (left stats, right inbox, top time, bottom keybinds)
5. `dlc/city/src/game/ui/mod.rs` — UI root markers
6. `dlc/city/src/game/mod.rs` — plugin system wiring
7. `dlc/city/src/game/systems/day_cycle.rs` — apply_day_summary_rollover (where level-ups happen)
8. `dlc/city/src/game/systems/interruptions.rs` — where social graph changes happen
9. `dlc/city/src/game/systems/tests.rs` — all existing tests

## Deliverables

### 1. Relationship milestone event (events.rs + resources.rs)

Add a new event:
```rust
#[derive(Event, Debug, Clone)]
pub struct RelationshipMilestone {
    pub coworker_id: u8,
    pub coworker_name: String,
    pub milestone: MilestoneKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MilestoneKind {
    Friendly,       // affinity >= 25
    Trusted,        // trust >= 25
    CloseFriend,    // affinity >= 50
    DeepTrust,      // trust >= 50
    Rival,          // affinity <= -25
    Distrusted,     // trust <= -25
}
```

Put `MilestoneKind` in `resources.rs` (it's a type, not an event).
Register the event in the plugin.

### 2. Milestone detection system (new system in systems/)

Add a system `check_relationship_milestones` that:
- Runs in `OfficeSimSet::Economy` (after interruption deltas applied), only in `InDay` state
- Tracks which milestones have already fired per coworker using a `FiredMilestones` resource:
```rust
#[derive(Resource, Debug, Clone, Default)]
pub struct FiredMilestones {
    pub fired: HashSet<(u8, MilestoneKind)>,
}
```
- For each coworker, checks if affinity/trust crosses a threshold that hasn't fired yet
- Emits `RelationshipMilestone` event for new crossings
- Register `FiredMilestones` in the plugin. Save/load it (add to snapshot with `#[serde(default)]`).

Wire this system into the plugin in `OfficeSimSet::Economy`, after `update_day_outcome_preview`.

### 3. Career readout on HUD (ui/hud.rs)

Add a new section to the left stats panel showing:
```
Lv.1 ■■□□□□□□□□ 0 XP
Streak: 0 | Burnout: 0
```

This means adding two new marker components (`HudLevelText`, `HudStreakText`) and
updating `update_hud` to read from `CareerProgression`.

The XP bar uses filled/empty squares: 10 segments, proportion = xp / xp_for_next_level.
Use `CareerProgression::xp_for_level()` to get the threshold.

### 4. Unlock catalog display on HUD (ui/hud.rs)

Below the career readout, show active unlocks as colored text:
```
[✓] Quick Coffee  [✓] Efficient Processing
[ ] Conflict Training  [ ] Escalation License
```

Add a `HudUnlocksText` marker component. Read from `UnlockCatalogState`.
Green for unlocked, dark gray for locked.

### 5. Coworker relationship panel on HUD (ui/hud.rs)

Add a small panel below the right inbox panel showing coworker relationships:
```
COWORKERS
Marta (Mgr)  ♥ 2  ★ 4
Leo (Clerk)  ♥ 0  ★ 0
Sana (Analyst) ♥ 0  ★ 0
Ira (Coord)  ♥ 0  ★ 0
Noah (Intern) ♥ 0  ★ 0
```

Where ♥ = affinity, ★ = trust. Color code:
- affinity >= 25: green
- affinity <= -25: red
- otherwise: gray
Same for trust.

Add a `HudCoworkerPanel` marker and `HudCoworkerContent` for the text.
Read from `SocialGraphState`.

### 6. Milestone notification toast (ui/hud.rs or new ui/toast.rs)

When a `RelationshipMilestone` event fires, show a brief toast notification
in the center of the screen. Use a simple approach:

Add a `ToastState` resource:
```rust
#[derive(Resource, Debug, Clone, Default)]
pub struct ToastState {
    pub message: String,
    pub remaining_ticks: u32,
}
```

Add a `ToastRoot` marker component in `ui/mod.rs`. Spawn a centered text node
on `OnEnter(InDay)`. A system `update_toast` in `OfficeSimSet::Ui`:
- Reads `RelationshipMilestone` events, formats them as toast messages:
  - Friendly: "{name} warms up to you!"
  - Trusted: "{name} is starting to trust you."
  - CloseFriend: "You and {name} are close friends now!"
  - DeepTrust: "{name} trusts you completely."
  - Rival: "Things are tense with {name}..."
  - Distrusted: "{name} doesn't trust you anymore."
- Sets `remaining_ticks` to 180 (about 3 seconds at 60fps)
- Each tick decrements. When 0, hide the toast.

Register `ToastState` resource in plugin (transient, do NOT save).

### 7. Tests

Add these tests to `tests.rs`:

1. `relationship_milestone_fires_at_affinity_threshold` — manually set a coworker's
   affinity to 24, send interruption that raises it to 25+, run update, assert
   `RelationshipMilestone` event with `MilestoneKind::Friendly` is emitted.

2. `relationship_milestone_does_not_refire` — fire the milestone once, then trigger
   again — assert only 1 event total (FiredMilestones prevents refire).

3. `career_progression_display_reflects_level_and_xp` — set CareerProgression level=3,
   xp=50, assert the HUD text contains "Lv.3" (this is a light integration test).

All existing 44 tests MUST continue to pass.

## Important notes

- The HUD is already complex with many query parameters. Use `#[allow(clippy::too_many_arguments, clippy::type_complexity)]` as needed.
- MilestoneKind needs Hash+Eq for the HashSet key.
- `CoworkerRole` has 5 variants: Manager, Clerk, Analyst, Coordinator, Intern. Use short labels: Mgr, Clerk, Analyst, Coord, Intern.
- `CareerProgression` has an `xp_for_level` method that returns the XP threshold. Read it to understand the leveling curve.
- Keep toast rendering simple — a single centered text node with visibility toggling.

## Validation

```bash
cd /home/user/hearthfield/dlc/city && cargo check
cd /home/user/hearthfield/dlc/city && cargo test
cd /home/user/hearthfield/dlc/city && cargo clippy -- -D warnings
```

Done = all three pass with zero errors and zero warnings.
