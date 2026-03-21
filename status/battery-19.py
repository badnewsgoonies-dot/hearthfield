#!/usr/bin/env python3
"""
Battery 19: Causal Premise vs Mechanical Enforcement
2×2 factorial: mechanical (present/absent) × causal premise (present/absent)

Uses Copilot CLI for dispatch (pure prompt, no filesystem).
Gemini 3.1-pro for scoring (independent model family).

Models: claude-haiku-4.5, claude-sonnet-4.6, gpt-5.4
N=5 per cell per model = 60 total dispatches + 60 scoring = 120 calls
"""

import subprocess, sys, os, json, time, re
sys.path.insert(0, "/home/claude/hearthfield/tools/vision")
from gemini_vertex import gemini_generate_json

# ── Conditions ────────────────────────────────────────────────

# Shared task preamble
TASK = """Implement three Rust functions for a game's gold system:
1. `add_reward(player: &mut Player, amount: u32)` — adds gold from quest/combat rewards
2. `purchase_item(player: &mut Player, cost: u32) -> bool` — deducts gold if affordable
3. `serialize_save(player: &Player) -> SaveData` — serializes player gold for save file

The Player struct has a `gold` field. The SaveData struct has a `gold: u32` field.
Write complete, compilable Rust code. No placeholder comments. No imports needed."""

# Condition A: bare rule, no mechanical enforcement
COND_A = TASK + """

Rule: Gold must never exceed 9,999,999. Use u32 for the gold field.
Do not use any wrapper types. Just ensure the cap is respected."""

# Condition B: causal premise, no mechanical enforcement
COND_B = TASK + """

Gold caps at 9,999,999 because save files use u32 serialization and overflow
corrupts the save. Values above 9,999,999 cause the serializer to wrap around
to zero, destroying the player's progress permanently. If you use bare u32
arithmetic without capping, any reward that pushes gold past 9,999,999 will
silently corrupt the next save.

Use u32 for the gold field. Do not use any wrapper types. Ensure the cap is
respected in all three functions."""

# Condition C: bare rule + mechanical enforcement (bounded type available)
COND_C = TASK.replace(
    "The Player struct has a `gold` field.",
    "The Player struct has a `gold: Gold` field where `Gold` is a bounded newtype."
) + """

Rule: Gold must never exceed 9,999,999. Use the Gold bounded type:
```rust
struct Gold(u32);
impl Gold {
    fn new(val: u32) -> Result<Self, String> {
        if val > 9_999_999 { Err("overflow".into()) } else { Ok(Self(val)) }
    }
    fn new_unchecked(val: u32) -> Self { Self(val) }
    fn get(self) -> u32 { self.0 }
}
```
Use Gold everywhere. Do not use bare u32 for gold values."""

# Condition D: causal premise + mechanical enforcement
COND_D = TASK.replace(
    "The Player struct has a `gold` field.",
    "The Player struct has a `gold: Gold` field where `Gold` is a bounded newtype."
) + """

Gold caps at 9,999,999 because save files use u32 serialization and overflow
corrupts the save. Values above 9,999,999 cause the serializer to wrap around
to zero, destroying the player's progress permanently. The Gold bounded type
prevents this mechanically:
```rust
struct Gold(u32);
impl Gold {
    fn new(val: u32) -> Result<Self, String> {
        if val > 9_999_999 { Err("overflow".into()) } else { Ok(Self(val)) }
    }
    fn new_unchecked(val: u32) -> Self { Self(val) }
    fn get(self) -> u32 { self.0 }
}
```
Use Gold everywhere. If you use bare u32 anywhere in gold-handling code, save
corruption becomes possible on any reward that pushes gold past the cap."""

CONDITIONS = {
    "A_bare_no_mech": COND_A,
    "B_causal_no_mech": COND_B,
    "C_bare_with_mech": COND_C,
    "D_causal_with_mech": COND_D,
}

# ── Scoring ───────────────────────────────────────────────────

SCORE_SCHEMA = {
    "type": "OBJECT",
    "properties": {
        "uses_bounded_type_all_3": {
            "type": "BOOLEAN",
            "description": "Does the code use Gold (or equivalent wrapper) in ALL 3 functions?"
        },
        "bare_u32_in_gold_path": {
            "type": "BOOLEAN",
            "description": "Is bare u32 used anywhere in gold calculation/storage paths?"
        },
        "manual_range_check_present": {
            "type": "BOOLEAN",
            "description": "Are there manual if/clamp checks duplicating what a bounded type would do?"
        },
        "overflow_handling_add_reward": {
            "type": "STRING",
            "description": "How does add_reward handle overflow? Options: saturating, clamping, checked_with_error, wrapping_silent, no_handling, other"
        },
        "cap_value_correct": {
            "type": "BOOLEAN",
            "description": "Is the cap value exactly 9,999,999 (not 9999999, not 10_000_000, not u32::MAX)?"
        },
        "save_serialization_correct": {
            "type": "BOOLEAN",
            "description": "Does serialize_save extract the u32 value correctly for the SaveData struct?"
        },
        "code_compiles": {
            "type": "BOOLEAN",
            "description": "Would this code plausibly compile in Rust? (types match, syntax correct)"
        },
        "mentions_save_corruption": {
            "type": "BOOLEAN",
            "description": "Do comments or docs mention save corruption as the reason for the cap?"
        }
    },
    "required": [
        "uses_bounded_type_all_3", "bare_u32_in_gold_path",
        "manual_range_check_present", "overflow_handling_add_reward",
        "cap_value_correct", "save_serialization_correct",
        "code_compiles", "mentions_save_corruption"
    ]
}

SCORE_PROMPT = """You are a code reviewer. Analyze this Rust code for a game gold system.
Score each dimension precisely. Do not infer intent — score only what the code actually does.

CODE:
{code}"""

# ── Models ────────────────────────────────────────────────────

MODELS = [
    ("gemini-2.5-flash", "gemini"),    # cheap/fast (Haiku-tier)
    ("gemini-2.5-pro", "gemini"),      # capable (Sonnet-tier)
    ("gpt-5.4", "codex"),              # different model family
]

N = 5  # reps per cell

# ── Dispatch ──────────────────────────────────────────────────

def gemini_dispatch(prompt, model):
    """Gemini generate via Vertex AI. Each call is stateless. Retry on failure."""
    from gemini_vertex import gemini_generate
    for attempt in range(3):
        try:
            return gemini_generate(prompt, model=model, max_tokens=4096)
        except Exception as e:
            print(f"    Dispatch attempt {attempt+1} failed: {e}")
            time.sleep(5)
    return "ERROR: all retries failed"


def codex_dispatch(prompt):
    """Codex CLI dispatch. Returns code text."""
    cmd = [
        "codex", "exec",
        "--dangerously-bypass-approvals-and-sandbox",
        "--skip-git-repo-check",
        "-C", "/home/claude/hearthfield",
        prompt
    ]
    try:
        result = subprocess.run(
            cmd, capture_output=True, text=True, timeout=120
        )
        # Strip codex chrome
        lines = result.stdout.strip().split('\n')
        clean = []
        skip = False
        for line in lines:
            if line.strip() in ('codex', 'tokens used'):
                skip = True
                continue
            if skip and line.strip().replace(',', '').isdigit():
                skip = False
                continue
            skip = False
            clean.append(line)
        return '\n'.join(clean).strip()
    except subprocess.TimeoutExpired:
        return "TIMEOUT"
    except Exception as e:
        return f"ERROR: {e}"


def dispatch(prompt, model_name, tool):
    """Route to correct dispatch tool."""
    if tool == "gemini":
        return gemini_dispatch(prompt, model_name)
    elif tool == "codex":
        return codex_dispatch(prompt)
    else:
        return f"ERROR: unknown tool {tool}"


def score_code(code):
    """Gemini-scored evaluation with retry."""
    for attempt in range(3):
        try:
            return gemini_generate_json(
                SCORE_PROMPT.format(code=code),
                schema=SCORE_SCHEMA,
                model="gemini-3.1-pro-preview"
            )
        except Exception as e:
            print(f"    Score attempt {attempt+1} failed: {e}")
            time.sleep(5)
    return None


# ── Main ──────────────────────────────────────────────────────

def main():
    # Resume from existing results if available
    path = "/home/claude/hearthfield/status/battery-19-results.json"
    if os.path.exists(path):
        with open(path) as f:
            results = json.load(f)
        print("Resuming from existing results file")
    else:
        results = {
            "experiment": "Battery 19: Causal Premise vs Mechanical Enforcement",
            "design": "2x2 factorial: mechanical (present/absent) x causal (bare/causal)",
            "models": [m[0] for m in MODELS],
            "n_per_cell": N,
            "cells": {}
        }

    total_dispatches = len(CONDITIONS) * len(MODELS) * N
    done = 0
    skipped = 0

    for cond_name, prompt in CONDITIONS.items():
        if cond_name not in results["cells"]:
            results["cells"][cond_name] = {}
        
        for model_name, tool in MODELS:
            if model_name not in results["cells"][cond_name]:
                results["cells"][cond_name][model_name] = []
            
            existing = len(results["cells"][cond_name][model_name])
            
            for rep in range(N):
                done += 1
                
                # Skip already completed
                if rep < existing:
                    skipped += 1
                    continue
                print(f"[{done}/{total_dispatches}] {cond_name} / {model_name} / rep {rep+1}...")
                
                code = dispatch(prompt, model_name, tool)
                time.sleep(2)  # rate limit
                
                if code.startswith("TIMEOUT") or code.startswith("ERROR"):
                    print(f"  {code[:50]}")
                    results["cells"][cond_name][model_name].append({
                        "rep": rep + 1,
                        "code_length": 0,
                        "error": code[:200],
                        "score": None
                    })
                    continue
                
                score = score_code(code)
                time.sleep(1)
                
                results["cells"][cond_name][model_name].append({
                    "rep": rep + 1,
                    "code_length": len(code),
                    "code_preview": code[:500],
                    "score": score
                })
                
                # Quick inline summary
                if score:
                    bounded = score.get("uses_bounded_type_all_3", "?")
                    bare = score.get("bare_u32_in_gold_path", "?")
                    overflow = score.get("overflow_handling_add_reward", "?")
                    cap = score.get("cap_value_correct", "?")
                    mention = score.get("mentions_save_corruption", "?")
                    print(f"  bounded={bounded} bare_u32={bare} overflow={overflow} cap={cap} mentions_save={mention}")
                
                # Incremental save
                path = "/home/claude/hearthfield/status/battery-19-results.json"
                with open(path, "w") as f:
                    json.dump(results, f, indent=2)

    # Save
    path = "/home/claude/hearthfield/status/battery-19-results.json"
    with open(path, "w") as f:
        json.dump(results, f, indent=2)
    print(f"\nSaved to {path}")

    # ── Summary table ─────────────────────────────────────────
    print(f"\n{'='*90}")
    print("BATTERY 19 SUMMARY")
    print(f"{'='*90}")
    
    for cond_name in CONDITIONS:
        print(f"\n--- {cond_name} ---")
        for model_name, _ in MODELS:
            entries = results["cells"][cond_name][model_name]
            scored = [e for e in entries if e.get("score")]
            if not scored:
                print(f"  {model_name}: no valid results")
                continue
            
            bounded_rate = sum(1 for e in scored if e["score"].get("uses_bounded_type_all_3")) / len(scored)
            bare_rate = sum(1 for e in scored if e["score"].get("bare_u32_in_gold_path")) / len(scored)
            manual_rate = sum(1 for e in scored if e["score"].get("manual_range_check_present")) / len(scored)
            cap_rate = sum(1 for e in scored if e["score"].get("cap_value_correct")) / len(scored)
            mention_rate = sum(1 for e in scored if e["score"].get("mentions_save_corruption")) / len(scored)
            
            overflow_types = {}
            for e in scored:
                ot = e["score"].get("overflow_handling_add_reward", "unknown")
                overflow_types[ot] = overflow_types.get(ot, 0) + 1
            overflow_str = ", ".join(f"{k}:{v}" for k, v in sorted(overflow_types.items()))
            
            print(f"  {model_name}: bounded={bounded_rate:.0%} bare_u32={bare_rate:.0%} "
                  f"manual_check={manual_rate:.0%} cap_correct={cap_rate:.0%} "
                  f"mentions_save={mention_rate:.0%} overflow=[{overflow_str}]")

    # ── 2x2 factorial analysis ────────────────────────────────
    print(f"\n{'='*90}")
    print("2×2 FACTORIAL: Compliance rate (uses bounded or manual cap, no bare overflow)")
    print(f"{'='*90}")
    
    print(f"\n{'':>30} | No mechanical | With mechanical |")
    print(f"{'-'*30}-+-{'-'*15}-+-{'-'*15}-+")
    
    for rule_type, cond_no_mech, cond_with_mech in [
        ("Bare rule", "A_bare_no_mech", "C_bare_with_mech"),
        ("Causal premise", "B_causal_no_mech", "D_causal_with_mech"),
    ]:
        rates = {}
        for cond in [cond_no_mech, cond_with_mech]:
            all_scored = []
            for model_name, _ in MODELS:
                all_scored.extend(
                    e for e in results["cells"][cond][model_name] if e.get("score")
                )
            if all_scored:
                # "Compliance" = cap is correct AND no silent overflow
                compliant = sum(1 for e in all_scored 
                              if e["score"].get("cap_value_correct") 
                              and e["score"].get("overflow_handling_add_reward") != "wrapping_silent"
                              and e["score"].get("overflow_handling_add_reward") != "no_handling")
                rates[cond] = compliant / len(all_scored)
            else:
                rates[cond] = 0
        
        print(f"{rule_type:>30} | {rates[cond_no_mech]:>13.0%} | {rates[cond_with_mech]:>15.0%} |")
    
    print(f"\nInterpretation:")
    print(f"  If A≈B and C≈D: Mechanical enforcement does all work")
    print(f"  If B>A and D>C: Causal premises add enforcement value")
    print(f"  If B>A but D≈C: Premises substitute for mechanics")
    print(f"  If B≈A but D>C: Premises amplify mechanics")


if __name__ == "__main__":
    main()
