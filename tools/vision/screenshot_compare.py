#!/usr/bin/env python3
"""
screenshot_compare.py — Tier 3 Visual Regression Gate

Compares a test screenshot against a reference screenshot.
Outputs structured pass/fail JSON and an optional diff image.

For pixel art at fixed resolution: use --tolerance 0 (pixel-perfect).
For non-deterministic renderers: use --tolerance N (N differing pixels allowed).

Usage:
  python3 screenshot_compare.py --reference ref.png --test test.png
  python3 screenshot_compare.py --reference ref.png --test test.png --tolerance 0 --diff diff.png
  python3 screenshot_compare.py --reference-dir refs/ --test-dir tests/ --report report.json
"""
import argparse
import json
import sys
from pathlib import Path

try:
    from PIL import Image, ImageChops
except ImportError:
    print("Requires Pillow: pip install Pillow", file=sys.stderr)
    sys.exit(1)


def compare_images(ref_path: str, test_path: str, tolerance: int = 0,
                   diff_path: str | None = None, channel_threshold: int = 0) -> dict:
    """Compare two images pixel-by-pixel.

    Args:
        ref_path: Path to reference screenshot.
        test_path: Path to test screenshot.
        tolerance: Max number of differing pixels before failure. 0 = pixel-perfect.
        diff_path: If set, write a diff image highlighting changed pixels.
        channel_threshold: Per-channel difference below this is ignored (for anti-aliasing).

    Returns:
        Dict with pass/fail, counts, and metadata.
    """
    ref = Image.open(ref_path).convert("RGBA")
    test = Image.open(test_path).convert("RGBA")

    result = {
        "reference": str(ref_path),
        "test": str(test_path),
        "ref_size": list(ref.size),
        "test_size": list(test.size),
        "tolerance": tolerance,
        "channel_threshold": channel_threshold,
    }

    # Size mismatch is always a failure
    if ref.size != test.size:
        result["passed"] = False
        result["reason"] = "size_mismatch"
        result["diff_pixels"] = -1
        result["total_pixels"] = ref.size[0] * ref.size[1]
        return result

    total_pixels = ref.size[0] * ref.size[1]
    ref_data = ref.load()
    test_data = test.load()
    width, height = ref.size

    diff_count = 0
    diff_img = None
    if diff_path:
        diff_img = Image.new("RGBA", ref.size, (0, 0, 0, 255))
        diff_data = diff_img.load()

    for y in range(height):
        for x in range(width):
            rp = ref_data[x, y]
            tp = test_data[x, y]

            pixel_differs = False
            if channel_threshold == 0:
                pixel_differs = rp != tp
            else:
                pixel_differs = any(
                    abs(rp[c] - tp[c]) > channel_threshold for c in range(4)
                )

            if pixel_differs:
                diff_count += 1
                if diff_img is not None:
                    # Highlight diff in red, show matching pixels dimmed
                    diff_data[x, y] = (255, 0, 0, 255)
            elif diff_img is not None:
                # Dim the matching pixel
                avg = (rp[0] + rp[1] + rp[2]) // 3
                diff_data[x, y] = (avg // 3, avg // 3, avg // 3, 255)

    if diff_img is not None and diff_path:
        Path(diff_path).parent.mkdir(parents=True, exist_ok=True)
        diff_img.save(diff_path)
        result["diff_image"] = str(diff_path)

    passed = diff_count <= tolerance
    result["passed"] = passed
    result["diff_pixels"] = diff_count
    result["total_pixels"] = total_pixels
    result["diff_percent"] = round(diff_count / total_pixels * 100, 4) if total_pixels > 0 else 0.0
    result["reason"] = "pass" if passed else "pixel_diff_exceeded"

    return result


def compare_directories(ref_dir: str, test_dir: str, tolerance: int = 0,
                        diff_dir: str | None = None, channel_threshold: int = 0,
                        report_path: str | None = None) -> list[dict]:
    """Compare all matching PNG files between two directories."""
    ref_path = Path(ref_dir)
    test_path = Path(test_dir)
    diff_path_base = Path(diff_dir) if diff_dir else None

    ref_files = sorted(ref_path.glob("*.png"))
    results = []

    for ref_file in ref_files:
        test_file = test_path / ref_file.name
        if not test_file.exists():
            results.append({
                "reference": str(ref_file),
                "test": str(test_file),
                "passed": False,
                "reason": "test_file_missing",
                "diff_pixels": -1,
            })
            continue

        diff_out = None
        if diff_path_base:
            diff_out = str(diff_path_base / f"diff_{ref_file.name}")

        r = compare_images(str(ref_file), str(test_file), tolerance, diff_out, channel_threshold)
        results.append(r)

    # Check for extra test files not in reference
    ref_names = {f.name for f in ref_files}
    for test_file in sorted(test_path.glob("*.png")):
        if test_file.name not in ref_names:
            results.append({
                "reference": None,
                "test": str(test_file),
                "passed": False,
                "reason": "no_reference_exists",
                "diff_pixels": -1,
            })

    if report_path:
        Path(report_path).parent.mkdir(parents=True, exist_ok=True)
        summary = {
            "total": len(results),
            "passed": sum(1 for r in results if r["passed"]),
            "failed": sum(1 for r in results if not r["passed"]),
            "results": results,
        }
        Path(report_path).write_text(json.dumps(summary, indent=2))

    return results


def main():
    parser = argparse.ArgumentParser(description="Pixel-level screenshot comparison (Tier 3 gate)")
    parser.add_argument("--reference", help="Reference screenshot path")
    parser.add_argument("--test", help="Test screenshot path")
    parser.add_argument("--reference-dir", help="Directory of reference screenshots")
    parser.add_argument("--test-dir", help="Directory of test screenshots")
    parser.add_argument("--diff", help="Output diff image path (single-file mode)")
    parser.add_argument("--diff-dir", help="Output diff directory (directory mode)")
    parser.add_argument("--tolerance", type=int, default=0,
                        help="Max differing pixels before failure. 0=pixel-perfect (default)")
    parser.add_argument("--channel-threshold", type=int, default=0,
                        help="Per-channel difference below this is ignored (for anti-aliasing)")
    parser.add_argument("--report", help="JSON report output path (directory mode)")
    parser.add_argument("--json", action="store_true", help="Output JSON to stdout")
    args = parser.parse_args()

    if args.reference and args.test:
        result = compare_images(args.reference, args.test, args.tolerance,
                                args.diff, args.channel_threshold)
        if args.json:
            print(json.dumps(result, indent=2))
        else:
            status = "PASS" if result["passed"] else "FAIL"
            print(f"[{status}] {result['diff_pixels']}/{result['total_pixels']} pixels differ "
                  f"({result.get('diff_percent', 0)}%) — tolerance {args.tolerance}")
            if result.get("diff_image"):
                print(f"  Diff image: {result['diff_image']}")
        sys.exit(0 if result["passed"] else 1)

    elif args.reference_dir and args.test_dir:
        results = compare_directories(args.reference_dir, args.test_dir,
                                      args.tolerance, args.diff_dir,
                                      args.channel_threshold, args.report)
        passed = sum(1 for r in results if r["passed"])
        failed = sum(1 for r in results if not r["passed"])
        print(f"Results: {passed} passed, {failed} failed, {len(results)} total")
        for r in results:
            if not r["passed"]:
                ref = Path(r.get("reference", "?")).name if r.get("reference") else "?"
                print(f"  FAIL: {ref} — {r['reason']}")
        sys.exit(0 if failed == 0 else 1)

    else:
        parser.error("Provide either --reference/--test or --reference-dir/--test-dir")


if __name__ == "__main__":
    main()
