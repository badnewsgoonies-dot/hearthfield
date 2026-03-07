# Worker: FIX-PILOT-SAVE-GAPS-2 (6 Missing Resources in Pilot DLC Save)

## Scope
You may modify files under: dlc/pilot/src/

## Required reading
1. dlc/pilot/src/save/mod.rs (FULL file — SaveFile, SaveResources, SaveResources2, LoadResources, LoadResources2)
2. dlc/pilot/src/player/skills.rs (PilotSkills, SkillType, SkillState structs)
3. dlc/pilot/src/missions/story.rs (StoryProgress, StoryChapter)
4. dlc/pilot/src/economy/loans.rs (LoanPortfolio, Loan)
5. dlc/pilot/src/economy/insurance.rs (InsuranceState, InsurancePolicy, InsuranceClaim, CoverageType)
6. dlc/pilot/src/economy/business.rs (AirlineBusiness, HiredPilot, BusinessRoute, BusinessMilestone)
7. dlc/pilot/src/crew/relationships.rs (RelationshipDetails, RelationshipPhase)

## Bug: 6 Game State Resources Missing from SaveFile

These resources are initialized via init_resource but NOT included in SaveFile:
1. **PilotSkills** — all 8 pilot skill levels/XP lost on save/load
2. **StoryProgress** — entire 10-chapter story arc resets on reload
3. **LoanPortfolio** — financial obligations vanish on reload
4. **InsuranceState** — insurance policies/claims lost
5. **AirlineBusiness** — airline business progress lost
6. **RelationshipDetails** — crew relationship phases/abilities lost

## Fix Pattern (for EACH resource)

### Step 1: Add Serialize/Deserialize derives to ALL types in the chain

**dlc/pilot/src/player/skills.rs:**
- `SkillType`: add `Serialize, Deserialize` to existing derives
- `SkillState`: add `Serialize, Deserialize` to existing derives
- `PilotSkills`: add `Serialize, Deserialize, Default` to existing derives (it may already have manual Default impl — keep that, just add the derive if not conflicting, or add `#[serde(default)]`)
- Add `use serde::{Serialize, Deserialize};` at top

**dlc/pilot/src/missions/story.rs:**
- `StoryChapter`: add `Serialize, Deserialize`
- `StoryProgress`: add `Serialize, Deserialize`
- Add `use serde::{Serialize, Deserialize};` at top

**dlc/pilot/src/economy/loans.rs:**
- `Loan`: add `Serialize, Deserialize`
- `LoanPortfolio`: add `Serialize, Deserialize`
- Add `use serde::{Serialize, Deserialize};` at top

**dlc/pilot/src/economy/insurance.rs:**
- `CoverageType`: add `Serialize, Deserialize`
- `InsurancePolicy`: add `Serialize, Deserialize`
- `InsuranceClaim`: add `Serialize, Deserialize`
- `InsuranceState`: add `Serialize, Deserialize`
- Add `use serde::{Serialize, Deserialize};` at top

**dlc/pilot/src/economy/business.rs:**
- `AirlineBusiness`: add `Serialize, Deserialize`
- `HiredPilot`: add `Serialize, Deserialize`
- `BusinessRoute`: add `Serialize, Deserialize`
- `BusinessMilestone`: add `Serialize, Deserialize`
- Add `use serde::{Serialize, Deserialize};` at top

**dlc/pilot/src/crew/relationships.rs:**
- `RelationshipPhase`: add `Serialize, Deserialize`
- `RelationshipDetails`: add `Serialize, Deserialize, Clone`
- Add `use serde::{Serialize, Deserialize};` at top

### Step 2: Add fields to SaveFile in dlc/pilot/src/save/mod.rs
Add these 6 fields (all with `#[serde(default)]`):
```rust
#[serde(default)] pub pilot_skills: PilotSkills,
#[serde(default)] pub story_progress: StoryProgress,
#[serde(default)] pub loan_portfolio: LoanPortfolio,
#[serde(default)] pub insurance_state: InsuranceState,
#[serde(default)] pub airline_business: AirlineBusiness,
#[serde(default)] pub relationship_details: RelationshipDetails,
```

### Step 3: Add to SaveResources2/LoadResources2 (or create SaveResources3/LoadResources3 if hitting 16-field limit)
SaveResources2 currently has 3 fields. You can add up to 13 more before hitting Bevy's SystemParam limit.
```rust
pub pilot_skills: Res<'w, PilotSkills>,
pub story_progress: Res<'w, StoryProgress>,
pub loan_portfolio: Res<'w, LoanPortfolio>,
pub insurance_state: Res<'w, InsuranceState>,
pub airline_business: Res<'w, AirlineBusiness>,
pub relationship_details: Res<'w, RelationshipDetails>,
```
And matching ResMut versions in LoadResources2.

### Step 4: Wire into save_gather and load_apply
In `save_gather`: add `field: ext2.field.clone()` for each new field
In `load_apply`: add `*ext2.field = file.field` for each new field

### Step 5: Add required imports at top of save/mod.rs
Import the new types from their modules.

## Validation
```
cd dlc/pilot && cargo check && cargo test --test headless && cargo clippy -- -D warnings
```

## When done
Write completion report to status/workers/fix-pilot-save-gaps-2.md
