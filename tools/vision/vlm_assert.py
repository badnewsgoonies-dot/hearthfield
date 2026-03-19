#!/usr/bin/env python3
"""
vlm_assert.py — Tier 4 VLM Visual Assertion Gate

Sends a screenshot to a vision-language model with structured assertions.
Returns pass/fail per assertion with reasoning.

The vision agent never sees source code — only the rendered output.
This is the Godogen principle: grounding against actual pixels, not code.

Usage:
  # Single assertion
  python3 vlm_assert.py --image screenshot.png \
    --assertion "The player character is visible on screen" \
    --provider anthropic

  # Multiple assertions
  python3 vlm_assert.py --image screenshot.png \
    --assertions "Player visible" "HUD shows health" "Crops in rows" \
    --provider openai

  # From a manifest
  python3 vlm_assert.py --image screenshot.png \
    --assertions-file assertions.txt \
    --provider anthropic --model claude-sonnet-4-20250514
"""
import argparse
import base64
import json
import os
import sys
from pathlib import Path


SYSTEM_PROMPT = """You are a visual QA agent. You evaluate screenshots against specific assertions.

For each assertion, respond with a JSON object:
{
  "assertions": [
    {
      "assertion": "the text of the assertion",
      "passed": true or false,
      "confidence": 0.0-1.0,
      "evidence": "what you see that supports or contradicts the assertion",
      "location": "where in the image (optional)"
    }
  ],
  "overall_passed": true if ALL assertions pass else false,
  "summary": "one sentence overall assessment"
}

Be precise. If you cannot determine whether an assertion is true from the image alone,
set passed=false and explain why in the evidence field. Do not guess.
Respond ONLY with the JSON object, no markdown, no preamble."""


def _load_image_b64(path: str) -> tuple[str, str]:
    """Load image as base64 and detect media type."""
    data = Path(path).read_bytes()
    b64 = base64.standard_b64encode(data).decode("utf-8")
    ext = Path(path).suffix.lower()
    media_types = {
        ".png": "image/png",
        ".jpg": "image/jpeg",
        ".jpeg": "image/jpeg",
        ".gif": "image/gif",
        ".webp": "image/webp",
    }
    return b64, media_types.get(ext, "image/png")


def _call_anthropic(image_b64: str, media_type: str, prompt: str,
                    model: str = "claude-sonnet-4-20250514") -> str:
    """Call Anthropic Messages API with vision."""
    import urllib.request

    api_key = os.environ.get("ANTHROPIC_API_KEY")
    if not api_key:
        raise ValueError("ANTHROPIC_API_KEY not set")

    body = json.dumps({
        "model": model,
        "max_tokens": 2000,
        "system": SYSTEM_PROMPT,
        "messages": [{
            "role": "user",
            "content": [
                {
                    "type": "image",
                    "source": {
                        "type": "base64",
                        "media_type": media_type,
                        "data": image_b64,
                    }
                },
                {"type": "text", "text": prompt}
            ]
        }]
    }).encode()

    req = urllib.request.Request(
        "https://api.anthropic.com/v1/messages",
        data=body,
        headers={
            "Content-Type": "application/json",
            "x-api-key": api_key,
            "anthropic-version": "2023-06-01",
        },
        method="POST",
    )
    with urllib.request.urlopen(req, timeout=120) as resp:
        data = json.loads(resp.read())
    return data["content"][0]["text"]


def _call_openai(image_b64: str, media_type: str, prompt: str,
                 model: str = "gpt-4o") -> str:
    """Call OpenAI Chat Completions API with vision."""
    import urllib.request

    api_key = os.environ.get("OPENAI_API_KEY")
    if not api_key:
        raise ValueError("OPENAI_API_KEY not set")

    body = json.dumps({
        "model": model,
        "max_tokens": 2000,
        "messages": [
            {"role": "system", "content": SYSTEM_PROMPT},
            {
                "role": "user",
                "content": [
                    {
                        "type": "image_url",
                        "image_url": {
                            "url": f"data:{media_type};base64,{image_b64}"
                        }
                    },
                    {"type": "text", "text": prompt}
                ]
            }
        ]
    }).encode()

    req = urllib.request.Request(
        "https://api.openai.com/v1/chat/completions",
        data=body,
        headers={
            "Content-Type": "application/json",
            "Authorization": f"Bearer {api_key}",
        },
        method="POST",
    )
    with urllib.request.urlopen(req, timeout=120) as resp:
        data = json.loads(resp.read())
    return data["choices"][0]["message"]["content"]


def _call_gemini(image_b64: str, media_type: str, prompt: str,
                 model: str = "gemini-2.5-flash") -> str:
    """Call Gemini API with vision."""
    try:
        import google.generativeai as genai
    except ImportError:
        raise ImportError("pip install google-generativeai")

    api_key = os.environ.get("GEMINI_API_KEY") or os.environ.get("GOOGLE_API_KEY")
    if not api_key:
        raise ValueError("GEMINI_API_KEY not set")

    genai.configure(api_key=api_key)
    m = genai.GenerativeModel(model, system_instruction=SYSTEM_PROMPT)

    import io
    image_bytes = base64.standard_b64decode(image_b64)
    from PIL import Image
    img = Image.open(io.BytesIO(image_bytes))

    response = m.generate_content([prompt, img])
    return response.text


def _call_vertex(image_b64: str, media_type: str, prompt: str,
                  model: str = "gemini-3-flash-preview") -> str:
    """Call Gemini via Vertex AI (works from containers where consumer API is blocked)."""
    import sys
    sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
    from gemini_vertex import gemini_vision_b64

    full_prompt = SYSTEM_PROMPT + "\n\n" + prompt
    return gemini_vision_b64(image_b64, media_type, full_prompt, model=model)


PROVIDERS = {
    "anthropic": _call_anthropic,
    "openai": _call_openai,
    "gemini": _call_gemini,
    "vertex": _call_vertex,
}


def evaluate(image_path: str, assertions: list[str], provider: str = "anthropic",
             model: str | None = None) -> dict:
    """Run VLM evaluation on a screenshot against assertions.

    Returns structured JSON with per-assertion pass/fail.
    """
    b64, media_type = _load_image_b64(image_path)

    assertions_text = "\n".join(f"- {a}" for a in assertions)
    prompt = f"Evaluate this screenshot against the following assertions:\n\n{assertions_text}"

    call_fn = PROVIDERS.get(provider)
    if not call_fn:
        raise ValueError(f"Unknown provider: {provider}. Use: {list(PROVIDERS.keys())}")

    kwargs = {}
    if model:
        kwargs["model"] = model

    raw_response = call_fn(b64, media_type, prompt, **kwargs)

    # Parse JSON from response (strip markdown fences if present)
    cleaned = raw_response.strip()
    if cleaned.startswith("```"):
        cleaned = cleaned.split("\n", 1)[1] if "\n" in cleaned else cleaned[3:]
    if cleaned.endswith("```"):
        cleaned = cleaned[:-3]
    cleaned = cleaned.strip()

    try:
        result = json.loads(cleaned)
    except json.JSONDecodeError:
        result = {
            "assertions": [],
            "overall_passed": False,
            "summary": "Failed to parse VLM response as JSON",
            "raw_response": raw_response,
        }

    result["image"] = str(image_path)
    result["provider"] = provider
    result["model"] = model or "default"
    return result


def main():
    parser = argparse.ArgumentParser(description="VLM visual assertion gate (Tier 4)")
    parser.add_argument("--image", required=True, help="Screenshot to evaluate")
    parser.add_argument("--assertion", help="Single assertion string")
    parser.add_argument("--assertions", nargs="+", help="Multiple assertion strings")
    parser.add_argument("--assertions-file", help="File with one assertion per line")
    parser.add_argument("--provider", default="anthropic",
                        choices=list(PROVIDERS.keys()),
                        help="VLM provider (default: anthropic)")
    parser.add_argument("--model", help="Model override (default: provider default)")
    parser.add_argument("--output", help="Write JSON result to file")
    parser.add_argument("--json", action="store_true", help="Output JSON to stdout")
    args = parser.parse_args()

    # Collect assertions
    all_assertions = []
    if args.assertion:
        all_assertions.append(args.assertion)
    if args.assertions:
        all_assertions.extend(args.assertions)
    if args.assertions_file:
        lines = Path(args.assertions_file).read_text().strip().splitlines()
        all_assertions.extend(l.strip() for l in lines if l.strip() and not l.startswith("#"))

    if not all_assertions:
        parser.error("Provide at least one assertion via --assertion, --assertions, or --assertions-file")

    result = evaluate(args.image, all_assertions, args.provider, args.model)

    if args.output:
        Path(args.output).parent.mkdir(parents=True, exist_ok=True)
        Path(args.output).write_text(json.dumps(result, indent=2))

    if args.json or not args.output:
        print(json.dumps(result, indent=2))

    passed = result.get("overall_passed", False)
    if not args.json:
        status = "PASS" if passed else "FAIL"
        print(f"\n[{status}] {result.get('summary', '')}")
        for a in result.get("assertions", []):
            mark = "✓" if a.get("passed") else "✗"
            print(f"  {mark} {a.get('assertion', '?')} — {a.get('evidence', '')}")

    sys.exit(0 if passed else 1)


if __name__ == "__main__":
    main()
