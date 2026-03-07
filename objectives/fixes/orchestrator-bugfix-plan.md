# Orchestrator Bugfix Plan — Second Zero Audit Fixes

## Context
A full player-journey audit traced every action from game launch to sleep.
The audit found 12 bugs across 3 priority tiers. This plan fixes the Critical (2) and Medium (6) bugs.
Low-priority polish items are deferred.

## Bug Registry

### CRITICAL

#### BUG-1: Mine entry not wired (MineEntrance → Mine)
- **Root cause**: `src/player/interaction.rs` `edge_transition()` has no rule for MineEntrance cave entry
- **Current state**: Lines ~117-123 only have MineEntrance south exit back to Forest
- **Fix**: Add a transition rule: when player walks onto the cave entrance tile (approx y=0 or specific tile), emit `MapTransitionEvent { to_map: MapId::Mine, to_x: ..., to_y: ... }`
- **Files**: `src/player/interaction.rs`
- **Validation**: `cargo check && cargo clippy -- -D warnings`

#### BUG-2: Seed planting expects F (interact), not Space (tool_use) — unintuitive
- **Root cause**: Seed planting is wired through interact_dispatch (F key), not through ToolUseEvent (Space key)
- **Fix**: Add a path so that when player presses Space with seeds selected in hotbar on a tilled tile, it fires the same planting logic. OR add a clear `[Space] Plant` interaction prompt when standing on tilled soil with seeds selected.
- **Files**: `src/player/interact_dispatch.rs` or `src/farming/mod.rs`, `src/ui/hud.rs` (prompt)
- **Validation**: `cargo check && cargo clippy -- -D warnings`

### MEDIUM

#### BUG-3: Dialogue shows npc_id not display name
- **Root cause**: `src/ui/dialogue_box.rs` lines ~79, ~172 use raw `npc_id` string
- **Fix**: Look up display name from `NpcRegistry` or `NpcData` and use that instead
- **Files**: `src/ui/dialogue_box.rs`
- **Validation**: `cargo check && cargo clippy -- -D warnings`

#### BUG-4: Knockout teleports to MineEntrance, not home
- **Root cause**: `src/mining/combat.rs` line ~313 sends `MapTransitionEvent` to `MapId::MineEntrance`
- **Fix**: Change to `MapId::PlayerHouse` with bed coordinates (~12,3) so player "wakes up at home"
- **Files**: `src/mining/combat.rs`
- **Validation**: `cargo check && cargo clippy -- -D warnings`

#### BUG-5: Chest overlay doesn't block movement/input
- **Root cause**: `InputBlocks` resource exists but nothing sets it when chest is open
- **Fix**: Set `InputBlocks` movement/tool flags when chest opens, clear on close
- **Files**: `src/world/chests.rs`
- **Validation**: `cargo check && cargo clippy -- -D warnings`

#### BUG-6: Esc conflicts with F1/F2/F4 overlays
- **Root cause**: Calendar/stats/settings overlays run in `Playing` state, Esc also triggers pause transition
- **Fix**: In `menu_input.rs` `gameplay_state_transitions`, skip pause if any overlay is active. Or have overlay close systems consume the Esc input so it doesn't propagate.
- **Files**: `src/ui/menu_input.rs` and/or `src/ui/calendar_screen.rs`, `src/ui/stats_screen.rs`, `src/ui/settings_screen.rs`
- **Validation**: `cargo check && cargo clippy -- -D warnings`

#### BUG-7: Crafting C key dual-path
- **Root cause**: Both `trigger_crafting_key` and `gameplay_state_transitions` respond to C
- **Fix**: Remove the duplicate path — crafting should go through one canonical route
- **Files**: `src/ui/menu_input.rs` or `src/crafting/mod.rs`
- **Validation**: `cargo check && cargo clippy -- -D warnings`

#### BUG-8: day_end_coins SFX unmapped
- **Root cause**: Shipping bin day-end emits `day_end_coins` but no audio mapping exists
- **Fix**: Either add the SFX asset mapping or remove the emit if no asset exists
- **Files**: `src/economy/mod.rs` or equivalent SFX registry
- **Validation**: `cargo check && cargo clippy -- -D warnings`

## Execution Order
1. BUG-1 (mine entry) — blocks player progression
2. BUG-2 (seed planting) — blocks intuitive farming
3. BUG-3 (dialogue name) — visible every NPC interaction
4. BUG-4 (knockout destination) — wrong behavior
5. BUG-5 (chest input blocking) — exploitable
6. BUG-6 (esc overlay conflict) — annoying
7. BUG-7 (crafting dual path) — potential double-fire
8. BUG-8 (SFX mapping) — cosmetic

## Gate After Each Fix
```bash
cargo check
cargo clippy -- -D warnings
shasum -a 256 -c .contract.sha256
```
All three must pass. If any fails, fix before moving to next bug.

## Git Protocol
- Branch: `claude/multi-agent-orchestration-vSy7F`
- Commit after each bug fix with message: `fix(domain): BUG-N description`
- Push after all fixes complete: `git push -u origin claude/multi-agent-orchestration-vSy7F`
