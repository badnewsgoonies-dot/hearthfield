# Social Systems Gameplay Audit

Audited: 2026-03-03  
Method: End-to-end trace — input → event → system → state change → feedback

---

## NPC INTERACTION

| # | Action | Status | Chain Traced | Issues Found |
|---|--------|--------|--------------|--------------|
| 1 | **TALK TO NPC** | ✅ YES | Player presses F → `handle_npc_interaction` checks proximity (1.5 tiles) → builds lines filtered by hearts tier (0/3/6/9), season, weather, birthday → emits `DialogueStartEvent{npc_id, lines, portrait_index}` → `listen_for_dialogue_start` spawns dialogue box → `GameState::Dialogue` set → NPC portrait + name + text shown | Dialogue lines are NOT filtered by time-of-day (season/weather filters exist but no 6 AM vs 6 PM variation) |
| 2 | **GIVE GIFT** | ✅ YES | Player presses G (gift hotkey) with item selected → `handle_gift_input` checks adjacency + not-gifted-today + item is giftable → emits `GiftGivenEvent{npc_id, item_id, preference}` + `ItemRemovedEvent` → `handle_gifts` applies points (Loved +80, Liked +45, Neutral +20, Disliked −20, Hated −40) with ×8 birthday multiplier → updates `Relationships.friendship` + `gifted_today` flag → toast shows reaction + point delta (e.g. "Margaret loved your gift! ♥♥♥ (+640)") → NPC emote bubble spawned → NPC-specific gift response dialogue triggered | Gift preference is resolved at event time, not re-evaluated; no diminishing returns on repeated same-item gifting |
| 3 | **NPC SCHEDULES** | ✅ YES | `update_npc_schedules` reads `Calendar` each frame → resolves active schedule entry via priority chain (Festival > Rain/Storm > Weekend > Weekday) → sets `target_x/target_y` → `move_npcs_toward_targets` lerps NPC to target each frame; seasonal schedule variants loaded on `OnEnter(Playing)` and refreshed on `SeasonChangeEvent`; high-heart NPCs (800+ pts) have 30% chance to visit farm on weekday mornings | None |
| 4 | **NPC BIRTHDAYS** | ✅ YES | Birthday stored per NPC as `(season, day)` in `NpcDef`; `handle_npc_interaction` in dialogue.rs checks current calendar against NPC birthday and prepends special birthday greeting lines; `handle_gift_input` in gifts.rs also checks birthday and applies ×8 point multiplier on gift giving | No dedicated "birthday missed" penalty or reminder notification to player |

---

## QUESTS

| # | Action | Status | Chain Traced | Issues Found |
|---|--------|--------|--------------|--------------|
| 5 | **RECEIVE QUEST** | ✅ YES | `post_daily_quests` fires on `DayEndEvent` → generates 2-3 random quests (6 types) with random NPC giver, 3–7 day deadline, scaled gold rewards → immediately appended to `QuestLog.active` → `QuestPostedEvent` fired → toast notification shown | Quests are auto-accepted with no player choice; no "decline quest" mechanic |
| 6 | **DELIVERY QUEST** | ✅ YES | `track_quest_progress` listens to `ItemPickupEvent` → matches item_id to `QuestObjective::Deliver` → increments `delivered` counter → when `delivered >= quantity`, emits `QuestCompletedEvent` → `handle_quest_completed` gives gold via `GoldChangeEvent`, adds reward items to inventory, adds friendship to giver NPC, shows completion toast | Delivery objective triggers on item pickup, not on explicit hand-off to NPC; thematically off but mechanically functional |
| 7 | **GATHER QUEST** | ✅ YES | `track_quest_progress` listens to `CropHarvestedEvent` → matches crop_id to `QuestObjective::Harvest` → increments `harvested` counter → auto-completes when target met → same reward pipeline as delivery | Mine quests (`QuestObjective::Mine`) use `ItemPickupEvent` for ore/minerals with same auto-complete logic |
| 8 | **SLAY QUEST** | ✅ YES | `track_monster_slain` listens to `MonsterSlainEvent` from mining domain → matches `monster_kind` to `QuestObjective::Slay` → increments `slain` counter → auto-completes when `slain >= quantity` → reward pipeline fires | `MonsterSlainEvent` origin in mining domain not independently verified in this audit; assume correct based on quest system integration |
| 9 | **TALK QUEST** | ⚠️ PARTIAL | `track_quest_progress` listens to `GiftGivenEvent` as the trigger for `QuestObjective::Talk` → sets `talked = true` when `npc_id` matches → quest completes | **BUG:** Talk quest completes on *gift-giving*, not on *talking to* the NPC. Player must gift the NPC (consuming an item) to satisfy a "talk to NPC" quest. `DialogueStartEvent` is not listened to for Talk quest completion. |
| 10 | **QUEST DISPLAY** | ❌ NO | Quest data lives in `QuestLog.active` (resource) with full progress state; `PlayerInput` struct has `open_journal` field | **No quest log UI implemented.** No screen, panel, or HUD element queries `QuestLog`. Players have no in-game way to view active quests, progress toward objectives, or quest history. Journal input is defined but has no handler. |

---

## ROMANCE

| # | Action | Status | Chain Traced | Issues Found |
|---|--------|--------|--------------|--------------|
| 11 | **BUILD HEARTS** | ✅ YES | Gift giving → `Relationships.friendship` updated → `update_relationship_stages` runs each frame → converts points to hearts (pts / 100, cap 10) → auto-advances `RelationshipStage` (Stranger → Acquaintance → Friend → CloseFriend at 6+ hearts); heart-tiered dialogue unlocks at 0/3/6/9 hearts; stages never demote below current once Dating+ is reached | No heart-event cutscenes triggered at thresholds (e.g., no "4-heart event" cinematic); only dialogue tier changes |
| 12 | **DATING** | ✅ YES | Player buys bouquet from shop (500 gold) → uses item on adjacent NPC via `BouquetGivenEvent` → `handle_bouquet` validates: NPC is marriageable, player not married, NPC not already Dating+, **8+ hearts required**, bouquet in inventory → consumes bouquet → `RelationshipStage` set to `Dating` → toast confirmation shown | No NPC-specific acceptance/rejection dialogue; immediate silent state change |
| 13 | **PROPOSAL** | ✅ YES | Player buys mermaid_pendant (5,000 gold) → uses item on adjacent NPC via `ProposalEvent` → `handle_proposal` validates: NPC is `Dating`, **10 hearts required**, **house tier ≥ Big** → consumes pendant → `RelationshipStage` set to `Engaged` → `WeddingTimer` set to 3 days → toast "Wedding will be in 3 days!" | House tier gate (must upgrade house before proposing) may be unclear to players; no in-game hint |
| 14 | **MARRIAGE** | ✅ YES | `tick_wedding_timer` on `DayEndEvent` decrements `WeddingTimer.days_remaining` → when 0, emits `WeddingEvent` → `handle_wedding` sets `RelationshipStage::Married`, populates `MarriageState{spouse, wedding_date, days_married=0, spouse_happiness=50}`, sets `Relationships.spouse` → toast "You married [name]!"; post-marriage: `spouse_daily_action` fires at 8 AM each day with random helpful actions (water crops, feed animals, give breakfast item, repair fence, stand on porch); `update_spouse_happiness` tracks happiness decay (−3/day if not interacted, +2/day if gifted, range −100 to +100) | No NPC sprite repositioned to player house; spouse happiness uses `gifted_today` as proxy for "talked to" (must gift spouse daily to prevent happiness decay, talking alone is insufficient) |

---

## Summary

| Domain | Full YES | Partial | No |
|--------|----------|---------|-----|
| NPC Interaction | 4/4 | 0 | 0 |
| Quests | 4/6 | 1 | 1 |
| Romance | 4/4 | 0 | 0 |

**Critical issues:**
1. **Quest #9 (Talk Quest)** — Completed by gifting, not talking. `DialogueStartEvent` must be added as a trigger for `QuestObjective::Talk`.
2. **Quest #10 (Quest Display)** — No UI exists. `open_journal` input is dead code. Quest system is invisible to the player.

**Minor issues:**
- Spouse happiness requires daily *gifting* (not just talking) to prevent decay — semantically wrong.
- No time-of-day dialogue variation (morning/afternoon/evening).
- House tier upgrade gate for proposals has no player-facing hint.
