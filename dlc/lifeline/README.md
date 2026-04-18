# Lifeline (Hospital Shift Sim DLC)

Sibling DLC to `precinct` (police). Where precinct asks "what kind of cop
will you be?", lifeline asks "what kind of healer." Same shift-cycle
topology; same 12-domain split; direct analogs for Case ↔ Patient,
Evidence ↔ Diagnostic, Patrol ↔ Rounds, PartnerArc ↔ MentorArc.

## Status

**Scaffold only.** All 12 domain plugins are empty stubs. The frozen
shared contract is complete (patients, diagnostics, NPCs, 12 core
events, mentor arc). `cargo check -p lifeline` is green.

| | |
|---|---|
| LOC | 849 (scaffold only) |
| Files | 16 |
| `cargo check -p lifeline` | ✅ green, zero warnings |
| Workspace member | ✅ registered in root Cargo.toml |
| Briefcase code attendant | ✅ ingested at gen 40814 |
| Briefcase verify profile | ✅ `lifeline_check` registered |
| Domain logic | ⬜ all 12 domains empty |
| Tests | ⬜ none yet |

## Topology

```
dlc/lifeline/
├── Cargo.toml
├── src/
│   ├── main.rs           # LifelinePlugin wiring
│   ├── shared/mod.rs     # FROZEN contract (types, events, resources)
│   └── domains/
│       ├── mod.rs        # 12-module roster
│       ├── calendar/     # shift clock, week rhythm
│       ├── player/       # movement, stamina
│       ├── world/        # 12 hospital maps (ER, ICU, OR, pharmacy…)
│       ├── ui/           # HUD, chart, dispatch screens
│       ├── patients/     # case analog — admission lifecycle
│       ├── diagnostics/  # evidence analog — labs, imaging, vitals
│       ├── rounds/       # patrol analog — walking the wards
│       ├── pharmacy/     # medication dispensing
│       ├── skills/       # Diagnostics, Surgery, BedsideManner, etc.
│       ├── economy/      # salary, supply budget
│       ├── npcs/         # doctors, nurses, admins, families
│       └── save/         # state serialization
```

## Contract summary

**Frozen types** (`shared/mod.rs`):

- `GameState` — Boot, MainMenu, OnShift, ShiftSummary, Paused, Dialogue
- `Rank` — Intern, Resident, Attending, ChiefOfMedicine
- `ShiftType` / `ShiftClock` / `DayOfWeek` — time
- `MapId` — 12 hospital locations with `display_name()`
- `Patient` / `PatientAcuity` (Routine, Urgent, Critical, Recovering, Palliative)
- `Diagnostic` / `DiagnosticKind` (Vitals, BloodPanel, Imaging, Biopsy, Interview, Observation)
- `Npc` / `NpcRole` (ChiefOfStaff, SeniorDoctor, Colleague, Nurse, Pharmacist, Administrator, Patient, Family, Specialist)
- `MentorArc` / `MentorStage` (Cool, Cordial, Respected, Trusted, Indispensable) — the partner-arc analog for the resident who shadows early-career

**Core events** (12 declared, room for briefcase to append via end-of-file anchor):

```
PatientAssignedEvent, PatientDischargedEvent, PatientDeclineEvent,
DiagnosticCollectedEvent, ShiftStartEvent, ShiftEndEvent,
MapTransitionEvent, DialogueStartEvent, DialogueEndEvent,
NpcTrustChangeEvent, ToastEvent, XpGainedEvent
```

## Briefcase bloom — attempted, blocked on transform shape

Tried `compile_chain` with `allowed_transform_ids=["hearthfield_wire_event",
"hearthfield_add_use_import"]` to wire the four patient-lifecycle events
into the patients domain plugin. Dry-run surfaced two blocking shape
mismatches:

1. **`hearthfield_add_use_import` compiled directive double-prefixed
   the path** — emitted `dlc/lifeline/dlc/lifeline/src/domains/patients/mod.rs`.
   The LLM compile step treats `dlc_path` and `target_file` as separate
   path components even when `target_file` is already repo-relative.

2. **`hearthfield_wire_event` is hardcoded for greenfield's
   topology.** Its source_request specifies: "Wire `.add_event::<events::E>()`
   into the plugin build chain for an Event that already exists in
   `events.rs`" and the compiled regex targets `src/game/mod.rs` with
   `app.init_state::<PatientsPlugin>()`.

   Lifeline (like precinct) uses `src/main.rs` + per-domain plugin
   structs, not a `src/game/mod.rs` + state types. The regex would
   find no match and `expected_count=1` would trip. Every wire-event
   step in the dry-run chain would have failed.

The transform library was tuned for `dlc/greenfield`'s specific shape.
**It does not yet port to the standard DLC topology used by precinct,
skywarden, or lifeline.** Either the transforms need variants that
accept `plugin_struct_type` (not just `plugin_state_type`) and target
files like `src/domains/{domain}/mod.rs`, or the compile_chain LLM
needs better prompting to rewrite the anchors per-DLC.

**Practical implication:** bloom the standard-topology DLCs via direct
`str_replace` edits the way precinct's Wave 8 slices did. Briefcase
becomes useful for these once the transform library is retargeted.

## Next slice candidates

- Author `patients` domain plugin body: register the four lifecycle
  events, add `PatientBoard` resource init, add a minimal system stub
  that reads `PatientAssignedEvent`. Cargo-gate via `lifeline_check`.
- Author `calendar` domain plugin body: `ShiftClock` resource init + a
  tick system + `ShiftStartEvent` / `ShiftEndEvent` emission. Small,
  self-contained, no cross-domain wiring.
- First content: Okafor analog — a Chief of Staff with authored
  dialogue profile, wired to the `dialogue_profile` lookup pattern
  the way `det_vasquez` was in precinct.
