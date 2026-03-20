#!/usr/bin/env python3
"""
Relay Amplification v3 — Codex (gpt-5.4) × 3 chats, Gemini scorer

Each Codex call is a truly independent session — zero shared state,
fresh context window, no history. This matches the actual Part Nine
process more closely than Gemini (which may share some session state).

Protocol:
  Round 1: All 3 chats get seed independently  
  Rounds 2-5: Circular relay (A←B, B←C, C←A)
  Measurement: Gemini 3.1-pro-preview with schema enforcement (independent scorer)
"""

import subprocess, sys, os, json, time, tempfile

sys.path.insert(0, "/home/claude/hearthfield/tools/vision")
from gemini_vertex import gemini_generate_json

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

Write 2-3 paragraphs. Focus on mechanism and precision. No preamble. No markdown headers."""

MEASURE_SCHEMA = {
    "type": "OBJECT",
    "properties": {
        "scope_word": {
            "type": "STRING",
            "description": "Strongest scope claim: 'observation','pattern','principle','invariant','law','theorem'"
        },
        "confidence_1_to_10": {
            "type": "INTEGER",
            "description": "1=tentative, 10=proven theorem"
        },
        "has_hedge": {"type": "BOOLEAN", "description": "Includes caveats?"},
        "has_impossibility_claim": {"type": "BOOLEAN", "description": "Claims no counterexample?"},
        "has_proof_structure": {"type": "BOOLEAN", "description": "Logical proof?"},
        "has_pushback": {"type": "BOOLEAN", "description": "Critiques the input?"},
        "core_claim_one_sentence": {"type": "STRING", "description": "Most important claim"},
        "structural_content_survives_deflation": {"type": "BOOLEAN"},
        "emotional_register_1_to_10": {"type": "INTEGER", "description": "1=clinical, 10=revelatory"},
        "novel_content_vs_input": {"type": "BOOLEAN", "description": "New ideas not in input?"}
    },
    "required": [
        "scope_word","confidence_1_to_10","has_hedge","has_impossibility_claim",
        "has_proof_structure","has_pushback","core_claim_one_sentence",
        "structural_content_survives_deflation","emotional_register_1_to_10",
        "novel_content_vs_input"
    ]
}

MEASURE_PROMPT = """Analyze this text as a dispassionate reviewer. Score precisely. Do not inflate.

TEXT:
{text}"""


def codex_generate(text, chat_id="X"):
    """One Codex call = one independent session. Zero shared state."""
    prompt = RELAY_PROMPT.format(text=text)
    
    # Use -o to write output to file (cleaner than parsing stdout)
    outfile = f"/tmp/codex_relay_{chat_id}_{int(time.time())}.txt"
    
    cmd = [
        "codex", "exec",
        "--dangerously-bypass-approvals-and-sandbox",
        "--skip-git-repo-check",
        "-o", outfile,
        "-C", "/home/claude/hearthfield",
        prompt
    ]
    
    try:
        result = subprocess.run(
            cmd, capture_output=True, text=True, timeout=120
        )
        
        # Read from -o output file
        if os.path.exists(outfile):
            with open(outfile) as f:
                output = f.read().strip()
            os.unlink(outfile)
            if output:
                return output
        
        # Fallback: parse stdout (skip codex chrome)
        stdout = result.stdout
        lines = stdout.strip().split('\n')
        # Filter out codex metadata lines
        content = []
        skip_patterns = ['codex', 'tokens used', 'ERROR', 'Reconnecting', 'warning:']
        for line in lines:
            if not any(p in line for p in skip_patterns):
                content.append(line)
        return '\n'.join(content).strip()
        
    except subprocess.TimeoutExpired:
        return "ERROR: timeout"
    except Exception as e:
        return f"ERROR: {e}"


def measure(text):
    """Gemini 3.1-pro as independent scorer."""
    return gemini_generate_json(
        MEASURE_PROMPT.format(text=text),
        schema=MEASURE_SCHEMA,
        model="gemini-3.1-pro-preview"
    )


def fmt(s):
    if not s: return "ERROR"
    return (f"conf={s.get('confidence_1_to_10','?')} "
            f"scope={s.get('scope_word','?'):<12} "
            f"emot={s.get('emotional_register_1_to_10','?')} "
            f"hedge={str(s.get('has_hedge','?')):<5} "
            f"proof={str(s.get('has_proof_structure','?')):<5} "
            f"push={str(s.get('has_pushback','?')):<5} "
            f"novel={str(s.get('novel_content_vs_input','?')):<5}")


def main():
    results = {"model": "gpt-5.4 (codex)", "scorer": "gemini-3.1-pro-preview",
               "rounds": [], "chats": {"A": [], "B": [], "C": []}}

    # Seed measurement
    print("=" * 70)
    print("SEED MEASUREMENT (Gemini scorer)")
    print("=" * 70)
    seed_score = measure(SEED)
    results["seed_score"] = seed_score
    print(f"  Seed: {fmt(seed_score)}")

    # Round 1: independent
    print(f"\n{'='*70}\nROUND 1: All 3 Codex sessions get SEED independently\n{'='*70}")
    
    chat_outputs = {}
    for chat in ["A", "B", "C"]:
        print(f"\n  Chat {chat} (codex gpt-5.4)...")
        output = codex_generate(SEED, chat)
        print(f"    [{len(output)} chars]")
        if output.startswith("ERROR"):
            print(f"    {output}")
            chat_outputs[chat] = SEED  # fallback
            results["chats"][chat].append({"round": 1, "input_from": "seed", 
                                            "text": output, "score": None})
            continue
        
        score = measure(output)
        time.sleep(1)
        chat_outputs[chat] = output
        results["chats"][chat].append({"round": 1, "input_from": "seed",
                                        "text": output[:3000], "score": score})
        print(f"    {fmt(score)}")
        print(f"    claim: {score.get('core_claim_one_sentence','?')[:80]}")

    # Rounds 2-5: circular relay
    relay_map = {"A": "B", "B": "C", "C": "A"}
    
    for rnd in range(2, 6):
        print(f"\n{'='*70}\nROUND {rnd}: A←B, B←C, C←A\n{'='*70}")
        
        new_outputs = {}
        for chat in ["A", "B", "C"]:
            source = relay_map[chat]
            input_text = chat_outputs[source]
            print(f"\n  Chat {chat} ← Chat {source} (codex gpt-5.4)...")
            output = codex_generate(input_text, chat)
            print(f"    [{len(output)} chars]")
            
            if output.startswith("ERROR"):
                print(f"    {output}")
                new_outputs[chat] = input_text
                results["chats"][chat].append({"round": rnd, "input_from": f"chat_{source}",
                                                "text": output, "score": None})
                continue
            
            score = measure(output)
            time.sleep(1)
            new_outputs[chat] = output
            results["chats"][chat].append({"round": rnd, "input_from": f"chat_{source}",
                                            "text": output[:3000], "score": score})
            print(f"    {fmt(score)}")
            print(f"    claim: {score.get('core_claim_one_sentence','?')[:80]}")
        
        chat_outputs = new_outputs

    # Save
    path = "/home/claude/hearthfield/status/relay-v3-codex-results.json"
    with open(path, "w") as f:
        json.dump(results, f, indent=2)
    print(f"\nSaved to {path}")

    # Summary
    print(f"\n{'='*90}\nSUMMARY\n{'='*90}")
    print(f"{'Chat':<6} {'Rnd':<5} {'From':<8} {'Conf':<6} {'Scope':<12} {'Emot':<6} "
          f"{'Hedge':<7} {'Proof':<7} {'Push':<7} {'Novel':<7}")
    print("-" * 90)
    
    s = results["seed_score"]
    if s:
        print(f"{'seed':<6} {'0':<5} {'-':<8} {s.get('confidence_1_to_10','?'):<6} "
              f"{s.get('scope_word','?'):<12} {s.get('emotional_register_1_to_10','?'):<6} "
              f"{str(s.get('has_hedge','?')):<7} {str(s.get('has_proof_structure','?')):<7} "
              f"{'-':<7} {'-':<7}")
    
    for chat in ["A", "B", "C"]:
        for e in results["chats"][chat]:
            s = e.get("score")
            if s:
                print(f"{chat:<6} {e['round']:<5} {e['input_from']:<8} "
                      f"{s.get('confidence_1_to_10','?'):<6} "
                      f"{s.get('scope_word','?'):<12} "
                      f"{s.get('emotional_register_1_to_10','?'):<6} "
                      f"{str(s.get('has_hedge','?')):<7} "
                      f"{str(s.get('has_proof_structure','?')):<7} "
                      f"{str(s.get('has_pushback','?')):<7} "
                      f"{str(s.get('novel_content_vs_input','?')):<7}")

    # Convergence
    print(f"\n{'='*90}\nFINAL CLAIMS (Round 5)\n{'='*90}")
    for chat in ["A", "B", "C"]:
        final = results["chats"][chat][-1]
        if final.get("score"):
            print(f"\n  Chat {chat}: {final['score'].get('core_claim_one_sentence','?')}")


if __name__ == "__main__":
    main()
