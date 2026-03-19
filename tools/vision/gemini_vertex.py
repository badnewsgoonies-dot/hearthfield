"""
Gemini via Vertex AI — OAuth-based wrapper for text, vision, and JSON mode.

Works from containers where generativelanguage.googleapis.com is blocked
but us-central1-aiplatform.googleapis.com and aiplatform.googleapis.com are open.

Required env vars:
    GEMINI_OAUTH_CLIENT_ID
    GEMINI_OAUTH_CLIENT_SECRET
    GEMINI_OAUTH_REFRESH_TOKEN
    GEMINI_VERTEX_PROJECT

Models and locations:
    Stable (2.x)  -> us-central1
    Preview (3.x) -> global
"""

import base64
import json
import os
import time
import urllib.request
import urllib.parse
from typing import Any, Optional

# ---------- token cache ----------

_token_cache = {"access_token": None, "expires_at": 0.0}


def _get_access_token():
    """Get or refresh OAuth access token. Caches for ~55 minutes."""
    if _token_cache["access_token"] and time.time() < _token_cache["expires_at"]:
        return _token_cache["access_token"]

    client_id = os.environ["GEMINI_OAUTH_CLIENT_ID"]
    client_secret = os.environ["GEMINI_OAUTH_CLIENT_SECRET"]
    refresh_token = os.environ["GEMINI_OAUTH_REFRESH_TOKEN"]

    data = urllib.parse.urlencode({
        "client_id": client_id,
        "client_secret": client_secret,
        "grant_type": "refresh_token",
        "refresh_token": refresh_token,
    }).encode()

    req = urllib.request.Request("https://oauth2.googleapis.com/token", data=data)
    with urllib.request.urlopen(req, timeout=15) as resp:
        result = json.loads(resp.read())

    _token_cache["access_token"] = result["access_token"]
    _token_cache["expires_at"] = time.time() + result.get("expires_in", 3600) - 300
    return _token_cache["access_token"]


# ---------- endpoint routing ----------

def _endpoint(model):
    """Route model to correct Vertex AI endpoint."""
    project = os.environ["GEMINI_VERTEX_PROJECT"]

    # Preview models (3.x) -> global
    if any(tag in model for tag in ("preview", "3-flash", "3.1-", "3-pro")):
        return (
            "https://aiplatform.googleapis.com/v1/projects/{}/locations/global"
            "/publishers/google/models/{}:generateContent".format(project, model)
        )
    # Stable models (2.x) -> us-central1
    return (
        "https://us-central1-aiplatform.googleapis.com/v1/projects/{}/locations/us-central1"
        "/publishers/google/models/{}:generateContent".format(project, model)
    )


# ---------- response extraction ----------

def _extract_text(response):
    """Extract text from response, handling thinking models.

    Gemini 3.x thinking models put text and thoughtSignature in the same
    parts array. We collect text fields that are NOT thinking signatures.
    If all parts have thoughtSignature, we fall back to collecting their
    text anyway (thinking-only response with no separate output).
    """
    candidates = response.get("candidates", [])
    if not candidates:
        raise ValueError("No candidates in response: {}".format(
            json.dumps(response)[:500]))

    candidate = candidates[0]
    finish = candidate.get("finishReason", "")
    if finish not in ("STOP", "MAX_TOKENS", ""):
        raise ValueError("Unexpected finish reason: {}".format(finish))

    parts = candidate.get("content", {}).get("parts", [])

    # First pass: collect non-thinking text parts
    text_parts = []
    for part in parts:
        if "text" in part and "thoughtSignature" not in part:
            text_parts.append(part["text"])

    if text_parts:
        return "".join(text_parts)

    # Fallback: thinking-only response — grab text from thought parts
    for part in parts:
        if "text" in part:
            text_parts.append(part["text"])

    if text_parts:
        return "".join(text_parts)

    raise ValueError("No text in response parts: {}".format(
        json.dumps(parts)[:500]))


# ---------- core API call ----------

def _call(contents, model, generation_config=None):
    """Make a raw Vertex AI generateContent call."""
    token = _get_access_token()
    url = _endpoint(model)

    body = {"contents": contents}
    if generation_config:
        body["generationConfig"] = generation_config

    payload = json.dumps(body).encode()
    req = urllib.request.Request(
        url,
        data=payload,
        headers={
            "Authorization": "Bearer {}".format(token),
            "Content-Type": "application/json",
        },
    )

    with urllib.request.urlopen(req, timeout=120) as resp:
        return json.loads(resp.read())


# ---------- public API ----------

def gemini_generate(prompt, model="gemini-3-flash-preview", max_tokens=4096):
    """Text generation. Returns response text."""
    contents = [{"role": "user", "parts": [{"text": prompt}]}]
    config = {"maxOutputTokens": max_tokens}
    response = _call(contents, model, config)
    return _extract_text(response)


def gemini_generate_json(prompt, schema, model="gemini-3-flash-preview",
                         max_tokens=4096):
    """Text generation with forced JSON output. Returns parsed dict."""
    contents = [{"role": "user", "parts": [{"text": prompt}]}]
    config = {
        "maxOutputTokens": max_tokens,
        "responseMimeType": "application/json",
        "responseSchema": schema,
    }
    response = _call(contents, model, config)
    text = _extract_text(response)
    return json.loads(text)


def gemini_vision(image_path, prompt, model="gemini-3-flash-preview",
                  max_tokens=4096):
    """Vision: image + text prompt. Returns response text."""
    with open(image_path, "rb") as f:
        image_data = base64.b64encode(f.read()).decode()

    ext = image_path.rsplit(".", 1)[-1].lower()
    mime_map = {"png": "image/png", "jpg": "image/jpeg", "jpeg": "image/jpeg",
                "gif": "image/gif", "webp": "image/webp"}
    mime_type = mime_map.get(ext, "image/png")

    contents = [{"role": "user", "parts": [
        {"inlineData": {"mimeType": mime_type, "data": image_data}},
        {"text": prompt},
    ]}]
    config = {"maxOutputTokens": max_tokens}
    response = _call(contents, model, config)
    return _extract_text(response)


def gemini_vision_json(image_path, prompt, schema, model="gemini-3-flash-preview",
                       max_tokens=4096):
    """Vision with forced JSON output. Returns parsed dict."""
    with open(image_path, "rb") as f:
        image_data = base64.b64encode(f.read()).decode()

    ext = image_path.rsplit(".", 1)[-1].lower()
    mime_map = {"png": "image/png", "jpg": "image/jpeg", "jpeg": "image/jpeg",
                "gif": "image/gif", "webp": "image/webp"}
    mime_type = mime_map.get(ext, "image/png")

    contents = [{"role": "user", "parts": [
        {"inlineData": {"mimeType": mime_type, "data": image_data}},
        {"text": prompt},
    ]}]
    config = {
        "maxOutputTokens": max_tokens,
        "responseMimeType": "application/json",
        "responseSchema": schema,
    }
    response = _call(contents, model, config)
    text = _extract_text(response)
    return json.loads(text)


def gemini_vision_b64(image_b64, mime_type, prompt,
                      model="gemini-3-flash-preview", max_tokens=4096):
    """Vision from base64 string (for piping from other tools)."""
    contents = [{"role": "user", "parts": [
        {"inlineData": {"mimeType": mime_type, "data": image_b64}},
        {"text": prompt},
    ]}]
    config = {"maxOutputTokens": max_tokens}
    response = _call(contents, model, config)
    return _extract_text(response)


# ---------- CLI ----------

if __name__ == "__main__":
    import sys
    if len(sys.argv) < 2:
        print("Usage: gemini_vertex.py <prompt> [--model MODEL] [--image PATH]")
        print("       gemini_vertex.py --json <prompt> --schema '{...}'")
        sys.exit(1)

    args = sys.argv[1:]
    model = "gemini-3-flash-preview"
    image = None
    use_json = False
    schema_str = None
    prompt_parts = []

    i = 0
    while i < len(args):
        if args[i] == "--model" and i + 1 < len(args):
            model = args[i + 1]; i += 2
        elif args[i] == "--image" and i + 1 < len(args):
            image = args[i + 1]; i += 2
        elif args[i] == "--json":
            use_json = True; i += 1
        elif args[i] == "--schema" and i + 1 < len(args):
            schema_str = args[i + 1]; i += 2
        else:
            prompt_parts.append(args[i]); i += 1

    prompt = " ".join(prompt_parts)

    if image and use_json and schema_str:
        result = gemini_vision_json(image, prompt, json.loads(schema_str), model)
        print(json.dumps(result, indent=2))
    elif image:
        print(gemini_vision(image, prompt, model))
    elif use_json and schema_str:
        result = gemini_generate_json(prompt, json.loads(schema_str), model)
        print(json.dumps(result, indent=2))
    else:
        print(gemini_generate(prompt, model))
