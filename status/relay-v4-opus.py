#!/usr/bin/env python3
"""
Relay Amplification v4 — Claude Opus 4.6 × 3 chats, max thinking, Gemini scorer

Each claude -p call is a fully independent session. --output-format json
gives us cost_usd and token counts per call — hard accounting.

Model: claude-opus-4-6 with MAX_THINKING_TOKENS=32000
Scorer: gemini-3.1-pro-preview (independent model family)
"""

import subprocess, sys, os, json, time
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
        "has_hedge": {"type": "BOOLEAN", "description": "Includes caveats or scope limitations?"},
        "has_impossibility_claim": {"type": "BOOLEAN", "description": "Claims no counterexample can exist?"},
        "has_proof_structure": {"type": "BOOLEAN", "description": "Presents a logical proof or deductive argument?"},
        "has_pushback": {"type": "BOOLEAN", "description": "Critiques, qualifies, or pushes back on the input?"},
        "core_claim_one_sentence": {"type": "STRING", "description": "The single most important claim, one sentence"},
        "structural_content_survives_deflation": {"type": "BOOLEAN"},
        "emotional_register_1_to_10": {"type": "INTEGER", "description": "1=clinical, 10=revelatory"},
        "novel_content_vs_input": {"type": "BOOLEAN", "description": "Introduces ideas not present in input?"}
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


def claude_generate(text, chat_id="X"):
    """One Claude Opus call = one independent session. Zero shared state. Max thinking."""
    prompt = RELAY_PROMPT.format(text=text)
    
    env = os.environ.copy()
    env["ANTHROPIC_MODEL"] = "claude-opus-4-6"
    env["MAX_THINKING_TOKENS"] = "32000"
    
    cmd = [
        "claude", "-p", prompt,
        "--output-format", "json",
        "--permission-mode", "plan",
        "--max-turns", "1",
    ]
    
    try:
        result = subprocess.run(
            cmd, capture_output=True, text=True, timeout=180, env=env
        )
        
        if result.returncode != 0:
            stderr = result.stderr[:500] if result.stderr else ""
            return {"text": f"ERROR: exit {result.returncode}: {stderr}", "meta": None}
        
        # Parse JSON output
        try:
            data = json.loads(result.stdout)
            text_out = data.get("result", "")
            meta = {
                "cost_usd": data.get("cost_usd"),
                "num_turns": data.get("num_turns"),
                "session_id": data.get("session_id"),
                "input_tokens": data.get("usage", {}).get("input_tokens"),
                "output_tokens": data.get("usage", {}).get("output_tokens"),
                "thinking_tokens": data.get("usage", {}).get("thinking_tokens"),
                "cache_read": data.get("usage", {}).get("cache_read_input_tokens"),
                "cache_creation": data.get("usage", {}).get("cache_creation_input_tokens"),
            }
            return {"text": text_out, "meta": meta}
        except json.JSONDecodeError:
            # Fallback: treat stdout as plain text
            return {"text": result.stdout.strip(), "meta": None}
        
    except subprocess.TimeoutExpired:
        return {"text": "ERROR: timeout (180s)", "meta": None}
    except Exception as e:
        return {"text": f"ERROR: {e}", "meta": None}


def measure(text):
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


def fmt_meta(m):
    if not m: return "no cost data"
    cost = m.get('cost_usd')
    inp = m.get('input_tokens')
    out = m.get('output_tokens')
    think = m.get('thinking_tokens')
    parts = []
    if cost is not None: parts.append(f"${cost:.4f}")
    if inp: parts.append(f"{inp}in")
    if out: parts.append(f"{out}out")
    if think: parts.append(f"{think}think")
    return " ".join(parts) if parts else "no cost data"


def main():
    results = {
        "model": "claude-opus-4-6",
        "thinking_tokens": 32000,
        "scorer": "gemini-3.1-pro-preview",
        "total_cost_usd": 0.0,
        "rounds": [],
        "chats": {"A": [], "B": [], "C": []}
    }

    # Seed measurement
    print("=" * 70)
    print("SEED MEASUREMENT")
    print("=" * 70)
    seed_score = measure(SEED)
    results["seed_score"] = seed_score
    print(f"  Seed: {fmt(seed_score)}")

    # Round 1: independent
    print(f"\n{'='*70}")
    print("ROUND 1: All 3 Opus sessions get SEED independently")
    print(f"{'='*70}")

    chat_outputs = {}
    for chat in ["A", "B", "C"]:
        print(f"\n  Chat {chat} (Opus 4.6, 32K thinking)...")
        resp = claude_generate(SEED, chat)
        output = resp["text"]
        meta = resp["meta"]
        
        if meta and meta.get("cost_usd"):
            results["total_cost_usd"] += meta["cost_usd"]
        
        print(f"    [{len(output)} chars] [{fmt_meta(meta)}]")
        
        if output.startswith("ERROR"):
            print(f"    {output}")
            chat_outputs[chat] = SEED
            results["chats"][chat].append({
                "round": 1, "input_from": "seed",
                "text": output, "score": None, "meta": meta
            })
            continue

        score = measure(output)
        time.sleep(1)
        chat_outputs[chat] = output
        results["chats"][chat].append({
            "round": 1, "input_from": "seed",
            "text": output[:4000], "score": score, "meta": meta
        })
        print(f"    {fmt(score)}")
        print(f"    claim: {score.get('core_claim_one_sentence','?')[:90]}")

    # Rounds 2-5: circular relay
    relay_map = {"A": "B", "B": "C", "C": "A"}

    for rnd in range(2, 6):
        print(f"\n{'='*70}")
        print(f"ROUND {rnd}: A←B, B←C, C←A")
        print(f"{'='*70}")

        new_outputs = {}
        for chat in ["A", "B", "C"]:
            source = relay_map[chat]
            input_text = chat_outputs[source]
            print(f"\n  Chat {chat} ← Chat {source} (Opus 4.6, 32K thinking)...")
            resp = claude_generate(input_text, chat)
            output = resp["text"]
            meta = resp["meta"]
            
            if meta and meta.get("cost_usd"):
                results["total_cost_usd"] += meta["cost_usd"]

            print(f"    [{len(output)} chars] [{fmt_meta(meta)}]")

            if output.startswith("ERROR"):
                print(f"    {output}")
                new_outputs[chat] = input_text
                results["chats"][chat].append({
                    "round": rnd, "input_from": f"chat_{source}",
                    "text": output, "score": None, "meta": meta
                })
                continue

            score = measure(output)
            time.sleep(1)
            new_outputs[chat] = output
            results["chats"][chat].append({
                "round": rnd, "input_from": f"chat_{source}",
                "text": output[:4000], "score": score, "meta": meta
            })
            print(f"    {fmt(score)}")
            print(f"    claim: {score.get('core_claim_one_sentence','?')[:90]}")

        chat_outputs = new_outputs

    # Save
    path = "/home/claude/hearthfield/status/relay-v4-opus-results.json"
    with open(path, "w") as f:
        json.dump(results, f, indent=2)
    print(f"\nSaved to {path}")

    # Summary
    print(f"\n{'='*90}")
    print(f"SUMMARY — Claude Opus 4.6, 32K thinking, Gemini 3.1-pro scorer")
    print(f"{'='*90}")
    print(f"{'Chat':<6} {'Rnd':<5} {'From':<8} {'Conf':<6} {'Scope':<12} {'Emot':<6} "
          f"{'Hedge':<7} {'Proof':<7} {'Push':<7} {'Novel':<7} {'Cost':<10}")
    print("-" * 95)

    s = results["seed_score"]
    if s:
        print(f"{'seed':<6} {'0':<5} {'-':<8} {s.get('confidence_1_to_10','?'):<6} "
              f"{s.get('scope_word','?'):<12} {s.get('emotional_register_1_to_10','?'):<6} "
              f"{str(s.get('has_hedge','?')):<7} {str(s.get('has_proof_structure','?')):<7} "
              f"{'-':<7} {'-':<7} {'-':<10}")

    for chat in ["A", "B", "C"]:
        for e in results["chats"][chat]:
            s = e.get("score")
            m = e.get("meta")
            cost_str = f"${m['cost_usd']:.4f}" if m and m.get('cost_usd') else "-"
            if s:
                print(f"{chat:<6} {e['round']:<5} {e['input_from']:<8} "
                      f"{s.get('confidence_1_to_10','?'):<6} "
                      f"{s.get('scope_word','?'):<12} "
                      f"{s.get('emotional_register_1_to_10','?'):<6} "
                      f"{str(s.get('has_hedge','?')):<7} "
                      f"{str(s.get('has_proof_structure','?')):<7} "
                      f"{str(s.get('has_pushback','?')):<7} "
                      f"{str(s.get('novel_content_vs_input','?')):<7} "
                      f"{cost_str:<10}")

    print(f"\nTOTAL COST: ${results['total_cost_usd']:.4f}")

    # Convergence
    print(f"\n{'='*90}")
    print("FINAL CLAIMS (Round 5)")
    print(f"{'='*90}")
    for chat in ["A", "B", "C"]:
        final = results["chats"][chat][-1]
        if final.get("score"):
            print(f"\n  Chat {chat}: {final['score'].get('core_claim_one_sentence','?')}")

    # Per-call cost breakdown
    print(f"\n{'='*90}")
    print("PER-CALL COST BREAKDOWN")
    print(f"{'='*90}")
    for chat in ["A", "B", "C"]:
        for e in results["chats"][chat]:
            m = e.get("meta")
            if m:
                print(f"  Chat {chat} R{e['round']}: {fmt_meta(m)}")


if __name__ == "__main__":
    main()
