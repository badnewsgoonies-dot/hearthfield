# Worker: Precinct Domain (Wave 2)

## Scope (mechanically enforced — edits outside will be reverted)
You may only create/modify files under: `dlc/police/src/domains/precinct/`

## Required reading (read BEFORE writing code)
1. `dlc/police/docs/spec.md` — full game spec
2. `dlc/police/docs/domains/precinct.md` — YOUR domain spec (CRITICAL — read every line, especially the enumerative data tables)
3. `dlc/police/src/shared/mod.rs` — the type contract. Import from here. Do NOT redefine types.
4. `dlc/police/src/main.rs` — plugin registration

## Interpretation contract
Before writing any code, extract from your domain spec:
- The preferred approach for each major decision
- The tempting alternative and what it breaks
- The drift cue that signals wrong interpretation
- All quantitative targets (exact numbers, formulas, counts)

## Required imports
All types listed under "Key Types" in your domain spec. Import from `crate::shared` only.
Do NOT redefine any shared types locally.

## Deliverables
Create `dlc/police/src/domains/precinct/mod.rs` with:
- `pub struct PrecinctPlugin;` implementing `Plugin`
- All systems listed in the domain spec
- All static data (case definitions, evidence types, dispatch events) — include EVERY item, do not summarize
- Integration tests (minimum 5 per domain spec)

## Failure patterns to avoid
- Local type redefinition (import from contract)
- Procedural generation instead of hand-authored data
- Real-time timers instead of shift-based counting
- Framework/infrastructure building instead of feature implementation
- Summarizing data tables (if spec says 25 cases, implement 25 cases)

## Validation
```bash
cargo check -p precinct
cargo test -p precinct
```

## When done
Report what was implemented, targets hit, and what is now player-reachable.
