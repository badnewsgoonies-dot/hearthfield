# Bounded Types Wiring Plan

**Generated:** 2026-03-19  
**Scope:** Wire `PlayerState.stamina`, `PlayerState.health`, `PlayerState.gold` to bounded newtypes  
**Constraint:** Do NOT modify `src/shared/mod.rs` or `src/shared/bounded_types.rs`

---

## 1. Generated API (from `#[game_value]` macro in `ironclad/src/game_value.rs`)

The macro generates for each type (e.g. `Stamina(f32)`):

```rust
impl Stamina {
    pub fn new(val: f32) -> Result<Self, String>       // range-checked
    pub fn new_unchecked(val: f32) -> Self             // bypass check
    pub fn get(self) -> f32                            // extract inner
}
impl Deref for Stamina { type Target = f32; }          // &Stamina -> &f32
impl Display for Stamina { ... }                       // format transparent
impl Serialize/Deserialize for Stamina { ... }         // transparent serde
```

**What Deref makes transparent:**
- Method calls: `player.stamina.min(x)` auto-derefs to `f32::min` ✅
- Display/format: `format!("{}", player.stamina)` uses `Display` ✅
- Explicit deref: `*player.stamina` yields `f32` ✅

**What Deref does NOT cover:**
- Binary operators: `player.stamina <= 0.0` — Rust won't auto-deref for `PartialOrd<f32>` ❌
- Arithmetic: `player.stamina - ev.amount` — `Sub<f32>` not implemented ❌
- Compound assignment: `player.gold -= cost` — `SubAssign<u32>` not implemented ❌
- Assignment: `player.stamina = 5.0` — type mismatch `f32` vs `Stamina` ❌
- Casts: `player.gold as f32` — can't cast newtype directly ❌

---

## 2. Files with Direct `PlayerState` Field Access

### 2a. `stamina: f32` → `Stamina`

**`src/player/tools.rs`** (4 accesses)
| Line | Pattern | Transparent? | Change Needed |
|------|---------|-------------|---------------|
| 88 | `if player_state.stamina < cost` | ❌ | `*player_state.stamina < cost` |
| 154 | `player_state.stamina = (player_state.stamina - ev.amount).max(0.0)` | ❌ | `player_state.stamina = Stamina::new_unchecked((player_state.stamina.get() - ev.amount).max(0.0))` |
| 167 | `if player_state.stamina <= threshold` | ❌ | `*player_state.stamina <= threshold` |
| 176 | `if player_state.stamina > threshold` | ❌ | `*player_state.stamina > threshold` |

**`src/crafting/buffs.rs`** (4 accesses)
| Line | Pattern | Transparent? | Change Needed |
|------|---------|-------------|---------------|
| 194 | `let before = player_state.stamina;` | ⚠️ | `let before: Stamina = player_state.stamina;` — type annotation needed if used in subtraction |
| 196 | `player_state.stamina = (player_state.stamina + restored).min(player_state.max_stamina)` | ❌ | `Stamina::new_unchecked((player_state.stamina.get() + restored).min(player_state.max_stamina))` |
| 197 | `player_state.stamina - before` | ❌ | `player_state.stamina.get() - before.get()` |
| 349 | `player_state.stamina = player_state.stamina.min(original)` | ❌ | `Stamina::new_unchecked(player_state.stamina.get().min(original))` (note: `.min()` deref works on RHS, but assignment target type is `Stamina`) |

**`src/player/interaction.rs`** (2 accesses)
| Line | Pattern | Transparent? | Change Needed |
|------|---------|-------------|---------------|
| 671 | `if player_state.stamina <= 0.0` | ❌ | `*player_state.stamina <= 0.0` |
| 685 | `else if player_state.stamina > 0.0` | ❌ | `*player_state.stamina > 0.0` |

**`src/ui/tutorial.rs`** (1 access)
| Line | Pattern | Transparent? | Change Needed |
|------|---------|-------------|---------------|
| 104 | `player_state.stamina < 20.0` | ❌ | `*player_state.stamina < 20.0` |

**`src/crafting/buffs.rs`** also reads `player_state.max_stamina` (`f32`) for `.min()` clamping — that field stays `f32`, so no change needed there.

---

### 2b. `health: f32` → `Health`

**`src/mining/combat.rs`** (2 accesses — only file)
| Line | Pattern | Transparent? | Change Needed |
|------|---------|-------------|---------------|
| 251 | `player_state.health = (player_state.health - damage).max(0.0)` | ❌ | `player_state.health = Health::new_unchecked((player_state.health.get() - damage).max(0.0))` |
| 288 | `if player_state.health <= 0.0` | ❌ | `*player_state.health <= 0.0` |

> Note: `monster.health`, `rock.health`, `obj_data.health`, `data.health` in other files are **not** `PlayerState.health` — they are component fields on separate structs. No changes needed in those files.

---

### 2c. `gold: u32` → `Gold`

**`src/economy/gold.rs`** (8 accesses — primary gold mutation site)
| Line | Pattern | Transparent? | Change Needed |
|------|---------|-------------|---------------|
| 24 | `player_state.gold = player_state.gold.saturating_add(gain)` | ❌ | `player_state.gold = Gold::new_unchecked(player_state.gold.get().saturating_add(gain))` |
| 28 | `format!("… {}", player_state.gold)` | ✅ | None (Display impl) |
| 32 | `if player_state.gold >= cost` | ❌ | `*player_state.gold >= cost` |
| 33 | `player_state.gold -= cost` | ❌ | `player_state.gold = Gold::new_unchecked(player_state.gold.get() - cost)` |
| 37 | `format!("… {}", player_state.gold)` | ✅ | None |
| 44 | `format!("… {}", player_state.gold)` | ✅ | None |
| 48 | `.saturating_add(player_state.gold as u64)` | ❌ | `player_state.gold.get() as u64` |
| 49 | `player_state.gold = 0` | ❌ | `player_state.gold = Gold::new_unchecked(0)` |

**`src/economy/shop.rs`** (9 accesses, including tests)
| Line | Pattern | Transparent? | Change Needed |
|------|---------|-------------|---------------|
| 87 | `player_state.gold` (format arg) | ✅ | None |
| 107 | `let gold = player_state.gold;` | ⚠️ | `let gold: Gold = player_state.gold;` — or `player_state.gold.get()` if used as u32 |
| 208 | `if player_state.gold < max_cost` | ❌ | `*player_state.gold < max_cost` |
| 211 | `have: player_state.gold` (struct field of type u32) | ❌ | `have: player_state.gold.get()` |
| 226 | `player_state.gold = player_state.gold.saturating_sub(total_cost)` | ❌ | `player_state.gold = Gold::new_unchecked(player_state.gold.get().saturating_sub(total_cost))` |
| 272 | `player_state.gold = player_state.gold.saturating_add(total_revenue)` | ❌ | `Gold::new_unchecked(player_state.gold.get().saturating_add(total_revenue))` |
| 343 | `ps.gold = gold;` (test setup, gold: u32) | ❌ | `ps.gold = Gold::new_unchecked(gold)` |
| 355/402/423 | `assert_eq!(player.gold, 400)` etc. | ❌ | `assert_eq!(*player.gold, 400)` or `.get()` |

**`src/economy/achievements.rs`** (3 accesses, all tests/checks)
| Line | Pattern | Transparent? | Change Needed |
|------|---------|-------------|---------------|
| 234 | `player.gold >= 1_000_000` | ❌ | `*player.gold >= 1_000_000` |
| 614 | `player.gold = 999_999` (test) | ❌ | `player.gold = Gold::new_unchecked(999_999)` |
| 629 | `player.gold = 1_000_000` (test) | ❌ | `player.gold = Gold::new_unchecked(1_000_000)` — exceeds declared max 999_999; `new()` would fail; use `new_unchecked` |

> ⚠️ **Edge case**: `Gold` is declared `#[game_value(min = 0, max = 999999)]`. The achievement test sets `player.gold = 1_000_000` to verify the millionaire check. `Gold::new(1_000_000)` returns `Err`. Tests must use `new_unchecked` for this value, or the max bound in `bounded_types.rs` must be raised (not allowed here). Flag for the implementer.

**`src/economy/evaluation.rs`** (1 access)
| Line | Pattern | Transparent? | Change Needed |
|------|---------|-------------|---------------|
| 202 | `if player_state.gold >= 1_000_000` | ❌ | `*player_state.gold >= 1_000_000` |

**`src/economy/tool_upgrades.rs`** (4 accesses, all tests)
| Line | Pattern | Transparent? | Change Needed |
|------|---------|-------------|---------------|
| 173/188/202/215 | `player.gold = 10_000` etc. | ❌ | `player.gold = Gold::new_unchecked(10_000)` |

**`src/mining/combat.rs`** (1 access)
| Line | Pattern | Transparent? | Change Needed |
|------|---------|-------------|---------------|
| 295 | `player_state.gold as f32 * 0.10` | ❌ | `player_state.gold.get() as f32 * 0.10` |

**`src/mining/transitions.rs`** (1 access)
| Line | Pattern | Transparent? | Change Needed |
|------|---------|-------------|---------------|
| 85 | `player_state.gold as f32 * 0.10` | ❌ | `player_state.gold.get() as f32 * 0.10` |

**`src/save/mod.rs`** (1 access)
| Line | Pattern | Transparent? | Change Needed |
|------|---------|-------------|---------------|
| 467 | `gold: self.player_state.gold` (assigned to u32 field in save struct) | ❌ | `gold: self.player_state.gold.get()` |

**`src/ui/building_upgrade_menu.rs`** (2 accesses)
| Line | Pattern | Transparent? | Change Needed |
|------|---------|-------------|---------------|
| 395 | `if player_state.gold < entry.cost_gold` | ❌ | `*player_state.gold < entry.cost_gold` |
| 398 | `format!("… {}", player_state.gold)` | ✅ | None |

**`src/ui/hud.rs`** (1 access)
| Line | Pattern | Transparent? | Change Needed |
|------|---------|-------------|---------------|
| 1002 | `format!("{} G", player.gold)` | ✅ | None |

**`src/ui/main_menu.rs`** (1 access — `info.gold`, not `PlayerState.gold`)
> `info.gold` is a separate save-info struct field (type u32), not `PlayerState`. No change needed when wiring `PlayerState.gold`.

**`src/ui/shop_screen.rs`** (6 accesses)
| Line | Pattern | Transparent? | Change Needed |
|------|---------|-------------|---------------|
| 190 | `format!("{} G", player.gold)` | ✅ | None |
| 419 | `format!("{} G", player.gold)` | ✅ | None |
| 491 | `if listing.price > player.gold` | ❌ | `listing.price > *player.gold` |
| 633 | `if player.gold >= listing.price` | ❌ | `*player.gold >= listing.price` |
| 640 | `player.gold -= listing.price` | ❌ | `player.gold = Gold::new_unchecked(player.gold.get() - listing.price)` |
| 679 | `player.gold += price` | ❌ | `player.gold = Gold::new_unchecked(player.gold.get() + price)` |

---

## 3. Deref Summary

| Operation | Via Deref? | Required Change |
|-----------|-----------|----------------|
| `format!("{}", x)` | ✅ Display impl | None |
| `x.method()` where method is on inner type | ✅ auto-deref | None |
| `x == y`, `x < y` (comparing to raw primitive) | ❌ | `*x == y` or `x.get() == y` |
| `x - y`, `x + y` arithmetic with raw primitive | ❌ | `x.get() - y` |
| `x as T` cast | ❌ | `x.get() as T` |
| `x += y`, `x -= y` compound assignment | ❌ | `x = Wrapper::new_unchecked(x.get() op y)` |
| `x = raw_value` assignment | ❌ | `x = Wrapper::new_unchecked(raw_value)` |
| `x = x.saturating_add(y)` | ❌ | `x = Wrapper::new_unchecked(x.get().saturating_add(y))` |
| Struct literal or test setup | ❌ | `field: Wrapper::new_unchecked(val)` |

**Bottom line:** Deref is helpful for format strings and some method calls, but the vast majority of access sites require explicit `.get()` or `new_unchecked()` changes. There is no zero-change migration path.

---

## 4. Priority Order for Wiring

### Priority 1 — `health: f32` → `Health` ✅ *Safest first*
- **Only 2 callsites**, both in a single file (`src/mining/combat.rs`)
- No test code touching it
- Simple patterns: one comparison, one arithmetic-assign
- No cross-domain entanglement
- **Risk:** Very low. 2 mechanical changes.

### Priority 2 — `stamina: f32` → `Stamina`
- **11 callsites** across 4 files: `player/tools.rs`, `crafting/buffs.rs`, `player/interaction.rs`, `ui/tutorial.rs`
- All in the player/crafting domain cluster
- Mix of comparisons and mutation; one tricky `before` variable binding
- No casts or saturating ops
- **Risk:** Low-medium. Contained domain. 8–10 mechanical changes.

### Priority 3 — `gold: u32` → `Gold` ⚠️ *Most complex, do last*
- **30+ callsites** across 10 files spanning every major domain
- Compound assignment operators (`+=`, `-=`) in shop_screen require careful rewrite
- `as f32` casts in mining require `.get()`
- Test setup via direct assignment in 3 separate test modules
- **Critical edge case:** `Gold::new(1_000_000)` panics/errors because `bounded_types.rs` declares `max = 999_999`. The millionaire achievement test (`achievements.rs:629`) and evaluation check (`evaluation.rs:202`) both use 1,000,000. The implementer must either:
  - Use `new_unchecked(1_000_000)` in tests (acceptable for test setup)
  - Or raise the max in `bounded_types.rs` to 1_000_000 (but that file is off-limits here)
  - Flag this as a spec conflict before implementing
- **Risk:** Medium-high. Most files, most operator variety, spec conflict on max bound.

---

## 5. File-by-File Change Count Summary

| File | Field | Callsites | Transparent | Needs Change |
|------|-------|-----------|-------------|-------------|
| `src/mining/combat.rs` | health | 2 | 0 | 2 |
| `src/player/interaction.rs` | stamina | 2 | 0 | 2 |
| `src/ui/tutorial.rs` | stamina | 1 | 0 | 1 |
| `src/player/tools.rs` | stamina | 4 | 0 | 4 |
| `src/crafting/buffs.rs` | stamina | 4 | 0 | 4 |
| `src/economy/evaluation.rs` | gold | 1 | 0 | 1 |
| `src/mining/transitions.rs` | gold | 1 | 0 | 1 |
| `src/mining/combat.rs` | gold | 1 | 0 | 1 |
| `src/save/mod.rs` | gold | 1 | 0 | 1 |
| `src/ui/building_upgrade_menu.rs` | gold | 2 | 1 | 1 |
| `src/ui/hud.rs` | gold | 1 | 1 | 0 |
| `src/ui/shop_screen.rs` | gold | 6 | 2 | 4 |
| `src/economy/achievements.rs` | gold | 3 | 0 | 3 |
| `src/economy/tool_upgrades.rs` | gold | 4 | 0 | 4 |
| `src/economy/gold.rs` | gold | 8 | 3 | 5 |
| `src/economy/shop.rs` | gold | 9 | 1 | 8 |
| **TOTAL** | | **50** | **8 (16%)** | **42 (84%)** |

---

## 6. Notes for Implementers

1. **`new_unchecked` is the right tool for internal mutation** — game code modifying stamina/health/gold in response to events should prefer `new_unchecked` since the values have already been clamped by game logic (e.g. `.max(0.0)`). Use `new()` only at user-facing input boundaries.

2. **`max_stamina` and `max_health` stay `f32`** — these are not in `bounded_types.rs` and are not part of this wiring. Arithmetic like `stamina.get().min(max_stamina)` is idiomatic.

3. **Gold max = 999_999 conflict** — the achievement system tests `player.gold = 1_000_000` but `Gold::new(1_000_000)` returns `Err`. Raise the cap in `bounded_types.rs` to `1_000_000` or higher, or use `new_unchecked` in tests only. This is a spec decision, not a mechanical change.

4. **Serde is transparent** — `Gold`, `Stamina`, `Health` all serialize as their raw primitive. Save file format is unchanged.

5. **`fishing/treasure.rs` uses `contents.gold`** — this is a `TreasureContents.gold: u32` field, not `PlayerState.gold`. No change needed there.

6. **Suggested wiring command to verify no regressions after each swap:**
   ```
   cargo check && cargo test --test headless && cargo clippy -- -D warnings
   ```
