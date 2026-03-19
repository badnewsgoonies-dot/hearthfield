#!/usr/bin/env python3
import argparse
from pathlib import Path


ARTIFACTS_DIR = Path("artifacts")


def extract_analysis(text: str) -> str:
    marker = "## Analysis"
    if marker in text:
        return text.split(marker, 1)[1].strip()
    return text.strip()


def collect_inputs(paths):
    if paths:
        return [Path(p) for p in paths]
    return sorted(ARTIFACTS_DIR.glob("vision_lane_seg*.md"))


def main() -> int:
    parser = argparse.ArgumentParser(description="Synthesize vision lane segment artifacts.")
    parser.add_argument("--inputs", nargs="*", help="Input files (default: artifacts/vision_lane_seg*.md)")
    parser.add_argument("--out", default="artifacts/vision_lane_synthesis.md", help="Output markdown file")
    args = parser.parse_args()

    inputs = collect_inputs(args.inputs)
    if not inputs:
        print("No input files found.")
        return 1

    out_path = Path(args.out)
    out_path.parent.mkdir(parents=True, exist_ok=True)

    lines = [
        "# Vision Lane Synthesis",
        "",
        "Sources:",
    ]
    for path in inputs:
        lines.append(f"- {path}")
    lines.append("")

    for path in inputs:
        if not path.exists():
            lines.append(f"## {path.name}")
            lines.append("")
            lines.append("Missing input file.")
            lines.append("")
            continue
        text = path.read_text(encoding="utf-8")
        analysis = extract_analysis(text)
        lines.append(f"## {path.name}")
        lines.append("")
        lines.append(analysis)
        lines.append("")

    out_path.write_text("\n".join(lines).rstrip() + "\n", encoding="utf-8")
    print(f"Wrote {out_path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
