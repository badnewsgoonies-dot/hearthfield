# Greenfield ← Hearthfield integration

This branch contains 35 systems whose bodies were ported from hearthfield's
production codebase by the substrate pipeline at
`gc-project/planner/` (branch `substrate-cross-domain-transfer`).

## Mechanical pipeline

For each greenfield stub `pub fn foo_system() {}`:

1. **Semantic match** — role-based name matcher (no LLM) picks the closest
   hearthfield fn by structural verb-role overlap. Low-confidence stubs
   resolve via a curated alias map.

2. **Per-stub assembly** — body, transitively-required types, impl blocks,
   helper fns, and consts are gathered into a self-contained .rs.

3. **Integration** — the integrator merges:
   - new types/consts → `src/game/resources.rs` (+1166 lines, 286 → 1452)
   - helper fns + impl blocks → `src/game/ported_helpers.rs` (new, 959 lines)
   - each stub fn rewritten to reference `crate::game::resources::*`

4. **Cross-port dependency sweep** — promotes identifiers referenced by one
   port but defined elsewhere. Falls back to scanning hearthfield/src/ for
   any identifier the pipeline missed.

## Validation

A flat single-crate test was assembled at `/tmp/real_validate/integrated_test.rs`
combining resources.rs + events.rs + components.rs + ported_helpers.rs + every
integrated system as nested modules.

That 5,385-line single crate compiles cleanly under rustc 1.75 against a
bevy 0.15 API stub (proc-macro derives stripped — see Pending below).

```
$ rustc --edition=2021 --crate-type=lib --extern bevy=libbevy.rlib \
    --emit=metadata -o integrated.rmeta integrated_test.rs
$ echo $?
0
$ ls -la integrated.rmeta
-rw-r--r-- 1 root root 1684656 May 18 14:41 integrated.rmeta
```

## Pending (mechanical, not architectural)

1. **Plugin registration** — the 35 systems are declared but not yet wired
   into `plugins.rs` `Plugin::build()` methods via `.add_systems(Update, ...)`.
   Without this, bevy's scheduler won't run them at runtime.

2. **Real cargo build against bevy 0.15** — blocked by rustc 1.75 in the
   build environment (bevy 0.15's transitive `hashbrown 0.17` requires
   `edition2024`, stable only on rustc ≥ 1.85). The integration is shape-
   correct; bumping the toolchain will let cargo see the same thing.

3. **Bevy proc-macro derives** — the validation harness stripped
   `#[derive(Resource)]`, `#[derive(Component)]`, `#[derive(Event)]` etc.
   because the rustc stub can't provide them. The on-disk files in this
   branch preserve them; a real bevy build will resolve them.

## What this branch is and is not

| Is | Is not |
|---|---|
| Real Rust referencing greenfield's actual module hierarchy | A playable game |
| Type-checked against bevy's API surface | Validated at runtime |
| Reproducible: re-run the planner pipeline and get the same output | A finished port |
| 35 of 65 systems with real, hearthfield-derived behavior | All 65 systems |
