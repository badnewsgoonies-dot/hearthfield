#!/usr/bin/env python3
import re
from pathlib import Path

ARTIFACTS_DIR = Path("artifacts")
MASTER_REPORT = ARTIFACTS_DIR / "MASTER_REPORT.md"

def extract_info(file_path):
    content = file_path.read_text()
    
    # Try to find game name from Analysis section
    game_name = "Unknown"
    name_match = re.search(r'analysis of (?:its core loop, UI elements, and "juice" (game feel)|its core loop, UI, and "juice"|its core loop, UI elements, and "juice"|its game design fundamentals) for \*\*?(.*?)\*\*?', content, re.IGNORECASE)
    if not name_match:
        name_match = re.search(r'breakdown of (?:the core loop, UI elements, and "juice" (game feel)|its core loop, UI elements, and "juice" (game feel)) for \*\*?(.*?)\*\*?', content, re.IGNORECASE)
    if not name_match:
        # Fallback to heading logic
        name_match = re.search(r'## Analysis\s+Based on the trailer for \*\*?(.*?)\*\*?', content, re.IGNORECASE)
    
    if name_match:
        game_name = name_match.group(1).strip("*")
    
    # Extract Core Loop
    core_loop = "Not found"
    loop_match = re.search(r'### 1. (?:The )?Core Loop(.*?)(?=### 2|$)', content, re.DOTALL | re.IGNORECASE)
    if loop_match:
        loop_text = loop_match.group(1).strip()
        # Summarize to first few lines or bullets
        summary = []
        for line in loop_text.splitlines():
            line = line.strip()
            if line.startswith(('*', '-')):
                summary.append(line.split(':')[0].strip('* -'))
        if summary:
            core_loop = " → ".join(summary)
        else:
            core_loop = loop_text.split('.')[0]
            
    # Extract UI
    ui = "Not found"
    ui_match = re.search(r'### 2. UI Elements(.*?)(?=### 3|$)', content, re.DOTALL | re.IGNORECASE)
    if ui_match:
        ui_text = ui_match.group(1).strip()
        items = []
        for line in ui_text.splitlines():
            line = line.strip()
            if line.startswith(('*', '-')):
                items.append(line.split(':')[0].strip('* -'))
        if items:
            ui = ", ".join(items[:5])
        else:
            ui = ui_text.split('.')[0]

    # Extract Juice
    juice = "Not found"
    juice_match = re.search(r'### 3. Game Feel (?:.*?)(juice)(.*?)(?=#|$)', content, re.DOTALL | re.IGNORECASE)
    if juice_match:
        juice_text = juice_match.group(2).strip()
        items = []
        for line in juice_text.splitlines():
            line = line.strip()
            if line.startswith(('*', '-')):
                items.append(line.split(':')[0].strip('* -'))
        if items:
            juice = ", ".join(items[:5])
        else:
            juice = juice_text.split('.')[0]

    return {
        "name": game_name,
        "loop": core_loop,
        "ui": ui,
        "juice": juice,
        "file": file_path.name
    }

def main():
    files = sorted(list(ARTIFACTS_DIR.glob("vision_analysis_*.md")), key=lambda x: int(re.search(r'(\d+)', x.name).group(1)))
    
    report_lines = [
        "# Vision Analysis Master Report",
        "",
        "This master report consolidates all artifacts matching artifacts/vision_analysis_*.md.",
        "",
        "Compiled summaries (Game Name | Core Loop | UI | Juice | Source File):",
        "",
        "| Game Name | Core Loop | UI | Juice | Source File |",
        "|-----------|-----------|----|-------|-------------|"
    ]
    
    for f in files:
        info = extract_info(f)
        report_lines.append(f"| {info['name']} | {info['loop']} | {info['ui']} | {info['juice']} | {info['file']} |")
        
    report_lines.extend([
        "",
        "# Notes",
        "- Each row summarizes Game Name, an abbreviated Core Loop, key visible UI elements, and concise \"juice\" (game-feel) observations derived from the source artifact files.",
        f"- Source files: artifacts/vision_analysis_1..{len(files)}.md",
        "",
        "# Next steps",
        "- If additional vision_analysis_*.md artifacts are added, re-run this synthesis to append rows."
    ])
    
    MASTER_REPORT.write_text("\n".join(report_lines))
    print(f"Generated report with {len(files)} entries.")

if __name__ == "__main__":
    main()
