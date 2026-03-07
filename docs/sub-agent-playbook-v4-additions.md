# Sub-Agent Playbook v4 Additions

These additions are meant to extend the existing playbook with the failure classes observed in the office-DLC audit: entrypoint drift, asset dead-ends, unreachable content, save/load drift, and disconnected event buses.

## Recommended Additions

1. Add an `EntryPoint Gate`.
- Require every wave to name the exact runtime surface being built: host app, standalone crate, specific binary, or specific branch/folder.
- Why: otherwise teams build a real game in a side crate that never gets wired into the player-facing runtime.
- Tempting alternative: "it compiles somewhere, so it's fine."
- Consequence: false progress; shipped branch contains code that is not actually launch-path reachable.

2. Add a `First-60-Seconds Gate`.
- Before any mid/late-game work, require: boot, main menu, new game/load game, first spawn, first movement, first interaction, first save-worthy state.
- Why: this catches "beautiful dead game" failure early.
- Drift cue: workers build systems that only matter after minute 10 while first-seconds path is still fragile.

3. Add an `Asset Reachability Gate`.
- For every asset: classify as `runtime-used`, `present-but-unreferenced`, or `referenced-but-missing`.
- Why: "asset exists" is not the same as "player can ever see it."
- This catches dead art and dead UI in one pass.

4. Add a `Definition -> Acquisition -> Use/Sell -> Save` gate.
- Every content unit must trace through four questions:
- Is it defined?
- Can the player obtain it?
- Can the player do something meaningful with it?
- Does it survive save/load correctly?
- This is the cleanest way to kill fake depth.

5. Add an `Event Bus Connectivity Gate`.
- Every emitted event must have at least one runtime reader.
- Every reader-only event type must have a runtime producer.
- Why: disconnected events feel "implemented" in code review but are inert in play.

6. Add a `State Transition Audit`.
- Enumerate all game states and all legal transitions.
- Require each state to have:
- entry condition
- exit condition
- owner systems
- forbidden systems
- This prevents ghost states and declared-but-never-entered modes.

7. Add a `Save/Load Round-Trip Gate`.
- Snapshot mid-session state, reload, and assert:
- location/state restored
- progression restored
- generated content not duplicated/regenerated incorrectly
- no `OnEnter` system stomps restored state
- This is especially important for simulation games.

8. Add a `Tracked Noise Hygiene Rule`.
- Explicitly forbid tracked build outputs like `target/`, `dist/`, generated fingerprints, temp saves unless intentional.
- Why: they create false diff volume, hide real work, and make orchestration slower.

## Paste-In Section

```md
## Phase 5A — Reality Gates (required before any wave is considered complete)

### 5A.1 EntryPoint Gate
Name the exact player-facing runtime surface:
- binary / crate / branch / folder / launch command

Pass only if the implemented work is reachable from that runtime.
Code that compiles in an unwired side surface does not count as progress.

### 5A.2 First-60-Seconds Gate
Validate the player path from launch to first meaningful interaction:
- boot
- menu
- new game / load game
- spawn
- movement
- first interaction
- first persistent state change

If this path is unstable, stop all deeper feature work.

### 5A.3 Asset Reachability Gate
Classify every asset as:
- runtime-used
- present-but-unreferenced
- referenced-but-missing

No asset-heavy wave closes without this report.

### 5A.4 Content Reachability Gate
For every gameplay content unit:
- defined
- obtainable
- usable / sellable / consumable / progressable
- save/load safe

If a unit fails any step, mark it as dead content.

### 5A.5 Event Connectivity Gate
For each event:
- list producers
- list consumers

Fail if any event has no runtime producer or no runtime consumer unless explicitly marked future-work.

### 5A.6 Save/Load Round-Trip Gate
Create state, save, reload, verify:
- same location/state
- same progression/resources
- no duplicate generation
- no `OnEnter` overwrite drift
```

## Two Meta Tweaks

- Add a machine-readable worker report alongside markdown, like `status/workers/domain.json`. That makes later audits much easier.
- Add a hard rule that every worker report must state: "what is now player-reachable because of this work." That one sentence kills a lot of fake completion.
