# Animal Cap Enforcement — Implementation Notes

**File modified:** `src/animals/spawning.rs` (only)

## What was added

Added cap enforcement in `handle_animal_purchase` immediately after the existing building-presence check and before any spawn logic. Two new system parameters were added:

- `animal_query: Query<&Animal>` — counts live animal entities by kind
- `mut toast_writer: EventWriter<ToastEvent>` — sends player-visible feedback

## Cap rules implemented

| Housing type | Animals | Max capacity |
|---|---|---|
| Coop | Chicken, Duck, Rabbit | `coop_level * 4` (4 / 8 / 12) |
| Barn | Cow, Sheep, Goat, Pig | `barn_level * 4` (4 / 8 / 12) |
| Roamers | Horse, Cat, Dog | 1 each (max 3 total) |

Roamers are capped at 1 per kind since the player can only own one horse, one cat, and one dog.

## Behaviour on cap hit

- `ToastEvent` is sent with `duration_secs: 3.0` and a descriptive message:
  - Coop full: `"Your coop is full! Upgrade to house more animals."`
  - Barn full: `"Your barn is full! Upgrade to house more animals."`
  - Roamer duplicate: `"You already have a [horse/cat/dog]!"`
- The purchase event loop `continue`s, skipping the spawn.
- **Gold is NOT refunded** — per spec, the shop system handles that separately.

## Pre-existing build errors (unrelated)

`cargo check` reports 3 errors in other files that pre-date this change:
- `src/npcs/quests.rs:791,793` — dereference of `u8` 
- `src/economy/shipping.rs:150` — missing `duration_secs` field in `ToastEvent`

These are not caused by and not fixed by this change.
