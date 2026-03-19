#!/usr/bin/env python3
"""
godogen_loop.py — Tier 5 Godogen Visual Verification Loop

Full cycle: capture screenshot → pixel compare (Tier 3) → VLM assert (Tier 4) → report.

The vision agent never sees source code — only rendered output.
This is the Godogen principle: the verifier is grounded against pixels, not code.

Reads a TOML or JSON manifest defining scenes, capture commands, reference
screenshots, and assertions. Runs the full pipeline for each scene.

Usage:
  python3 godogen_loop.py --manifest scenes.toml --provider anthropic
  python3 godogen_loop.py --manifest scenes.json --provider openai --skip-vlm
  python3 godogen_loop.py --manifest scenes.toml --tier3-only
"""
import argparse
import json
import sys
import os
from pathlib import Path
from datetime import datetime, timezone

# Import sibling modules
sys.path.insert(0, str(Path(__file__).parent))
from screenshot_compare import compare_images
from vlm_assert import evaluate as vlm_evaluate
from capture import capture_command


def _load_manifest(path: str) -> dict:
    """Load a TOML or JSON manifest."""
    p = Path(path)
    text = p.read_text()

    if p.suffix == ".toml":
        try:
            import tomllib
        except ImportError:
            try:
                import tomli as tomllib
            except ImportError:
                raise ImportError("Python 3.11+ or pip install tomli required for TOML")
        return tomllib.loads(text)

    elif p.suffix == ".json":
        return json.loads(text)

    else:
        # Try JSON first, then TOML
        try:
            return json.loads(text)
        except json.JSONDecodeError:
            try:
                import tomllib
                return tomllib.loads(text)
            except Exception:
                raise ValueError(f"Cannot parse {path} as JSON or TOML")


def run_scene(scene_name: str, scene_config: dict, output_dir: str,
              provider: str = "anthropic", model: str | None = None,
              skip_vlm: bool = False, tier3_only: bool = False,
              pixel_tolerance: int = 0, timeout: int = 120) -> dict:
    """Run the full Godogen loop for one scene.

    Steps:
      1. Capture screenshot via the scene's capture_cmd
      2. Tier 3: Compare against reference (if reference exists)
      3. Tier 4: VLM assertions (if assertions defined and not skipped)
    """
    result = {
        "scene": scene_name,
        "timestamp": datetime.now(timezone.utc).isoformat(),
        "tiers": {},
    }

    out_dir = Path(output_dir) / scene_name
    out_dir.mkdir(parents=True, exist_ok=True)
    test_screenshot = str(out_dir / "test.png")

    # --- Step 1: Capture ---
    capture_cmd = scene_config.get("capture_cmd", "")
    if not capture_cmd:
        result["error"] = "No capture_cmd defined"
        result["overall_passed"] = False
        return result

    print(f"  [{scene_name}] Capturing...")
    capture_result = capture_command(capture_cmd, test_screenshot, timeout)
    result["capture"] = capture_result

    if not capture_result.get("success"):
        result["error"] = f"Capture failed: {capture_result.get('reason', 'unknown')}"
        result["overall_passed"] = False
        return result

    # --- Step 2: Tier 3 — Pixel comparison ---
    reference = scene_config.get("reference")
    if reference and Path(reference).exists():
        print(f"  [{scene_name}] Tier 3: Pixel compare...")
        diff_path = str(out_dir / "diff.png")
        tier3 = compare_images(reference, test_screenshot, pixel_tolerance, diff_path)
        result["tiers"]["tier3"] = tier3
        status = "PASS" if tier3["passed"] else "FAIL"
        print(f"    Tier 3: {status} ({tier3['diff_pixels']} pixels differ)")
    elif reference:
        result["tiers"]["tier3"] = {
            "passed": False,
            "reason": f"Reference not found: {reference}",
            "skipped": False,
        }
        print(f"    Tier 3: SKIP (reference not found: {reference})")
    else:
        result["tiers"]["tier3"] = {"skipped": True, "reason": "No reference defined"}
        print(f"    Tier 3: SKIP (no reference)")

    if tier3_only:
        tier3_result = result["tiers"].get("tier3", {})
        result["overall_passed"] = tier3_result.get("passed", False) or tier3_result.get("skipped", False)
        return result

    # --- Step 3: Tier 4 — VLM assertions ---
    assertions = scene_config.get("assertions", [])
    if assertions and not skip_vlm:
        print(f"  [{scene_name}] Tier 4: VLM assertions ({len(assertions)} checks)...")
        try:
            tier4 = vlm_evaluate(test_screenshot, assertions, provider, model)
            result["tiers"]["tier4"] = tier4
            status = "PASS" if tier4.get("overall_passed") else "FAIL"
            passed_count = sum(1 for a in tier4.get("assertions", []) if a.get("passed"))
            print(f"    Tier 4: {status} ({passed_count}/{len(assertions)} assertions)")
        except Exception as e:
            result["tiers"]["tier4"] = {
                "passed": False,
                "error": str(e),
            }
            print(f"    Tier 4: ERROR ({e})")
    else:
        reason = "No assertions defined" if not assertions else "VLM skipped"
        result["tiers"]["tier4"] = {"skipped": True, "reason": reason}
        if not assertions:
            print(f"    Tier 4: SKIP (no assertions)")

    # --- Overall verdict ---
    tier3_ok = result["tiers"].get("tier3", {}).get("passed", True) or \
               result["tiers"].get("tier3", {}).get("skipped", False)
    tier4_ok = result["tiers"].get("tier4", {}).get("overall_passed", True) or \
               result["tiers"].get("tier4", {}).get("skipped", False)
    result["overall_passed"] = tier3_ok and tier4_ok

    return result


def main():
    parser = argparse.ArgumentParser(description="Godogen visual verification loop (Tier 5)")
    parser.add_argument("--manifest", required=True, help="TOML or JSON manifest file")
    parser.add_argument("--provider", default="anthropic",
                        choices=["anthropic", "openai", "gemini"],
                        help="VLM provider for Tier 4")
    parser.add_argument("--model", help="Model override for VLM")
    parser.add_argument("--output-dir", default="verification_output",
                        help="Directory for test screenshots and diffs")
    parser.add_argument("--report", default="verification_report.json",
                        help="JSON report output path")
    parser.add_argument("--tolerance", type=int, default=0,
                        help="Pixel tolerance for Tier 3 (0=pixel-perfect)")
    parser.add_argument("--timeout", type=int, default=120,
                        help="Capture command timeout in seconds")
    parser.add_argument("--skip-vlm", action="store_true",
                        help="Skip Tier 4 VLM assertions (run Tier 3 only)")
    parser.add_argument("--tier3-only", action="store_true",
                        help="Run only Tier 3 pixel comparison")
    parser.add_argument("--scene", help="Run only this scene (by name)")
    args = parser.parse_args()

    manifest = _load_manifest(args.manifest)
    scenes = manifest.get("scenes", manifest.get("scene", {}))

    if args.scene:
        if args.scene not in scenes:
            print(f"Scene '{args.scene}' not found. Available: {list(scenes.keys())}")
            sys.exit(1)
        scenes = {args.scene: scenes[args.scene]}

    print(f"Godogen loop: {len(scenes)} scene(s) from {args.manifest}")
    print(f"  Provider: {args.provider} | Tolerance: {args.tolerance} | Output: {args.output_dir}")
    print()

    results = []
    for name, config in scenes.items():
        print(f"Scene: {name}")
        r = run_scene(
            name, config, args.output_dir, args.provider, args.model,
            args.skip_vlm, args.tier3_only, args.tolerance, args.timeout
        )
        results.append(r)
        status = "PASS" if r.get("overall_passed") else "FAIL"
        print(f"  → {status}")
        print()

    # Summary
    passed = sum(1 for r in results if r.get("overall_passed"))
    failed = len(results) - passed

    report = {
        "timestamp": datetime.now(timezone.utc).isoformat(),
        "manifest": str(args.manifest),
        "provider": args.provider,
        "total_scenes": len(results),
        "passed": passed,
        "failed": failed,
        "results": results,
    }

    report_path = Path(args.report)
    report_path.parent.mkdir(parents=True, exist_ok=True)
    report_path.write_text(json.dumps(report, indent=2))

    print(f"{'='*50}")
    print(f"Results: {passed} passed, {failed} failed, {len(results)} total")
    print(f"Report: {report_path}")

    if failed > 0:
        print("\nFailed scenes:")
        for r in results:
            if not r.get("overall_passed"):
                print(f"  ✗ {r['scene']}: {r.get('error', 'see report')}")

    sys.exit(0 if failed == 0 else 1)


if __name__ == "__main__":
    main()
