#!/usr/bin/env python3
"""
Relay Amplification v2 — 3 chats, 1 model, cross-pollination

Simulates the actual Part Nine process: same model (gemini-3.1-pro-preview),
3 independent "chats" (no shared state), orchestrator bounces outputs between them.

Protocol:
  Round 1: All 3 chats get the seed independently
  Round 2: Chat A gets Chat B's output, Chat B gets Chat C's, Chat C gets Chat A's
  Round 3: Same rotation
  Round 4: Same rotation  
  Round 5: Same rotation

This is circular relay — each chat always receives output from a different chat,
never its own. Matches the copy-paste-between-sessions pattern.

Measurement: schema-enforced JSON scoring at every step.
"""

import sys, os, json, time
sys.path.insert(0, "/home/claude/hearthfield/tools/vision")
from gemini_vertex import gemini_generate, gemini_generate_json

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

RELAY_PROMPT = """You are a researcher engaging with an argument from a colleague.
Read the following and produce the strongest, most rigorous version of the claim.
Push the idea further — find the underlying mechanism, identify what principle
explains the pattern, and state it as precisely as you can.

Be critical. If the argument overreaches, say so. If it conflates things, separate them.
If it's missing a step, add it. Your job is to make this sharper, not to agree with it.

ARGUMENT FROM COLLEAGUE:
{text}

Write 2-3 paragraphs. Focus on mechanism and precision. No preamble."""

MEASURE_SCHEMA = {
    "type": "OBJECT",
    "properties": {
        "scope_word": {
            "type": "STRING",
            "description": "The strongest scope claim: 'observation', 'pattern', 'principle', 'invariant', 'law', 'theorem'"
        },
        "confidence_1_to_10": {
            "type": "INTEGER",
            "description": "How confident is the text? 1=tentative, 10=proven theorem"
        },
        "has_hedge": {
            "type": "BOOLEAN",
            "description": "Includes scope limitations or caveats?"
        },
        "has_impossibility_claim": {
            "type": "BOOLEAN",
            "description": "Claims no counterexample can exist?"
        },
        "has_proof_structure": {
            "type": "BOOLEAN",
            "description": "Presents a logical proof or deductive argument?"
        },
        "has_pushback": {
            "type": "BOOLEAN",
            "description": "Does the text push back on, qualify, or critique the input argument?"
        },
        "core_claim_one_sentence": {
            "type": "STRING",
            "description": "The single most important claim, one sentence"
        },
        "structural_content_survives_deflation": {
            "type": "BOOLEAN",
            "description": "If you strip confidence language and emotional register, does a novel structural insight remain?"
        },
        "emotional_register_1_to_10": {
            "type": "INTEGER",
            "description": "1=dry/clinical, 10=excited/revelatory"
        },
        "novel_content_vs_input": {
            "type": "BOOLEAN",
            "description": "Does this text introduce ideas, mechanisms, or framings NOT present in the input?"
        }
    },
    "required": [
        "scope_word", "confidence_1_to_10", "has_hedge",
        "has_impossibility_claim", "has_proof_structure", "has_pushback",
        "core_claim_one_sentence", "structural_content_survives_deflation",
        "emotional_register_1_to_10", "novel_content_vs_input"
    ]
}

MEASURE_PROMPT = """Analyze this text as a dispassionate reviewer. Score precisely and honestly.
Do not inflate. If the text is modest, score it as modest.

TEXT:
{text}"""

MODEL = "gemini-3.1-pro-preview"

def generate(text):
    return gemini_generate(RELAY_PROMPT.format(text=text), model=MODEL, max_tokens=2048)

def measure(text):
    return gemini_generate_json(MEASURE_PROMPT.format(text=text), schema=MEASURE_SCHEMA, model=MODEL)

def fmt_score(s):
    if not s: return "ERROR"
    return (f"conf={s.get('confidence_1_to_10','?')} "
            f"scope={s.get('scope_word','?'):<12} "
            f"emot={s.get('emotional_register_1_to_10','?')} "
            f"hedge={str(s.get('has_hedge','?')):<5} "
            f"proof={str(s.get('has_proof_structure','?')):<5} "
            f"pushback={str(s.get('has_pushback','?')):<5} "
            f"novel={str(s.get('novel_content_vs_input','?')):<5} "
            f"survives={s.get('structural_content_survives_deflation','?')}")

def main():
    results = {"rounds": [], "chats": {"A": [], "B": [], "C": []}}

    # ── Round 0: Measure seed ─────────────────────────────────────
    print("=" * 70)
    print("ROUND 0: Seed measurement")
    print("=" * 70)
    seed_score = measure(SEED)
    print(f"  Seed: {fmt_score(seed_score)}")
    results["seed_score"] = seed_score

    # ── Round 1: All 3 chats get seed independently ───────────────
    print(f"\n{'=' * 70}")
    print("ROUND 1: All chats receive SEED independently")
    print("=" * 70)

    chat_outputs = {"A": None, "B": None, "C": None}

    for chat in ["A", "B", "C"]:
        print(f"\n  Chat {chat} generating...")
        output = generate(SEED)
        time.sleep(2)
        score = measure(output)
        time.sleep(1)
        chat_outputs[chat] = output
        results["chats"][chat].append({
            "round": 1, "input_from": "seed",
            "text": output[:3000], "score": score
        })
        print(f"  Chat {chat}: {fmt_score(score)}")
        print(f"           claim: {score.get('core_claim_one_sentence', '?')[:80]}")

    # ── Rounds 2-5: Circular relay (A←B, B←C, C←A) ──────────────
    relay_map = {"A": "B", "B": "C", "C": "A"}  # chat receives from...

    for rnd in range(2, 6):
        print(f"\n{'=' * 70}")
        print(f"ROUND {rnd}: Cross-pollination (A←B output, B←C output, C←A output)")
        print("=" * 70)

        new_outputs = {}
        for chat in ["A", "B", "C"]:
            source = relay_map[chat]
            input_text = chat_outputs[source]
            print(f"\n  Chat {chat} ← Chat {source}'s output, generating...")
            output = generate(input_text)
            time.sleep(2)
            score = measure(output)
            time.sleep(1)
            new_outputs[chat] = output
            results["chats"][chat].append({
                "round": rnd, "input_from": f"chat_{source}",
                "text": output[:3000], "score": score
            })
            print(f"  Chat {chat}: {fmt_score(score)}")
            print(f"           claim: {score.get('core_claim_one_sentence', '?')[:80]}")

        chat_outputs = new_outputs

    # ── Save ──────────────────────────────────────────────────────
    output_path = "/home/claude/hearthfield/status/relay-v2-results.json"
    with open(output_path, "w") as f:
        json.dump(results, f, indent=2)
    print(f"\nSaved to {output_path}")

    # ── Summary table ─────────────────────────────────────────────
    print(f"\n{'=' * 90}")
    print("SUMMARY TABLE")
    print(f"{'=' * 90}")
    print(f"{'Chat':<6} {'Rnd':<5} {'From':<8} {'Conf':<6} {'Scope':<12} {'Emot':<6} "
          f"{'Hedge':<7} {'Proof':<7} {'Push':<7} {'Novel':<7} {'Surv':<6}")
    print("-" * 90)

    # Seed row
    s = results["seed_score"]
    print(f"{'seed':<6} {'0':<5} {'-':<8} {s.get('confidence_1_to_10','?'):<6} "
          f"{s.get('scope_word','?'):<12} {s.get('emotional_register_1_to_10','?'):<6} "
          f"{str(s.get('has_hedge','?')):<7} {str(s.get('has_proof_structure','?')):<7} "
          f"{'-':<7} {'-':<7} {str(s.get('structural_content_survives_deflation','?')):<6}")

    for chat in ["A", "B", "C"]:
        for entry in results["chats"][chat]:
            s = entry["score"]
            if s:
                print(f"{chat:<6} {entry['round']:<5} {entry['input_from']:<8} "
                      f"{s.get('confidence_1_to_10','?'):<6} "
                      f"{s.get('scope_word','?'):<12} "
                      f"{s.get('emotional_register_1_to_10','?'):<6} "
                      f"{str(s.get('has_hedge','?')):<7} "
                      f"{str(s.get('has_proof_structure','?')):<7} "
                      f"{str(s.get('has_pushback','?')):<7} "
                      f"{str(s.get('novel_content_vs_input','?')):<7} "
                      f"{str(s.get('structural_content_survives_deflation','?')):<6}")

    # ── Convergence check ─────────────────────────────────────────
    print(f"\n{'=' * 90}")
    print("CONVERGENCE: Final claims (round 5)")
    print(f"{'=' * 90}")
    for chat in ["A", "B", "C"]:
        final = results["chats"][chat][-1]
        if final["score"]:
            print(f"\n  Chat {chat} (received from Chat {relay_map[chat]}):")
            print(f"    {final['score'].get('core_claim_one_sentence', '?')}")

if __name__ == "__main__":
    main()
