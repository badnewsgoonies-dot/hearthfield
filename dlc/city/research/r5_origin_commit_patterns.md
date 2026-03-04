# R5 Origin Commit Patterns (Early History Scan)

Scope reviewed: first 50 chronological commits (`4d61117` -> `63d04aa`, 2026-02-26 to 2026-02-27).
Filter applied: excluded `city_office_worker_dlc` paths (none appeared in this window).

## Good Patterns

- Contract-first sequencing worked: `4d61117` established `src/shared/mod.rs` and 12 domain stubs; `a5fb75e` and `474903f` landed explicit contract additions before large feature waves.
- Parallel waves were strongest when broken into follow-up slices: `14a0be0` (quest board, 2 files) and `aa0f8b1` (sprinkler + buffs, 4 files) were easier to integrate than earlier mega waves.
- Testing investments clearly stabilized delivery: `cef42ae` created the headless ECS harness, then `7ea49b1` and `e1df4be` expanded coverage heavily.
- Architecture corrections were made explicitly instead of hidden drift: `92596f4` moved cross-domain utilities into `shared`; `6868082` unified input abstraction across systems.

## Bad Patterns

- Early mega-commits raised integration risk: `3ef6aa4` (+8,886 LOC, 40 files), `1702100` (+7,482 LOC, 31 files), and merge `73f04d1` (+18,624 LOC, 78 files).
- Reactive bug-fix churn followed large bursts: `d803c3f` fixed 3 critical gameplay bugs right after Phase 2 waves; `3711bd3` and `f142acf` show additional post-wave stabilization.
- Commit scope drift hurt history quality: `043d82f` is labeled as a chore note but includes major gameplay/UI/test changes (507 insertions, 191 deletions).
- Warning suppression created deferred cleanup debt: `c9d8e78` added broad `#[allow(dead_code)]` usage during asset wiring.
- Infra/content bundling created noisy reviews: `5a10bf8` mixed WASM pipeline work with 214-file broad asset/config churn.

## Immediate Adjustments (next 2 rotations)

- R5 adjustment: start with a contract delta commit, then enforce vertical slices with small file sets and at least one deterministic/headless assertion per slice.
- R5 adjustment: block `WIP`-style multi-domain mega commits on integration branches.
- R6 adjustment: schedule one explicit boundary audit to catch cross-domain leakage early (repeat `92596f4` behavior before drift expands).
- R6 adjustment: remove any temporary warning suppressions introduced in R5 within the same rotation.
- R6 adjustment: reserve a dedicated bug-bash/hardening step before the next feature burst to avoid reactive fix chains.

## Process Guardrails

- Flag commits above ~1,200 insertions or >20 files for mandatory split/review planning.
- Disallow `WIP` commits on integration branches.
- Reject `chore/docs` commits that also modify gameplay/system behavior.
- Require shared-contract changes to include wiring and headless test updates in the same PR.
- Disallow new `#[allow(dead_code)]` unless linked to a near-term cleanup task.
- Keep infra/build changes separate from gameplay/content changes.
- End each feature wave with a hardening + deterministic test expansion checkpoint.
