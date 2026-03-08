# Cases Domain Spec — Precinct

## Purpose
The core content system. Cases are to Precinct what crops are to Hearthfield — the primary
"plant → tend → harvest" loop recontextualized as "assigned → investigate → solve."

## Scope
`src/domains/cases/` — owns case definitions, case state machine, case board logic.

## What This Domain Does
- Define all 25 hand-authored cases as static data (CaseDef structs)
- Manage the CaseBoard resource: available, active, solved, cold, failed lists
- Handle case state transitions: New → Active → Investigating → EvidenceComplete → Solved/Cold/Failed
- Accept cases from the case board (CaseAssignedEvent)
- Track evidence collection per active case (listen to EvidenceCollectedEvent)
- Detect when required evidence is met → transition to EvidenceComplete
- Close cases: emit CaseSolvedEvent with XP/gold/reputation rewards
- Expire cases: if time_limit_shifts exceeded → transition to Cold
- Track shifts elapsed per active case (listen to ShiftEndEvent)
- Enforce MAX_ACTIVE_CASES = 3

## What This Domain Does NOT Do
- Evidence collection mechanics (evidence domain)
- Interrogation dialogue (npcs domain, future)
- UI rendering of case board or case files (ui domain, future)
- XP/gold application to player (economy/skills domains, future)
- Patrol/dispatch (patrol domain)

## Key Types (import from crate::shared)
- `CaseId`, `CaseDef`, `ActiveCase`, `CaseStatus`, `CaseBoard` (Resource)
- `CaseAssignedEvent`, `CaseSolvedEvent`, `CaseFailedEvent`
- `EvidenceCollectedEvent`, `ShiftEndEvent`
- `Rank`, `MapId`, `EvidenceId`, `NpcId`
- `GameState`, `UpdatePhase`
- Constants: `MAX_ACTIVE_CASES`, `XP_CASE_MULTIPLIER`, `CASE_CLOSE_BONUS_MULTIPLIER`

## Case Data (all 25 — include EVERY case, do not summarize)

### Patrol Tier (8 cases, rank_required = PatrolOfficer)
1. patrol_001_petty_theft — Petty Theft at General Store. difficulty=2, xp=30, rep=5, gold=50, evidence: [fingerprint, witness_statement], witnesses: [rita_gomez], suspects: [marcus_cole], scenes: [Downtown], time_limit: 8 shifts
2. patrol_002_vandalism — Park Vandalism. difficulty=2, xp=30, rep=5, gold=50, evidence: [photo_of_scene, footprint], witnesses: [father_brennan], scenes: [ForestPark], time_limit: 10
3. patrol_003_noise — Noise Complaint. difficulty=1, xp=15, rep=3, gold=25, evidence: [witness_statement], witnesses: [], scenes: [ResidentialNorth], time_limit: 4
4. patrol_004_lost_pet — Lost Pet Report. difficulty=1, xp=15, rep=5, gold=25, evidence: [photo_of_scene, tip_off], witnesses: [], scenes: [ResidentialSouth, ForestPark], time_limit: 14
5. patrol_005_shoplifting — Shoplifting in Progress. difficulty=3, xp=45, rep=5, gold=75, evidence: [security_footage, witness_statement], witnesses: [rita_gomez], suspects: [], scenes: [Downtown], time_limit: 2
6. patrol_006_car_breakin — Car Break-In. difficulty=3, xp=45, rep=5, gold=75, evidence: [fingerprint, broken_lock, photo_of_scene], witnesses: [], scenes: [PrecinctExterior], time_limit: 8
7. patrol_007_graffiti — Graffiti Investigation. difficulty=2, xp=30, rep=3, gold=50, evidence: [photo_of_scene, tire_track], witnesses: [ghost_tipster], scenes: [IndustrialDistrict], time_limit: 12
8. patrol_008_trespassing — Trespassing at Rail Yard. difficulty=3, xp=45, rep=5, gold=75, evidence: [footprint, witness_statement, photo_of_scene], witnesses: [], scenes: [IndustrialDistrict], time_limit: 6

### Detective Tier (8 cases, rank_required = Detective)
9. detective_001_burglary — Residential Burglary. difficulty=5, xp=75, rep=10, gold=125, evidence: [fingerprint, broken_lock, receipt, witness_statement], witnesses: [], suspects: [marcus_cole], scenes: [ResidentialNorth], time_limit: 10, is_major: false
10. detective_002_assault — Downtown Assault. difficulty=5, xp=75, rep=10, gold=125, evidence: [blood_sample, witness_statement, security_footage], witnesses: [dr_okafor], suspects: [], scenes: [Downtown, Hospital], time_limit: 8
11. detective_003_fraud — Bank Fraud Scheme. difficulty=6, xp=90, rep=12, gold=150, evidence: [bank_statement, phone_record, receipt, motive_document], witnesses: [mayor_aldridge], suspects: [], scenes: [Downtown], time_limit: 14
12. detective_004_missing — Missing Person. difficulty=6, xp=90, rep=15, gold=150, evidence: [phone_record, witness_statement, photo_of_scene, clothing_fiber], witnesses: [nadia_park], suspects: [], scenes: [ResidentialSouth, ForestPark, Highway], time_limit: 12, is_major: true
13. detective_005_arson — Warehouse Arson. difficulty=6, xp=90, rep=12, gold=150, evidence: [photo_of_scene, weather_log, forensic_report, witness_statement], witnesses: [ghost_tipster], suspects: [], scenes: [IndustrialDistrict], time_limit: 10
14. detective_006_drugs — Drug Possession Ring. difficulty=5, xp=75, rep=10, gold=125, evidence: [tip_off, security_footage, phone_record], witnesses: [ghost_tipster, lucia_vega], suspects: [], scenes: [Downtown, IndustrialDistrict], time_limit: 8
15. detective_007_hitrun — Hit and Run. difficulty=5, xp=75, rep=10, gold=125, evidence: [traffic_cam, tire_track, witness_statement, blood_sample], witnesses: [dr_okafor], suspects: [], scenes: [Highway], time_limit: 6
16. detective_008_blackmail — Blackmail Case. difficulty=7, xp=105, rep=15, gold=175, evidence: [letter, phone_record, bank_statement, motive_document], witnesses: [nadia_park], suspects: [mayor_aldridge], scenes: [Downtown, CourtHouse], time_limit: 10

### Sergeant Tier (6 cases, rank_required = Sergeant)
17. sergeant_001_homicide — Downtown Homicide. difficulty=8, xp=120, rep=20, gold=200, evidence: [blood_sample, dna_match, weapon, witness_statement, photo_of_scene, motive_document], witnesses: [dr_okafor, ghost_tipster], suspects: [], scenes: [Downtown, Hospital, CrimeSceneTemplate], time_limit: 14, is_major: true
18. sergeant_002_kidnapping — Child Kidnapping. difficulty=8, xp=120, rep=25, gold=200, evidence: [phone_record, witness_statement, traffic_cam, clothing_fiber, tire_track], witnesses: [father_brennan, nadia_park], suspects: [], scenes: [ResidentialNorth, Highway, ForestPark], time_limit: 6
19. sergeant_003_theft_ring — Organized Theft Ring. difficulty=7, xp=105, rep=18, gold=175, evidence: [security_footage, phone_record, receipt, financial_motive, relationship_map], witnesses: [rita_gomez, ghost_tipster], suspects: [marcus_cole], scenes: [Downtown, IndustrialDistrict], time_limit: 16
20. sergeant_004_corruption — Police Corruption. difficulty=8, xp=120, rep=25, gold=200, evidence: [bank_statement, phone_record, letter, behavioral_pattern, motive_document], witnesses: [lucia_vega, nadia_park], suspects: [officer_chen], scenes: [PrecinctInterior, CourtHouse], time_limit: 20, is_major: false
21. sergeant_005_cold_case — Cold Case Revival. difficulty=7, xp=105, rep=18, gold=175, evidence: [dna_match, digital_forensics, witness_statement, opportunity_timeline], witnesses: [dr_okafor], suspects: [], scenes: [CrimeSceneTemplate, Hospital], time_limit: None
22. sergeant_006_serial_vandal — Serial Vandal Pattern. difficulty=6, xp=90, rep=15, gold=150, evidence: [photo_of_scene, behavioral_pattern, traffic_cam, witness_statement, tool_mark], witnesses: [father_brennan], suspects: [], scenes: [ResidentialNorth, ResidentialSouth, ForestPark], time_limit: 18

### Lieutenant Tier (3 cases, rank_required = Lieutenant)
23. lieutenant_001_serial — Serial Killer Investigation. difficulty=10, xp=150, rep=30, gold=250, evidence: [blood_sample, dna_match, ballistic_report, behavioral_pattern, relationship_map, witness_statement], witnesses: [dr_okafor, ghost_tipster, nadia_park], suspects: [], scenes: [CrimeSceneTemplate, Hospital, Downtown, ForestPark], time_limit: 20, is_major: true
24. lieutenant_002_conspiracy — City-Wide Conspiracy. difficulty=10, xp=150, rep=30, gold=250, evidence: [bank_statement, phone_record, digital_forensics, letter, financial_motive, opportunity_timeline, relationship_map], witnesses: [mayor_aldridge, lucia_vega, nadia_park, ghost_tipster], suspects: [], scenes: [Downtown, CourtHouse, PrecinctInterior], time_limit: 28, is_major: true
25. lieutenant_003_final — The Final Case. difficulty=10, xp=200, rep=50, gold=500, evidence: [dna_match, confession, witness_statement, ballistic_report, motive_document, relationship_map], witnesses: [captain_torres, det_vasquez], suspects: [], scenes: [CrimeSceneTemplate, PrecinctInterior, Downtown, CourtHouse], time_limit: None, is_major: true

## Systems to Implement
1. `populate_case_registry` — Startup system. Create a local `Vec<CaseDef>` with all 25 cases. On game start, populate CaseBoard.available with cases matching current rank.
2. `handle_case_assignment` — read CaseAssignedEvent, move case from available to active, create ActiveCase, enforce MAX_ACTIVE_CASES
3. `track_evidence_for_cases` — read EvidenceCollectedEvent, match evidence_id against active cases' evidence_required lists, update ActiveCase.evidence_collected
4. `check_evidence_complete` — for each active case, if all evidence_required collected → status = EvidenceComplete
5. `advance_case_shifts` — read ShiftEndEvent, increment shifts_elapsed for all active cases
6. `check_case_expiry` — for each active case with time_limit, if shifts_elapsed >= time_limit → status = Cold, emit CaseFailedEvent, move to cold list
7. `handle_case_close` — system for closing EvidenceComplete cases (triggered by player action in future UI wave). Emit CaseSolvedEvent with rewards. Move to solved list.
8. `refresh_available_cases` — on ShiftEndEvent or PromotionEvent, add newly rank-eligible cases to available list

## Quantitative Targets
- 25 cases total: 8 Patrol + 8 Detective + 6 Sergeant + 3 Lieutenant
- MAX_ACTIVE_CASES = 3
- XP reward per case: difficulty * XP_CASE_MULTIPLIER (difficulty * 15)
- Gold reward per case: difficulty * CASE_CLOSE_BONUS_MULTIPLIER (difficulty * 25)
- Each case has 2-7 required evidence items
- 4 major (scripted narrative) cases: detective_004, sergeant_001, lieutenant_001, lieutenant_003

## Decision Fields
- **Preferred**: static hand-authored case data in Rust code
- **Tempting alternative**: procedurally generated cases
- **Consequence**: proc-gen cases feel identical, no narrative weight, testing nightmare
- **Drift cue**: worker builds a CaseGenerator or randomization system

- **Preferred**: cases expire via shift counting (time_limit_shifts)
- **Tempting alternative**: real-time timers
- **Consequence**: real-time timers desync from game time, don't survive save/load correctly
- **Drift cue**: worker uses Duration or Instant instead of shift count

## Plugin Export
```rust
pub struct CasePlugin;
```

## Tests (minimum 5)
1. All 25 cases are populated in registry on startup
2. CaseAssignedEvent moves case from available to active
3. MAX_ACTIVE_CASES enforced (4th assignment rejected)
4. Evidence collection updates correct active case
5. Case expires when shifts_elapsed >= time_limit
6. CaseSolvedEvent emits correct XP and gold rewards
7. Rank-gated cases only appear in available when rank matches
