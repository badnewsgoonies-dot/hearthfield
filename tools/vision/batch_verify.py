#!/usr/bin/env python3
"""
batch_verify.py — Batch Visual Verification Runner

Reads a manifest and runs verification across all scenes.
Outputs CSV-compatible results for integration with spawn_agents_on_csv.

Usage:
  # Run all scenes at Tier 3 (pixel compare only)
  python3 batch_verify.py --manifest scenes.toml --mode tier3

  # Run all scenes at Tier 4 (pixel + VLM)
  python3 batch_verify.py --manifest scenes.toml --mode tier4 --provider anthropic

  # Generate a CSV task file for factory dispatch
  python3 batch_verify.py --manifest scenes.toml --generate-csv tasks.csv

  # Run from a pre-generated CSV (compatible with spawn_agents_on_csv)
  python3 batch_verify.py --from-csv tasks.csv
"""
import argparse
import csv
import json
import sys
from pathlib import Path
from datetime import datetime, timezone

sys.path.insert(0, str(Path(__file__).parent))


def _load_manifest(path: str) -> dict:
    """Load TOML or JSON manifest."""
    p = Path(path)
    text = p.read_text()
    if p.suffix == ".toml":
        try:
            import tomllib
        except ImportError:
            import tomli as tomllib
        return tomllib.loads(text)
    return json.loads(text)


def generate_csv(manifest_path: str, csv_path: str, mode: str = "tier3",
                 provider: str = "anthropic", output_dir: str = "verification_output"):
    """Generate a CSV task file from a manifest. Each row = one scene verification."""
    manifest = _load_manifest(manifest_path)
    scenes = manifest.get("scenes", {})

    rows = []
    for name, config in scenes.items():
        row = {
            "scene_name": name,
            "capture_cmd": config.get("capture_cmd", ""),
            "reference": config.get("reference", ""),
            "assertions": json.dumps(config.get("assertions", [])),
            "mode": mode,
            "provider": provider,
            "output_dir": output_dir,
        }
        rows.append(row)

    fieldnames = ["scene_name", "capture_cmd", "reference", "assertions",
                  "mode", "provider", "output_dir"]

    Path(csv_path).parent.mkdir(parents=True, exist_ok=True)
    with open(csv_path, "w", newline="") as f:
        writer = csv.DictWriter(f, fieldnames=fieldnames)
        writer.writeheader()
        writer.writerows(rows)

    print(f"Generated {csv_path} with {len(rows)} tasks")
    return csv_path


def run_from_csv(csv_path: str, report_path: str = "batch_report.json"):
    """Run verification for each row in a CSV task file."""
    from godogen_loop import run_scene

    results = []
    with open(csv_path) as f:
        reader = csv.DictReader(f)
        rows = list(reader)

    print(f"Running {len(rows)} verification tasks from {csv_path}")

    for i, row in enumerate(rows, 1):
        name = row["scene_name"]
        config = {
            "capture_cmd": row["capture_cmd"],
            "reference": row.get("reference", ""),
            "assertions": json.loads(row.get("assertions", "[]")),
        }
        mode = row.get("mode", "tier3")
        provider = row.get("provider", "anthropic")
        output_dir = row.get("output_dir", "verification_output")

        print(f"\n[{i}/{len(rows)}] {name} (mode={mode})")

        r = run_scene(
            name, config, output_dir, provider,
            skip_vlm=(mode == "tier3"),
            tier3_only=(mode == "tier3"),
        )
        results.append(r)

        status = "PASS" if r.get("overall_passed") else "FAIL"
        print(f"  → {status}")

    # Write report
    passed = sum(1 for r in results if r.get("overall_passed"))
    failed = len(results) - passed

    report = {
        "timestamp": datetime.now(timezone.utc).isoformat(),
        "source_csv": csv_path,
        "total": len(results),
        "passed": passed,
        "failed": failed,
        "results": results,
    }

    Path(report_path).parent.mkdir(parents=True, exist_ok=True)
    Path(report_path).write_text(json.dumps(report, indent=2))

    print(f"\n{'='*50}")
    print(f"Results: {passed} passed, {failed} failed")
    print(f"Report: {report_path}")

    return report


def run_from_manifest(manifest_path: str, mode: str, provider: str,
                      output_dir: str, report_path: str, tolerance: int = 0):
    """Run verification directly from manifest without CSV intermediate."""
    from godogen_loop import run_scene

    manifest = _load_manifest(manifest_path)
    scenes = manifest.get("scenes", {})

    results = []
    for i, (name, config) in enumerate(scenes.items(), 1):
        print(f"\n[{i}/{len(scenes)}] {name}")
        r = run_scene(
            name, config, output_dir, provider,
            skip_vlm=(mode == "tier3"),
            tier3_only=(mode == "tier3"),
            pixel_tolerance=tolerance,
        )
        results.append(r)
        status = "PASS" if r.get("overall_passed") else "FAIL"
        print(f"  → {status}")

    passed = sum(1 for r in results if r.get("overall_passed"))
    failed = len(results) - passed

    report = {
        "timestamp": datetime.now(timezone.utc).isoformat(),
        "manifest": manifest_path,
        "mode": mode,
        "total": len(results),
        "passed": passed,
        "failed": failed,
        "results": results,
    }

    Path(report_path).parent.mkdir(parents=True, exist_ok=True)
    Path(report_path).write_text(json.dumps(report, indent=2))

    print(f"\n{'='*50}")
    print(f"Results: {passed} passed, {failed} failed")
    print(f"Report: {report_path}")

    sys.exit(0 if failed == 0 else 1)


def main():
    parser = argparse.ArgumentParser(description="Batch visual verification runner")
    parser.add_argument("--manifest", help="TOML/JSON manifest file")
    parser.add_argument("--mode", default="tier3", choices=["tier3", "tier4", "tier5"],
                        help="Verification tier (default: tier3)")
    parser.add_argument("--provider", default="anthropic",
                        choices=["anthropic", "openai", "gemini"])
    parser.add_argument("--output-dir", default="verification_output")
    parser.add_argument("--report", default="batch_report.json")
    parser.add_argument("--tolerance", type=int, default=0)
    parser.add_argument("--generate-csv", help="Generate CSV task file instead of running")
    parser.add_argument("--from-csv", help="Run from a CSV task file")
    args = parser.parse_args()

    if args.generate_csv:
        if not args.manifest:
            parser.error("--manifest required with --generate-csv")
        generate_csv(args.manifest, args.generate_csv, args.mode,
                     args.provider, args.output_dir)
    elif args.from_csv:
        run_from_csv(args.from_csv, args.report)
    elif args.manifest:
        run_from_manifest(args.manifest, args.mode, args.provider,
                          args.output_dir, args.report, args.tolerance)
    else:
        parser.error("Provide --manifest, --from-csv, or --generate-csv")


if __name__ == "__main__":
    main()
