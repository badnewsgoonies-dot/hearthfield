#!/usr/bin/env bash
set -euo pipefail

usage() {
    cat <<'EOF'
Usage:
  bash scripts/launch-lane.sh \
    --lane-id <lane-id> \
    --branch <branch-name> \
    --worktree <path> \
    --codex-home <path> \
    --objective <path> \
    [--base <ref>] \
    [--output-last <path>] \
    [--run-log <path>]

Creates or reuses an isolated lane environment, syncs the current launch state,
and starts a Codex lane run non-interactively.

Defaults:
  --base        HEAD
  --output-last <worktree>.last
  --run-log     <worktree>.run.jsonl
EOF
}

die() {
    printf 'launch-lane: %s\n' "$*" >&2
    exit 1
}

build_shell_command() {
    local out=""
    local arg
    for arg in "$@"; do
        printf -v out '%s%q ' "$out" "$arg"
    done
    printf '%s' "${out% }"
}

LANE_ID=""
BRANCH_NAME=""
WORKTREE=""
CODEX_HOME_DIR=""
OBJECTIVE=""
BASE_REF="HEAD"
OUTPUT_LAST=""
RUN_LOG=""

while [[ $# -gt 0 ]]; do
    case "$1" in
        --lane-id)
            LANE_ID="${2:-}"
            shift 2
            ;;
        --branch)
            BRANCH_NAME="${2:-}"
            shift 2
            ;;
        --worktree)
            WORKTREE="${2:-}"
            shift 2
            ;;
        --codex-home)
            CODEX_HOME_DIR="${2:-}"
            shift 2
            ;;
        --objective)
            OBJECTIVE="${2:-}"
            shift 2
            ;;
        --base)
            BASE_REF="${2:-}"
            shift 2
            ;;
        --output-last)
            OUTPUT_LAST="${2:-}"
            shift 2
            ;;
        --run-log)
            RUN_LOG="${2:-}"
            shift 2
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            die "unknown argument: $1"
            ;;
    esac
done

[[ -n "$LANE_ID" ]] || die "--lane-id is required"
[[ -n "$BRANCH_NAME" ]] || die "--branch is required"
[[ -n "$WORKTREE" ]] || die "--worktree is required"
[[ -n "$CODEX_HOME_DIR" ]] || die "--codex-home is required"
[[ -n "$OBJECTIVE" ]] || die "--objective is required"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
WORKTREE_ABS="$WORKTREE"
CODEX_HOME_ABS="$CODEX_HOME_DIR"
OBJECTIVE_ABS="$OBJECTIVE"

if [[ "$WORKTREE_ABS" != /* ]]; then
    WORKTREE_ABS="$REPO_ROOT/$WORKTREE_ABS"
fi
if [[ "$CODEX_HOME_ABS" != /* ]]; then
    CODEX_HOME_ABS="$REPO_ROOT/$CODEX_HOME_ABS"
fi
if [[ "$OBJECTIVE_ABS" != /* ]]; then
    OBJECTIVE_ABS="$REPO_ROOT/$OBJECTIVE_ABS"
fi

[[ -f "$OBJECTIVE_ABS" ]] || die "objective not found: $OBJECTIVE_ABS"

if [[ -z "$OUTPUT_LAST" ]]; then
    OUTPUT_LAST="$CODEX_HOME_ABS/run/${LANE_ID}.last"
fi
if [[ -z "$RUN_LOG" ]]; then
    RUN_LOG="$CODEX_HOME_ABS/run/${LANE_ID}.run.jsonl"
fi

mkdir -p "$(dirname "$WORKTREE_ABS")" "$CODEX_HOME_ABS"

if [[ ! -d "$WORKTREE_ABS/.git" && ! -f "$WORKTREE_ABS/.git" ]]; then
    git -C "$REPO_ROOT" worktree add "$WORKTREE_ABS" -b "$BRANCH_NAME" "$BASE_REF" >/dev/null
fi

mkdir -p "$CODEX_HOME_ABS"
cp "$HOME/.codex/auth.json" "$CODEX_HOME_ABS/" 2>/dev/null || die "missing ~/.codex/auth.json"
cp "$HOME/.codex/config.toml" "$CODEX_HOME_ABS/" 2>/dev/null || die "missing ~/.codex/config.toml"
cp "$HOME/.codex/version.json" "$CODEX_HOME_ABS/" 2>/dev/null || true
mkdir -p "$CODEX_HOME_ABS/skills"
mkdir -p "$CODEX_HOME_ABS/run"
rsync -a "$HOME/.codex/skills/" "$CODEX_HOME_ABS/skills/" >/dev/null

rsync -a "$REPO_ROOT/docs/" "$WORKTREE_ABS/docs/" >/dev/null
rsync -a "$REPO_ROOT/objectives/" "$WORKTREE_ABS/objectives/" >/dev/null
rsync -a "$REPO_ROOT/status/" "$WORKTREE_ABS/status/" >/dev/null
cp "$REPO_ROOT/AGENTS.md" "$WORKTREE_ABS/AGENTS.md"
cp "$REPO_ROOT/objectives/TEMPLATE.md" "$WORKTREE_ABS/objectives/TEMPLATE.md" 2>/dev/null || true
cp "$REPO_ROOT/tests/headless.rs" "$WORKTREE_ABS/tests/headless.rs" 2>/dev/null || true

RUN_CMD="$(build_shell_command env "CODEX_HOME=$CODEX_HOME_ABS" codex exec --dangerously-bypass-approvals-and-sandbox --json -o "$OUTPUT_LAST" -C "$WORKTREE_ABS" -)"

{
    setsid bash -lc "$RUN_CMD < $(printf '%q' "$OBJECTIVE_ABS") > $(printf '%q' "$RUN_LOG") 2>&1" >/dev/null 2>&1 &
    printf '%s' "$!"
} | {
    IFS= read -r pid || true
    [[ -n "$pid" ]] || die "failed to capture launched lane pid"
    pgid="$(ps -o pgid= -p "$pid" | tr -d ' ' || true)"
    printf 'lane launched\n'
    printf '  lane_id: %s\n' "$LANE_ID"
    printf '  pid: %s\n' "$pid"
    printf '  pgid: %s\n' "${pgid:-unknown}"
    printf '  worktree: %s\n' "$WORKTREE_ABS"
    printf '  codex_home: %s\n' "$CODEX_HOME_ABS"
    printf '  objective: %s\n' "$OBJECTIVE_ABS"
    printf '  output_last: %s\n' "$OUTPUT_LAST"
    printf '  run_log: %s\n' "$RUN_LOG"
}
