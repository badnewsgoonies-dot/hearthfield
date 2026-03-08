# Worker: Skills Domain (Wave 3)

## Scope (mechanically enforced — edits outside will be reverted)
You may only create/modify files under: `dlc/police/src/domains/skills/`

## Required reading (read BEFORE writing code)
1. `dlc/police/docs/spec.md` — full game spec
2. `dlc/police/docs/domains/skills.md` — YOUR domain spec (CRITICAL — read every line, all enumerative data)
3. `dlc/police/src/shared/mod.rs` — the type contract. Import from here. Do NOT redefine types.
4. `dlc/police/src/main.rs` — plugin registration

## Interpretation contract
Before writing code, extract from your domain spec:
- Preferred approach for each decision + tempting alternative + what breaks
- All quantitative targets (exact numbers, formulas, counts)
- All enumerative data (NPC definitions, perk definitions, salary values)

## Required imports
All types listed under "Key Types" in your domain spec. Import from `crate::shared` only.

## Deliverables
Create `dlc/police/src/domains/skills/mod.rs` with:
- `pub struct SkillsPlugin;` implementing `Plugin`
- All systems, static data, and helper functions listed in the domain spec
- Include ALL data (12 NPCs, 20 perks, all salary/promotion constants) — do NOT summarize
- Integration tests (minimum 5 per spec)

## Critical anti-pattern
ALL state mutations must go through events. Do NOT directly mutate PlayerState.gold or Economy.reputation.
Route through GoldChangeEvent and update handlers. This is the #1 lesson from the Hearthfield audit.

## Failure patterns to avoid
- Local type redefinition (import from contract)
- Direct state mutation bypassing events  
- Generic stat boosts instead of named perks (skills)
- Single friendship bar instead of trust/pressure (npcs)
- Summarizing enumerative data

## Validation
```bash
cargo check -p precinct
cargo test -p precinct
```

## When done
Report what was implemented, targets hit, and what is now player-reachable.
