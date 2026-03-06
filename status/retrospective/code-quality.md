# Code Quality Audit (Worker 3)

Date: 2026-03-06
Repo: `/home/user/hearthfield`

## 1) `cargo clippy -- -D warnings`

Result: **PASS**

- Command: `cargo clippy -- -D warnings`
- Outcome: finished successfully (`dev` profile), no warnings/errors under `-D warnings`.

## 2) Largest Rust source files (`src`)

Command: `find src -name '*.rs' -exec wc -l {} + | sort -rn | head -20`

```
  46454 total
   2315 src/shared/mod.rs
   2251 src/world/objects.rs
   1915 src/data/items.rs
   1847 src/npcs/schedules.rs
   1591 src/ui/hud.rs
   1485 src/data/npcs.rs
   1162 src/save/mod.rs
   1098 src/world/maps.rs
   1027 src/npcs/quests.rs
    914 src/world/mod.rs
    827 src/calendar/mod.rs
    733 src/npcs/dialogue.rs
    712 src/data/recipes.rs
    693 src/ui/shop_screen.rs
    689 src/calendar/festivals.rs
    663 src/input/mod.rs
    642 src/crafting/machines.rs
    586 src/npcs/romance.rs
    538 src/farming/render.rs
```

## 3) Marker grep: `TODO|FIXME|HACK|XXX`

Commands run:
- `rg -n --hidden --glob '!.git' "TODO|FIXME|HACK|XXX"`
- `rg -n --hidden --glob '!.git' "TODO|FIXME|HACK|XXX" | wc -l`
- `rg -n --hidden --glob '!.git' "TODO|FIXME|HACK|XXX" src tests | wc -l`
- `rg -n "TODO|FIXME|HACK|XXX" --glob '**/*.rs' | wc -l`

Counts:
- Whole repo (excluding `.git`): **20** matches
- `src` + `tests`: **0** matches
- All Rust files (`**/*.rs`): **0** matches

Representative examples (non-code/docs/status context):
- `dlc/city/TODO.md:1`
- `objectives/fix-pilot-load-game.md:4`
- `status/workers/impl_shipping.md:27`

## 4) `unwrap()` usage

Commands:
- `rg -n "unwrap\(\)" src tests`
- `rg -n "unwrap\(\)" src tests | wc -l`
- `rg -n "unwrap\(\)" src tests | cut -d: -f1 | sort | uniq -c | sort -rn`

Counts:
- `src` + `tests`: **30**
- By file:
  - `tests/headless.rs`: **17**
  - `src/shared/mod.rs`: **8**
  - `src/npcs/schedules.rs`: **5**

Representative examples:
- `tests/headless.rs:238` (`farm_state.crops.get(...).unwrap()`)
- `src/shared/mod.rs:1927` (`tier.next().unwrap()`)
- `src/npcs/schedules.rs:1783` (`enhanced_schedule(...).unwrap()`)

Observation: `unwrap()` is concentrated in tests and test-like sections; no broad runtime spread.

## 5) `clone()` usage and potential overuse patterns

Commands:
- `rg -n "\.clone\(" src tests`
- `rg -n "\.clone\(" src tests | wc -l`
- `rg -n "\.clone\(" src tests | cut -d: -f1 | sort | uniq -c | sort -rn | head -20`

Counts:
- `src` + `tests`: **496**
- All Rust files (`**/*.rs`): **782**

Top files (`src` + `tests`):
- `src/save/mod.rs`: 41
- `src/ui/hud.rs`: 36
- `src/world/objects.rs`: 33
- `src/npcs/quests.rs`: 24
- `src/ui/journal_screen.rs`: 21

Representative patterns:
- Resource snapshot/build structs with many field clones:
  - `src/save/mod.rs:526-560` (many `.clone()` assignments)
- UI handle/text cloning in builder flows:
  - `src/ui/hud.rs:225`, `src/ui/hud.rs:496`
- ID/string propagation in events/maps:
  - `src/npcs/quests.rs:686`, `src/economy/shipping.rs:162`

Assessment: high clone volume is expected in ECS/event + Bevy handle/value passing, but `src/save/mod.rs` is a hotspot worth targeted profiling/review for avoidable full-structure clones.

## 6) `#[allow(clippy::...)` suppressions

Commands:
- `rg -n "#\[allow\(clippy::" src tests`
- `rg -n "#\[allow\(clippy::" src tests | wc -l`
- `rg -n "#\[allow\(clippy::" src | wc -l`
- category rollup by lint name

Counts:
- `src` + `tests`: **70**
- `src` only: **69**
- Category rollup (`src` + `tests`):
  - `too_many_arguments`: **65**
  - `type_complexity`: **4**
  - `assertions_on_constants`: **1**

Representative examples:
- `src/world/mod.rs:289` (`too_many_arguments`)
- `src/save/mod.rs:480` (`too_many_arguments`)
- `src/world/weather_fx.rs:198` (`type_complexity`)
- `tests/headless.rs:2806` (`assertions_on_constants`)

Assessment: suppressions are numerous and mostly clustered around function signature complexity.

## 7) `tests/headless.rs` coverage assessment

Inventory:
- `tests/headless.rs` has **90 tests** (`rg -n "^fn test_" tests/headless.rs | wc -l`)
- File size: ~100k lines includes broad gameplay/system tests.

Covered well (representative):
- Farming growth/water/death/season checks (`test_crop_*`, `test_reset_soil_watered_state`)
- Shipping day-end accounting (`test_shipping_bin_*`, multi-day accumulation)
- Animals happiness/feed/starvation/quality (`test_animal_*`, `test_quality_from_happiness`)
- Calendar progression/festival/day math
- Economy stats/gold events
- Sprinklers (placement + affected tiles)
- Building/tool upgrades
- Romance progression + wedding timer
- Quest completion/expiry
- Evaluation trigger/scoring/candles
- Some low-level rendering/math behavior (`ysort`, logical position, animation state helpers)

Not covered or thinly covered:
- Save/load serialization and migration paths (`src/save/mod.rs`)
- Mining gameplay loop (combat/movement/transitions/ladder/rock-breaking)
- Fishing runtime loop (cast/minigame/resolve integration)
- Full NPC schedule/dialogue/map-event runtime integration
- UI screen interaction behavior (most UI logic untested in integration form)
- Error-path/negative-path robustness in many systems (tests are mostly happy-path and rule-path)

## 8) Error handling assessment (`Result` vs panic)

Commands:
- `rg -n -- "->\s*Result<" src | wc -l`
- `rg -n -- "->\s*Option<" src | wc -l`
- `rg -n "panic!\(|expect\(|unwrap\(\)" src tests`

Counts:
- `Result` return signatures in `src`: **7**
- `Option` return signatures in `src`: **33**
- `panic!/expect()/unwrap()` in `src` + `tests`: **33** (30 unwrap + 3 panic/expect)

Panic/expect examples:
- `tests/keybinding_duplicates.rs:41` (`panic!` in test assertion path)
- `tests/headless.rs:2925` (`panic!` in test assertion path)
- `src/ui/main_menu.rs:439` (`expect("path should resolve")`)

Assessment:
- Production code relies more on state/event flow and `Option` than explicit `Result` plumbing.
- Panic-style handling is mostly confined to tests.
- One production `expect` in `src/ui/main_menu.rs` is a crash risk if DLC path resolution fails unexpectedly; this is the primary runtime panic-style hotspot observed.

## 9) Overall quality notes

- Strict clippy gate currently passes, which is a strong baseline.
- Main maintainability pressure points are large modules, heavy `.clone()` density in certain hotspots, and high `clippy::too_many_arguments` suppression count.
- Test breadth in `tests/headless.rs` is good for farming/economy/social progression, with notable gaps in save/load, mining, fishing integration, and broader UI/runtime error paths.
