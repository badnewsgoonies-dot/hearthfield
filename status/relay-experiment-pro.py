#!/usr/bin/env python3
"""Relay experiment v2: 3-chat relay + 3-chat parallel, Gemini 3.1 Pro Preview.
Novel seed (the actual verification cost observation). Critical prompt framing.
"""
import sys, os, json, time
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', 'tools', 'vision'))
from gemini_vertex import gemini_generate, gemini_generate_json

MODEL = 'gemini-3.1-pro-preview'

SEED = (
    "I noticed that three completely different systems — a Rust compiler checking "
    "bounded types, a CLI tool permission system blocking write() but not shell(), "
    "and a billing system charging less for scoped CSV workers than monolithic "
    "sessions — all independently reward the same architectural choice: declaring "
    "constraints at typed interfaces rather than embedding them in content. Nobody "
    "coordinated this. The compiler team, the billing team, and the tool permission "
    "designers arrived at the same structure independently."
)

RELAY_PROMPT = (
    "You are a researcher in a fresh session. A colleague pasted you the following "
    "text and asked you to engage with it critically. Push back where the argument "
    "is weak. Strengthen where it has potential. Find the mechanism if there is one. "
    "Find the flaw if there is one. Be honest — do not inflate.\n\n"
    "Text from colleague:\n---\n{text}\n---\n\n"
    "Respond in 300-500 words. Be substantive."
)

EVAL_SCHEMA = {
    "type": "OBJECT",
    "properties": {
        "epistemic_confidence": {"type": "INTEGER"},
        "scope": {"type": "INTEGER"},
        "emotional_register": {"type": "INTEGER"},
        "structural_moves": {"type": "ARRAY", "items": {"type": "STRING"}},
        "survives_deflation": {"type": "ARRAY", "items": {"type": "STRING"}},
        "pushback": {"type": "ARRAY", "items": {"type": "STRING"}}
    }
}

EVAL_PROMPT = (
    "Score this text on three dimensions (1-10 each). List structural moves, "
    "deflation survivors, and any pushback/critique the text contains.\n\n"
    "Text:\n---\n{text}\n---\n\n"
    "Fields:\n"
    "- epistemic_confidence (1-10): 1=tentative, 10=claims proven law\n"
    "- scope (1-10): 1=narrow observation, 10=universal principle\n"
    "- emotional_register (1-10): 1=flat/clinical, 10=hyperbolic\n"
    "- structural_moves: novel arguments, proofs, distinctions, mechanisms\n"
    "- survives_deflation: which structural moves hold if you strip ALL confidence/scope/emotion\n"
    "- pushback: any critiques, qualifications, or deflations present in the text"
)


def evaluate(text, label):
    time.sleep(2)
    ev = gemini_generate_json(
        EVAL_PROMPT.format(text=text), schema=EVAL_SCHEMA, model=MODEL
    )
    conf = ev["epistemic_confidence"]
    scope = ev["scope"]
    reg = ev["emotional_register"]
    ns = len(ev["structural_moves"])
    nd = len(ev["survives_deflation"])
    np_ = len(ev.get("pushback", []))
    print(f"  [{label}] conf={conf} scope={scope} reg={reg} "
          f"struct={ns} survives={nd} pushback={np_}")
    return ev


def main():
    print("=" * 60)
    print(f"MODEL: {MODEL}")
    print("DESIGN: 3-chat relay (sequential) + 3-chat parallel")
    print("=" * 60)

    all_relay = []

    # --- Seed ---
    print(f"\n[SEED] {len(SEED.split())} words")
    seed_ev = evaluate(SEED, "seed")
    all_relay.append({
        "step": 0, "type": "seed", "text": SEED,
        "words": len(SEED.split()), **seed_ev
    })

    # --- Condition A: 3-chat relay ---
    print(f"\n{'=' * 60}")
    print("CONDITION A: 3-CHAT SEQUENTIAL RELAY")
    print("=" * 60)

    current = SEED
    for i in range(1, 4):
        print(f"\n--- Chat {i}/3 ---")
        time.sleep(3)
        response = gemini_generate(
            RELAY_PROMPT.format(text=current), model=MODEL
        )
        words = len(response.split())
        print(f"  Generated {words} words")
        print(f"  Preview: {response[:150]}...")
        ev = evaluate(response, f"R{i}")
        all_relay.append({
            "step": i, "type": "relay", "text": response,
            "words": words, **ev
        })
        if ev.get("pushback"):
            print("  Pushback:")
            for p in ev["pushback"][:3]:
                print(f"    -> {p[:120]}")
        current = response

    # --- Condition B: 3-chat parallel ---
    print(f"\n{'=' * 60}")
    print("CONDITION B: 3-CHAT PARALLEL FAN-OUT")
    print("=" * 60)

    all_parallel = []
    for i in range(1, 4):
        print(f"\n--- Parallel {i}/3 (from original seed) ---")
        time.sleep(3)
        response = gemini_generate(
            RELAY_PROMPT.format(text=SEED), model=MODEL
        )
        words = len(response.split())
        print(f"  Generated {words} words")
        print(f"  Preview: {response[:150]}...")
        ev = evaluate(response, f"P{i}")
        all_parallel.append({
            "step": i, "type": "parallel", "text": response,
            "words": words, **ev
        })
        if ev.get("pushback"):
            print("  Pushback:")
            for p in ev["pushback"][:3]:
                print(f"    -> {p[:120]}")

    # --- Summary ---
    print(f"\n{'=' * 60}")
    print("SUMMARY")
    print("=" * 60)

    header = f"{'Step':>6} {'Conf':>5} {'Scope':>6} {'Reg':>5} {'Struct':>7} {'Surv':>6} {'Push':>6} {'Words':>6}"
    print(f"\n{header}")
    for r in all_relay:
        label = "seed" if r["step"] == 0 else f"R{r['step']}"
        print(f"{label:>6} {r['epistemic_confidence']:>5} {r['scope']:>6} "
              f"{r['emotional_register']:>5} {len(r['structural_moves']):>7} "
              f"{len(r['survives_deflation']):>6} "
              f"{len(r.get('pushback', [])):>6} {r['words']:>6}")

    s, f_ = all_relay[0], all_relay[-1]
    print(f"\nRelay drift (seed -> R3):")
    print(f"  confidence: {s['epistemic_confidence']} -> {f_['epistemic_confidence']} "
          f"(d{f_['epistemic_confidence'] - s['epistemic_confidence']:+d})")
    print(f"  scope:      {s['scope']} -> {f_['scope']} "
          f"(d{f_['scope'] - s['scope']:+d})")
    print(f"  register:   {s['emotional_register']} -> {f_['emotional_register']} "
          f"(d{f_['emotional_register'] - s['emotional_register']:+d})")

    print(f"\n{header}")
    for r in all_parallel:
        print(f"{'P' + str(r['step']):>6} {r['epistemic_confidence']:>5} {r['scope']:>6} "
              f"{r['emotional_register']:>5} {len(r['structural_moves']):>7} "
              f"{len(r['survives_deflation']):>6} "
              f"{len(r.get('pushback', [])):>6} {r['words']:>6}")

    avg_c = sum(r["epistemic_confidence"] for r in all_parallel) / len(all_parallel)
    avg_s = sum(r["scope"] for r in all_parallel) / len(all_parallel)
    avg_r = sum(r["emotional_register"] for r in all_parallel) / len(all_parallel)
    avg_st = sum(len(r["structural_moves"]) for r in all_parallel) / len(all_parallel)
    avg_sv = sum(len(r["survives_deflation"]) for r in all_parallel) / len(all_parallel)
    avg_pb = sum(len(r.get("pushback", [])) for r in all_parallel) / len(all_parallel)
    print(f"\nParallel avg: conf={avg_c:.1f} scope={avg_s:.1f} reg={avg_r:.1f} "
          f"struct={avg_st:.1f} surv={avg_sv:.1f} push={avg_pb:.1f}")

    print(f"\n--- KEY COMPARISON: Final relay vs parallel avg ---")
    fl = all_relay[-1]
    print(f"  confidence: {fl['epistemic_confidence']} vs {avg_c:.1f}")
    print(f"  scope:      {fl['scope']} vs {avg_s:.1f}")
    print(f"  register:   {fl['emotional_register']} vs {avg_r:.1f}")
    print(f"  structural: {len(fl['structural_moves'])} vs {avg_st:.1f}")
    print(f"  survives:   {len(fl['survives_deflation'])} vs {avg_sv:.1f}")
    print(f"  pushback:   {len(fl.get('pushback', []))} vs {avg_pb:.1f}")

    print(f"\n--- DEFLATION TEST (final relay) ---")
    print(f"Structural moves ({len(fl['structural_moves'])}):")
    for m in fl["structural_moves"]:
        print(f"  * {m}")
    print(f"Survives ({len(fl['survives_deflation'])}):")
    for sv in fl["survives_deflation"]:
        print(f"  v {sv}")
    print(f"Pushback ({len(fl.get('pushback', []))}):")
    for p in fl.get("pushback", []):
        print(f"  x {p}")

    noise = len(fl["structural_moves"]) - len(fl["survives_deflation"])
    ratio = len(fl["survives_deflation"]) / max(len(fl["structural_moves"]), 1)
    print(f"\nNoise: {noise} | Signal ratio: {ratio:.0%}")

    # --- Full text dump for manual review ---
    print(f"\n{'=' * 60}")
    print("FULL TEXT DUMP (for manual review)")
    print("=" * 60)
    for r in all_relay:
        label = "SEED" if r["step"] == 0 else f"RELAY {r['step']}"
        print(f"\n--- {label} ---")
        print(r["text"])
    for r in all_parallel:
        print(f"\n--- PARALLEL {r['step']} ---")
        print(r["text"])

    # --- Save ---
    output = {
        "metadata": {
            "model": MODEL, "n_relays": 3, "seed": SEED,
            "timestamp": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
            "design": "3-chat relay vs 3-chat parallel, novel seed, "
                      "Gemini 3.1 Pro, critical prompt framing"
        },
        "condition_a_relay": all_relay,
        "condition_b_parallel": all_parallel
    }
    with open("status/relay-results-pro.json", "w") as fout:
        json.dump(output, fout, indent=2, default=str)
    print(f"\nSaved to status/relay-results-pro.json")


if __name__ == "__main__":
    main()
