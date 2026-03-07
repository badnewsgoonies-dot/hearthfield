# Dialogue Expansion Report

## Scope
- Updated `src/data/npcs.rs` for NPCs `sam` and `nora`.
- Expanded/rewrote heart dialogue tiers `3`, `6`, and `9` for both characters.

## What Was Changed
- Kept tier `0` dialogue unchanged for both NPCs.
- Updated Sam tiers `3/6/9` to match profile:
  - Energetic musician voice with beach and guitar references.
  - Increasing trust and warmth by tier.
  - Tier `9` now includes emotional tension between leaving town for music and wanting to stay connected to home.
- Updated Nora tiers `3/6/9` to match profile:
  - Practical, wise farming mentorship tone.
  - Increasing personal trust and warmth by tier.
  - Tier `9` now shares multi-generation farm history and explicit emotional warmth.

## Constraint Checks
- 2-3 lines per added tier: satisfied (`3` lines each tier for both NPCs).
- Progressive warmth/personal depth: satisfied.
- Placement: dialogue tiers remain after tier `0` inserts and before `NpcDef` creation blocks.
- File modification scope:
  - Code changes: `src/data/npcs.rs`
  - Report written: `status/workers/dialogue_expansion.md`
