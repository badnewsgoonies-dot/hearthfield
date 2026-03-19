#!/usr/bin/env python3
import re
from pathlib import Path

ARTIFACTS_DIR = Path("artifacts")
MASTER_REPORT = ARTIFACTS_DIR / "MASTER_REPORT_100.md"

def extract_info(file_path):
    content = file_path.read_text()
    
    # Robust Regex Extraction
    game_name = "Unknown"
    # Look for name in various headers
    name_match = re.search(r'# Vision Analysis: (.*)', content)
    if not name_match:
        name_match = re.search(r'analysis of (?:.*? )for \*\*?(.*?)\*\*?', content, re.IGNORECASE)
    if not name_match:
        name_match = re.search(r'trailer for \*\*?(.*?)\*\*?', content, re.IGNORECASE)
    
    if name_match:
        game_name = name_match.group(1).strip("* ").split("—")[0].strip()
    
    # Check for video_title in the bullet list
    if game_name == "Unknown":
        title_match = re.search(r'- \*\*video_title\*\*: (.*)', content)
        if title_match:
            game_name = title_match.group(1).strip()
        elif "hls_264_master" in content:
            # Try to get it from the source URL filename if possible
            game_name = "Steam Indie #" + str(re.search(r'(\d+)', file_path.name).group(1))
    
    core_loop = "Not found"
    loop_match = re.search(r'Core Loop.*?\n(.*?)(?=###|$)', content, re.DOTALL | re.IGNORECASE)
    if loop_match:
        core_loop = loop_match.group(1).strip().replace('\n', ' ').replace('*', '')[:100] + "..."

    juice = "Not found"
    juice_match = re.search(r'(?:Juice|Game Feel).*?\n(.*?)(?=###|$)', content, re.DOTALL | re.IGNORECASE)
    if juice_match:
        juice = juice_match.group(1).strip().replace('\n', ' ').replace('*', '')[:100] + "..."

    return {
        "name": game_name,
        "loop": core_loop,
        "juice": juice,
        "file": file_path.name
    }

def main():
    files = sorted(list(ARTIFACTS_DIR.glob("vision_analysis_*.md")), key=lambda x: int(re.search(r'(\d+)', x.name).group(1) if re.search(r'(\d+)', x.name) else 0))
    
    report_lines = [
        "# The Indie Game Design Bible (100 Games)",
        "",
        f"Analysis of {len(files)} top-tier indie games.",
        "",
        "| Game Name | Core Loop (Brief) | Juice / Feel | Source |",
        "|-----------|-------------------|--------------|--------|"
    ]
    
    print(f"Synthesizing {len(files)} reports...")
    
    for f in files:
        if "debug" in f.name: continue
        try:
            info = extract_info(f)
            # Escape pipes
            name = info['name'].replace('|', '-')
            loop = info['loop'].replace('|', '-')
            juice = info['juice'].replace('|', '-')
            
            report_lines.append(f"| {name} | {loop} | {juice} | {info['file']} |")
        except Exception as e:
            print(f"Skipping {f.name}: {e}")
            
    MASTER_REPORT.write_text("\n".join(report_lines))
    print(f"Bible written to {MASTER_REPORT}")

if __name__ == "__main__":
    main()
