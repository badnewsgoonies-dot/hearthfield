# Worker: NPCS

## Scope (mechanically enforced — your edits outside this path will be reverted)
You may only modify files under: src/npcs/
Out-of-scope edits will be silently reverted after you finish.

## Required reading (read these files from disk before writing any code)
1. GAME_SPEC.md
2. docs/domains/npcs.md
3. src/shared/mod.rs (the type contract — import from here, do not redefine)

## Required imports (use exactly, do not redefine locally)
- `Npc`, `NpcDef`, `NpcRegistry`, `NpcSchedule`, `ScheduleEntry`, `NpcId`
- `GiftPreference` (Loved +80, Liked +45, Neutral +20, Disliked -20, Hated -40)
- `Relationships`, `RelationshipStage`, `RelationshipStages`
- `MarriageState`, `SpouseAction`
- `Quest`, `QuestLog`, `QuestObjective`
- `Calendar`, `Season`, `DayOfWeek`, `Weather`, `MapId`
- `GridPosition`, `LogicalPosition`, `YSorted`
- `Inventory`, `ItemId`, `PlayerInput`, `GameState`, `InputBlocks`
- Events: `DialogueStartEvent`, `DialogueEndEvent`, `GiftGivenEvent`, `QuestPostedEvent`, `QuestAcceptedEvent`, `QuestCompletedEvent`, `BouquetGivenEvent`, `ProposalEvent`, `WeddingEvent`, `SpouseActionEvent`, `DayEndEvent`, `CropHarvestedEvent`, `MonsterSlainEvent`
- Constants: `FRIENDSHIP_PER_HEART` (100), `MAX_HEARTS` (10), `MAX_FRIENDSHIP` (1000)
- Functions: `grid_to_world_center()`

## Deliverables
- `src/npcs/mod.rs` — `NpcPlugin`
- `src/npcs/definitions.rs` — NPC personality/gift data
- `src/npcs/dialogue.rs` — Dialogue flow system
- `src/npcs/gifts.rs` — Gift giving and preference calculation
- `src/npcs/quests.rs` — Quest posting, tracking, and completion
- `src/npcs/romance.rs` — Bouquet, proposal, wedding flow
- `src/npcs/schedule.rs` — Schedule-based NPC movement
- `src/npcs/schedules.rs` — Schedule data definitions
- `src/npcs/spawning.rs` — NPC entity spawning per map
- `src/npcs/animation.rs` — Walk/idle NPC animations
- `src/npcs/emotes.rs` — Emote bubble system
- `src/npcs/map_events.rs` — Map-specific NPC events

## Quantitative targets (non-negotiable)
- 10 named NPCs with unique schedules and gift preferences
- 5+ dialogue lines per NPC per season
- 3+ gift responses per NPC (loved/liked/disliked)
- 2 marriage candidates
- Gift points: Loved +80, Liked +45, Neutral +20, Disliked -20, Hated -40
- Birthday multiplier: 8x
- 12 quests (3 per season)
- Romance: bouquet at 8 hearts, pendant at 10 hearts + Big house

## Validation (run before reporting done)
```
cargo check
cargo test --test headless
cargo clippy -- -D warnings
```
Done = all three commands pass with zero errors and zero warnings.

## When done
Write completion report to status/workers/npcs.md containing:
- Files created/modified (with line counts)
- What was implemented
- Quantitative targets hit (with actual counts)
- Shared type imports used
- Validation results (pass/fail)
- Known risks for integration
