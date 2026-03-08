# Wave 3 Scope Plan — Precinct Police DLC

## Inputs Read
- `dlc/police/docs/spec.md`
- `dlc/police/src/shared/mod.rs`
- `dlc/police/MANIFEST.md`

## Planning Note
`MANIFEST.md` still reflects the bootstrap phase. This plan follows the requested working status instead:
- Wave 1 done: `calendar`, `player`, `world`, `ui`
- Wave 2 in progress: `cases`, `evidence`, `patrol`, `precinct`

## Recommendation
Wave 3 should include:
- `skills`
- `economy`
- `npcs`

Wave 4 should hold:
- `save`

## Why This Cut
Wave 1 and Wave 2 already establish most of the runtime seams these three domains need:
- `cases` already emits case completion and failure outcomes.
- `patrol` already emits XP-related events and exposes dispatch state.
- `evidence` already uses the skill-quality formula, but it is still hard-wired to a placeholder skill level.
- `cases` already embeds `NpcId` witness and suspect links, and shared events for dialogue and interrogation already exist.
- `precinct` already has placeholders for captain, locker, dispatch, partner-facing interactions, and career-facing rooms.

That makes `skills`, `economy`, and `npcs` the best Wave 3 set because they convert the current mechanical loop into a real progression-and-relationships loop. `save` is different: it is a cross-cutting integration layer that becomes cheaper and safer once the last gameplay resources have stabilized.

## Recommended In-Wave Order
1. `skills`
2. `economy`
3. `npcs`

Reason:
- `skills` removes the current placeholder XP and evidence-quality seam.
- `economy` can then consume stable XP totals plus case outcomes for salary, reputation, and promotions.
- `npcs` is the broadest surface area and benefits from having progression and career hooks already in place.

## Wave 3 Domains

## `skills`

### 1. Dependencies From Wave 1 and Wave 2
- Wave 1 `calendar`: rank tier timing, paused time behavior, and shift cadence for progression pacing.
- Wave 1 `player`: fatigue, stress, movement, and equipment hooks that perks can modify.
- Wave 1 `ui`: `GameState::SkillTree` already exists in the contract and is the natural menu target.
- Wave 2 `cases`: `CaseSolvedEvent` carries case XP rewards.
- Wave 2 `evidence`: evidence quality formula is already defined and currently waiting on a real investigation level.
- Wave 2 `patrol`: dispatch resolution already emits XP-related events and has patrol-specific perk hooks.
- Wave 2 `precinct`: records, locker, and captain-facing interactions are the natural unlock points for leadership and investigation perks.

### 2. Shared Types It Needs
- `Skills`
- `SkillTree`
- `GameState`
- `CaseSolvedEvent`
- `EvidenceCollectedEvent`
- `DispatchResolvedEvent`
- `InterrogationEndEvent`
- `XpGainedEvent`
- `SkillPointSpentEvent`
- `ShiftClock`
- Constants: `SKILL_POINT_INTERVAL`, `XP_PER_EVIDENCE`, `XP_PER_INTERROGATION`, `XP_PER_FAVOR`

### 3. Key Systems To Implement
- Static perk definitions for all 4 trees and 5 levels each.
- XP intake and normalization from cases, evidence, patrol, interrogation, and favors.
- Skill point award logic every 100 XP.
- Point spending with per-tree cap enforcement and `SkillPointSpentEvent`.
- Shared perk queries or helper functions so other domains can read live perk state.
- Integration pass on existing Wave 2 seams:
  - evidence quality must read `Skills.investigation_level`
  - patrol perks must gate dispatch intel and travel improvements
  - leadership perks must expose budget-request and partner-bonus hooks

### 4. Estimated Complexity
- `500-800 LOC`

### 5. Cross-Domain Integration Risks
- XP can be double-counted if both raw gameplay events and `XpGainedEvent` are consumed without a single ownership rule.
- The evidence domain currently assumes a placeholder skill level; switching to live skills must preserve the exact frozen formula.
- Several perks need behavior in other domains, so a tree UI without downstream hooks would create fake progression.
- `GameState::SkillTree` exists, but the current UI plugin does not yet render that screen.

## `economy`

### 1. Dependencies From Wave 1 and Wave 2
- Wave 1 `calendar`: salary cadence, weekly expense timing, and rank-based pay rates.
- Wave 1 `player`: `PlayerState.gold` is already the player-facing wallet shown in HUD.
- Wave 1 `ui`: the HUD already surfaces gold, and `GameState::CareerView` exists for future career screens.
- Wave 2 `cases`: `CaseSolvedEvent`, `CaseFailedEvent`, and `CaseBoard.total_cases_solved` drive pay, reputation, and promotion checks.
- Wave 2 `precinct`: captain's office is the natural place for promotion checks and department budget requests.
- Wave 2 `patrol`: responded versus ignored calls are likely reputation inputs, even if the first pass keeps that simple.
- Wave 3 `skills`: promotion logic needs stable `Skills.total_xp`.

### 2. Shared Types It Needs
- `Economy`
- `PlayerState`
- `ShiftClock`
- `Rank`
- `CaseBoard`
- `Skills`
- `GameState`
- `CaseSolvedEvent`
- `CaseFailedEvent`
- `ShiftEndEvent`
- `GoldChangeEvent`
- `PromotionEvent`
- Constants: `PATROL_SALARY`, `DETECTIVE_SALARY`, `SERGEANT_SALARY`, `LIEUTENANT_SALARY`, `FAILED_CASE_PENALTY`, `PROMOTION_DETECTIVE_XP`, `PROMOTION_DETECTIVE_CASES`, `PROMOTION_DETECTIVE_REP`, `PROMOTION_SERGEANT_XP`, `PROMOTION_SERGEANT_CASES`, `PROMOTION_SERGEANT_REP`, `PROMOTION_LIEUTENANT_XP`, `PROMOTION_LIEUTENANT_CASES`, `PROMOTION_LIEUTENANT_REP`, `MAX_REPUTATION`, `MIN_REPUTATION`

### 3. Key Systems To Implement
- Salary payout on `ShiftEndEvent` using `Rank::salary()`.
- Case reward and failure application:
  - gold to `PlayerState.gold`
  - reputation to `Economy.reputation`
  - lifetime earnings to `Economy.total_earned`
- Weekly expense deductions and budget tracking.
- Promotion eligibility checks using shift-gated rank, total XP, cases solved, and reputation.
- Captain/career-facing hooks for promotion feedback and department budget access.
- Clamp and audit systems for reputation bounds and budget integrity.

### 4. Estimated Complexity
- `450-750 LOC`

### 5. Cross-Domain Integration Risks
- Promotion has the highest seam risk in the DLC:
  - `calendar` currently advances rank by shift count
  - `economy` also needs promotion requirements based on XP, solved cases, and reputation
  - Wave 3 must define one source of truth for rank change versus promotion eligibility
- Gold has split ownership pressure between `PlayerState.gold` and `Economy.total_earned`.
- The captain's office interaction currently lives in `precinct`, so career actions need a clean event or resource seam instead of direct coupling.
- Reputation tuning can accidentally lock players out of case flow if penalties are too aggressive.

## `npcs`

### 1. Dependencies From Wave 1 and Wave 2
- Wave 1 `calendar`: schedules depend on hour, day, and weather.
- Wave 1 `world`: NPC placement depends on map presence and transition handling.
- Wave 1 `player`: proximity interaction and map position are needed for dialogue and interrogation entry.
- Wave 1 `ui`: `GameState::Dialogue` and `GameState::Interrogation` already exist in the contract.
- Wave 2 `cases`: witness and suspect IDs are already embedded in case definitions, and active cases already track interviewed/interrogated sets.
- Wave 2 `evidence`: testimonial evidence and confessions should emerge from NPC interactions.
- Wave 2 `precinct`: partner conversations, interrogation room flow, captain/mentor interactions, and records-room hooks all point here.
- Wave 2 `patrol`: dispatch scenes can surface witnesses, informants, or suspect encounters later in the wave.

### 2. Shared Types It Needs
- `NpcRegistry`
- `NpcDef`
- `NpcRelationship`
- `ScheduleEntry`
- `Npc`
- `NpcId`
- `NpcRole`
- `PartnerArc`
- `PartnerStage`
- `CaseBoard`
- `ShiftClock`
- `MapId`
- `PlayerState`
- `GameState`
- `DialogueStartEvent`
- `DialogueEndEvent`
- `InterrogationStartEvent`
- `InterrogationEndEvent`
- `NpcTrustChangeEvent`
- `EvidenceCollectedEvent`
- Constants: `MAX_TRUST`, `MIN_TRUST`, `MAX_PRESSURE`

### 3. Key Systems To Implement
- Static data population for all 12 named NPCs, default relationships, and schedules.
- Schedule resolution by time, day, and weather with map-based spawn or presence updates.
- Dialogue start/end handling from proximity interactions.
- Trust and pressure mutation systems with clamp rules and dialogue flags.
- Witness interview and suspect interrogation flow that updates active cases.
- Emission of testimonial evidence such as `witness_statement`, `tip_off`, or `confession` where appropriate.
- Partner arc progression and bonus unlock tracking.
- Favor completion hooks that feed both trust and XP progression.

### 4. Estimated Complexity
- `900-1400 LOC`

### 5. Cross-Domain Integration Risks
- This domain has the widest integration surface in the DLC and will touch cases, evidence, precinct, UI, and world behavior at once.
- The current world implementation only fully supports the precinct map, while NPC schedules assume town-wide movement across many maps.
- Dialogue and interrogation game states exist, but the UI layer for them is not implemented yet.
- The same NPC can be a witness in one case and a suspect in another, so case-scoped interaction state must stay separate from global relationship state.
- Testimonial evidence ownership must stay clear so `npcs` does not duplicate logic already owned by `evidence` or `cases`.

## Hold For Wave 4

## `save`

### Recommendation
Defer `save` to Wave 4.

### Why It Should Wait
- It depends on the final runtime shape of `skills`, `economy`, and `npcs`, which are all still unimplemented.
- It must serialize nearly every gameplay resource:
  - `ShiftClock`
  - `PlayerState`
  - `Inventory`
  - `CaseBoard`
  - `EvidenceLocker`
  - `NpcRegistry`
  - `PartnerArc`
  - `Economy`
  - `Skills`
  - `PatrolState`
- It also needs a clean policy for what is reconstructed versus serialized for world and precinct runtime entities.
- Save/load is the most failure-sensitive domain in the DLC because bad ownership or ordering bugs produce corrupted or divergent state rather than isolated feature regressions.
- The spec explicitly requires full-state round-trip fidelity and auto-save at shift end, which is easiest to verify once the gameplay model is no longer moving.

### Wave 4 Entry Criteria
- Wave 3 resources and events are stable.
- Promotion, XP, trust/pressure, and partner-arc ownership rules are settled.
- The map/state reconstruction strategy is documented for load-time rehydration.
- Round-trip tests can be written against the final resource set instead of placeholder defaults.

## Bottom Line
Wave 3 should complete the progression and relationship layer of Precinct:
- `skills` makes XP and perk identity real
- `economy` makes salary, reputation, and promotions real
- `npcs` makes witnesses, suspects, and partner dynamics real

`save` should wait for Wave 4 because it is the integration capstone, not the next gameplay unlock.
