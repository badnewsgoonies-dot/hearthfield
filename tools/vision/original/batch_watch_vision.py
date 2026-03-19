#!/usr/bin/env python3
"""
Batch processor for watch_vision.py.
Reads URLs from artifacts/video_targets.txt and runs analysis for each.
Simplifies URL handling by delegating robustness to watch_vision.py.
"""
import os
import subprocess
import sys
import re
import argparse
from pathlib import Path

TARGETS_FILE = Path("artifacts/video_targets.txt")
TOOL_SCRIPT = Path("tools/watch_vision.py")
ARTIFACTS_DIR = Path("artifacts")

def get_next_index():
    """Finds the next available index for vision_analysis_N.md"""
    existing_files = list(ARTIFACTS_DIR.glob("vision_analysis_*.md"))
    max_idx = 0
    for f in existing_files:
        match = re.search(r'vision_analysis_(\d+)\.md', f.name)
        if match:
            idx = int(match.group(1))
            if idx > max_idx:
                max_idx = idx
    return max_idx + 1

def main():
    global TARGETS_FILE
    
    parser = argparse.ArgumentParser()
    parser.add_argument("--targets", help="Path to targets file")
    parser.add_argument("--skip", type=int, default=0, help="Number of URLs to skip")
    parser.add_argument("--limit", type=int, default=0, help="Max URLs to process (0=all)")
    args = parser.parse_args()
    
    if args.targets:
        TARGETS_FILE = Path(args.targets)

    if not TARGETS_FILE.exists():
        print(f"Error: Targets file not found at {TARGETS_FILE}")
        sys.exit(1)

    urls = [line.strip() for line in TARGETS_FILE.read_text().splitlines() 
            if line.strip() and not line.strip().startswith("#")]

    if not urls:
        print("No valid URLs found in targets file.")
        sys.exit(0)
        
    # Apply Skip/Limit
    if args.skip > 0:
        urls = urls[args.skip:]
    if args.limit > 0:
        urls = urls[:args.limit]

    start_index = get_next_index()
    print(f"Found {len(urls)} videos to process. Starting file index at: {start_index}")

    for i, url in enumerate(urls, start_index):
        print(f"\n[{i-start_index+1}/{len(urls)}] Processing (File #{i}): {url}")
        
        # Pass the ORIGINAL URL directly to the tool.
        # The tool now handles resolution, retries, and errors internally.
        output_filename = f"vision_analysis_{i}.md"
        
        cmd = [
            sys.executable, str(TOOL_SCRIPT),
            url,  # Positional argument supported by new tool
            "--prompt", "Study this indie gameplay trailer. Identify the core loop, UI elements, and game feel (juice).",
            "--out", output_filename
        ]
        
        try:
            # We don't check=True because the tool now exits 0 even on failure (writing a failure artifact)
            subprocess.run(cmd)
            print(f"Finished processing {output_filename}")
                
        except Exception as e:
            print(f"Unexpected batch error for video {i}: {e}")

if __name__ == "__main__":
    main()