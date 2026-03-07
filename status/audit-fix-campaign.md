# Audit Fix Campaign — March 4, 2026

## Source: Deep Research audit (verified against code)

## Confirmed Bugs (all verified by orchestrator)
1. **R key collision** — shared/mod.rs:1638+1643, both tool_secondary and open_relationships = KeyR
2. **Animal atlas mismatch** — animals/mod.rs slices character_spritesheet.png as 192×256 but image is 192×192
3. **Hotbar icon 28px** — ui/hud.rs:405-406 uses 28.0px for 16px source (1.75x fractional)
4. **Bed skips cutscene** — player/interact_dispatch.rs sends DayEndEvent+toast, no cutscene queue
5. **Hardcoded key strings** — ui/hud.rs:905,913,933,935 literal [F] and [R] instead of reading KeyBindings

## Wave Plan

### Wave 0: Contract Amendment (sequential, blocking)
- Fix R key: open_relationships → KeyCode::KeyL in shared/mod.rs
- Re-checksum
- This is integration-tier work, single focused worker

### Wave 1: Implementation (parallel, 3 workers)
- Worker A: animal atlas fix (src/animals/)
- Worker B: hotbar icons + prompt strings from KeyBindings (src/ui/)
- Worker C: bed sleep cutscene unification (src/player/ + src/calendar/)

### Wave 2: Regression Audit (single worker)
- Verify all 5 fixes landed correctly
- Run cargo check, clippy
- Report any new issues

## Current Phase: COMPLETE
## Blockers: None

## Results
- Wave 0 (contract amendment): DONE — ecfa262
- Wave 1 (3 parallel workers): DONE — 1dc3680
- Wave 2 (regression audit): DONE — all 5 fixes verified, SHIP verdict
- Total cost: 5 premium requests (1 contract + 3 impl + 1 audit attempt)
- Total time: ~5 minutes wall clock
- Scope violations: 0
- Gate failures: 0
