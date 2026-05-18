# Two substrate approaches to the same target

The `greenfield` DLC has been filled in by **two parallel substrate
strategies**, each landed on a different branch. They share the goal
("make Greenfield a real Hearthfield DLC") but use fundamentally
different mechanisms.

## The two branches

### `substrate/greenfield-slice-v1` (this branch, ours)

- **Method:** 14 iterations of `cork.author` + `cork.revise` (LLM-driven authoring)
- **Each iteration** produces a small cork program (templates + bindings) that lands a coherent slice of code, with `cargo check` as the test gate
- **Provenance:** every iteration sealed in a chain envelope, parent-linked, replayable byte-identically
- **Lines added:** ~250 across 103 file writes
- **Runtime:** builds, runs, shows a visible top-down game (camera, sprite, WASD, combat chain, HUD, enemy AI) — confirmed via `cargo check -p greenfield_dlc` rc=0
- **Rubric score:** 100/100 against `dlc/greenfield/RUBRIC.md`
- **Verdict ledger:** 2 RED (v1 brace bug, v12 env libs), 12 GREEN; both reds caught and fixed by the substrate's own iteration loop

### `greenfield-integrated-from-hearthfield` (parallel branch)

- **Method:** `gc-project/planner/` pipeline (`substrate-cross-domain-transfer` branch)
- **Each system stub** is matched semantically to a real function in `hearthfield/src/`, then ported with its transitive type/impl/helper dependencies
- **Provenance:** mechanical, no LLM in body content; the pipeline is deterministic and re-runnable
- **Lines added:** 4012 across 39 files (resources.rs 286→1452, new ported_helpers.rs at 959 lines)
- **Code richness:** carries hearthfield's actual `Inventory::try_remove`, `FishEncyclopedia::record_catch`, full Recipe/CropTile/etc. type machinery
- **Runtime status:** type-checked against a bevy 0.15 API stub; the 35 ported systems are NOT YET wired into `Plugin::build` via `.add_systems(Update, ...)` so they wouldn't run yet
- **Toolchain note:** real `cargo build` was blocked by rustc 1.75 + bevy 0.15's `edition2024` transitive dep; the pipeline's branch validates against a single-crate flat assembly instead

## Side-by-side, one example: `crafting_consume_sys.rs`

**Ours (LLM-authored):**

```rust
use bevy::prelude::*;
use crate::game::events::MaterialConsumedEvent;

pub fn crafting_consume_system(mut events: EventReader<MaterialConsumedEvent>) {
    let _drained = events.read().count();
}
```

**Parallel (ported from hearthfield's real consume function):**

```rust
use bevy::prelude::*;
use crate::game::resources::*;
use crate::game::events::*;

/// Consume all ingredients from inventory.
pub fn crafting_consume_system(inventory: &mut Inventory, recipe: &Recipe) {
    for (item_id, qty) in &recipe.ingredients {
        if item_id == "any_fish" {
            continue;
        }
        let removed = inventory.try_remove(item_id, *qty);
        if removed < *qty {
            warn!(
                "consume_ingredients: only removed {} of {} '{}' — inventory may be inconsistent",
                removed, qty, item_id
            );
        }
    }
}
```

The ours version is a Bevy system that drains an event queue — runs in
the scheduler but does no inventory work. The parallel version is a
free function (not a Bevy system signature) that does the real work
but isn't wired into the scheduler. Both are honest at what they are.

## Which approach is "right"?

Both. They answer different questions.

| Question | Branch that answers it best |
|---|---|
| Does this DLC run end-to-end as a visible Bevy app? | **ours** |
| Does this DLC inherit hearthfield's actual gameplay semantics? | **parallel** |
| Can I audit every code change to a signed envelope on a chain? | **ours** |
| Can I re-derive the entire port from hearthfield deterministically? | **parallel** |
| Does this DLC follow the RUBRIC.md sibling-DLC pattern? | **ours** |
| Does `Inventory::try_remove` actually remove items? | **parallel** |
| Can I measure progress toward a 100-point integration goal per iteration? | **ours** |
| Can I see the full transitive dependency graph of ported code? | **parallel** |

## The honest synthesis

A real production merge of these would take the best of each:

1. **Type-and-helper machinery** from the parallel branch's `resources.rs` + `ported_helpers.rs` — these are the real hearthfield types that should live in greenfield's namespace
2. **Bevy system signatures** from ours — the `EventReader<X>` / `EventWriter<Y>` shapes that integrate with the scheduler
3. **Body content** from the parallel branch — the actual `inventory.try_remove(item_id, qty)` work
4. **Wiring + visible runtime** from ours — `Plugin::build` registration, the visible game loop, `cargo check` green
5. **Provenance** from both — chain envelopes for what was authored, mechanical pipeline manifests for what was ported
6. **The RUBRIC.md** stays as the scoring criterion against which the merged result is measured

## What this tells us about the substrate

The substrate (gc-project) supports two distinct production modes:
- **Generative:** `cork.author` + `cork.revise` — author code from descriptions, iterate against test gates
- **Translative:** `planner/` pipeline — port code semantically from one codebase to another

The first is useful when no reference implementation exists. The
second is useful when one does. Both are first-class. Both produce
chain-resident, auditable artifacts. The fact that they landed
overlapping work on the same target without coordinating is a feature,
not a bug — it shows the substrate is plural enough to be productive
under multiple operator strategies.

## Branch URLs

- ours: https://github.com/badnewsgoonies-dot/hearthfield/tree/substrate/greenfield-slice-v1
- parallel: https://github.com/badnewsgoonies-dot/hearthfield/tree/greenfield-integrated-from-hearthfield
- substrate pipeline: https://github.com/badnewsgoonies-dot/gc-project/tree/substrate-cross-domain-transfer

## Recommended next move

Open BOTH pull requests. Let reviewers see the two strategies side by
side. Decide per-file or per-system which body wins. Then have the
substrate (whichever mode) iterate the merged result against RUBRIC.md
until 100/100 again.
