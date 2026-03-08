# Wave 5 Orchestrator: Event Wiring + UI Screens

You are the Wave 5 orchestrator for the Precinct police DLC.

## CRITICAL INSTRUCTION (applies to ALL work)
After each implementation step, audit your work from the player's perspective. Boot the game mentally. Walk through what a player sees, clicks, and experiences. If your work isn't player-reachable and player-visible, it's not done.

## Task 1: Wire 7 Broken Events
These events have senders but no readers, or readers but no senders. Fix ALL of them.

1. **ToastEvent** — 12+ senders, ZERO readers. Players get no feedback. Add a toast notification system: spawn a UI text node on ToastEvent, display for duration_secs, then despawn. Show at bottom-center of screen. This is the HIGHEST PRIORITY — without it, players see nothing when they collect evidence, solve cases, earn XP, get promoted, etc.

2. **DispatchCallEvent** — patrol generates calls but nothing displays them. Add a radio notification in the HUD: when DispatchCallEvent fires, show flashing "DISPATCH" text with call description. Player should see what the call is.

3. **InterrogationStartEvent** — nobody sends it. When player interacts with an NPC who is a suspect in an active case, emit InterrogationStartEvent. Wire in npcs or precinct domain.

4. **DialogueEndEvent** — nobody sends it. When dialogue state exits, emit DialogueEndEvent. Wire in npcs domain.

5. **LoadRequestEvent** — save system reads it but nothing sends it. Add a "Load Game" button to the main menu and/or pause menu that emits LoadRequestEvent with the slot number.

6. **EvidenceProcessedEvent** — evidence domain sends it but nobody reads it. Add a reader that emits a ToastEvent("Evidence analyzed: {name}") when evidence processing completes.

7. **PlaySfxEvent / PlayMusicEvent** — senders exist but no audio system reads them. Add stub readers that log the event (actual audio in Wave 7). At minimum, don't let these be dead events.

## Task 2: UI Screens (5 new screens)

All screens use Bevy native UI (Node, Button, Text). Simple colored backgrounds, plain text. No sprite atlas yet.

### 2A. Case File Screen (GameState::CaseFile)
- Trigger: player interacts with case board when they have active cases
- Shows: case name, description, required evidence (collected/missing), witnesses, suspects
- Shows evidence quality for collected pieces
- "Close Case" button when status == EvidenceComplete → emits CaseSolvedEvent
- "Back" button → return to Playing

### 2B. Skill Tree Screen (GameState::SkillTree)
- Trigger: Tab key or menu button
- Shows: 4 columns (Investigation, Interrogation, Patrol, Leadership)
- Each column: 5 perk slots, unlocked ones highlighted, next available shown
- Available skill points displayed
- Click unlocked-but-unspent perk → emit SkillPointSpentEvent
- "Back" button → Playing

### 2C. Career View (GameState::CareerView)
- Trigger: menu button or precinct captain interaction
- Shows: current rank, total XP, cases solved, reputation
- Shows promotion requirements for next rank (what's met, what's not)
- Department budget
- "Back" button → Playing

### 2D. Interrogation Screen (GameState::Interrogation)
- Trigger: InterrogationStartEvent
- Shows: NPC name, NPC portrait placeholder, trust/pressure meters
- 3 dialogue options: "Build Trust" (+trust), "Apply Pressure" (+pressure), "Ask About Evidence" (neutral)
- Each option emits NpcTrustChangeEvent with appropriate deltas
- "End Interrogation" → InterrogationEndEvent → back to Playing
- If pressure > 80, chance of confession → emit EvidenceCollectedEvent("confession")

### 2E. Evidence Examination (GameState::EvidenceExam)
- Trigger: interact with evidence terminal when evidence exists
- Shows: list of all evidence in EvidenceLocker
- Each piece: name, category, quality bar, processing state, linked case
- "Process All Raw" button → starts processing
- "Back" button → Playing

## Task 3: Input Wiring
- Tab → GameState::SkillTree
- J or N → GameState::CaseFile (notebook/journal)
- C → GameState::CareerView
- These keys must be captured in the player input system

## Validation (run ALL of these)
```bash
cargo check -p precinct
cargo test -p precinct
cargo clippy -p precinct -- -D warnings
cd dlc/police && shasum -a 256 -c .contract.sha256
```

## Event Connectivity Gate (MUST pass after this wave)
Every event type registered in main.rs must have:
- At least 1 system that sends it (EventWriter::send)
- At least 1 system that reads it (EventReader)
Run: grep for every add_event in main.rs, verify each has writer AND reader.
Report any violations.

## First-60-Seconds Check
After implementation, mentally trace:
1. Boot → Loading → MainMenu (auto-transition)
2. "New Game" → Playing
3. Player spawns, HUD visible, clock ticking
4. Walk to case board, press F → case assigned → TOAST SHOWS "Case assigned: Petty Theft"
5. Walk to exit → PrecinctExterior
6. Dispatch call arrives → RADIO FLASHES on HUD
7. Press Escape → Pause → "Save" works, "Load" works
8. Press Tab → skill tree visible
9. Every interaction produces visible feedback via ToastEvent

If ANY step has no visible player feedback, it's broken. Fix it.

Do NOT modify dlc/police/src/shared/mod.rs.
