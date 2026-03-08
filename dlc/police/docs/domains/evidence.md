# Evidence Domain Spec — Precinct

## Purpose
Evidence collection, processing, and management. Replaces Hearthfield's item gathering
(foraging, fishing, mining) with a police investigation mechanic.

## Scope
`src/domains/evidence/` — owns evidence definitions, collection, processing pipeline, locker management.

## What This Domain Does
- Define all 30 evidence types as static data
- Handle evidence collection: player at crime scene + interact → EvidenceCollectedEvent
- Calculate evidence quality: base_quality + skill_bonus - weather_penalty
- Manage EvidenceLocker resource: store collected pieces
- Process evidence: raw → processing (1 shift) → analyzed (on ShiftEndEvent)
- Link evidence to cases via case_id
- Emit EvidenceProcessedEvent when processing completes

## What This Domain Does NOT Do
- Case state machine (cases domain)
- Player movement to crime scenes (player/world domains)
- UI for evidence examination (ui domain, future)
- Skill level lookups (skills domain, future — use default 0 for Wave 2)

## Key Types (import from crate::shared)
- `EvidenceId`, `EvidenceCategory`, `EvidenceProcessingState`, `EvidencePiece`, `EvidenceLocker` (Resource)
- `EvidenceCollectedEvent`, `EvidenceProcessedEvent`
- `ShiftEndEvent`, `ShiftClock` (read weather)
- `CaseId`, `MapId`
- `GameState`, `UpdatePhase`
- Constants: `EVIDENCE_BASE_QUALITY`, `EVIDENCE_SKILL_BONUS`, `EVIDENCE_MAX_QUALITY`, `EVIDENCE_WEATHER_PENALTY`, `XP_PER_EVIDENCE`

## Evidence Type Definitions (all 30)

### Physical (5)
- fingerprint — "Fingerprint lifted from surface"
- footprint — "Shoe impression found at scene"
- weapon — "Weapon recovered from scene"
- clothing_fiber — "Fiber sample from clothing"
- tool_mark — "Tool impression on lock or surface"

### Documentary (5)
- receipt — "Transaction receipt"
- letter — "Written correspondence"
- phone_record — "Call log or text messages"
- security_footage — "Security camera recording"
- bank_statement — "Financial account records"

### Testimonial (5)
- witness_statement — "Sworn witness account"
- alibi — "Alibi documentation"
- confession — "Suspect confession"
- tip_off — "Anonymous tip"
- recording_911 — "Emergency call recording"

### Forensic (5)
- blood_sample — "Blood evidence"
- dna_match — "DNA analysis result"
- ballistic_report — "Ballistics analysis"
- toxicology — "Toxicology screen results"
- digital_forensics — "Digital device analysis"

### Environmental (5)
- photo_of_scene — "Crime scene photograph"
- weather_log — "Weather conditions record"
- traffic_cam — "Traffic camera footage"
- broken_lock — "Damaged lock or entry point"
- tire_track — "Tire impression"

### Circumstantial (5)
- motive_document — "Evidence of motive"
- opportunity_timeline — "Timeline of suspect movements"
- behavioral_pattern — "Pattern of behavior analysis"
- financial_motive — "Financial gain evidence"
- relationship_map — "Relationship connections diagram"

## Systems to Implement
1. `populate_evidence_registry` — Startup. Create local HashMap<EvidenceId, EvidenceDef> with all 30 types (id, name, category, description).
2. `collect_evidence` — read EvidenceCollectedEvent. Create EvidencePiece with:
   - quality = min(EVIDENCE_MAX_QUALITY, EVIDENCE_BASE_QUALITY + (skill_level * EVIDENCE_SKILL_BONUS) - weather_penalties)
   - weather_penalties: sum of 0.1 for each active condition (Rainy, Foggy; Night counts if hour >= 22 or hour < 6)
   - processing_state = Raw
   - collected_shift = ShiftClock.shift_number
   - collected_map = from event or PlayerState.position_map
   - Add to EvidenceLocker.pieces
3. `process_evidence` — read ShiftEndEvent. For each piece with state == Processing → set to Analyzed, emit EvidenceProcessedEvent.
4. `start_processing` — when player interacts with evidence room (future UI trigger). For each Raw piece → set to Processing. (For Wave 2: provide a public function `start_processing_evidence(locker, evidence_id)` that can be called from precinct domain.)

## Quantitative Targets
- 30 evidence types: 5 per category × 6 categories
- Quality formula: min(0.95, 0.5 + skill*0.05 - weather*0.1)
- Processing time: 1 shift (set to Processing on interact, Analyzed on next ShiftEndEvent)
- Default skill_level for Wave 2: 0 (no skills domain yet)

## Decision Fields
- **Preferred**: quality calculated at collection time with formula from contract
- **Tempting alternative**: fixed quality per evidence type
- **Consequence**: no skill progression payoff, investigation skill tree becomes pointless
- **Drift cue**: worker hardcodes quality to 0.5 or 1.0 for all evidence

- **Preferred**: processing takes 1 shift (state change on ShiftEndEvent)
- **Tempting alternative**: instant processing or real-time timer
- **Consequence**: instant removes the time-management tension; real-time desync from game time
- **Drift cue**: worker sets Analyzed immediately on collection

## Plugin Export
```rust
pub struct EvidencePlugin;
```

## Tests (minimum 5)
1. All 30 evidence types registered
2. Evidence quality formula: base 0.5 with skill 0 and clear weather = 0.5
3. Evidence quality with weather penalty: rainy = 0.4
4. Evidence quality caps at EVIDENCE_MAX_QUALITY (0.95)
5. Processing state transitions: Raw → Processing → Analyzed over 1 shift
6. EvidenceProcessedEvent emits when processing completes
7. Evidence links to correct case_id
