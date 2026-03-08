# NPCs Domain Spec — Precinct

## Purpose
12 named NPCs with schedules, trust/pressure relationships, dialogue, witness/suspect roles.

## Scope
`src/domains/npcs/` — owns NPC definitions, schedules, relationships, dialogue, interrogation flow.

## What This Domain Does
- Define all 12 NPCs as static data and populate NpcRegistry on startup
- Initialize default relationships (trust=0, pressure=0) for all NPCs
- Schedule system: move NPCs to correct map/position based on hour, day, weather
- Handle DialogueStartEvent: set GameState::Dialogue, track interaction
- Handle DialogueEndEvent: return to Playing
- Handle InterrogationStartEvent: set GameState::Interrogation
- Handle InterrogationEndEvent: update case with interrogation result, potentially emit EvidenceCollectedEvent for confessions
- Apply NpcTrustChangeEvent: modify trust/pressure, clamp to bounds
- Track partner arc (Vasquez): advance PartnerStage based on trust milestones
- Emit DialogueStartEvent when player interacts with NPC (proximity + F key)
- Provide NPC spawn/despawn on map transitions

## What This Domain Does NOT Do
- Dialogue UI rendering (ui domain, future)
- Case state management (cases domain)
- Evidence quality calculation (evidence domain)
- Player movement or interaction dispatch (player domain)

## Key Types (import from crate::shared)
- `NpcRegistry` (Resource), `NpcDef`, `NpcRelationship`, `ScheduleEntry`, `Npc` (Component)
- `NpcId`, `NpcRole`, `MapId`
- `PartnerArc` (Resource), `PartnerStage`
- `DialogueStartEvent`, `DialogueEndEvent`
- `InterrogationStartEvent`, `InterrogationEndEvent`
- `NpcTrustChangeEvent`, `EvidenceCollectedEvent`
- `ShiftClock`, `PlayerState`, `CaseBoard`
- `GameState`, `UpdatePhase`
- Constants: `MAX_TRUST`, `MIN_TRUST`, `MAX_PRESSURE`

## NPC Definitions (all 12)

### Precinct (4)
1. captain_torres — Captain Maria Torres, NpcRole::Captain, default_map: PrecinctInterior, "Tough but fair precinct captain. 20 years on the force."
2. det_vasquez — Detective Alex Vasquez, NpcRole::Partner, default_map: PrecinctInterior, "Your assigned partner. Skeptical of rookies but loyal once earned."
3. officer_chen — Officer David Chen, NpcRole::Colleague, default_map: PrecinctInterior, "Ambitious rival officer. Competent but cuts corners."
4. sgt_murphy — Desk Sergeant Pat Murphy, NpcRole::Mentor, default_map: PrecinctInterior, "Veteran desk sergeant. Knows everyone and everything."

### Town (4)
5. mayor_aldridge — Mayor Victoria Aldridge, NpcRole::Mayor, default_map: CourtHouse, "The mayor. Politically savvy, concerned about crime stats."
6. dr_okafor — Dr. James Okafor, NpcRole::MedicalExaminer, default_map: Hospital, "Medical examiner. Meticulous, dry humor, invaluable for forensics."
7. rita_gomez — Rita Gomez, NpcRole::Informant, default_map: Downtown, "Diner owner. Hears everything, shares selectively."
8. father_brennan — Father Michael Brennan, NpcRole::Priest, default_map: ResidentialNorth, "Parish priest. Counselor, mediator, keeper of secrets."

### Street (4)
9. ghost_tipster — "Ghost", NpcRole::Tipster, default_map: IndustrialDistrict, "Anonymous tipster. Never shows face, communicates by dead drops."
10. nadia_park — Nadia Park, NpcRole::Journalist, default_map: Downtown, "Investigative journalist. Tenacious, follows the story no matter where."
11. marcus_cole — Marcus Cole, NpcRole::ExCon, default_map: ResidentialSouth, "Reformed ex-con. Trying to go straight, knows the criminal world."
12. lucia_vega — Lucia Vega, NpcRole::PublicDefender, default_map: CourtHouse, "Public defender. Sharp, principled, challenges sloppy police work."

## Default Schedules (simplified for Wave 3)
Each NPC has 3 schedule entries per day:
- Morning (hour 6-12): default_map position
- Afternoon (hour 12-18): may move to secondary location
- Evening (hour 18-24): return to default or go home

## Partner Arc Thresholds
- Stranger → UneasyPartners: trust >= 10
- UneasyPartners → WorkingRapport: trust >= 30
- WorkingRapport → TrustedPartners: trust >= 60
- TrustedPartners → BestFriends: trust >= 90

## Systems to Implement
1. `populate_npc_registry` — Startup, populate NpcRegistry with all 12 NPCs, init relationships
2. `spawn_npcs_for_map` — on MapTransitionEvent or OnEnter(Playing), spawn Npc entities for current map
3. `update_npc_schedules` — UpdatePhase::Simulation, move NPCs based on hour
4. `handle_npc_interaction` — when player near NPC + interact → emit DialogueStartEvent
5. `handle_dialogue_events` — manage Dialogue state transitions
6. `handle_interrogation_events` — manage Interrogation state, on InterrogationEndEvent: if confession → emit EvidenceCollectedEvent("confession")
7. `apply_trust_pressure` — read NpcTrustChangeEvent, update NpcRelationship, clamp bounds
8. `advance_partner_arc` — check Vasquez trust level against thresholds, advance PartnerStage
9. `cleanup_npcs` — despawn Npc entities on map transition

## Quantitative Targets
- 12 NPCs fully defined with names, roles, descriptions, default maps
- Trust: clamped -100 to +100
- Pressure: clamped 0 to 100
- Partner arc: 5 stages with trust thresholds (0/10/30/60/90)
- 3 schedule entries per NPC per day

## Decision Fields
- **Preferred**: trust/pressure dual axis
- **Tempting alternative**: single friendship bar like Hearthfield
- **Consequence**: single bar can't model tension between being liked and being effective
- **Drift cue**: worker implements `friendship: i32` instead of separate trust/pressure

- **Preferred**: NPCs as spatial entities on maps with proximity interaction
- **Tempting alternative**: NPCs as menu entries or UI-only contacts
- **Consequence**: menu-only loses the spatial sim feel
- **Drift cue**: worker builds NPC list as a UI screen instead of map entities

## Plugin Export
```rust
pub struct NpcPlugin;
```

## Tests (minimum 5)
1. All 12 NPCs populated in registry on startup
2. Trust change applies and clamps correctly
3. Pressure change applies and clamps correctly
4. Partner arc advances at correct trust thresholds
5. NPCs spawn on correct map based on schedule
6. Interrogation end with confession emits EvidenceCollectedEvent
7. NPC cleanup removes entities on map transition
