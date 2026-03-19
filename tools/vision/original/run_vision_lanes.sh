#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage: scripts/run_vision_lanes.sh --url <youtube_url> [--segments N] [--segment-seconds N] [--state-out PATH] [--prefix NAME]

Defaults:
  --segments 4
  --segment-seconds 30
  --state-out artifacts/vision_lanes_state_<timestamp>.json
  --prefix vision_lane

Example:
  scripts/run_vision_lanes.sh --url "https://www.youtube.com/watch?v=aqz-KE-bpKQ" --segments 4 --segment-seconds 30 --prefix bunny
EOF
}

URL=""
SEGMENTS="4"
SEGMENT_SECONDS="30"
STATE_OUT=""
PREFIX="vision_lane"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --url)
      URL="${2:-}"
      shift 2
      ;;
    --segments)
      SEGMENTS="${2:-}"
      shift 2
      ;;
    --segment-seconds)
      SEGMENT_SECONDS="${2:-}"
      shift 2
      ;;
    --state-out)
      STATE_OUT="${2:-}"
      shift 2
      ;;
    --prefix)
      PREFIX="${2:-}"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "Unknown arg: $1" >&2
      usage
      exit 1
      ;;
  esac
done

if [[ -z "$URL" ]]; then
  echo "Missing --url" >&2
  usage
  exit 1
fi

if [[ -z "$STATE_OUT" ]]; then
  ts="$(date +%Y%m%d-%H%M%S)"
  STATE_OUT="artifacts/vision_lanes_state_${ts}.json"
fi

export VISION_LANES_URL="$URL"
export VISION_LANES_SEGMENTS="$SEGMENTS"
export VISION_LANES_SEGMENT_SECONDS="$SEGMENT_SECONDS"
export VISION_LANES_STATE_OUT="$STATE_OUT"
export VISION_LANES_PREFIX="$PREFIX"

python3 - <<'PY'
import json
import os
import shlex
from pathlib import Path

url = os.environ["VISION_LANES_URL"]
segments = int(os.environ["VISION_LANES_SEGMENTS"])
segment_seconds = int(os.environ["VISION_LANES_SEGMENT_SECONDS"])
state_out = Path(os.environ["VISION_LANES_STATE_OUT"])
prefix = os.environ["VISION_LANES_PREFIX"]

allowed_files = [
    f"artifacts/{prefix}_seg{i}.md" for i in range(segments)
]

lane_commands = []
for idx in range(segments):
    seg_index = idx + 1  # 1-based for watch_vision.py
    out_file = f"artifacts/{prefix}_seg{idx}.md"
    log_file = f"/tmp/{prefix}_seg{idx}.log"
    cmd = (
        "python3 tools/watch_vision.py "
        f"--url {shlex.quote(url)} "
        f"--segments {segments} "
        f"--segment-seconds {segment_seconds} "
        f"--segment-index {seg_index} "
        f"--out {out_file} "
        f"> {log_file} 2>&1"
    )
    lane_commands.append(cmd)

goal = f"Run watch_vision in {segments} parallel lane segments ({segment_seconds}s each) for {url} and synthesize results."

state = {
    "goal": goal,
    "repo_root": str(Path.cwd()),
    "session_id": f"vision-lanes-{state_out.stem}",
    "phases": [
        {
            "phase_id": "phase-1-vision-lanes",
            "intent": f"Run watch_vision for {segments} sequential segments in parallel lanes.",
            "allowed_files": allowed_files,
            "definition_of_done": [f"{p} exists and is non-empty" for p in allowed_files],
            "estimated_lanes": segments,
            "estimated_rounds": 1,
            "depends_on": [],
            "read_only_files": [
                "tools/watch_vision.py",
                "tools/synthesize_vision_segments.py",
            ],
            "lane_commands": lane_commands,
            "status": "pending",
            "batches_run": 0,
            "batch_results": [],
        },
        {
            "phase_id": "phase-2-synthesize",
            "intent": "Synthesize the lane outputs into a single summary file.",
            "allowed_files": [
                f"artifacts/{prefix}_synthesis.md"
            ],
            "definition_of_done": [
                f"artifacts/{prefix}_synthesis.md exists and is non-empty"
            ],
            "estimated_lanes": 1,
            "estimated_rounds": 1,
            "depends_on": [
                "phase-1-vision-lanes"
            ],
            "read_only_files": [
                "tools/synthesize_vision_segments.py",
            ] + allowed_files,
            "lane_commands": [
                "python3 tools/synthesize_vision_segments.py "
                + " ".join(allowed_files)
                + f" --out artifacts/{prefix}_synthesis.md"
            ],
            "status": "pending",
            "batches_run": 0,
            "batch_results": [],
        }
    ],
    "current_phase_idx": 0,
    "status": "pending",
    "started_at": None,
    "updated_at": None,
    "total_batches": 0,
    "lessons": [],
    "rationale": "Deterministic direct lane commands for vision runs and synthesis.",
    "conversation_history": [],
    "accumulated_knowledge": []
}

state_out.parent.mkdir(parents=True, exist_ok=True)
state_out.write_text(json.dumps(state, indent=2))
print(state_out)
PY

STATE_PATH="$(python3 - <<'PY'
import os
print(os.environ["VISION_LANES_STATE_OUT"])
PY
)"

GOAL="$(python3 - <<'PY'
import json
import os
from pathlib import Path
state = json.loads(Path(os.environ["VISION_LANES_STATE_OUT"]).read_text())
print(state["goal"])
PY
)"

export LANE_PARALLELISM="${LANE_PARALLELISM:-$SEGMENTS}"
python3 agents/strategic_orchestrator.py --goal "$GOAL" --repo "$(pwd)" --state-file "$STATE_PATH"
