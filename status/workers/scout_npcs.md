# NPC System Audit Report

**Files read:** `src/npcs/mod.rs`, `dialogue.rs`, `gifts.rs`, `quests.rs`, `romance.rs`,
`schedules.rs`, `schedule.rs`, `spawning.rs`, `map_events.rs`, `definitions.rs`, `animation.rs`, `emotes.rs`

---

## 1. NPC Roster (10 NPCs)

| ID           | Name        | Role / Notes                            |
|--------------|-------------|-----------------------------------------|
| `margaret`   | Margaret    | Baker                                   |
| `marco`      | Marco       | Chef / innkeeper                        |
| `lily`       | Lily        | Florist (youthful)                      |
| `old_tom`    | Old Tom     | Fisherman                               |
| `elena`      | Elena       | Blacksmith                              |
| `mira`       | Mira        | Traveling merchant                      |
| `doc`        | Doc         | Doctor                                  |
| `mayor_rex`  | Mayor Rex   | Mayor / civic leader                    |
| `sam`        | Sam         | Musician                                |
| `nora`       | Nora        | Farmer / farming mentor                 |

Canonical source: `src/npcs/definitions.rs:11` (`ALL_NPC_IDS`).  
NPC data (gift preferences, dialogue, birthdays) is loaded by `DataPlugin` from `src/data/npcs.rs` during `OnEnter(GameState::Loading)`.

---

## 2. Relationship / Heart System & Gifting

### Hearts and friendship points
- Friendship is stored as raw integer points in `Relationships.friendship`.
- Hearts = `friendship / FRIENDSHIP_PER_HEART`, capped at `MAX_HEARTS` (10).
- Friendship stages auto-update via `update_relationship_stages` each frame:
  - 0–1 hearts → Stranger  
  - 2–3 hearts → Acquaintance  
  - 4–5 hearts → Friend  
  - 6+ hearts → CloseFriend (ceiling until bouquet is given)  
  - Dating / Engaged / Married are only set by the romance systems, never demoted automatically.

### Gift giving (`src/npcs/gifts.rs`)
1. Player presses the **secondary tool key** (G by default) while adjacent to an NPC in `GameState::Playing`.
2. The currently selected hotbar item is used as the gift. **Tools and Special items cannot be gifted.**
3. One gift is allowed per NPC per day (`Relationships.gifted_today`).
4. The gift preference is looked up from `NpcDef.gift_preferences` (populated by `DataPlugin`). If no entry exists, defaults to `Neutral`.
5. Friendship delta:

   | Preference | Points |
   |-----------|--------|
   | Loved     | +80    |
   | Liked     | +45    |
   | Neutral   | +20    |
   | Disliked  | −20    |
   | Hated     | −40    |

6. **Birthday multiplier: ×8** — giving a loved gift on an NPC's birthday yields +640 points.
7. An `NpcEmoteEvent` fires a floating emote bubble (heart, smile, sad, angry, etc.).
8. A toast notification is shown to the player.
9. Dialogue response (`build_gift_response_lines`) is sent via `DialogueStartEvent`.

### Friendship decay (`src/npcs/map_events.rs:handle_day_end`)
- `GiftDecayTracker` counts consecutive days each NPC has gone without a gift.
- After 7 days: −2 friendship per day until a gift resets the counter.
- A toast fires when a full heart is lost.
- `gifted_today` is cleared at end of day.

### Dialogue tiers
`build_dialogue_lines` in `dialogue.rs` selects heart-tier dialogue from `NpcDef.heart_dialogue`:
- Tier 0: 0–2 hearts  
- Tier 3: 3–5 hearts  
- Tier 6: 6–8 hearts  
- Tier 9: 9–10 hearts  

Contextual lines (birthday, weather, season) are prepended. If already gifted today, an early-return "come back tomorrow" line fires.

---

## 3. Quest System (`src/npcs/quests.rs`)

### Quest types (6)
| Type     | Objective struct              | Tracking event(s)                  |
|----------|-------------------------------|------------------------------------|
| Deliver  | `QuestObjective::Deliver`     | `ItemPickupEvent`                  |
| Harvest  | `QuestObjective::Harvest`     | `CropHarvestedEvent`               |
| Catch    | `QuestObjective::Catch`       | `ItemPickupEvent` (fish item id)   |
| Mine     | `QuestObjective::Mine`        | `ItemPickupEvent`                  |
| Talk     | `QuestObjective::Talk`        | `GiftGivenEvent` (gift = visit)    |
| Slay     | `QuestObjective::Slay`        | `MonsterSlainEvent`                |

### Quest lifecycle
1. **Post** — `post_daily_quests` listens to `DayEndEvent` and generates 2–3 random quests for the next day. Each is added directly to `QuestLog.active` and a `QuestPostedEvent` is fired.
2. **Accept** — `handle_quest_accepted` confirms via toast (quests are auto-accepted today; no separate UI accept step).
3. **Track** — `track_quest_progress` monitors `CropHarvestedEvent`, `ItemPickupEvent`, `GiftGivenEvent`. `track_monster_slain` handles `MonsterSlainEvent` separately.
4. **Complete** — `handle_quest_completed` removes the quest from `QuestLog.active`, awards gold, reward items, and friendship with the giver NPC, and sends a toast.
5. **Expire** — `expire_quests` decrements `days_remaining` on `DayEndEvent`. Quests with `days_remaining <= 1` are removed and a toast fires.

### Rewards
- Gold: randomly scaled via `scaled_reward()`, clamped to tier bounds (Early: 100–300 g, Mid: 300–600 g, Late: 500–1000 g).
- Friendship: +20 to +70 depending on tier and type.
- All quests carry `reward_items: Vec<(String, u8)>` but none of the current templates populate it (empty).

---

## 4. Bugs Found

### BUG 1 — **CRITICAL: Slay quests never complete** (`quests.rs`)

`track_monster_slain` increments `slain` when a `MonsterSlainEvent` matches but **never fires `QuestCompletedEvent`** when `slain >= quantity`. The quest will sit in `QuestLog.active` until it expires, awarding nothing.

```rust
// quests.rs (end of file)
pub fn track_monster_slain(
    mut events: EventReader<MonsterSlainEvent>,
    mut quest_log: ResMut<QuestLog>,
) {
    for event in events.read() {
        for quest in quest_log.active.iter_mut() {
            if let QuestObjective::Slay { ref monster_kind, quantity, ref mut slain } = quest.objective {
                if *monster_kind == event.monster_kind && *slain < *quantity {
                    *slain += 1;
                    // BUG: no check for slain >= quantity, no QuestCompletedEvent sent
                }
            }
        }
    }
}
```

Fix: add a `completed_writer: EventWriter<QuestCompletedEvent>` parameter and emit the event when `slain >= quantity`, mirroring what `track_quest_progress` does for all other types.

---

### BUG 2 — **Birthday dialogue addresses NPC by their own name** (`dialogue.rs:123`)

```rust
lines.push(format!(
    "Oh! Today is my birthday! I can't believe you remembered, {}!",
    npc_def.name   // <-- uses the NPC's own name, not the player's
));
```

The NPC ends up saying e.g. *"I can't believe you remembered, Margaret!"* — the NPC addresses themselves. The format string was likely intended to use the player's display name (not available in this function's signature) or should simply be removed.

---

### BUG 3 — **Stale NPC IDs in `TALK_NPCS` fallback constant** (`quests.rs:~175`)

```rust
const TALK_NPCS: &[&str] = &[
    "mayor_thomas", "elena", "marcus", "dr_iris", "old_pete",
    "chef_rosa", "miner_gil", "librarian_faye", "farmer_dale", "child_lily",
];
```

Of the 10 entries only `"elena"` matches a real NPC ID. The others (`mayor_thomas`, `marcus`, `dr_iris`, `old_pete`, `chef_rosa`, `miner_gil`, `librarian_faye`, `farmer_dale`, `child_lily`) are stale placeholder names from a prior design iteration. This fallback is used when `npc_registry.npcs.is_empty()` — which should not happen in production but will silently generate Talk quests targeting nonexistent NPCs if triggered (e.g., in tests). The constant is also never actually used in the happy path (the code favors `npc_registry.npcs.keys()`).

---

### BUG 4 — **Talk quest description misleads: gift required, not just talking** (`quests.rs`)

The `QuestObjective::Talk` type is described as "visit an NPC", and the `talk_description` text says things like *"Give them a gift to complete."* Completion is tracked via `GiftGivenEvent`, not `DialogueStartEvent`. Merely talking to the target NPC (pressing F) will **not** complete the quest — the player must also give a gift. Since the gift description accurately mentions this, it's technically consistent but the quest *type* being named "Talk" is misleading.

---

### BUG 5 — **Missing seasonal dialogue comments (content gaps)** (`dialogue.rs:npc_season_comment`)

Several NPC × season combinations have no entry in the match and silently return `None` (no seasonal line). These are content omissions rather than crashes, but may be unintentional:

| NPC       | Missing season |
|-----------|---------------|
| lily      | Fall           |
| doc       | Summer         |
| margaret  | Summer         |
| old_tom   | Spring         |
| elena     | Fall           |

---

## 5. Marriage System Overview (`src/npcs/romance.rs`)

### Progression stages

```
Stranger → Acquaintance → Friend → CloseFriend
                                         ↓  (give Bouquet, 8+ hearts)
                                      Dating
                                         ↓  (give Mermaid Pendant, 10 hearts, Big house)
                                      Engaged
                                         ↓  (3 days pass via WeddingTimer)
                                      Married
```

### Requirements
| Step        | Item consumed    | Heart requirement | Other requirement     |
|-------------|-----------------|-------------------|-----------------------|
| Bouquet     | `bouquet`        | ≥ 8              | Not already married   |
| Proposal    | `mermaid_pendant`| ≥ 10             | `HouseTier::Big`      |

- Only NPCs with `NpcDef.is_marriageable = true` are eligible.
- The player can only have one spouse (`MarriageState.spouse`).

### Post-marriage
- Each morning at **8:00 AM** game time, `spouse_daily_action` rolls one of:
  - 40%: Water 6 random unwatered crop tiles
  - 25%: Feed all animals
  - 15%: Give a random breakfast item (egg, pancakes, toast, porridge, or fruit salad)
  - 10%: Repair a fence (currently no mechanical effect — placeholder)
  - 10%: Stand on porch (cosmetic only)

### Happiness tracking
- `MarriageState.spouse_happiness` starts at 50 (neutral-positive), range −100 to 100.
- Each day: +2 if player gifted the spouse; −3 otherwise.
- Warnings fire at 0 and −50 via toast notifications.
- No consequence for happiness reaching −100 (no divorce / separation mechanic implemented).

### Spouse happiness proxy issue (design note)
`update_spouse_happiness` uses `relationships.gifted_today` as a proxy for "talked today." Talking to the spouse without gifting them does **not** increase happiness. This may be intentional but could surprise players expecting conversation to suffice.

---

*Report generated from code read only — no source files were modified.*
