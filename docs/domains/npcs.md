# Domain Spec: NPCs & Dialogue

## Scope
`src/npcs/` — `mod.rs`, `definitions.rs`, `dialogue.rs`, `gifts.rs`, `quests.rs`, `romance.rs`, `schedule.rs`, `schedules.rs`, `spawning.rs`, `animation.rs`, `emotes.rs`, `map_events.rs`

## Responsibility
NPC definitions, schedule-based movement, dialogue trees, gift giving and preference responses, friendship progression, quest system, romance/marriage system, NPC animation and emotes.

## Shared Contract Types (import from `crate::shared`)
- `Npc` (Component — id, name)
- `NpcDef` (id, name, birthday_season, birthday_day, gift_preferences, default_dialogue, heart_dialogue, is_marriageable, sprite_index, portrait_index)
- `NpcRegistry` (Resource — npcs, schedules)
- `NpcSchedule`, `ScheduleEntry` (time, map, x, y)
- `NpcId` (= String)
- `GiftPreference` (Loved +80, Liked +45, Neutral +20, Disliked -20, Hated -40)
- `Relationships` (Resource — friendship, gifted_today, spouse)
- `RelationshipStage` (Stranger → Acquaintance → Friend → CloseFriend → Dating → Engaged → Married)
- `RelationshipStages` (Resource — stages HashMap)
- `MarriageState` (Resource — spouse, wedding_date, days_married, spouse_happiness)
- `Quest`, `QuestLog`, `QuestObjective`
- `Calendar`, `Season`, `DayOfWeek`, `Weather`
- `MapId`, `GridPosition`, `LogicalPosition`, `YSorted`
- `Inventory`, `ItemId`
- `PlayerInput`, `GameState`, `InputBlocks`
- Events: `DialogueStartEvent`, `DialogueEndEvent`, `GiftGivenEvent`, `QuestPostedEvent`, `QuestAcceptedEvent`, `QuestCompletedEvent`, `BouquetGivenEvent`, `ProposalEvent`, `WeddingEvent`, `SpouseActionEvent`, `DayEndEvent`, `CropHarvestedEvent`, `MonsterSlainEvent`
- Constants: `FRIENDSHIP_PER_HEART` (100), `MAX_HEARTS` (10), `MAX_FRIENDSHIP` (1000)
- Functions: `grid_to_world_center()`

## Quantitative Targets
- 10 named NPCs with unique schedules, dialogue, and gift preferences
- Each NPC: 5+ unique dialogue lines per season, 3+ gift responses (loved/liked/disliked)
- 2 marriage candidates
- Birthday: +8x gift points on birthday
- Gift preferences: each NPC has 2-3 loved, 3-5 liked, 2-3 hated items
- Friendship stages: Stranger (0), Acquaintance (200), Friend (400), CloseFriend (600), Dating (800+bouquet), Engaged (1000+pendant)
- 12 quests: 3 per season (mix of Deliver, Catch, Harvest, Mine, Talk, Slay)
- Quest rewards: 100-1000g + optional items + friendship bonus
- Romance: bouquet at 8 hearts, pendant at 10 hearts + Big house, wedding 3 days after proposal

## Constants & Formulas
- Friendship per gift: Loved +80, Liked +45, Neutral +20, Disliked -20, Hated -40
- Birthday multiplier: 8x (e.g., Loved on birthday = +640)
- Daily friendship decay: 0 (no decay)
- Hearts = friendship_points / 100, max 10
- Spouse happiness: -100 to 100, +5 if talked, -2 per day if ignored
- NPC walk speed: 40.0 pixels/sec (half player speed)
- Schedule: weekday/weekend split, rain override, festival override

## Key Systems
1. `npc_spawning` — spawn NPC entities per map from `NpcRegistry`
2. `schedule_system` — move NPCs to schedule waypoints based on time/day/weather
3. `dialogue_system` — on interact with NPC, start dialogue, manage conversation flow
4. `gift_system` — handle gift giving, calculate preference, adjust friendship
5. `quest_posting` — post quests on bulletin board per season
6. `quest_tracking` — listen to game events, update quest objectives, fire completion
7. `romance_system` — handle bouquet/proposal/wedding flow
8. `spouse_actions` — daily spouse behavior (water crops, feed animals, give breakfast)
9. `npc_animation` — walk/idle animations based on movement state
10. `emote_system` — display emote bubbles on gift reaction, quest completion

## Does NOT Handle
- NPC data loading from tables (data domain)
- Dialogue box UI rendering (ui domain)
- Quest log UI display (ui domain)
- NPC portrait rendering in dialogue (ui domain)
- Gift item definitions (data domain)
