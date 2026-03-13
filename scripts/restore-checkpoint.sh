#!/usr/bin/env bash
set -euo pipefail

usage() {
    cat <<'EOF'
Usage:
  bash scripts/restore-checkpoint.sh \
    [--manifest <path> | --label <label>] \
    [--checkpoint-ledger <path>] \
    [--force] \
    [--resume-interactive] \
    [--resume-exec <prompt>]

Defaults:
  --checkpoint-ledger status/foreman/checkpoints.yaml

Behavior:
  - verifies the checkpoint manifest against the current worktree and session store
  - prints restore commands by default
  - with --resume-interactive, launches `codex resume`
  - with --resume-exec, launches `codex exec resume "<prompt>"`
  - with --force, prints mismatches but still allows resume
EOF
}

die() {
    printf 'restore-checkpoint: %s\n' "$*" >&2
    exit 1
}

sha256_file() {
    shasum -a 256 "$1" | awk '{print $1}'
}

sha256_text() {
    printf '%s' "$1" | shasum -a 256 | awk '{print $1}'
}

join_lines() {
    paste -sd ',' | sed 's/,/, /g'
}

MANIFEST_ARG=""
LABEL=""
CHECKPOINT_LEDGER_REL="status/foreman/checkpoints.yaml"
FORCE=0
RESUME_INTERACTIVE=0
RESUME_EXEC_PROMPT=""

while [[ $# -gt 0 ]]; do
    case "$1" in
        --manifest)
            MANIFEST_ARG="${2:-}"
            shift 2
            ;;
        --label)
            LABEL="${2:-}"
            shift 2
            ;;
        --checkpoint-ledger)
            CHECKPOINT_LEDGER_REL="${2:-}"
            shift 2
            ;;
        --force)
            FORCE=1
            shift
            ;;
        --resume-interactive)
            RESUME_INTERACTIVE=1
            shift
            ;;
        --resume-exec)
            RESUME_EXEC_PROMPT="${2:-}"
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

if [[ -z "$MANIFEST_ARG" && -z "$LABEL" ]]; then
    die "provide --manifest or --label"
fi

if [[ -n "$MANIFEST_ARG" && -n "$LABEL" ]]; then
    die "use only one of --manifest or --label"
fi

if [[ "$RESUME_INTERACTIVE" -eq 1 && -n "$RESUME_EXEC_PROMPT" ]]; then
    die "use only one of --resume-interactive or --resume-exec"
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
CHECKPOINT_LEDGER_ABS="$REPO_ROOT/$CHECKPOINT_LEDGER_REL"

resolve_manifest() {
    if [[ -n "$MANIFEST_ARG" ]]; then
        if [[ "$MANIFEST_ARG" = /* ]]; then
            printf '%s' "$MANIFEST_ARG"
        else
            printf '%s' "$REPO_ROOT/$MANIFEST_ARG"
        fi
        return
    fi

    [[ -f "$CHECKPOINT_LEDGER_ABS" ]] || die "checkpoint ledger not found: $CHECKPOINT_LEDGER_REL"
    python3 - "$CHECKPOINT_LEDGER_ABS" "$REPO_ROOT" "$LABEL" <<'PY'
import sys
from pathlib import Path

ledger = Path(sys.argv[1])
repo_root = Path(sys.argv[2])
label = sys.argv[3]
current = None
manifest = None

for raw in ledger.read_text().splitlines():
    line = raw.rstrip()
    if line.startswith("- label: "):
        current = line.split(": ", 1)[1]
        continue
    if current == label and line.strip().startswith("manifest: "):
        manifest = line.strip().split(": ", 1)[1]

if not manifest:
    sys.exit(1)

path = Path(manifest)
if not path.is_absolute():
    path = repo_root / path
print(path)
PY
}

MANIFEST_ABS="$(resolve_manifest)" || die "could not resolve manifest"
[[ -f "$MANIFEST_ABS" ]] || die "manifest not found: $MANIFEST_ABS"

read_manifest_vars() {
    python3 - "$MANIFEST_ABS" <<'PY'
import sys
from pathlib import Path

manifest = Path(sys.argv[1])
data = {}
for raw in manifest.read_text().splitlines():
    if not raw or raw.startswith("  - ") or raw.startswith("    ") or raw.startswith("#"):
        continue
    if ":" not in raw:
        continue
    key, value = raw.split(":", 1)
    data[key.strip()] = value.strip()

for key in [
    "label",
    "parent_session",
    "snapshot_session",
    "snapshot_session_file",
    "codex_home",
    "repo",
    "worktree",
    "branch",
    "head_commit",
    "ledger",
    "dispatch_state_hash",
    "agents_hash",
    "state_hash",
    "snapshot_session_hash",
    "tracked_status_hash",
    "tracked_diff_hash",
    "untracked_hash",
]:
    print(f"{key}={data.get(key, '')}")
PY
}

eval "$(read_manifest_vars)"

[[ -n "${snapshot_session:-}" ]] || die "manifest missing snapshot_session"
[[ -n "${snapshot_session_file:-}" ]] || die "manifest missing snapshot_session_file"
[[ -n "${repo:-}" ]] || die "manifest missing repo"
[[ -n "${worktree:-}" ]] || die "manifest missing worktree"

MANIFEST_REPO="$repo"
MANIFEST_WORKTREE="$worktree"
MANIFEST_BRANCH="$branch"
MANIFEST_HEAD="$head_commit"
MANIFEST_LEDGER_REL="$ledger"
MANIFEST_CODEX_HOME="${codex_home:-}"
MANIFEST_SESSION_FILE="$snapshot_session_file"

if [[ -z "$MANIFEST_CODEX_HOME" ]]; then
    MANIFEST_CODEX_HOME="$(python3 - "$MANIFEST_SESSION_FILE" <<'PY'
import sys
from pathlib import Path
path = Path(sys.argv[1]).resolve()
parts = list(path.parts)
idx = parts.index("sessions")
print(Path(*parts[:idx]))
PY
)"
fi

[[ -d "$MANIFEST_WORKTREE" ]] || die "worktree missing: $MANIFEST_WORKTREE"
[[ -d "$MANIFEST_CODEX_HOME" ]] || die "codex_home missing: $MANIFEST_CODEX_HOME"
[[ -f "$MANIFEST_SESSION_FILE" ]] || die "snapshot session file missing: $MANIFEST_SESSION_FILE"

MISMATCHES=()

CURRENT_TOPLEVEL="$(git -C "$MANIFEST_WORKTREE" rev-parse --show-toplevel)"
[[ "$CURRENT_TOPLEVEL" == "$MANIFEST_WORKTREE" ]] || MISMATCHES+=("worktree is no longer a git worktree root")

CURRENT_BRANCH="$(git -C "$MANIFEST_WORKTREE" rev-parse --abbrev-ref HEAD)"
[[ "$CURRENT_BRANCH" == "$MANIFEST_BRANCH" ]] || MISMATCHES+=("branch mismatch: current=$CURRENT_BRANCH manifest=$MANIFEST_BRANCH")

CURRENT_HEAD="$(git -C "$MANIFEST_WORKTREE" rev-parse --verify HEAD)"
[[ "$CURRENT_HEAD" == "$MANIFEST_HEAD" ]] || MISMATCHES+=("head commit mismatch: current=$CURRENT_HEAD manifest=$MANIFEST_HEAD")

CURRENT_TRACKED_STATUS="$(git -C "$MANIFEST_WORKTREE" status --short --untracked-files=all)"
CURRENT_TRACKED_STATUS_HASH="$(sha256_text "$CURRENT_TRACKED_STATUS")"
[[ "$CURRENT_TRACKED_STATUS_HASH" == "$tracked_status_hash" ]] || MISMATCHES+=("tracked status hash mismatch")

CURRENT_TRACKED_DIFF_HASH="$(git -C "$MANIFEST_WORKTREE" diff --binary HEAD | shasum -a 256 | awk '{print $1}')"
[[ "$CURRENT_TRACKED_DIFF_HASH" == "$tracked_diff_hash" ]] || MISMATCHES+=("tracked diff hash mismatch")

CURRENT_UNTRACKED_LIST="$(git -C "$MANIFEST_WORKTREE" status --short --untracked-files=all | awk '/^\?\?/ {sub(/^\?\? /, ""); print}' | sort)"
CURRENT_UNTRACKED_HASH="$(sha256_text "$CURRENT_UNTRACKED_LIST")"
[[ "$CURRENT_UNTRACKED_HASH" == "$untracked_hash" ]] || MISMATCHES+=("untracked hash mismatch")

WORKTREE_AGENTS="$MANIFEST_WORKTREE/AGENTS.md"
WORKTREE_STATE="$MANIFEST_WORKTREE/.memory/STATE.md"
WORKTREE_LEDGER="$MANIFEST_WORKTREE/$MANIFEST_LEDGER_REL"

[[ -f "$WORKTREE_AGENTS" ]] || MISMATCHES+=("AGENTS.md missing in worktree")
[[ -f "$WORKTREE_STATE" ]] || MISMATCHES+=(".memory/STATE.md missing in worktree")
[[ -f "$WORKTREE_LEDGER" ]] || MISMATCHES+=("$MANIFEST_LEDGER_REL missing in worktree")

if [[ -f "$WORKTREE_AGENTS" ]]; then
    [[ "$(sha256_file "$WORKTREE_AGENTS")" == "$agents_hash" ]] || MISMATCHES+=("AGENTS.md hash mismatch")
fi
if [[ -f "$WORKTREE_STATE" ]]; then
    [[ "$(sha256_file "$WORKTREE_STATE")" == "$state_hash" ]] || MISMATCHES+=(".memory/STATE.md hash mismatch")
fi
if [[ -f "$WORKTREE_LEDGER" ]]; then
    [[ "$(sha256_file "$WORKTREE_LEDGER")" == "$dispatch_state_hash" ]] || MISMATCHES+=("$MANIFEST_LEDGER_REL hash mismatch")
fi

[[ "$(sha256_file "$MANIFEST_SESSION_FILE")" == "$snapshot_session_hash" ]] || MISMATCHES+=("snapshot session file hash mismatch")

if [[ ${#MISMATCHES[@]} -gt 0 && "$FORCE" -ne 1 ]]; then
    printf 'restore-checkpoint: checkpoint verification failed:\n' >&2
    printf '  - %s\n' "${MISMATCHES[@]}" >&2
    exit 1
fi

printf 'checkpoint restore ok\n'
printf '  label: %s\n' "$label"
printf '  snapshot_session: %s\n' "$snapshot_session"
printf '  worktree: %s\n' "$MANIFEST_WORKTREE"
printf '  branch: %s\n' "$MANIFEST_BRANCH"
printf '  head_commit: %s\n' "$MANIFEST_HEAD"
printf '  codex_home: %s\n' "$MANIFEST_CODEX_HOME"
if [[ ${#MISMATCHES[@]} -gt 0 ]]; then
    printf '  mismatches (forced):\n'
    printf '    - %s\n' "${MISMATCHES[@]}"
fi
printf '  resume: cd %s && env CODEX_HOME=%s codex resume %s\n' "$MANIFEST_WORKTREE" "$MANIFEST_CODEX_HOME" "$snapshot_session"
printf '  exec_resume_template: cd %s && env CODEX_HOME=%s codex exec resume %s \"<prompt>\"\n' "$MANIFEST_WORKTREE" "$MANIFEST_CODEX_HOME" "$snapshot_session"

if [[ "$RESUME_INTERACTIVE" -eq 1 ]]; then
    cd "$MANIFEST_WORKTREE"
    exec env CODEX_HOME="$MANIFEST_CODEX_HOME" codex resume "$snapshot_session"
fi

if [[ -n "$RESUME_EXEC_PROMPT" ]]; then
    cd "$MANIFEST_WORKTREE"
    exec env CODEX_HOME="$MANIFEST_CODEX_HOME" codex exec resume "$snapshot_session" "$RESUME_EXEC_PROMPT"
fi
