# Hearthfield Launch Tranche Plan

Purpose: operationalize the reconstruction target into bounded execution tranches.

This is a launch plan, not a theory note.

## Global Rules

- Run at most 2-3 active slices at once.
- Integration is continuous after each tranche.
- No tranche is considered done at green gates alone.
- All progress must be serialized to disk.
- The top-level orchestrator reads tranche outputs, not every worker log.

## Tranche Layout

### Tranche 0 — Control Plane

Goal:

- freeze the execution scaffolding
- initialize the dispatch ledger
- generate tranche packets for the first execution batch

Outputs:

- `status/foreman/dispatch-state.yaml`
- `status/launch/tranche-0-report.md`
- `objectives/launch/tranche-1-*.md`

### Tranche 1 — Playable Spine

Target surfaces:

- boot -> main menu
- player spawn
- house exit / sleep
- basic day progression
- core farm day-one loop

### Tranche 2 — World Graph Spine

Target surfaces:

- map registry / loading
- map transitions
- collision
- interior / exterior parity
- weather / lighting / y-sort

### Tranche 3 — Economy + Progression Spine

Target surfaces:

- gold
- shipping
- shops
- upgrades
- achievements / evaluation

### Tranche 4 — Social Spine

Target surfaces:

- NPC definitions
- schedules
- dialogue
- gifts
- quests
- romance

### Tranche 5 — System Breadth

Target surfaces:

- crafting
- cooking
- machines
- fishing
- mining

### Tranche 6 — UI Breadth

Target surfaces:

- HUD
- menus
- inventory / journal / map / calendar / stats / relationships
- shop / crafting / chest / building upgrade
- dialogue / cutscenes / tutorials / toast / audio

### Tranche 7 — Save + Parity + Hardening

Target surfaces:

- save/load parity
- asset / visual parity pass
- regression / hardening pass

## Concurrency Guidance

Recommended active slices:

- Tranche 1: `player_day_start`, `farm_core_loop`
- Tranche 2: `world_graph_travel`, `collision_and_lighting`
- Tranche 3: `shipping_and_gold`, `town_shop_loop`
- Tranche 4: `social_loop`
- Tranche 5: `crafting_loop`, `fishing_loop`, `mining_loop`
- Tranche 6: `ui_screen_family`
- Tranche 7: `save_load_roundtrip`, `visual parity`, `hardening`

## Required Checkpoint Outputs Per Tranche

- updated `status/foreman/dispatch-state.yaml`
- tranche report under `status/launch/`
- gate results
- residual risks
- integration note or explicit reason why integration is deferred
