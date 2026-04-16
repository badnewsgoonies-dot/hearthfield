# greenfield_dlc

A DLC skeleton shaped exactly for the GC-OS write-op transforms
(`briefcase_v015.hearthfield_transforms`). Every structural anchor in
this crate matches what the transforms expect:

- `app.init_state::<GreenfieldState>()` is the Plugin::build chain root
  where `hearthfield_add_event` / `hearthfield_register_system` splice
  new plumbing.
- `src/game/events.rs` has its `\Z` end-of-file anchor ready for unit
  Event structs.
- `src/game/components.rs` has the same for Component structs.
- `src/game/systems/` hosts module files that `hearthfield_register_system`
  wires into the build chain once their fns exist.

## Propagation contract

The point of this crate is that the **mechanical** transforms can
build it up from an inbox without human intervention, one cargo-gated
write-op at a time. Populate `.gc-write-ops-inbox.json` at the
hearthfield repo root and run:

```
python3 -m orchestrator.daemon --mode write \
  --project /home/geni/hearthfield \
  --cargo-package greenfield_dlc
```

Each entry is applied, `cargo check -p greenfield_dlc` runs, and the
change is kept (on green) or reverted byte-for-byte (on regression).
Successes and failures land in `.gc-write-ops-log/{success,failure}/`.

## Not here (yet)

Things the current transform set can't author:

- Fielded Events / Components (only unit structs)
- Resources (no `hearthfield_add_resource` yet)
- New system-module files with fn bodies (register_system only wires
  pre-existing fns)
- Use imports, enums, trait impls

These are additive transforms — each one a self-contained regex-splice
on a stable anchor. Add them, re-run write-mode, watch the crate grow.
