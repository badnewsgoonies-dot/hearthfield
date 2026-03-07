# Precinct — A Police Sim (Hearthfield DLC)

## Vision

A cozy-but-tense police simulator in the spirit of Harvest Moon. Where Hearthfield
asks "what kind of farmer will you be?", Precinct asks "what kind of cop will you be?"
Top-down 16×16 pixel art. Daily shift rhythm, case investigation, evidence collection,
NPC relationships (partner, captain, informants, witnesses, suspects), rank progression,
and a gentle narrative about a rookie officer finding their place in a small-town precinct.

This DLC is designed to become the new core game base. Its architecture must be cleaner,
more modular, and more content-complete than Hearthfield's base game.

## Art Style

- 16×16 pixel art, muted blue/grey/amber palette (noir-adjacent but not grim)
- 2x or 3x upscale rendering (nearest-neighbor), matching Hearthfield
- Four distinct "season" palettes mapped to rank tiers:
  - Patrol Officer (blue/grey — rookie, daytime shifts)
  - Detective (amber/brown — investigation focus, night access)
  - Sergeant (green/silver — leadership, wider jurisdiction)
  - Lieutenant (gold/navy — command cases, full city access)
- Day/night tint overlay: warm morning → neutral midday → amber dusk → blue night
- Weather affects visibility and NPC behavior (rain, fog, clear, snow)

## Core Loop

```
Start shift → Check case board at precinct → Patrol / respond to dispatch calls →
Arrive at scene → Investigate (collect evidence, interview witnesses) →
Return to precinct → Process evidence → Interrogate suspects →
File case report → End shift → Rank XP awarded → Sleep →
Next shift → Cases progress / new cases arrive
```

### "Better Than Hearthfield" Metrics (non-negotiable targets)

| Metric | Hearthfield | Precinct Target |
|--------|-------------|-----------------|
| First-60-seconds path | Boot → farm → water crop | Boot → precinct → first dispatch call |
| Content units (items/cases) | 15 crops + 20 fish + 20 recipes = 55 | 25 cases + 30 evidence types + 15 perks = 70 |
| NPC depth | 10 NPCs, gift system, 2 romance | 12 NPCs, trust/pressure system, partner arc |
| Save/load coverage | Full state serialization | Full state + mid-case progress + evidence chain |
| Maps | 10 | 12 |
| Core systems | 12 domains | 12 domains |
| LOC target | 50,440 | ≥55,000 (stretch: 65,000) |
| Test target | Hearthfield has minimal tests | ≥100 tests, every domain must have ≥5 |

---

## 12 Domains

### 1. Calendar & Shifts (`calendar`)

Recontextualization: Hearthfield's seasons → rank tiers. Hearthfield's days → shifts.

- **Shift cycle:** 3 shifts per week (Mon/Wed/Fri OR Tue/Thu/Sat), 8 hours each
  - Morning shift: 06:00–14:00
  - Afternoon shift: 14:00–22:00
  - Night shift: 22:00–06:00 (unlocked at Detective rank)
- **Time scale:** 1 real second = 2 game minutes (1 shift = ~4 real minutes)
- **Week:** 7 days, 3 on-duty + 4 off-duty (off-duty: explore, build relationships, train)
- **Rank progression:** replaces seasons as the macro-progression axis
  - Patrol Officer: shifts 1–28 (tutorial arc, simple cases)
  - Detective: shifts 29–56 (investigation cases, night access)
  - Sergeant: shifts 57–84 (leadership, multi-scene cases)
  - Lieutenant: shifts 85–112 (command, city-wide cases, endgame)
- **Calendar events:** 1 major case per rank tier (scripted narrative case)
- **Weather:** Sunny, Rainy, Foggy, Snowy — affects patrol visibility and NPC schedules
- **Time pauses:** in menus, dialogue, evidence examination, interrogation

**Decision field:**
- **Preferred:** shift-based progression (finite shifts per rank tier)
- **Tempting alternative:** real-time day/night with infinite days
- **Consequence:** infinite days removes pacing pressure; cases never feel urgent
- **Drift cue:** worker implements open-ended day counter without rank-tier gating

### 2. Player (`player`)

The player character is a police officer. Movement, interaction, and fatigue mirror Hearthfield's player but recontextualized.

- 4-directional movement with walk/run animations
- **Equipment slots:** badge (always), sidearm (holstered/drawn), flashlight (night), radio, notebook
- **Fatigue system:** 100 base, actions cost 2–10 fatigue
  - Investigating a scene: 5 fatigue
  - Chasing a suspect: 10 fatigue
  - Interrogation: 8 fatigue
  - Patrol walking: 1 fatigue per 5 minutes
  - Coffee break: restores 25 fatigue, costs 15 minutes
  - Meal break: restores 50 fatigue, costs 30 minutes
- **Stress system:** 0–100, separate from fatigue
  - Witnessing violence: +15 stress
  - Failed interrogation: +10 stress
  - Successful case close: -20 stress
  - Partner conversation: -10 stress
  - Stress > 80: performance penalties (slower evidence collection, dialogue options locked)
- Inventory: 12 evidence slots + 6 personal item slots
- Collision with terrain, objects, NPCs

**Decision field:**
- **Preferred:** fatigue AND stress as dual resource constraints
- **Tempting alternative:** single "energy" bar like Hearthfield
- **Consequence:** single bar loses the tension between physical and mental strain
- **Drift cue:** worker merges stress into fatigue or vice versa

### 3. World & Maps (`world`)

Tilemap-based, 16×16 tiles. Maps defined as RON files (matching Hearthfield pattern).

- **12 maps:**
  1. Precinct interior (32×24) — hub, case board, evidence room, offices
  2. Precinct exterior / parking lot (24×24) — patrol car, entrance
  3. Downtown (48×48) — shops, restaurant, bank, alley network
  4. Residential North (40×40) — houses, park, school
  5. Residential South (40×40) — apartments, convenience store, laundromat
  6. Industrial District (32×32) — warehouses, docks, rail yard
  7. Highway / outskirts (48×24) — speed trap, rest stop, gas station
  8. Forest Park (32×32) — trails, campsite, ranger station
  9. Crime scene template (24×24) — dynamically dressed per case
  10. Hospital (24×24) — morgue, ER, witness interviews
  11. Court House (24×16) — case filing, judge's chambers
  12. Player's apartment (16×16) — off-duty hub, sleep, personal items

- Map transitions: edge-walking, door interaction, or patrol car (fast travel between exteriors)
- Collision layer: solid, walkable, water, crime-scene-tape (restricted)
- Day/night visual variants for all exterior maps
- Weather overlay system: rain particles, fog opacity, snow accumulation

**Decision field:**
- **Preferred:** 12 maps with crime scene template that gets dynamically dressed
- **Tempting alternative:** 20+ small maps with unique crime scenes each
- **Consequence:** 20 maps = massive asset burden, most become dead content
- **Drift cue:** worker creates many maps without the template/dressing system

### 4. Cases (`cases`)

The core content system. Cases replace crops as the primary "plant → tend → harvest" loop.

- **25 cases total across 4 rank tiers:**
  - Patrol (cases 1–8): petty theft, vandalism, noise complaint, lost pet, shoplifting, car break-in, graffiti, trespassing
  - Detective (cases 9–16): burglary, assault, fraud, missing person, arson, drug possession, hit-and-run, blackmail
  - Sergeant (cases 17–22): homicide, kidnapping, organized theft ring, corruption, cold case, serial vandal
  - Lieutenant (cases 23–25): serial killer, major conspiracy, final narrative case

- **Case structure (each case has):**
  - `CaseId: String` (e.g., "patrol_001_petty_theft")
  - `rank_required: Rank`
  - `evidence_required: Vec<EvidenceId>` — minimum evidence to close
  - `witnesses: Vec<NpcId>` — who can be interviewed
  - `suspects: Vec<NpcId>` — who can be interrogated
  - `scenes: Vec<MapId>` — where evidence can be found
  - `time_limit_shifts: Option<u8>` — some cases expire (cold after N shifts)
  - `reward_xp: u32`
  - `reward_reputation: i32`
  - `difficulty: u8` (1–10)
  - `narrative_text: String` — case board description

- **Case states:** `New → Active → Investigating → EvidenceComplete → Interrogating → Solved | Cold | Failed`
- **Max 3 active cases simultaneously** (cognitive load constraint)
- **1 major scripted case per rank tier** (narrative arc, multi-shift, unique dialogue)

**Quantitative targets:**
- 8 Patrol cases, 8 Detective cases, 6 Sergeant cases, 3 Lieutenant cases = 25 total
- Each case requires 2–6 evidence items to close
- Average case takes 1–3 shifts to solve
- Major cases take 4–8 shifts

**Decision field:**
- **Preferred:** fixed case pool with rank gating and time pressure
- **Tempting alternative:** procedurally generated infinite cases
- **Consequence:** proc-gen cases feel identical; no narrative weight; testing nightmare
- **Drift cue:** worker builds a case generator instead of hand-authored case data

### 5. Evidence (`evidence`)

Evidence collection replaces Hearthfield's item gathering (foraging, fishing, mining).

- **30 evidence types across 6 categories:**
  - Physical (5): fingerprint, footprint, weapon, clothing fiber, tool mark
  - Documentary (5): receipt, letter, phone record, security footage, bank statement
  - Testimonial (5): witness statement, alibi, confession, tip-off, 911 recording
  - Forensic (5): blood sample, DNA match, ballistic report, toxicology, digital forensics
  - Environmental (5): photo of scene, weather log, traffic cam, broken lock, tire track
  - Circumstantial (5): motive document, opportunity timeline, behavioral pattern, financial motive, relationship map

- **Evidence has quality:** `quality: f32` (0.0–1.0)
  - Quality affects interrogation effectiveness and case close rating
  - Higher player skill → higher evidence quality
  - `base_quality = 0.5 + (skill_level * 0.05)` capped at 0.95
  - Rain/fog/night: quality penalty of 0.1 per condition

- **Evidence processing:** raw evidence → precinct lab → processed evidence (takes 1 shift)
- **Evidence chain:** each piece links to a case via `CaseId`; orphaned evidence is flagged

**Decision field:**
- **Preferred:** 30 distinct evidence types with quality scaling
- **Tempting alternative:** generic "clue" item with no differentiation
- **Consequence:** generic clues make investigation feel like fetch quests
- **Drift cue:** worker defines fewer than 5 evidence categories or omits quality

### 6. NPCs & Relationships (`npcs`)

12 named NPCs with schedules, trust/pressure dynamics, and role-based dialogue.

- **12 NPCs:**
  - **Precinct (4):** Captain Torres (boss), Detective Vasquez (partner), Officer Chen (rival), Desk Sergeant Murphy (mentor)
  - **Town (4):** Mayor Aldridge (political), Dr. Okafor (medical examiner), Rita Gomez (diner owner/informant), Father Brennan (priest/counselor)
  - **Street (4):** "Ghost" (anonymous tipster), Nadia Park (journalist), Marcus Cole (reformed ex-con), Lucia Vega (public defender)

- **Relationship system: trust + pressure (dual axis)**
  - Trust: -100 to +100 — built through honest dialogue, keeping promises, helping
  - Pressure: 0 to 100 — built through aggressive interrogation, threats, leverage
  - NPCs respond differently based on trust/pressure balance
  - High trust: NPCs volunteer information, offer side quests
  - High pressure: NPCs comply but may give false info, relationship degrades

- **Partner arc (Vasquez):** deepest relationship track
  - Stages: Stranger → Uneasy Partners → Working Rapport → Trusted Partners → Best Friends
  - Partner provides gameplay bonuses at higher stages (faster evidence, dialogue hints)
  - 10 partner-specific dialogue events across the game

- **NPC schedules:** location varies by time/day/weather (matching Hearthfield pattern)
- **Gift system replaced by favor system:** do favors for NPCs to build trust
- **NPCs as witnesses/suspects:** same NPC can be witness in one case, suspect in another

**Decision field:**
- **Preferred:** trust/pressure dual axis
- **Tempting alternative:** single friendship bar like Hearthfield
- **Consequence:** single bar can't model the tension between being liked and being effective
- **Drift cue:** worker implements `friendship: i32` instead of separate trust/pressure

### 7. Economy & Career (`economy`)

- **Salary:** base pay per shift, modified by rank and performance
  - Patrol: 80 gold/shift
  - Detective: 120 gold/shift
  - Sergeant: 160 gold/shift
  - Lieutenant: 200 gold/shift
  - Case close bonus: `difficulty * 25` gold
  - Failed case penalty: -50 gold
- **Expenses:** equipment maintenance (20/week), apartment rent (100/week), coffee (5/break)
- **Reputation:** -100 to +100, affects case assignments and NPC availability
  - +reputation: better cases, NPC cooperation, promotion eligibility
  - -reputation: desk duty, restricted access, forced case reassignment
- **Promotion requirements:**
  - Detective: 200 XP + 3 cases solved + reputation ≥ 10
  - Sergeant: 500 XP + 8 cases solved + reputation ≥ 25
  - Lieutenant: 1000 XP + 16 cases solved + reputation ≥ 50
- **Department budget:** shared resource for equipment requests (radio upgrades, forensic tools)

**Decision field:**
- **Preferred:** salary + reputation + department budget as three economic axes
- **Tempting alternative:** just gold like Hearthfield
- **Consequence:** gold-only loses the "public servant under scrutiny" feel
- **Drift cue:** worker drops reputation or budget system

### 8. Skills & Progression (`skills`)

Replaces Hearthfield's tool upgrades with skill trees.

- **4 skill trees, 5 levels each (20 total perks):**
  - **Investigation:** faster evidence collection, higher quality, see hidden clues
    - L1: Quick Search (+20% evidence speed)
    - L2: Keen Eye (spot hidden evidence in scenes)
    - L3: Forensic Intuition (+0.1 base evidence quality)
    - L4: Cold Case Reader (access cold case files)
    - L5: Master Investigator (all evidence at +0.15 quality)
  - **Interrogation:** more dialogue options, better pressure/trust yields
    - L1: Good Cop (trust-building dialogue options)
    - L2: Bad Cop (pressure dialogue options)
    - L3: Read The Room (see NPC trust/pressure values)
    - L4: Confession Artist (+25% confession chance)
    - L5: Master Interrogator (unlock all dialogue paths)
  - **Patrol:** faster movement, better pursuit, wider awareness
    - L1: Beat Knowledge (minimap shows NPC locations)
    - L2: Quick Response (-20% travel time)
    - L3: Pursuit Training (catch fleeing suspects)
    - L4: Community Trust (+5 trust with all town NPCs)
    - L5: Master Patrol (dispatch calls show difficulty rating)
  - **Leadership:** better partner bonuses, command options, resource access
    - L1: Radio Discipline (clearer dispatch information)
    - L2: Partner Synergy (+10% partner bonus effectiveness)
    - L3: Budget Request (access department budget for equipment)
    - L4: Task Delegation (assign simple tasks to AI officers)
    - L5: Master Commander (direct multi-unit responses)

- **XP sources:**
  - Case solved: `case_difficulty * 15` XP
  - Evidence collected: 5 XP per piece
  - Successful interrogation: 20 XP
  - Patrol event resolved: 10 XP
  - Favor completed: 8 XP

- **Skill point award:** 1 point per 100 XP, player chooses which tree

**Decision field:**
- **Preferred:** 4 trees × 5 levels with explicit perk definitions
- **Tempting alternative:** generic XP → stat boost
- **Consequence:** generic boosts have no player identity; "what kind of cop" question disappears
- **Drift cue:** worker implements flat stat increases instead of named perks

### 9. Patrol & Dispatch (`patrol`)

The "outdoor gameplay" domain — replaces farming's outdoor loop.

- **Patrol mode:** player walks/drives through maps, dispatch radio fires events
- **Dispatch events (random, 1–3 per shift):**
  - Traffic stop (easy, 5 fatigue, 10 XP)
  - Noise complaint (easy, 3 fatigue, 5 XP)
  - Shoplifter in progress (medium, 8 fatigue, 15 XP)
  - Domestic disturbance (medium, 10 fatigue, 20 XP, stress +10)
  - Suspicious vehicle (varies, may lead to case evidence)
  - Officer needs backup (hard, 15 fatigue, 25 XP, stress +15)
- **Dispatch frequency:** `base_rate = 0.15` events per game-hour, modified by area and time
  - Downtown: ×1.5 rate
  - Residential: ×1.0 rate
  - Industrial: ×1.2 rate (night: ×2.0)
  - Night shifts: ×1.5 global modifier
- **Patrol car:** fast travel between exterior map zones, uses fuel (refuel at precinct)
  - Fuel: 100 max, travel costs 10–20 per trip, refuel is free at precinct

**Decision field:**
- **Preferred:** dispatch events as the patrol content, not random encounters
- **Tempting alternative:** random NPC spawns that attack the player
- **Consequence:** random combat feels like a different game; police sim ≠ action RPG
- **Drift cue:** worker implements combat system instead of dispatch/response system

### 10. Precinct (`precinct`)

The "indoor hub" domain — replaces Hearthfield's farm buildings and crafting bench.

- **Case board:** displays available and active cases (max 3 active)
- **Evidence room:** process raw evidence into analyzed evidence (1 shift processing time)
- **Interrogation room:** conduct suspect interrogations (see NPC domain)
- **Captain's office:** receive assignments, request promotions, check reputation
- **Locker room:** equip/change equipment, store personal items
- **Break room:** coffee (+25 fatigue, -5 stress), meal (+50 fatigue, -10 stress), partner conversations
- **Records room:** review closed cases, check NPC backgrounds, access cold case files (Detective+)
- **Dispatch desk:** hear active dispatch calls, choose which to respond to

**Decision field:**
- **Preferred:** precinct as a hub with functional rooms
- **Tempting alternative:** single menu screen for all precinct actions
- **Consequence:** menu-only loses the spatial feel that makes sim games engaging
- **Drift cue:** worker builds precinct as a UI overlay instead of a walkable map

### 11. UI System (`ui`)

- **HUD:** shift clock, fatigue bar, stress bar, active case indicator, radio icon (flashes on dispatch)
- **Case file screen:** evidence collected, witnesses interviewed, suspects identified, case status
- **Evidence examination:** close-up view of evidence item with quality indicator
- **Interrogation UI:** dialogue tree with trust/pressure indicators, NPC portrait + expression
- **Notebook:** player's notes on active cases (auto-populated from discoveries)
- **Map screen:** shows current area, markers for case-relevant locations
- **Precinct screens:** case board, evidence room inventory, records search
- **Skill tree screen:** 4 trees with unlocked/available/locked perks
- **Career screen:** rank, XP, promotion requirements, statistics
- **Pause menu:** save, settings, quit
- **Main menu:** new game, load game (3 slots), settings
- **Shift summary:** end-of-shift report (cases progressed, evidence collected, XP earned)

**Target: 15 distinct UI screens** (Hearthfield has 23 but many are variants)

### 12. Save & Settings (`save`)

- **Full state serialization:**
  - Player state (position, fatigue, stress, inventory, equipment)
  - Calendar state (shift number, day, time, weather, rank)
  - All case states (active, evidence collected, witnesses interviewed, suspects status)
  - Evidence chain (every piece linked to its case)
  - NPC states (trust, pressure, schedule position, dialogue flags)
  - Skill tree state (points spent per tree)
  - Economy state (gold, reputation, department budget)
  - World state (map object states, patrol car position/fuel)
- **3 save slots** (matching Hearthfield)
- **Auto-save on shift end**
- **Save/load round-trip gate must pass:** save at any point, reload, verify identical state
- **Settings:** volume (music/sfx), window size, keybinds

**Decision field:**
- **Preferred:** serialize everything via serde, round-trip tested
- **Tempting alternative:** serialize only "important" state, reconstruct the rest
- **Consequence:** reconstructed state diverges from saved state; OnEnter bugs
- **Drift cue:** any resource that implements `Default` but not `Serialize`

---

## Technical Architecture

### Bevy Plugin Structure

```
main.rs
  → CalendarPlugin
  → PlayerPlugin
  → WorldPlugin
  → CasePlugin
  → EvidencePlugin
  → NpcPlugin
  → EconomyPlugin
  → SkillPlugin
  → PatrolPlugin
  → PrecinctPlugin
  → UiPlugin
  → SavePlugin
```

### Integration Pattern (from Hearthfield, improved)

- **Shared Resources:** `ShiftClock`, `PlayerState`, `Inventory`, `CaseBoard`, `EvidenceLocker`, `NpcRegistry`, `Economy`, `Skills`, `PatrolState`
- **Events:** `ShiftEndEvent`, `CaseAssignedEvent`, `EvidenceCollectedEvent`, `InterrogationStartEvent`, `DispatchCallEvent`, `PromotionEvent`, `NpcTrustChangeEvent`
- **States:** `GameState` (Loading, MainMenu, Playing, Paused, Dialogue, Interrogation, EvidenceExam, CaseFile, Precinct)
- **No direct cross-domain function calls** — everything through ECS Resources + Events + State transitions

### Build Pattern

- Standalone binary: `dlc/police/` with own `Cargo.toml`
- Bevy 0.15 (matching existing DLCs)
- Assets path: `../../assets` for shared Hearthfield assets + local `assets/` for police-specific
- Shared type contract: `src/shared/mod.rs` — all domains import from here
- Worker scope: each domain owns `src/domains/{domain}/` exclusively

---

## Constants & Formulas (frozen — workers must use these exactly)

```
TILE_SIZE = 16.0
PIXEL_SCALE = 3.0
SCREEN_WIDTH = 960.0
SCREEN_HEIGHT = 540.0

MAX_FATIGUE = 100.0
MAX_STRESS = 100.0
MAX_TRUST = 100
MIN_TRUST = -100
MAX_PRESSURE = 100
MAX_REPUTATION = 100
MIN_REPUTATION = -100

TIME_SCALE = 2.0  // game-minutes per real-second
SHIFT_DURATION_HOURS = 8
SHIFTS_PER_WEEK = 3

DISPATCH_BASE_RATE = 0.15  // events per game-hour
DISPATCH_NIGHT_MODIFIER = 1.5
DISPATCH_DOWNTOWN_MODIFIER = 1.5

EVIDENCE_BASE_QUALITY = 0.5
EVIDENCE_SKILL_BONUS = 0.05  // per skill level
EVIDENCE_MAX_QUALITY = 0.95
EVIDENCE_WEATHER_PENALTY = 0.1  // per active condition (rain, fog, night)

PATROL_SALARY = 80
DETECTIVE_SALARY = 120
SERGEANT_SALARY = 160
LIEUTENANT_SALARY = 200
CASE_CLOSE_BONUS_MULTIPLIER = 25  // difficulty * this
FAILED_CASE_PENALTY = 50

PROMOTION_DETECTIVE_XP = 200
PROMOTION_DETECTIVE_CASES = 3
PROMOTION_DETECTIVE_REP = 10
PROMOTION_SERGEANT_XP = 500
PROMOTION_SERGEANT_CASES = 8
PROMOTION_SERGEANT_REP = 25
PROMOTION_LIEUTENANT_XP = 1000
PROMOTION_LIEUTENANT_CASES = 16
PROMOTION_LIEUTENANT_REP = 50

XP_PER_EVIDENCE = 5
XP_PER_INTERROGATION = 20
XP_PER_PATROL_EVENT = 10
XP_PER_FAVOR = 8
XP_CASE_MULTIPLIER = 15  // case_difficulty * this
SKILL_POINT_INTERVAL = 100  // 1 point per this much XP

COFFEE_FATIGUE_RESTORE = 25
COFFEE_STRESS_RELIEF = 5
COFFEE_TIME_COST = 15  // minutes
MEAL_FATIGUE_RESTORE = 50
MEAL_STRESS_RELIEF = 10
MEAL_TIME_COST = 30  // minutes

FUEL_MAX = 100
FUEL_COST_PER_TRIP = 15  // average, varies 10-20

MAX_ACTIVE_CASES = 3
```
