# Worker: NPC Dialogue Depth + Gift Responses

## Scope (mechanically enforced)
You may only modify files under: src/data/ and src/npcs/
Out-of-scope edits will be silently reverted after you finish.

## Required reading
1. src/data/npcs.rs — current NPC definitions with heart_dialogue
2. src/npcs/dialogue.rs — dialogue system, gift response builder
3. src/shared/mod.rs — NpcDef, NpcRegistry types (import only)

## Task
Expand the dialogue variety for all 10 NPCs:

1. **Heart-tier dialogue**: Each NPC currently has 4 tiers (0, 3, 6, 9) with ~5-7 lines each. Add 3 MORE unique lines per tier per NPC. That means each tier should end up with 8-10 lines total.

2. **Gift response lines**: In `src/npcs/dialogue.rs`, the `build_gift_response_lines` function should return NPC-specific responses for each gift preference level:
   - Loved: 3 unique responses per NPC (ecstatic, grateful, personal)
   - Liked: 2 unique responses per NPC
   - Disliked: 2 unique responses per NPC
   - Hated: 2 unique responses per NPC
   Currently these may use generic fallbacks. Make each NPC's reactions reflect their personality.

3. **Marriage candidate dialogue**: For the 2 marriage candidates (check who they are in the data), add dating-stage dialogue (hearts 8+) that hints at romance progression.

## Character personalities (for writing dialogue)
- Margaret: Warm baker, nurturing, loves sharing food
- Marco: Passionate chef, dramatic about ingredients
- Lily: Energetic flower lover, optimistic, childlike wonder
- Old Tom: Wise fisherman, patient, dry humor
- Elena: Stoic blacksmith, practical, quietly caring
- Mira: Traveling merchant, worldly, mysterious past
- Doc: Caring doctor, analytical, health-focused
- Mayor Rex: Formal, civic-minded, secretly sentimental
- Sam: Young musician, dreamy, emotionally expressive
- Nora: Seasoned farmer, no-nonsense, deep knowledge

## Validation
```
cargo check
cargo test --test headless
cargo clippy -- -D warnings
```
Done = all three pass with zero errors and zero warnings.
