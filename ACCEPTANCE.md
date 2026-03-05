# ACCEPTANCE.md — Hearthfield
# Single source of truth for "definition of done"
# Last updated: 2026-03-05
#
# RULES:
# - The implementer (any worker/agent) may NOT edit this file.
# - If a check is wrong or impossible, write to CHANGE_REQUEST.md with justification.
# - A separate verifier agent may extend this file, but must cite the SPEC bullet for every addition.

---

## 1. Build & Verify Commands

The ONE command that means "this project is healthy":

```bash
./scripts/run-gates.sh
```

Which runs, in order:
1. Contract integrity check (`shasum -a 256 -c .contract.sha256`)
2. Type check (`cargo check`)
3. Integration tests (`cargo test --test headless`)
4. Lint gate (`cargo clippy -- -D warnings`)
5. Connectivity check (no hermetic domains)

DLC gates (added to run-gates.sh or run separately):
```bash
cargo test -p city_office_worker_dlc
cargo test -p skywarden
```

**Done = all of the above pass with zero failures and zero skipped tests.**

---

## 2. Non-Negotiable Constraints

Each must be mechanically enforced — not just stated.

| Constraint | How it's enforced |
|---|---|
| Type contract is frozen | `.contract.sha256` checksum in Gate 1 |
| No out-of-scope edits by workers | `scripts/clamp-scope.sh` post-revert |
| Zero clippy warnings | Gate 4: `-D warnings` flag |
| All domains import from shared contract | Gate 5: connectivity grep |
| No network calls at runtime | (add test guard — see Section 5) |
| Deterministic behavior with fixed seed | Tests: `fixed_seed_three_day_replay_is_deterministic` (City DLC Gate G3), equivalent needed in base game |
| Rust stable compiles | Implicit in Gate 2 |

---

## 3. Base Game — User-Visible Outcomes

Source of truth: `GAME_SPEC.md`, `PLAYER_EXPERIENCE_PLAN.md`, `ROADMAP_TO_SHIP.md`

A new player starting `cargo run -p hearthfield` must be able to:

### Phase 1: Playable (Days 1–7 without soft-locks)
- [ ] Game launches without crash or panic
- [ ] Player spawns at farm (not stuck on beach or in wall)
- [ ] Tutorial prompt appears on Day 1 (controls hint)
- [ ] Player can exit the house (door interaction works)
- [ ] Player can enter all 4 wired buildings without getting stuck
- [ ] Player can reach the shipping bin and interact with it
- [ ] Sleep interaction advances the day
- [ ] Days 1–7 completable without soft-lock or unrecoverable state

**Spec reference:** `ROADMAP_TO_SHIP.md` Phase 1 exit criteria

### Phase 2: Core Loops

**Farming** (verified working per ROADMAP):
- [x] Till → plant → water → grow → harvest → ship → receive gold
- [x] Crop quality: Normal/Silver/Gold/Iridium sell at 1.0x/1.25x/1.5x/2.0x
- [ ] Planted crop display (not colored dots — uses crop growth sprites)
- [ ] Wrong-season crop wither produces visible feedback

**Fishing** (needs verification):
- [ ] Cast → bite trigger → minigame → catch → added to inventory → can be shipped
- [ ] Fish selection filters by location, season, and time of day
- [ ] Fishing skill progresses (visible change in catch rate or unlock)
- [ ] Cast fails at non-water tile with toast "Can't fish here."
- [ ] Escaped fish produces toast "The fish got away!"

**Mining** (needs verification):
- [ ] Enter mine → break rocks → collect ores → fight enemies → reach ladder → descend
- [ ] Elevator unlocks appear at floors 5, 10, 15, 20
- [ ] Knockout: gold penalty applied, player teleports home, no save corruption
- [ ] Floor transition shows toast "Floor {n}"
- [ ] Player takes damage from enemies with visible feedback (toast or HP bar flash)

**Crafting** (needs verification):
- [ ] Open crafting bench → select valid recipe → craft → item added to inventory
- [ ] Insufficient ingredients: toast "Missing ingredients!" (no silent failure)
- [ ] Successful craft: toast "{item_name} crafted!"
- [ ] Cooking at kitchen stove works (requires house upgrade)
- [ ] Machines (furnace, preserves jar): can place, insert input, collect output

**Social** (needs verification):
- [ ] Talk to NPC → friendship gain > 0
- [ ] Gift item to NPC → correct point delta (Loved +80, Liked +45, Neutral +20, etc.)
- [ ] Birthday gift delivers 8× multiplier
- [ ] Quest accept → track → complete → reward

**Economy**:
- [ ] Buy seeds at General Store, receive correct item and correct gold deduction
- [ ] Tool upgrade at Blacksmith costs correct amount and delivers upgraded tool
- [ ] Shipping bin sale shows floating gold text "+{n}g"

**Spec reference:** `ROADMAP_TO_SHIP.md` Phase 2 + `GAME_SPEC.md`

---

## 4. City Office Worker DLC — User-Visible Outcomes

Source of truth: `dlc/city/README.md`, `dlc/city/CONTRACT.md`, `dlc/city/STATUS.md`

`cargo run -p city_office_worker_dlc` must:

- [ ] Launch and display the main menu without panic
- [ ] Start a new game and load into the office map

**3-day loop (minimum viable playthrough):**
- [ ] Day 1: Player can pull tasks from the board, complete at least one task before deadline, handle one interruption, submit end-of-day report
- [ ] Day 2: Stats from Day 1 (salary, reputation) persist correctly to Day 2
- [ ] Day 3: Day-scaling applies (Day 3 tasks harder/different than Day 1)
- [ ] Save → quit → load → resume from correct day and state

**Data integrity:**
- [ ] Task definitions are non-empty (≥ 1 task template per kind/priority combination)
- [ ] Task IDs are unique within a day
- [ ] End-of-day summary shows correct values (tasks completed, salary change, stress level)

**Quality gates (all 14 already green — must stay green):**
- [ ] G0–G14 all pass (see `dlc/city/STATUS.md`)
- [ ] `cargo test -p city_office_worker_dlc` passes (30 tests, zero skipped)
- [ ] `cargo clippy -p city_office_worker_dlc -- -D warnings` passes

---

## 5. Pilot DLC (Skywarden) — User-Visible Outcomes

Source of truth: `dlc/pilot/src/`, `dlc/pilot/tests/headless.rs`

`cargo run -p skywarden` must:

- [ ] Launch without panic (Main Menu appears)
- [ ] New Game flow loads the pilot into first assignment

**2-flight/duty-cycle loop (minimum viable playthrough):**
- [ ] Flight 1: Player can accept a flight assignment, complete it (or fail it gracefully), return to base
- [ ] Flight 2: Outcome from Flight 1 (reputation, fatigue) persists into Flight 2 setup
- [ ] No crash or panic during either flight

**Data integrity:**
- [ ] `airports` data: ≥ 1 airport, all fields non-empty, unique IDs
- [ ] `aircraft` data: ≥ 1 aircraft with valid stats
- [ ] `crew` data: ≥ 1 crew member with valid attributes
- [ ] No aircraft/crew/airport ID references unresolvable entities

**Test coverage:**
- [ ] `cargo test -p skywarden` passes (target ≥ 68 tests)
- [ ] Save/load round-trip test passes
- [ ] At least one property-style invariant test for flight simulation

---

## 6. Save System

Source of truth: `ROADMAP_TO_SHIP.md` Phase 5 + `docs/domains/save.md`

- [x] 37 data structures survive save/load round-trip (verified)
- [ ] Save at each season boundary → load → state matches pre-save state
- [ ] Save does not corrupt on mid-day save (stamina, inventory, time all correct)
- [ ] DLC saves are isolated from base game saves (no cross-contamination)

---

## 7. Invariants (Must Never Be Violated)

These are not features — they are constraints the world enforces.

| Invariant | Check form |
|---|---|
| Player HP ≥ 0 | Test: damage beyond max HP clamps, not underflows |
| Stamina ∈ [0, 100] | Test: overconsumption clamps at 0 |
| Day never goes backward | Test: sleep always advances day_index by exactly 1 |
| Friendship ∈ [0, 1000] (10 hearts × 100) | Test: gift overflow clamps |
| Gold never goes negative (buying fails, not deducts) | Test: buy with insufficient gold = rejection |
| Inventory slots: max 36 (12 hotbar + 24 backpack) | Test: overflow rejects, not corrupts |
| Type contract checksum unchanged | Gate 1 |

---

## 8. Anti-Cheating Rules (for any agent writing or modifying tests)

1. Every new test must include a comment citing the SPEC bullet it encodes:
   `// SPEC: GAME_SPEC.md – Farming loop: harvest → ship → gold`

2. Do NOT write tests that snapshot currently buggy behavior. If a test passes because of a bug, it is not a valid test.

3. If the SPEC is underspecified or inconsistent, write to `CHANGE_REQUEST.md` — do not silently encode your own spec.

4. Tests must be falsifiable: a wrong implementation must fail them. If a test passes trivially (e.g. `assert!(true)`), it is not a test.

5. Determinism tests must use a fixed seed and compare two independent runs, not just one.

---

## Completion Checklist

You are done **only** when:

- [ ] `./scripts/run-gates.sh` passes (base game: contract + check + tests + clippy + connectivity)
- [ ] `cargo test -p city_office_worker_dlc` passes (30 tests)
- [ ] `cargo test -p skywarden` passes (≥ 68 tests)
- [ ] All Phase 1 checkboxes above are checked
- [ ] All Phase 2 loop checkboxes above are checked (farming, fishing, mining, crafting, social, economy)
- [ ] City DLC 3-day loop checkboxes checked
- [ ] Pilot DLC 2-flight loop checkboxes checked
- [ ] All invariants hold (Section 7)
- [ ] `MANIFEST.md` updated with final status
- [ ] `status/AI_STATUS_OVERVIEW.md` exists with discovery report
