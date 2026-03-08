# 05 — Build Plan Template: The Perfect Build Orchestration Pattern

Load this FIFTH. This is the battle-tested template from Hearthfield's build, adapted for reuse.

## Origin
Created after a failed 48-hour build produced 34K LOC with 19 disconnected events, 14 unsaved
resources, and 80% of gameplay unreachable. Every rule exists because its absence caused a specific failure.

## 10 Iron Rules (apply to every phase, every worker)

1. **Loop Closure**: every EventReader MUST have a corresponding EventWriter in the same commit
2. **Save Completeness**: every Serialize resource appears in save/load in the same commit it's created
3. **One Coordinate System**: all positions use a single shared grid→world function
4. **One Collision System**: single CollisionMap resource, no separate collision resources
5. **No Placeholders**: use atlas sprites or colored rectangles, never Sprite::default()
6. **Real Sprites from Atlas**: atlas indices defined as shared constants (or colored rects for skeleton)
7. **Single Source of Truth**: one registry per data type, no duplicate definitions
8. **No #[allow(dead_code)]**: unwired code is a build failure, not a suppression target
9. **Integration Tests**: every agent delivers at least one end-to-end test
10. **Event Reader Symmetry**: every add_event has at least one sender AND one reader

## The Golden Path Pattern
Define the exact player journey (15-25 steps) BEFORE any code. Build Phase 1 as a single
sequential pass making every step work end-to-end. Example (Hearthfield, 23 steps):
Boot → Menu → New Game → Spawn → Walk → Till → Plant → Water → Sleep → Day advances →
Grow → Harvest → Ship → Buy → Craft → Place → Save → Load → Verify

## The Interaction Contract
For every player-facing action, specify:
- WHO sends the event (which system, which file)
- HOW the player triggers it (key, button, proximity)
- WHAT state gates apply (GameState, proximity, inventory)
- WHICH domain owns the handler

Categories: F-key proximity, R-key item use, tool actions (Space), UI buttons, automatic events.

## Phase Structure
| Phase | Agents | Goal |
|-------|--------|------|
| 0: Scaffold | Orchestrator | Contract, main.rs, empty stubs, specs |
| 1: Vertical Slice | 1 sequential | Golden path works end-to-end |
| 2: Domain Expansion | 3-5 parallel | All domains have real gameplay |
| 3: UI + Polish | 2 parallel | Every backend feature has UI |
| 4: Rendering | 1-2 | Real sprites and animation |
| 5: Testing | 1 | 50+ integration tests, zero warnings |
| 6: WASM/Deploy | 1 | Browser build, deployment |

## Audit Scripts (run at every gate)
- **event_audit**: every add_event has ≥1 sender and ≥1 reader
- **save_audit**: every Serialize+Resource appears in save/load
- **dead_code_audit**: every pub fn is either called or deleted

## What This Prevents (failure → prevention mapping)
| Failure | Prevention |
|---------|-----------|
| 19 events with no sender | Rule 1 + interaction contract + event audit |
| 14 unsaved resources | Rule 2 + save audit |
| Coordinate inconsistencies | Rule 3 + single conversion function |
| Player walks through walls | Rule 4 + CollisionMap sync |
| Colored rectangles everywhere | Rule 5 + Rule 6 |
| Duplicate registries | Rule 7 |
| 80 dead_code suppressions | Rule 8 |
| No integration tests | Rule 9 |
| Events firing into void | Rule 10 + event audit |
