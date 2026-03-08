# Worker: Save Domain (Wave 4)

## Scope (mechanically enforced)
You may only create/modify files under: `dlc/police/src/domains/save/`

## Required reading
1. `dlc/police/docs/spec.md`
2. `dlc/police/docs/domains/save.md` (CRITICAL)
3. `dlc/police/src/shared/mod.rs`
4. `dlc/police/src/main.rs`

## Interpretation contract
- Serialize ALL 10 resources listed in the spec (missing any = failure)
- Replace resources on load, don't patch entities
- JSON format, one file per slot
- Round-trip fidelity is the primary quality gate

## Required imports
From `crate::shared`: SaveRequestEvent, LoadRequestEvent, ShiftEndEvent, ShiftClock,
PlayerState, Inventory, CaseBoard, EvidenceLocker, NpcRegistry, PartnerArc, Economy,
Skills, PatrolState, GameState, NpcId, NpcRelationship

## Deliverables
- `pub struct SavePlugin;`
- FullSaveData struct with all 10 resources
- handle_save, handle_load, auto_save, ensure_save_dir systems
- Minimum 7 tests including round-trip

## Critical: The NpcRegistry contains both definitions (static) and relationships (mutable).
Only serialize the relationships (HashMap<NpcId, NpcRelationship>) and PartnerArc.
Definitions are reconstructed from the npcs domain on startup.

## Validation
```bash
cargo check -p precinct
cargo test -p precinct
```
