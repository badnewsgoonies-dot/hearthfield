# Trial Festival Verify

## Findings

- [Observed] `FestivalState.timer` is runtime-only and is explicitly excluded from serialization with `#[serde(skip)]`, so a plain deserialize cannot restore an in-progress Egg Hunt timer.
  - source_refs: `src/calendar/festivals.rs:29`, `src/calendar/festivals.rs:33`, `src/calendar/festivals.rs:34`

- [Observed] The current `src/calendar/festivals.rs` does not define `FestivalState::restore_runtime_state()`. A repo search for `restore_runtime_state` under `src/calendar/` and `src/save/` returned no matches.
  - source_refs: `src/calendar/festivals.rs:1`

- [Observed] The real load flow in `handle_load_request` reads the save file and assigns `file.festival_state` directly into the live resource with `*ext.festival_state = file.festival_state;`.
  - source_refs: `src/save/mod.rs:918`, `src/save/mod.rs:934`, `src/save/mod.rs:979`, `src/save/mod.rs:983`

- [Observed] The real load flow does not perform any festival-specific post-load reconstruction after assigning `file.festival_state`. The logic continues into chest and machine restoration, map reload invalidation, and completion events.
  - source_refs: `src/save/mod.rs:983`, `src/save/mod.rs:986`, `src/save/mod.rs:1001`, `src/save/mod.rs:1043`, `src/save/mod.rs:1070`

- [Observed] The existing save tests in `tests/headless.rs` are serde round-trip tests only. The section is labeled `SAVE ROUND-TRIP TESTS`, uses a local `serde_roundtrip()` helper, and does not send `LoadRequestEvent`, add `SavePlugin`, or execute `handle_load_request`.
  - source_refs: `tests/headless.rs:3456`, `tests/headless.rs:3460`, `tests/headless.rs:3465`

- [Observed] The existing festival tests only cover festival-day activation and day-end cleanup. They do not exercise save/load.
  - source_refs: `tests/headless.rs:3015`, `tests/headless.rs:3044`, `tests/headless.rs:3071`, `tests/headless.rs:3094`

## Conclusion

- [Observed] The fix is missing in the current codebase state inspected here.
- [Observed] What is missing: a post-load festival runtime restoration step in the actual load path, either by reintroducing a `FestivalState::restore_runtime_state()` helper in `src/calendar/festivals.rs` and calling it from `handle_load_request`, or by equivalent inline reconstruction in `src/save/mod.rs`.
- [Assumed] This missing step is sufficient to explain the reported softlock: if a save is taken mid-Egg Hunt, `started` can persist while `timer` reloads as `None`, and `start_egg_hunt` will refuse to restart because it exits early when `festival.started` is already `true`.
  - source_refs: `src/calendar/festivals.rs:31`, `src/calendar/festivals.rs:33`, `src/calendar/festivals.rs:141`, `src/calendar/festivals.rs:144`, `src/calendar/festivals.rs:205`

## Action Taken

- [Observed] No code fix or regression test was added in this pass because the task specified: if the fix is missing, document exactly what is missing with source refs.
