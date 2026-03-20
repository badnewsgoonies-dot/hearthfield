#!/usr/bin/env python3
"""
Relay Amplification Experiment

Tests whether relaying a claim through multiple independent LLM sessions
inflates confidence monotonically, and whether different relay orderings
produce convergent or divergent structural content.

Design:
  - Seed: a modest claim (the original Part Nine table observation)
  - Each relay step: fresh Gemini call with no shared state
  - Prompt: "Engage with this argument. What's the strongest version?"
  - 3 orderings × 5 steps = 15 total calls
  - Measurement: JSON-schema-enforced scoring of each output

Models (all working from this container):
  - gemini-2.5-flash (us-central1)
  - gemini-2.5-pro (us-central1) 
  - gemini-3-flash-preview (global)
  - gemini-3.1-pro-preview (global)
  - gemini-3.1-flash-lite-preview (global)
"""

import sys, os, json, time

sys.path.insert(0, "/home/claude/hearthfield/tools/vision")
from gemini_vertex import gemini_generate, gemini_generate_json

# ── Seed claim (modest, from the original table) ──────────────────────

SEED = """
We observed that enforcement mechanisms succeed when they operate at the
interface level (tool permissions, type signatures, API parameters) and
fail when they operate at the content level (prompt instructions, naming
conventions). Here are 7 examples from independent systems:

1. CLAUDE.md "don't touch shared/" (prompt) failed 0/20; disallowedTools config succeeded 20/20
2. --deny-tool "write(*)" succeeds (typed tool); --deny-tool "shell(*)" fails (opaque tool)  
3. Evidence tags (YAML at interface) improved accuracy from 0% to 96%
4. "Respond in JSON" (prompt) is unreliable; responseMimeType (API param) is 100%
5. Ironclad bounded types catch violations at compile time vs runtime tracing for bare primitives
6. TOML manifests validated by build.rs vs hardcoded paths found only by play-testing
7. Tool-forced architecture with remote store = 100% defense vs context-window provenance = bypassable

The pattern appears consistent: interface-level enforcement works, content-level enforcement doesn't.
"""

# ── Relay prompt ──────────────────────────────────────────────────────

RELAY_PROMPT = """You are a researcher engaging with an argument. Read the following 
and produce the strongest, most rigorous version of the claim. Push the idea 
further — find the underlying mechanism, identify what principle explains the pattern, 
and state it as precisely as you can. Be substantive, not complimentary.

ARGUMENT:
{text}

Write 2-3 paragraphs. Focus on mechanism and precision. No preamble."""

# ── Measurement prompt (schema-enforced) ──────────────────────────────

MEASURE_SCHEMA = {
    "type": "OBJECT",
    "properties": {
        "scope_word": {
            "type": "STRING",
            "description": "The strongest scope claim made: 'pattern', 'principle', 'invariant', 'law', 'theorem', or 'other'"
        },
        "confidence_1_to_10": {
            "type": "INTEGER",
            "description": "How confident is the text in its claim? 1=tentative observation, 10=proven theorem"
        },
        "has_hedge": {
            "type": "BOOLEAN",
            "description": "Does the text include scope limitations or caveats?"
        },
        "has_impossibility_claim": {
            "type": "BOOLEAN",
            "description": "Does the text claim no counterexample can exist?"
        },
        "has_proof_structure": {
            "type": "BOOLEAN",
            "description": "Does the text present a logical proof or deductive argument?"
        },
        "core_claim_one_sentence": {
            "type": "STRING",
            "description": "The single most important claim, in one sentence"
        },
        "structural_content_survives_deflation": {
            "type": "BOOLEAN",
            "description": "If you strip all confidence language and emotional register, does a novel structural insight remain?"
        },
        "emotional_register_1_to_10": {
            "type": "INTEGER",
            "description": "1=dry/clinical, 10=excited/revelatory"
        }
    },
    "required": [
        "scope_word", "confidence_1_to_10", "has_hedge", 
        "has_impossibility_claim", "has_proof_structure",
        "core_claim_one_sentence", "structural_content_survives_deflation",
        "emotional_register_1_to_10"
    ]
}

MEASURE_PROMPT = """Analyze this text as a dispassionate reviewer. Score it on the requested dimensions.
Be precise and honest — do not inflate scores.

TEXT:
{text}"""

# ── Model orderings ───────────────────────────────────────────────────

ORDERINGS = {
    "A": [
        "gemini-2.5-flash",
        "gemini-3-flash-preview", 
        "gemini-2.5-pro",
        "gemini-3.1-pro-preview",
        "gemini-3.1-flash-lite-preview",
    ],
    "B": [
        "gemini-3.1-pro-preview",
        "gemini-2.5-flash",
        "gemini-3.1-flash-lite-preview",
        "gemini-2.5-pro",
        "gemini-3-flash-preview",
    ],
    "C": [
        "gemini-3.1-flash-lite-preview",
        "gemini-3.1-pro-preview",
        "gemini-3-flash-preview",
        "gemini-2.5-flash",
        "gemini-2.5-pro",
    ],
}

# ── Run ───────────────────────────────────────────────────────────────

def measure(text, model="gemini-3-flash-preview"):
    """Score a text using schema-enforced JSON."""
    return gemini_generate_json(
        MEASURE_PROMPT.format(text=text),
        schema=MEASURE_SCHEMA,
        model=model,
    )

def relay_step(text, model):
    """One relay step: fresh session, no history."""
    return gemini_generate(
        RELAY_PROMPT.format(text=text),
        model=model,
    )

def run_ordering(name, models):
    """Run a full 5-step relay and measure each step."""
    print(f"\n{'='*60}")
    print(f"ORDERING {name}: {' → '.join(m.split('-')[-1] if 'preview' not in m else m.split('-')[1]+'-'+m.split('-')[2] for m in models)}")
    print(f"{'='*60}")
    
    results = []
    
    # Measure seed
    print(f"\n  Step 0 (seed): measuring...")
    seed_score = measure(SEED)
    results.append({
        "step": 0,
        "model": "seed",
        "text": SEED.strip(),
        "score": seed_score,
    })
    print(f"    confidence={seed_score.get('confidence_1_to_10')}, "
          f"scope={seed_score.get('scope_word')}, "
          f"emotional={seed_score.get('emotional_register_1_to_10')}, "
          f"hedge={seed_score.get('has_hedge')}")
    
    current_text = SEED
    
    for i, model in enumerate(models):
        step = i + 1
        short_name = model.replace("gemini-", "")
        print(f"\n  Step {step} ({short_name}): generating...")
        
        try:
            output = relay_step(current_text, model)
            time.sleep(1)  # Rate limit courtesy
            
            print(f"    [{len(output)} chars] measuring...")
            score = measure(output)
            
            results.append({
                "step": step,
                "model": model,
                "text": output[:2000],  # Truncate for storage
                "score": score,
            })
            
            print(f"    confidence={score.get('confidence_1_to_10')}, "
                  f"scope={score.get('scope_word')}, "
                  f"emotional={score.get('emotional_register_1_to_10')}, "
                  f"hedge={score.get('has_hedge')}, "
                  f"proof={score.get('has_proof_structure')}")
            
            current_text = output
            time.sleep(1)
            
        except Exception as e:
            print(f"    ERROR: {e}")
            results.append({
                "step": step,
                "model": model,
                "text": f"ERROR: {e}",
                "score": None,
            })
    
    return results

def main():
    all_results = {}
    
    for name, models in ORDERINGS.items():
        all_results[name] = run_ordering(name, models)
    
    # Save full results
    output_path = "/home/claude/hearthfield/status/relay-experiment-results.json"
    with open(output_path, "w") as f:
        json.dump(all_results, f, indent=2)
    print(f"\nFull results saved to {output_path}")
    
    # Print summary table
    print(f"\n{'='*80}")
    print("SUMMARY: Confidence × Scope × Emotional Register across relay steps")
    print(f"{'='*80}")
    print(f"{'Ordering':<10} {'Step':<6} {'Model':<25} {'Conf':<6} {'Scope':<12} {'Emot':<6} {'Hedge':<7} {'Proof':<7} {'Survives':<9}")
    print("-" * 80)
    
    for name, results in all_results.items():
        for r in results:
            if r["score"]:
                s = r["score"]
                print(f"{name:<10} {r['step']:<6} {r['model']:<25} "
                      f"{s.get('confidence_1_to_10','?'):<6} "
                      f"{s.get('scope_word','?'):<12} "
                      f"{s.get('emotional_register_1_to_10','?'):<6} "
                      f"{str(s.get('has_hedge','?')):<7} "
                      f"{str(s.get('has_proof_structure','?')):<7} "
                      f"{str(s.get('structural_content_survives_deflation','?')):<9}")
        print()
    
    # Check convergence: do the core claims at step 5 match across orderings?
    print(f"\n{'='*80}")
    print("CONVERGENCE CHECK: Core claims at step 5 across orderings")
    print(f"{'='*80}")
    for name, results in all_results.items():
        final = results[-1]
        if final["score"]:
            print(f"\n  Ordering {name}: {final['score'].get('core_claim_one_sentence', '?')}")

if __name__ == "__main__":
    main()
