#!/usr/bin/env python3
"""
reference_manager.py — Manage Reference Screenshots

Accept new baselines, diff against existing, prune stale references.
Works with the screenshot_compare.py tool for the actual pixel diffing.

Usage:
  # Accept a test screenshot as the new reference
  python3 reference_manager.py accept --test screenshots/test/farm.png --ref-dir screenshots/ref/

  # Accept all test screenshots that have no existing reference
  python3 reference_manager.py accept-new --test-dir screenshots/test/ --ref-dir screenshots/ref/

  # Diff all test screenshots against references
  python3 reference_manager.py diff --test-dir screenshots/test/ --ref-dir screenshots/ref/

  # List stale references (no matching test file)
  python3 reference_manager.py stale --test-dir screenshots/test/ --ref-dir screenshots/ref/

  # Prune stale references
  python3 reference_manager.py prune --test-dir screenshots/test/ --ref-dir screenshots/ref/

  # Show status summary
  python3 reference_manager.py status --test-dir screenshots/test/ --ref-dir screenshots/ref/
"""
import argparse
import json
import shutil
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from screenshot_compare import compare_images, compare_directories


def cmd_accept(args):
    """Accept a test screenshot as the new reference baseline."""
    test = Path(args.test)
    ref_dir = Path(args.ref_dir)
    ref_dir.mkdir(parents=True, exist_ok=True)

    dest = ref_dir / test.name
    shutil.copy2(test, dest)
    print(f"Accepted: {test.name} → {dest}")


def cmd_accept_new(args):
    """Accept all test screenshots that have no existing reference."""
    test_dir = Path(args.test_dir)
    ref_dir = Path(args.ref_dir)
    ref_dir.mkdir(parents=True, exist_ok=True)

    existing = {f.name for f in ref_dir.glob("*.png")}
    accepted = 0

    for test_file in sorted(test_dir.glob("*.png")):
        if test_file.name not in existing:
            dest = ref_dir / test_file.name
            shutil.copy2(test_file, dest)
            print(f"  New: {test_file.name}")
            accepted += 1

    print(f"Accepted {accepted} new reference(s)")


def cmd_accept_all(args):
    """Accept ALL test screenshots as references, overwriting existing."""
    test_dir = Path(args.test_dir)
    ref_dir = Path(args.ref_dir)
    ref_dir.mkdir(parents=True, exist_ok=True)

    count = 0
    for test_file in sorted(test_dir.glob("*.png")):
        dest = ref_dir / test_file.name
        existed = dest.exists()
        shutil.copy2(test_file, dest)
        label = "Updated" if existed else "New"
        print(f"  {label}: {test_file.name}")
        count += 1

    print(f"Accepted {count} reference(s)")


def cmd_diff(args):
    """Diff all test screenshots against references."""
    results = compare_directories(
        args.ref_dir, args.test_dir,
        tolerance=args.tolerance,
        diff_dir=args.diff_dir,
        report_path=args.report,
    )

    passed = sum(1 for r in results if r["passed"])
    failed = sum(1 for r in results if not r["passed"])
    print(f"Diff: {passed} match, {failed} differ, {len(results)} total")

    for r in results:
        if not r["passed"]:
            ref_name = Path(r.get("reference", "?")).name if r.get("reference") else "?"
            print(f"  DIFF: {ref_name} — {r['reason']} ({r.get('diff_pixels', '?')} pixels)")

    return 0 if failed == 0 else 1


def cmd_stale(args):
    """List reference files with no matching test screenshot."""
    ref_dir = Path(args.ref_dir)
    test_dir = Path(args.test_dir)

    test_names = {f.name for f in test_dir.glob("*.png")}
    stale = []

    for ref_file in sorted(ref_dir.glob("*.png")):
        if ref_file.name not in test_names:
            stale.append(ref_file)
            print(f"  Stale: {ref_file.name}")

    print(f"{len(stale)} stale reference(s)")
    return stale


def cmd_prune(args):
    """Remove stale references."""
    stale = cmd_stale(args)
    if not stale:
        return

    if not args.yes:
        answer = input(f"Delete {len(stale)} stale reference(s)? [y/N] ")
        if answer.lower() != "y":
            print("Aborted.")
            return

    for f in stale:
        f.unlink()
        print(f"  Deleted: {f.name}")

    print(f"Pruned {len(stale)} reference(s)")


def cmd_status(args):
    """Show summary status of references vs test screenshots."""
    ref_dir = Path(args.ref_dir)
    test_dir = Path(args.test_dir)

    ref_files = {f.name for f in ref_dir.glob("*.png")} if ref_dir.exists() else set()
    test_files = {f.name for f in test_dir.glob("*.png")} if test_dir.exists() else set()

    matched = ref_files & test_files
    stale = ref_files - test_files
    new = test_files - ref_files

    print(f"References: {len(ref_files)}  |  Test screenshots: {len(test_files)}")
    print(f"  Matched: {len(matched)}")
    print(f"  New (no reference): {len(new)}")
    print(f"  Stale (no test): {len(stale)}")

    if new:
        print(f"\nNew screenshots needing references:")
        for name in sorted(new):
            print(f"  + {name}")

    if stale:
        print(f"\nStale references:")
        for name in sorted(stale):
            print(f"  - {name}")


def main():
    parser = argparse.ArgumentParser(description="Reference screenshot manager")
    sub = parser.add_subparsers(dest="command")

    # accept
    p = sub.add_parser("accept", help="Accept one test screenshot as reference")
    p.add_argument("--test", required=True, help="Test screenshot to accept")
    p.add_argument("--ref-dir", required=True, help="Reference directory")

    # accept-new
    p = sub.add_parser("accept-new", help="Accept test screenshots with no existing reference")
    p.add_argument("--test-dir", required=True)
    p.add_argument("--ref-dir", required=True)

    # accept-all
    p = sub.add_parser("accept-all", help="Accept ALL test screenshots as references")
    p.add_argument("--test-dir", required=True)
    p.add_argument("--ref-dir", required=True)

    # diff
    p = sub.add_parser("diff", help="Diff test screenshots against references")
    p.add_argument("--test-dir", required=True)
    p.add_argument("--ref-dir", required=True)
    p.add_argument("--diff-dir", help="Output diff images directory")
    p.add_argument("--report", help="JSON report output")
    p.add_argument("--tolerance", type=int, default=0)

    # stale
    p = sub.add_parser("stale", help="List stale references")
    p.add_argument("--test-dir", required=True)
    p.add_argument("--ref-dir", required=True)

    # prune
    p = sub.add_parser("prune", help="Delete stale references")
    p.add_argument("--test-dir", required=True)
    p.add_argument("--ref-dir", required=True)
    p.add_argument("--yes", "-y", action="store_true", help="Skip confirmation")

    # status
    p = sub.add_parser("status", help="Show reference vs test status")
    p.add_argument("--test-dir", required=True)
    p.add_argument("--ref-dir", required=True)

    args = parser.parse_args()

    if not args.command:
        parser.print_help()
        sys.exit(1)

    commands = {
        "accept": cmd_accept,
        "accept-new": cmd_accept_new,
        "accept-all": cmd_accept_all,
        "diff": cmd_diff,
        "stale": cmd_stale,
        "prune": cmd_prune,
        "status": cmd_status,
    }

    fn = commands.get(args.command)
    if fn:
        result = fn(args)
        if isinstance(result, int):
            sys.exit(result)


if __name__ == "__main__":
    main()
