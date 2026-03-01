# Hearthfield — Long-Horizon Upgrade Strategy

## Orchestrator: Claude Opus (this session)
## Workers: GitHub Copilot CLI (Sonnet 4.6, `--yolo` mode)
## Architecture: Model-is-the-orchestrator, zero scaffold

---

## Current State Assessment

| Metric | Value |
|--------|-------|
| Source files | 116 .rs files |
| Lines of code | ~35K |
| Headless tests | 93 |
| Domain plugins | 13 |
| Orphaned events (handler, no sender) | 7 |
| Fire-and-forget events (sender, no handler) | 6 |
| Save system coverage | 13/~20 serializable resources |
| Sprite coverage | Partial (22 placeholders replaced, more remain) |
| WASM deployment | Configured but not working |

### The Core Problem

The game has **complete backend systems** and a **working vertical slice** (intro → tutorial → farming → day cycle) but the middle layer — where player actions connect to backend logic — is inconsistent. Some UIs bypass the event system entirely (shop does buy/sell inline), some events have no sender (quest acceptance, tool upgrades), and some handlers fire into the void (achievements, fishing level-up).

The result: a player can farm, walk around, talk to NPCs, fish, and sleep — but can't craft at a bench, can't accept quests, can't upgrade tools at the blacksmith, and achievements/progression tracking is silent.

---

## Strategy: Four Tiers, Parallel Workers

Each tier is a batch of copilot worker dispatches. Workers are scope-isolated to specific directories. The orchestrator (me) coordinates sequencing, validates results, and handles integration.

### Tier 0: Unblock Core Loops (3 workers, ~30 min)

**Goal:** Every gameplay system the player can reach should actually work end-to-end.

| Worker | Scope | Task |
|--------|-------|------|
| W0-A | `src/ui/crafting_screen.rs`, `src/crafting/` | Wire crafting confirmation: when player presses activate on a recipe, send `CraftItemEvent` instead of (or in addition to) doing inline logic. Connect `CloseCraftingEvent` to Escape. |
| W0-B | `src/npcs/quests.rs`, `src/npcs/dialogue.rs` | Wire quest acceptance: when NPC dialogue offers a quest and player accepts, send `QuestAcceptedEvent`. Add quest-offer dialogue nodes for at least 3 NPCs. |
| W0-C | `src/economy/blacksmith.rs`, `src/ui/building_upgrade_menu.rs` | Wire `ToolUpgradeRequestEvent`: add blacksmith-specific UI path (separate from building upgrades) that lets player select a tool tier and sends the event. |

**Integration check:** `cargo check` + run headless tests after all three complete.

### Tier 1: Complete the Event Graph (4 workers, ~45 min)

**Goal:** Eliminate all orphaned and fire-and-forget events. Every event in the system either has both a sender and a handler, or is removed.

| Worker | Scope | Task |
|--------|-------|------|
| W1-A | `src/economy/achievements.rs`, `src/ui/toast.rs` | Add handler for `AchievementUnlockedEvent` — display a toast notification and persist to save data. |
| W1-B | `src/fishing/` | Add handler for `FishingLevelUpEvent` — apply skill bonuses, show toast. Wire `ToolImpactEvent` if relevant to fishing rod upgrades, or remove if dead. |
| W1-C | `src/economy/blacksmith.rs`, `src/player/` | Add handler for `ToolUpgradeCompleteEvent` — when upgrade timer finishes, actually upgrade the tool in PlayerState and notify player. |
| W1-D | `src/shared/mod.rs` audit | Audit: remove truly dead events (e.g., `ConsumeItemEvent` if `EatFoodEvent` fully replaces it). Clean `QuestPostedEvent` if unused. Update `src/main.rs` registrations. |

**Integration check:** Zero orphaned events. `comm -23 readers.txt writers.txt` returns empty.

### Tier 2: Content & Depth (6 workers, ~90 min)

**Goal:** Transform from tech demo to playable game with 2+ hours of content.

| Worker | Scope | Task |
|--------|-------|------|
| W2-A | `src/npcs/definitions.rs`, `src/npcs/dialogue.rs` | Expand dialogue: each of 10 NPCs gets 5+ unique lines per season, 3+ gift responses (loved/liked/disliked), birthday dialogue. |
| W2-B | `src/npcs/quests.rs`, `src/data/` | Add 12 quests: 3 per season, mix of fetch (bring X items), social (talk to NPC), and farming (grow X crop). Wire quest completion checks. |
| W2-C | `src/calendar/festivals.rs` | Implement 4 festival events: spawn festival map decorations, festival-specific NPC dialogue, mini-game or judging mechanic, rewards. |
| W2-D | `src/mining/` | Polish mine: ensure 20 floors work, elevator unlocks every 5, combat balance pass (HP/damage numbers), gem drop tables match spec. |
| W2-E | `src/crafting/recipes.rs`, `src/data/recipes.rs` | Fill out recipes: ensure 20 crafting + 15 cooking recipes exist per spec. Wire machine processing (furnace, preserves jar, cheese press, loom) with correct timers. |
| W2-F | `src/animals/` | Polish animals: verify buy→feed→produce→sell loop works for chickens, cows, sheep. Add petting interaction. Baby→adult aging. Happiness decay. |

**Integration check:** Functional playthrough test — can a player complete 1 full in-game year touching every system?

### Tier 3: Polish & Deploy (5 workers, ~60 min)

**Goal:** Ship to itch.io as a playable web game.

| Worker | Scope | Task |
|--------|-------|------|
| W3-A | `src/ui/audio.rs`, `assets/` | Audio integration: background music per map/season, SFX for tools, footsteps, UI clicks, crop harvest, fishing splash. Use Bevy's audio. |
| W3-B | `src/ui/transitions.rs`, `src/ui/` | Screen polish: fade transitions between maps, screen shake on tool impact, particle effects for harvesting, weather visual effects. |
| W3-C | `src/save/mod.rs` | Save system audit: ensure ALL 20+ serializable resources are included. Add save slot selection UI. Test save/load round-trip for every resource. |
| W3-D | WASM build pipeline | Fix WASM build: resolve tonemapping/rendering pipeline for WebGL2, create index.html with loading screen, test in browser, create itch.io upload bundle. |
| W3-E | `src/input/mod.rs` | Touch input: add virtual joystick and action buttons for mobile. Detect touch vs keyboard. Map touch zones to existing PlayerInput resource. |

---

## Dispatch Protocol

For each worker, the orchestrator (me) will:

1. **Prepare context**: Generate a focused prompt with file paths, existing patterns to follow, explicit scope boundaries
2. **Dispatch**: `gh copilot -p "<prompt>" --yolo --add-dir /home/claude/hearthfield`
3. **Validate**: Check compilation, run affected tests, verify no scope violations
4. **Integrate**: If worker touched shared types, verify no downstream breakage
5. **Commit**: Stage and commit with descriptive message

### Parallelism Rules

- Workers within the same tier can run in parallel IF they don't share files
- `src/shared/mod.rs` is a serialization point — only one worker may modify it at a time
- `src/main.rs` modifications are orchestrator-only (I do the wiring)
- Cross-tier dependencies are sequential: Tier N must complete before Tier N+1

### Scope Enforcement

Per the paper's finding (Section 5, 0/20 prompt-based enforcement under compiler pressure), we use **mechanical enforcement**:
- Workers get `--add-dir` restricted to their domain
- Post-worker, orchestrator runs `git diff --name-only` and reverts any out-of-scope changes
- Shared type changes require orchestrator review before commit

---

## Estimated Budget

| Tier | Workers | Est. Premium Requests | Est. Time |
|------|---------|----------------------|-----------|
| 0 | 3 | 3-6 | 30 min |
| 1 | 4 | 4-8 | 45 min |
| 2 | 6 | 12-24 | 90 min |
| 3 | 5 | 10-20 | 60 min |
| **Total** | **18** | **29-58** | **~4 hours** |

Orchestrator (Claude Opus) cost: primarily context re-ingestion across dispatch cycles. The statefulness premium applies — each dispatch requires re-reading conversation history.

---

## Success Criteria

- [ ] Zero orphaned events (Tier 1 complete)
- [ ] All 13 domain plugins have working player-facing interactions
- [ ] 93+ headless tests passing (no regressions)
- [ ] Full year playthrough possible (Tier 2 complete)
- [ ] Playable in browser on itch.io (Tier 3 complete)
- [ ] Save/load preserves all game state

---

## What This Tests (Meta)

This upgrade campaign is itself an experiment in the pattern described in `model_is_the_orchestrator_draft_13.docx`:

- **Orchestrator**: Claude Opus in a consumer chat interface (claude.ai)
- **Workers**: GitHub Copilot CLI (Sonnet 4.6) dispatched via bash
- **Coordination**: Shared type contract (`src/shared/mod.rs`) + file-system scope isolation
- **Scaffold**: Zero — the orchestration logic is this conversation

If successful, this would demonstrate the paper's weak claim (model given a coordination template executes effectively) in a novel cross-vendor configuration: Anthropic orchestrator → GitHub/Anthropic worker, mediated through OAuth device flow authentication established during the same conversation.
