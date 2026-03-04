# Worker: City DLC R7 — Content Depth Slice A (Data + Systems)

## Goal

Add narrative content and variety to the City Office Worker DLC. Right now the simulation
has working mechanics but zero player-facing text — interruptions show a single hardcoded
string, tasks have no descriptions, and coworkers are invisible. This slice adds content
strings, expands the task/interruption template pools, wires `InterruptionKind` into live
systems, and adds a `ScenarioText` resource so the UI layer (Slice B) can display rich
context.

## Scope

You may modify these files:
- `dlc/city/src/game/resources.rs`
- `dlc/city/src/game/events.rs`
- `dlc/city/src/game/systems/interruptions.rs`
- `dlc/city/src/game/systems/task_board.rs`
- `dlc/city/src/game/systems/tasks.rs`
- `dlc/city/src/game/systems/day_cycle.rs`
- `dlc/city/src/game/ui/interruption.rs`
- `dlc/city/src/game/ui/task_board.rs`
- `dlc/city/src/game/ui/mod.rs`
- `dlc/city/src/game/mod.rs`
- `dlc/city/src/game/systems/mod.rs`
- `dlc/city/src/game/save/mod.rs`
- `dlc/city/src/game/systems/tests.rs`

Do NOT modify files outside the `dlc/city/` directory.

## Required reading (read these files FIRST)

1. `dlc/city/CONTRACT.md` (the type contract)
2. `dlc/city/src/game/resources.rs` (all resources)
3. `dlc/city/src/game/events.rs` (all events)
4. `dlc/city/src/game/systems/interruptions.rs` (current 6 scenarios)
5. `dlc/city/src/game/systems/task_board.rs` (current 12 task templates)
6. `dlc/city/src/game/ui/interruption.rs` (current hardcoded popup)
7. `dlc/city/src/game/ui/task_board.rs` (current plain text display)
8. `dlc/city/src/game/mod.rs` (plugin wiring)
9. `dlc/city/src/game/save/mod.rs` (save schema — read to avoid breaking)
10. `dlc/city/src/game/systems/tests.rs` (all 40 tests — must stay passing)

## Deliverables

### 1. Expand TaskKind (resources.rs)

Add 4 new variants to `TaskKind`:
```rust
pub enum TaskKind {
    DataEntry,
    Filing,
    EmailTriage,
    PermitReview,
    // NEW:
    PhoneCall,
    MeetingPrep,
    ReportWriting,
    BudgetReview,
}
```

Add a `description(&self) -> &'static str` method on `TaskKind`:
```rust
impl TaskKind {
    pub fn description(&self) -> &'static str {
        match self {
            Self::DataEntry => "Enter spreadsheet figures for quarterly report",
            Self::Filing => "Sort and file incoming documents to correct departments",
            Self::EmailTriage => "Read, prioritize, and route the email backlog",
            Self::PermitReview => "Check permit applications for compliance",
            Self::PhoneCall => "Return missed calls and take messages",
            Self::MeetingPrep => "Prepare agenda and handouts for upcoming meeting",
            Self::ReportWriting => "Draft the weekly status report for management",
            Self::BudgetReview => "Reconcile expense receipts against budget lines",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::DataEntry => "Data Entry",
            Self::Filing => "Filing",
            Self::EmailTriage => "Email",
            Self::PermitReview => "Permit",
            Self::PhoneCall => "Phone",
            Self::MeetingPrep => "Meeting",
            Self::ReportWriting => "Report",
            Self::BudgetReview => "Budget",
        }
    }
}
```

### 2. Wire InterruptionKind into live system (events.rs + interruptions.rs)

Change `InterruptionEvent` to carry a kind:
```rust
#[derive(Event, Debug, Clone, Copy)]
pub struct InterruptionEvent {
    pub kind: InterruptionKind,
}
```

Update `auto_trigger_interruptions` to pick a kind deterministically from seed:
```rust
fn pick_interruption_kind(seed: u64, day: u32, hour: u32) -> InterruptionKind {
    match (seed.wrapping_add(day as u64 * 13).wrapping_add(hour as u64 * 7)) % 4 {
        0 => InterruptionKind::ManagerRequest,
        1 => InterruptionKind::EmergencyMeeting,
        2 => InterruptionKind::SystemOutage,
        _ => InterruptionKind::CoworkerHelp,
    }
}
```

Update `handle_interruption_requests` to read the kind and use it when selecting scenarios.

### 3. Add ScenarioText resource (resources.rs)

Add a resource that holds the current interruption context for the UI to read:
```rust
#[derive(Resource, Debug, Clone, Default)]
pub struct ActiveInterruptionContext {
    pub kind: Option<InterruptionKind>,
    pub coworker_name: Option<String>,
    pub description: String,
}
```

Initialize and register this resource in the plugin. When an interruption fires,
populate it with the kind-specific text and involved coworker name. When resolved
(calm/panic), clear it.

Add descriptive text for each InterruptionKind:
- ManagerRequest: "Marta stops by your desk. 'Got a minute? I need an update on the Henderson file.'"
  (use actual manager name from SocialGraphState)
- EmergencyMeeting: "All hands! Emergency meeting in Conference Room B — the client changed the deadline."
- SystemOutage: "The network just went down. IT says 15 minutes, but who knows..."
- CoworkerHelp: "{coworker_name} leans over: 'Hey, can you help me figure out this spreadsheet formula?'"
  (use the selected teammate name from SocialGraphState)

### 4. Expand task templates (task_board.rs)

Expand `TASK_TEMPLATES` from 12 to 24 entries. Add templates for all 8 TaskKind variants
across multiple priority levels. Keep the same field structure. Example new entries:

```
PhoneCall/Low:      focus=20, stress=1, money=7,  rep=0, deadline=10
PhoneCall/High:     focus=48, stress=5, money=15, rep=2, deadline=52
MeetingPrep/Med:    focus=42, stress=4, money=13, rep=1, deadline=38
MeetingPrep/Crit:   focus=72, stress=9, money=25, rep=4, deadline=130
ReportWriting/Low:  focus=28, stress=2, money=10, rep=1, deadline=18
ReportWriting/High: focus=54, stress=6, money=17, rep=2, deadline=62
BudgetReview/Med:   focus=46, stress=5, money=14, rep=1, deadline=42
BudgetReview/Crit:  focus=74, stress=10, money=26, rep=4, deadline=136
PhoneCall/Med:      focus=32, stress=3, money=10, rep=1, deadline=24
MeetingPrep/High:   focus=60, stress=7, money=19, rep=3, deadline=80
ReportWriting/Med:  focus=36, stress=3, money=11, rep=1, deadline=30
BudgetReview/High:  focus=62, stress=7, money=20, rep=3, deadline=70
```

### 5. Expand interruption scenarios (interruptions.rs)

Expand `INTERRUPTION_SCENARIOS` from 6 to 12 entries. Add a `description: &'static str`
field to `InterruptionScenarioTemplate`. Each scenario should have a short text snippet
that the UI can show. Examples:

- "The printer is jammed again and everyone is looking at you."
- "Your manager scheduled a last-minute check-in."
- "The fire alarm went off — false alarm, but you lost your train of thought."
- "A client is on hold and nobody else is picking up."
- "Someone microwaved fish in the break room. The smell is unbearable."
- "The intern accidentally deleted a shared folder. Chaos ensues."

### 6. Update interruption UI (ui/interruption.rs)

Make the interruption popup text dynamic instead of hardcoded:
- Read from `ActiveInterruptionContext` resource
- Show the `kind` as a colored label (e.g., "MANAGER REQUEST" in orange, "SYSTEM OUTAGE" in red)
- Show the `description` as the body text
- Show the `coworker_name` if present

The popup structure stays the same (calm/panic buttons). Just replace the static text
with dynamic text from the resource.

### 7. Update task board UI (ui/task_board.rs)

- Use `TaskKind::label()` instead of the local `kind_label` function
- Add color coding per priority: Low=gray, Med=white, High=yellow, Critical=red
- Show reward: append `$XX` to each task line

### 8. Update save/mod.rs if needed

If you add any new Resource types (like `ActiveInterruptionContext`), ensure they
are either excluded from save (transient) or included with `#[serde(default)]`.
`ActiveInterruptionContext` is transient — don't save it.

### 9. Tests

Add these new tests to `tests.rs`:

1. `new_task_kinds_appear_in_seeded_board` — verify that over 10 days, all 8 TaskKinds
   appear at least once in seeded boards
2. `interruption_kind_is_deterministic_for_seed` — verify that the same seed+day produces
   the same InterruptionKind sequence
3. `active_interruption_context_populated_on_interrupt` — verify that after sending
   InterruptionEvent, the ActiveInterruptionContext resource has non-empty description
4. `active_interruption_context_cleared_on_resolve` — verify context clears after calm/panic

All existing 40 tests MUST continue to pass. Do NOT break any existing test.

## Validation

Run these commands and ensure zero errors, zero warnings:
```bash
cd dlc/city && cargo check
cd dlc/city && cargo test
cd dlc/city && cargo clippy -- -D warnings
```

Done = all three pass with zero errors and zero warnings, all existing tests still pass,
and new tests pass.
