# Quest Content Review (`src/npcs/quests.rs`)

## Scope completed
- Read `src/npcs/quests.rs` fully.
- Counted quest description/reward template variety per quest type.
- Added description variants where needed (minimum 3 per quest type).
- Normalized reward scaling into tier bands:
  - Early: `100-300g`
  - Mid: `300-600g`
  - Late: `500-1000g`
- Audited quest-referenced item IDs against `src/data/items.rs`.

## Variant counts by quest type

### Before changes
- Deliver:
  - Description variants: `1`
  - Reward templates: `10`
- Harvest:
  - Description variants: `1`
  - Reward templates: `10`
- Catch:
  - Description variants: `1`
  - Reward templates: `10`
- Mine:
  - Description variants: `1`
  - Reward templates: `10`
- Talk:
  - Description variants: `1`
  - Reward templates: `1` (single random range)
- Slay:
  - Description variants: `1`
  - Reward templates: `5`

### After changes
- Deliver:
  - Description variants: `3`
  - Reward templates: `10`
- Harvest:
  - Description variants: `3`
  - Reward templates: `10`
- Catch:
  - Description variants: `3`
  - Reward templates: `13`
- Mine:
  - Description variants: `3`
  - Reward templates: `10`
- Talk:
  - Description variants: `3`
  - Reward templates: `3`
- Slay:
  - Description variants: `3`
  - Reward templates: `5`

## Reward scaling verification
Implemented `RewardTier` + `scaled_reward(...)` with hard tier clamps:
- `Early -> clamp(100, 300)`
- `Mid -> clamp(300, 600)`
- `Late -> clamp(500, 1000)`

All quest types now pull from tiered templates and route reward calculation through this scaling helper.

## Item reference audit vs `src/data/items.rs`

### Invalid references found in original quest templates
- `clay`
- `carrot`
- `cabbage`
- `sunfish`
- `topaz`

### Resolution in `src/npcs/quests.rs`
- Replaced invalid IDs with valid item/fish IDs present in `items.rs`.
- Post-change cross-check found no remaining missing item IDs in quest template references.
