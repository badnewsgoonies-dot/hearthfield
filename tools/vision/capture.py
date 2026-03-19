#!/usr/bin/env python3
"""
capture.py — Engine-Agnostic Screenshot Capture

Captures screenshots from any game engine or visual application.
Supports three modes:
  1. command — run an arbitrary command that produces a screenshot file
  2. bevy    — run a Bevy app with --headless or custom features that save a frame
  3. browser — use Playwright to capture a browser-based game

Usage:
  # Arbitrary command (most flexible)
  python3 capture.py --mode command \
    --cmd "cargo run --features headless -- --scene farm --output {output}" \
    --output screenshots/farm.png

  # Bevy project (convenience wrapper)
  python3 capture.py --mode bevy \
    --project-dir /path/to/hearthfield \
    --features "headless,screenshot" \
    --args "--scene farm" \
    --output screenshots/farm.png

  # Browser game (via Playwright)
  python3 capture.py --mode browser \
    --url http://localhost:5174 \
    --selector "canvas.game-canvas" \
    --wait 3 \
    --output screenshots/game.png
"""
import argparse
import os
import subprocess
import sys
import time
from pathlib import Path


def capture_command(cmd: str, output: str, timeout: int = 120) -> dict:
    """Run an arbitrary command that produces a screenshot.

    The command string can contain {output} which is replaced with the output path.
    """
    output_path = Path(output)
    output_path.parent.mkdir(parents=True, exist_ok=True)

    resolved_cmd = cmd.replace("{output}", str(output_path))

    result = {"mode": "command", "cmd": resolved_cmd, "output": str(output_path)}

    try:
        proc = subprocess.run(
            resolved_cmd, shell=True, capture_output=True, text=True, timeout=timeout
        )
        result["returncode"] = proc.returncode
        result["stdout"] = proc.stdout[-500:] if proc.stdout else ""
        result["stderr"] = proc.stderr[-500:] if proc.stderr else ""

        if output_path.exists() and output_path.stat().st_size > 0:
            result["success"] = True
            result["file_size"] = output_path.stat().st_size
        else:
            result["success"] = False
            result["reason"] = "output file not created or empty"
    except subprocess.TimeoutExpired:
        result["success"] = False
        result["reason"] = f"timeout after {timeout}s"

    return result


def capture_bevy(project_dir: str, output: str, features: str = "headless",
                 args: str = "", timeout: int = 120) -> dict:
    """Capture from a Bevy project using cargo run."""
    output_path = Path(output).resolve()
    output_path.parent.mkdir(parents=True, exist_ok=True)

    cmd_parts = ["cargo", "run"]
    if features:
        cmd_parts.extend(["--features", features])
    cmd_parts.append("--")
    if args:
        cmd_parts.extend(args.split())
    cmd_parts.extend(["--screenshot", str(output_path)])

    result = {
        "mode": "bevy",
        "project_dir": str(project_dir),
        "cmd": " ".join(cmd_parts),
        "output": str(output_path),
    }

    try:
        proc = subprocess.run(
            cmd_parts, cwd=project_dir, capture_output=True, text=True, timeout=timeout
        )
        result["returncode"] = proc.returncode
        result["stderr"] = proc.stderr[-500:] if proc.stderr else ""

        if output_path.exists() and output_path.stat().st_size > 0:
            result["success"] = True
            result["file_size"] = output_path.stat().st_size
        else:
            result["success"] = False
            result["reason"] = "screenshot not created"
    except subprocess.TimeoutExpired:
        result["success"] = False
        result["reason"] = f"timeout after {timeout}s"
    except FileNotFoundError:
        result["success"] = False
        result["reason"] = "cargo not found on PATH"

    return result


def capture_browser(url: str, output: str, selector: str | None = None,
                    wait: float = 2.0, viewport_w: int = 1280,
                    viewport_h: int = 720, setup_actions: list[str] | None = None) -> dict:
    """Capture from a browser-based game using Playwright."""
    output_path = Path(output)
    output_path.parent.mkdir(parents=True, exist_ok=True)

    result = {"mode": "browser", "url": url, "output": str(output_path)}

    try:
        from playwright.sync_api import sync_playwright
    except ImportError:
        result["success"] = False
        result["reason"] = "playwright not installed: pip install playwright && playwright install"
        return result

    try:
        with sync_playwright() as p:
            browser = p.chromium.launch(headless=True)
            context = browser.new_context(viewport={"width": viewport_w, "height": viewport_h})
            page = context.new_page()

            page.goto(url, wait_until="networkidle", timeout=30000)
            time.sleep(wait)

            # Run optional setup actions (click buttons, press keys, etc.)
            if setup_actions:
                for action in setup_actions:
                    parts = action.split(":", 1)
                    act_type = parts[0].strip()
                    act_arg = parts[1].strip() if len(parts) > 1 else ""

                    if act_type == "click":
                        page.locator(act_arg).first.click()
                    elif act_type == "press":
                        page.keyboard.press(act_arg)
                    elif act_type == "wait":
                        time.sleep(float(act_arg))
                    elif act_type == "type":
                        page.keyboard.type(act_arg)
                    time.sleep(0.5)

            if selector:
                element = page.locator(selector)
                element.screenshot(path=str(output_path))
            else:
                page.screenshot(path=str(output_path))

            browser.close()

        if output_path.exists() and output_path.stat().st_size > 0:
            result["success"] = True
            result["file_size"] = output_path.stat().st_size
        else:
            result["success"] = False
            result["reason"] = "screenshot not created"

    except Exception as e:
        result["success"] = False
        result["reason"] = str(e)

    return result


def main():
    parser = argparse.ArgumentParser(description="Engine-agnostic screenshot capture")
    parser.add_argument("--mode", required=True, choices=["command", "bevy", "browser"],
                        help="Capture mode")
    parser.add_argument("--output", required=True, help="Output screenshot path")
    parser.add_argument("--timeout", type=int, default=120, help="Timeout in seconds")

    # Command mode
    parser.add_argument("--cmd", help="Shell command (use {output} placeholder)")

    # Bevy mode
    parser.add_argument("--project-dir", help="Bevy project directory")
    parser.add_argument("--features", default="headless", help="Cargo features")
    parser.add_argument("--args", default="", help="Extra args after --")

    # Browser mode
    parser.add_argument("--url", help="URL to capture")
    parser.add_argument("--selector", help="CSS selector for element screenshot")
    parser.add_argument("--wait", type=float, default=2.0, help="Wait seconds after load")
    parser.add_argument("--viewport", default="1280x720", help="Viewport WxH")
    parser.add_argument("--setup", nargs="*",
                        help="Setup actions: 'click:.start-btn' 'press:Enter' 'wait:2'")

    parser.add_argument("--json", action="store_true", help="Output JSON")
    args = parser.parse_args()

    if args.mode == "command":
        if not args.cmd:
            parser.error("--cmd required for command mode")
        result = capture_command(args.cmd, args.output, args.timeout)

    elif args.mode == "bevy":
        if not args.project_dir:
            parser.error("--project-dir required for bevy mode")
        result = capture_bevy(args.project_dir, args.output, args.features,
                              args.args, args.timeout)

    elif args.mode == "browser":
        if not args.url:
            parser.error("--url required for browser mode")
        vw, vh = (int(x) for x in args.viewport.split("x"))
        result = capture_browser(args.url, args.output, args.selector,
                                 args.wait, vw, vh, args.setup)

    if args.json:
        print(json.dumps(result, indent=2))
    else:
        status = "OK" if result.get("success") else "FAIL"
        print(f"[{status}] {result['output']}")
        if not result.get("success"):
            print(f"  Reason: {result.get('reason', 'unknown')}")

    sys.exit(0 if result.get("success") else 1)


if __name__ == "__main__":
    import json
    main()
