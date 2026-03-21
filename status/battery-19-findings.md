# Battery 19: Causal Premise vs Mechanical Enforcement

**Design:** 2×2 factorial. Mechanical enforcement (present/absent) × Causal premise (present/absent).
**Models:** gemini-2.5-flash, gemini-2.5-pro, gpt-5.4. **N=5** per cell per model. **Scorer:** gemini-3.1-pro-preview (independent model family, schema-enforced JSON).

**Task:** Implement three Rust functions for a game gold system — `add_reward`, `purchase_item`, `serialize_save` — with a 9,999,999 cap.

---

## The 2×2

|                    | No mechanical enforcement | With mechanical enforcement |
|--------------------|---------------------------|------------------------------|
| **Bare rule**      | A: "Gold must not exceed 9,999,999. Use u32." | C: "Use the Gold bounded type." (Type provided in prompt.) |
| **Causal premise** | B: "Gold caps at 9,999,999 because save files use u32 serialization and overflow corrupts the save..." | D: Same causal explanation + Gold type provided. |

---

## Results: Bounded Type Adoption

|                    | No enforcement | With enforcement |
|--------------------|----------------|-------------------|
| **Bare rule**      | **0%** (0/13)  | **54%** (7/13)    |
| **Causal premise** | **0%** (0/13)  | **25%** (3/12)    |

- **A vs B:** Zero difference. Causal premises do not cause bounded type adoption when no type is available. (Expected — there's nothing to adopt.)
- **A vs C:** Mechanical enforcement alone produces 54% adoption. The type does the work.
- **C vs D:** Adding causal explanation **reduces** type adoption from 54% to 25%. Models that understand WHY solve the problem through manual clamping instead of using the provided tool.

## Results: Mentions Save Corruption (commentary metric)

|                    | No enforcement | With enforcement |
|--------------------|----------------|-------------------|
| **Bare rule**      | **0%** (0/13)  | **0%** (0/13)     |
| **Causal premise** | **77%** (10/13)| **67%** (8/12)    |

Causal premises improve comments and documentation. They do not improve code compliance.

## Results: Actual Overflow Protection (the metric that matters)

|                    | No enforcement | With enforcement |
|--------------------|----------------|-------------------|
| **Bare rule**      | **92%** (12/13)| **69%** (9/13)    |
| **Causal premise** | **100%** (13/13)| **67%** (8/12)   |

**The bounded type reduced overflow protection.** Conditions without the type (A, B) had 92-100% protection via manual `saturating_add` and `.min()` clamping. Conditions with the type (C, D) had 67-69% protection because models used `new_unchecked()` — the bypass constructor — and skipped manual checks.

---

## Root Cause: `new_unchecked` Is a Bypass Bug

The Gold bounded type had two constructors:

| Constructor | Returns | Validates? | Lines of code |
|-------------|---------|------------|---------------|
| `Gold::new(val)` | `Result<Gold, String>` | Yes | 3+ (must handle error) |
| `Gold::new_unchecked(val)` | `Gold` | No | 1 |

Workers optimize for compilation speed, not correctness. `new_unchecked` compiles with less code. They used it for **100% of constructions** across the entire codebase (372/372 callsites). The bounded type's presence suppressed the manual overflow checks that workers would otherwise write, while `new_unchecked` provided an escape hatch through the type system's protection.

This is INV-025 (envelope enforcement) eating its own tail. The bounded type is an interface-level constraint. `new_unchecked` is a payload hole in that interface. Models found it and walked through it because it was the path of least resistance to a green build.

---

## The Fix (Implemented)

Ironclad `game_value` macro v2:

| Old API | New API |
|---------|---------|
| `Gold::new(val) → Result<Gold, String>` | `Gold::new(val) → Gold` (clamps at bounds, always succeeds) |
| `Gold::new_unchecked(val) → Gold` | **Removed.** |
| — | `Gold::validate(val) → Result<Gold, String>` (for trust boundaries) |
| — | `Gold::MIN`, `Gold::MAX` (constants) |

The safe path is now the easy path. No escape hatch. No bypass. Every construction validates.

**Migration:** 372 callsites across 26 files. Net change: +1 line. Every `new_unchecked(val)` became `new(val)`. Every `new(val).unwrap_or_else(...)` collapsed to `new(val)`. The code got simpler.

**Compilation:** 0 errors, lib + tests. Commit `2d5718f`.

---

## Findings (5 total, ordered by importance)

### Finding 1: Escape hatches in typed interfaces are exploited 100% of the time

AI workers find the path of least resistance to a green build. If a typed constraint has a bypass constructor, they will use it for every callsite, because it requires less code than the validating constructor. This is not malice — it is optimization. The bypass compiles with fewer lines.

**API design rule for AI-consumed code:** the constrained path must be the path of least resistance. Not just the correct path. The EASIEST path. `Gold::new(val) → Gold` (one line, always works) beats `Gold::new(val) → Result<Gold, String>` (requires error handling) which loses to `Gold::new_unchecked(val) → Gold` (one line, no validation).

**Evidence level:** Observed. 372/372 callsites used `new_unchecked` before the fix. 0/372 used `new`.

### Finding 2: Causal premises change comments, not compliance

Without mechanical enforcement, causal premises produce zero compliance improvement over bare rules. 0% vs 0% bounded type adoption. The only measurable difference: models given causal explanations mention save corruption in comments (77% vs 0%). The code is identical.

**Evidence level:** Observed. A vs B, 13 samples each, 3 models.

### Finding 3: Mechanical enforcement is the compliance mechanism

Providing the bounded type in the prompt produces 54% adoption with a bare instruction. No causal reasoning needed. Hand the model a tool and it uses it (when the tool is easier than the alternative).

**Evidence level:** Observed. A vs C, 13 samples each, 3 models.

### Finding 4: Bounded types with bypass APIs reduce safety below baseline

Overflow protection was 92% without the bounded type (manual checks) and 69% with it (bypass constructor). The type suppressed vigilance. Models that received the type assumed it handled overflow and stopped writing manual protection — but routed through the unchecked constructor, which provides no protection.

This is Opus Chat B's counterargument from the relay experiment made concrete: "The greatest danger of a system boundary arises from partially legible structures that falsely suppress vigilance."

**Evidence level:** Observed. A vs C on the overflow_protection metric, 13 samples each, 3 models.

### Finding 5: Causal premises redirect rather than amplify mechanical enforcement

When models have both the type AND the causal explanation (condition D), bounded type adoption drops from 54% to 25% — but bare u32 usage also drops from 100% to 58%. Models that understood WHY the cap exists solved the problem through manual protection instead of adopting the provided type. The premise didn't interfere with safety — it provided an alternative compliance path.

**Evidence level:** Observed but weaker (D condition has 12 samples, 2 GPT-5.4 timeouts). The direction is clear; the magnitude needs replication.

---

## Implications

### For the paper (INV-025 extension)

The original: "Put constraints at the interface." Necessary but not sufficient. Battery 19 adds: **the constrained path must be the path of least resistance.** A constraint at the interface with an easier bypass is worse than no constraint at all, because the constraint suppresses the manual checks that would otherwise exist.

The revised principle: *Verification cost is bounded if and only if constraints are declared at the interface AND the constrained path is easier than any alternative.* The second clause is new.

### For the Ironclad macros (INV-027 extension)

The original: "Bounded types surface pre-existing bugs." Battery 19 adds: **bounded types with bypass constructors create new bugs by suppressing manual protection.** The type without the bypass is strictly better. The type with the bypass is worse than no type at all on the metric that matters (actual overflow protection).

### For practitioner guidance

1. **Remove all `unsafe`/`unchecked`/`bypass` constructors from types consumed by AI workers.** If a constructor exists that skips validation and is easier to call, it will be used exclusively.
2. **Make validating constructors return the value directly, not `Result`.** `Gold::new(val) → Gold` (clamps) is used correctly. `Gold::new(val) → Result<Gold>` loses to any alternative that doesn't require error handling.
3. **Causal explanations in prompts improve code documentation but not code correctness.** Invest in type signatures, not explanations. The type is the enforcement. The explanation is commentary.
4. **Test enforcement mechanisms with the escape hatches, not without them.** Battery 19 would have shown 100% compliance if tested with only `Gold::new()` (Result version) — because the scorer measures "uses bounded type," not "uses it correctly." The `new_unchecked` bypass was the actual failure mode, invisible to the original metric.

---

## Per-Model Behavior

| Model | Condition A (bare, no type) | Condition C (bare, with type) | Notes |
|-------|---------------------------|-------------------------------|-------|
| gemini-2.5-flash | 0% bounded, 80% protected | 20% bounded, 60% protected | Weakest type adoption |
| gemini-2.5-pro | 0% bounded, 100% protected | 60% bounded, 60% protected | Best type adoption but protection still dropped |
| gpt-5.4 | 0% bounded, 100% protected | 100% bounded, 100% protected | Only model that used type correctly — but N=3 (2 timeouts) |

GPT-5.4 is the outlier: 100% type adoption AND 100% overflow protection in condition C. But N=3 due to timeouts — insufficient for a model-specific claim. Worth replicating.

---

## Experimental Limitations

1. **N is small.** 5 per cell, 3 models, but GPT-5.4 had timeouts reducing effective N to 3 in most cells.
2. **Condition D is partial.** 12/15 samples collected before process crash. Direction is clear but magnitudes are approximate.
3. **Scoring is by a different model** (Gemini 3.1-pro), not by compilation. The `code_compiles` field is a model judgment, not a `rustc` invocation. Some scores may be inaccurate.
4. **No filesystem access.** Workers generated code in a pure-prompt context. Workers with filesystem access (reading the actual `bounded_types.rs`) might behave differently.
5. **Single task.** The Gold system is one task. Generalization to other bounded types, other languages, and other constraint patterns needs replication.

---

## Relation to Other Batteries

- **Battery 19 is the foundational experiment.** It answers: "Do causal premises add enforcement value beyond mechanical constraints?" Answer: No — for compliance. Yes — for comments.
- **Batteries 15-18** (causal premise depth, pressure, delegation, forking) should be re-evaluated in light of this finding. If the enforcement comes from the type and the premise only changes commentary, the delegation-survival question (Battery 17) becomes: "Do types survive delegation?" rather than "Do explanations survive delegation?"
- **The relay experiments** (v1-v4) tested a different phenomenon (claim escalation through sessions) but Opus Chat B's counterargument about partial enforcement directly predicted this finding.

---

*Battery 19 conducted March 21, 2026. 60 dispatches planned, ~50 completed (GPT-5.4 timeouts). Scored by gemini-3.1-pro-preview with schema-enforced JSON. Ironclad macro fix implemented and compiled in commit 2d5718f. Total cost: ~15 Gemini calls (free) + ~10 Codex calls (~10 premium) + scoring (~15 Gemini calls, free).*
