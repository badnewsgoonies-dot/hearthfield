#!/usr/bin/env bash
set -euo pipefail

usage() {
    cat <<'EOF'
Usage:
  bash scripts/checkpoint-state.sh \
    --label <name> \
    --session <session-id> \
    --worktree <path> \
    [--allow-prefix <path>]... \
    [--strict-untracked] \
    [--dry-run] \
    [--ledger <path>] \
    [--codex-home <path>] \
    [--manifest-dir <path>] \
    [--checkpoint-ledger <path>]

Creates a composite orchestration checkpoint:
  1. forks the control-plane session in the exact worktree
  2. verifies the fork landed as a snapshot branch point
  3. records filesystem + ledger state in a manifest
  4. appends the checkpoint to a durable checkpoint ledger

Defaults:
  --allow-prefix      optional repeatable prefix used to define allowed lane diff state
  --strict-untracked  treat untracked files outside the allowlist as fatal
  --dry-run           run preflight only, do not fork or write outputs
  --ledger            status/foreman/dispatch-state.yaml
  --codex-home        \$CODEX_HOME or ~/.codex
  --manifest-dir      status/checkpoints
  --checkpoint-ledger status/foreman/checkpoints.yaml
EOF
}

die() {
    printf 'checkpoint-state: %s\n' "$*" >&2
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

sanitize_label() {
    printf '%s' "$1" | tr '[:upper:]' '[:lower:]' | tr -cs 'a-z0-9._-' '-'
}

build_shell_command() {
    local out=""
    local arg
    for arg in "$@"; do
        printf -v out '%s%q ' "$out" "$arg"
    done
    printf '%s' "${out% }"
}

matches_allow_prefix() {
    local path="$1"
    local prefix
    local norm
    if [[ ${#ALLOW_PREFIXES[@]} -eq 0 ]]; then
        return 0
    fi
    for prefix in "${ALLOW_PREFIXES[@]}"; do
        norm="${prefix%/}"
        if [[ "$path" == "$norm" || "$path" == "$norm/"* ]]; then
            return 0
        fi
    done
    return 1
}

filter_status_list() {
    local mode="$1"
    local line
    while IFS= read -r line; do
        [[ -n "$line" ]] || continue
        if matches_allow_prefix "$line"; then
            [[ "$mode" == "allowed" ]] && printf '%s\n' "$line"
        else
            [[ "$mode" == "outside" ]] && printf '%s\n' "$line"
        fi
    done
    return 0
}

LABEL=""
SESSION_ID=""
WORKTREE=""
LEDGER_REL="status/foreman/dispatch-state.yaml"
CODEX_HOME_DIR="${CODEX_HOME:-$HOME/.codex}"
MANIFEST_DIR_REL="status/checkpoints"
CHECKPOINT_LEDGER_REL="status/foreman/checkpoints.yaml"
ALLOW_PREFIXES=()
DRY_RUN=0
STRICT_UNTRACKED=0

while [[ $# -gt 0 ]]; do
    case "$1" in
        --label)
            LABEL="${2:-}"
            shift 2
            ;;
        --session)
            SESSION_ID="${2:-}"
            shift 2
            ;;
        --worktree)
            WORKTREE="${2:-}"
            shift 2
            ;;
        --allow-prefix)
            ALLOW_PREFIXES+=("${2:-}")
            shift 2
            ;;
        --strict-untracked)
            STRICT_UNTRACKED=1
            shift
            ;;
        --dry-run)
            DRY_RUN=1
            shift
            ;;
        --ledger)
            LEDGER_REL="${2:-}"
            shift 2
            ;;
        --codex-home)
            CODEX_HOME_DIR="${2:-}"
            shift 2
            ;;
        --manifest-dir)
            MANIFEST_DIR_REL="${2:-}"
            shift 2
            ;;
        --checkpoint-ledger)
            CHECKPOINT_LEDGER_REL="${2:-}"
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

[[ -n "$LABEL" ]] || die "--label is required"
[[ -n "$SESSION_ID" ]] || die "--session is required"
[[ -n "$WORKTREE" ]] || die "--worktree is required"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
WORKTREE_ABS="$(cd "$WORKTREE" && pwd)"
CODEX_HOME_ABS="$(cd "$CODEX_HOME_DIR" && pwd)"
LEDGER_ABS="$REPO_ROOT/$LEDGER_REL"
MANIFEST_DIR_ABS="$REPO_ROOT/$MANIFEST_DIR_REL"
CHECKPOINT_LEDGER_ABS="$REPO_ROOT/$CHECKPOINT_LEDGER_REL"
AGENTS_ABS="$REPO_ROOT/AGENTS.md"
STATE_ABS="$REPO_ROOT/.memory/STATE.md"
LOCKFILE_ABS="${CHECKPOINT_LEDGER_ABS}.lock"

[[ -d "$WORKTREE_ABS" ]] || die "worktree does not exist: $WORKTREE"
[[ -d "$CODEX_HOME_ABS" ]] || die "CODEX_HOME does not exist: $CODEX_HOME_DIR"
[[ -f "$LEDGER_ABS" ]] || die "ledger file does not exist: $LEDGER_REL"
[[ -f "$AGENTS_ABS" ]] || die "AGENTS.md missing at repo root"
[[ -f "$STATE_ABS" ]] || die ".memory/STATE.md missing at repo root"
[[ -d "$CODEX_HOME_ABS/sessions" ]] || die "CODEX_HOME has no sessions directory: $CODEX_HOME_ABS"

WORKTREE_AGENTS_ABS="$WORKTREE_ABS/AGENTS.md"
WORKTREE_STATE_ABS="$WORKTREE_ABS/.memory/STATE.md"
WORKTREE_LEDGER_ABS="$WORKTREE_ABS/$LEDGER_REL"

MAIN_COMMON_DIR="$(git -C "$REPO_ROOT" rev-parse --git-common-dir)"
WORKTREE_COMMON_DIR="$(git -C "$WORKTREE_ABS" rev-parse --git-common-dir)"
if [[ "$MAIN_COMMON_DIR" = /* ]]; then
    MAIN_COMMON_DIR="$(cd "$MAIN_COMMON_DIR" && pwd)"
else
    MAIN_COMMON_DIR="$(cd "$REPO_ROOT/$MAIN_COMMON_DIR" && pwd)"
fi
if [[ "$WORKTREE_COMMON_DIR" = /* ]]; then
    WORKTREE_COMMON_DIR="$(cd "$WORKTREE_COMMON_DIR" && pwd)"
else
    WORKTREE_COMMON_DIR="$(cd "$WORKTREE_ABS/$WORKTREE_COMMON_DIR" && pwd)"
fi
[[ "$MAIN_COMMON_DIR" == "$WORKTREE_COMMON_DIR" ]] || die "worktree is not attached to the same git common dir as the repo root"

WORKTREE_TOPLEVEL="$(git -C "$WORKTREE_ABS" rev-parse --show-toplevel)"
[[ "$WORKTREE_TOPLEVEL" == "$WORKTREE_ABS" ]] || die "worktree path is not a git worktree root: $WORKTREE_ABS"

HEAD_COMMIT="$(git -C "$WORKTREE_ABS" rev-parse --verify HEAD)"
BRANCH_NAME="$(git -C "$WORKTREE_ABS" rev-parse --abbrev-ref HEAD)"

SESSION_FILE="$(find "$CODEX_HOME_ABS/sessions" -type f -name "*${SESSION_ID}*.jsonl" | sort | tail -n 1)"
[[ -n "$SESSION_FILE" ]] || die "session id not found in CODEX_HOME: $SESSION_ID"

MERGE_CONFLICTS="$(git -C "$WORKTREE_ABS" diff --name-only --diff-filter=U)"
[[ -z "$MERGE_CONFLICTS" ]] || die "worktree has unresolved merge conflicts"

ACTIVE_MATCHES="$(ps -eo pid,cmd | rg -F "$WORKTREE_ABS" | rg -v 'checkpoint-state.sh|rg -F' || true)"
[[ -z "$ACTIVE_MATCHES" ]] || die "active process still references worktree; refusing checkpoint"

TRACKED_STATUS="$(git -C "$WORKTREE_ABS" status --short --untracked-files=all)"
TRACKED_DIFF_HASH="$(git -C "$WORKTREE_ABS" diff --binary HEAD | shasum -a 256 | awk '{print $1}')"
TRACKED_FILES="$(git -C "$WORKTREE_ABS" status --short | awk 'substr($0,1,2) != "??" {print substr($0,4)}' | sort)"
UNTRACKED_LIST="$(git -C "$WORKTREE_ABS" status --short --untracked-files=all | awk '/^\?\?/ {sub(/^\?\? /, ""); print}' | sort)"
UNTRACKED_HASH="$(sha256_text "$UNTRACKED_LIST")"
TRACKED_ALLOWED_FILES="$(printf '%s\n' "$TRACKED_FILES" | filter_status_list allowed)"
TRACKED_OUTSIDE_FILES="$(printf '%s\n' "$TRACKED_FILES" | filter_status_list outside)"
UNTRACKED_ALLOWED_FILES="$(printf '%s\n' "$UNTRACKED_LIST" | filter_status_list allowed)"
UNTRACKED_OUTSIDE_FILES="$(printf '%s\n' "$UNTRACKED_LIST" | filter_status_list outside)"

mkdir -p "$MANIFEST_DIR_ABS" "$(dirname "$CHECKPOINT_LEDGER_ABS")"

if [[ ${#ALLOW_PREFIXES[@]} -gt 0 && -n "$TRACKED_OUTSIDE_FILES" ]]; then
    die "tracked dirty files outside allowlist: $(printf '%s' "$TRACKED_OUTSIDE_FILES" | join_lines)"
fi

if [[ "$STRICT_UNTRACKED" -eq 1 && ${#ALLOW_PREFIXES[@]} -gt 0 && -n "$UNTRACKED_OUTSIDE_FILES" ]]; then
    die "untracked files outside allowlist: $(printf '%s' "$UNTRACKED_OUTSIDE_FILES" | join_lines)"
fi

exec 9>"$LOCKFILE_ABS"
command -v flock >/dev/null 2>&1 || die "flock is required"
flock -n 9 || die "another checkpoint transaction is already running"

PROMPT="Checkpoint label: ${LABEL}. Snapshot branch point only. Do not change files or run commands. Immediately acknowledge that this is a preserved fork point and wait."

if [[ "$DRY_RUN" -eq 1 ]]; then
    printf 'checkpoint preflight ok\n'
    printf '  label: %s\n' "$LABEL"
    printf '  session: %s\n' "$SESSION_ID"
    printf '  worktree: %s\n' "$WORKTREE_ABS"
    printf '  branch: %s\n' "$BRANCH_NAME"
    printf '  head_commit: %s\n' "$HEAD_COMMIT"
    if [[ ${#ALLOW_PREFIXES[@]} -gt 0 ]]; then
        printf '  allow_prefixes:\n'
        for prefix in "${ALLOW_PREFIXES[@]}"; do
            printf '    - %s\n' "$prefix"
        done
    fi
    printf '  strict_untracked: %s\n' "$STRICT_UNTRACKED"
    [[ -n "$TRACKED_FILES" ]] && printf '  tracked_dirty_files: %s\n' "$(printf '%s\n' "$TRACKED_FILES" | wc -l | tr -d ' ')"
    [[ -n "$UNTRACKED_LIST" ]] && printf '  untracked_files: %s\n' "$(printf '%s\n' "$UNTRACKED_LIST" | wc -l | tr -d ' ')"
    exit 0
fi

BEFORE_LIST_FILE="$(mktemp)"
find "$CODEX_HOME_ABS/sessions" -type f -name '*.jsonl' | sort > "$BEFORE_LIST_FILE"

FORK_CMD="$(build_shell_command codex fork "$SESSION_ID" "$PROMPT" --no-alt-screen -C "$WORKTREE_ABS")"

{
    env CODEX_HOME="$CODEX_HOME_ABS" timeout --signal=INT --kill-after=2s 10s \
        script -q -c "$FORK_CMD" /dev/null \
        >/dev/null 2>&1 || true
} 2>/dev/null

SNAPSHOT_FILE="$(python3 - "$BEFORE_LIST_FILE" "$CODEX_HOME_ABS" <<'PY'
import sys
from pathlib import Path

before_list_file = Path(sys.argv[1])
codex_home = Path(sys.argv[2])
before = set(before_list_file.read_text().splitlines())
current = {
    str(path)
    for path in sorted(codex_home.glob("sessions/**/*.jsonl"))
    if path.is_file()
}
created = sorted(current - before)
print(created[-1] if created else "", end="")
PY
)"

[[ -n "$SNAPSHOT_FILE" ]] || die "codex fork did not produce a verified snapshot session"

rm -f "$BEFORE_LIST_FILE"
[[ -f "$SNAPSHOT_FILE" ]] || die "snapshot session file missing: $SNAPSHOT_FILE"

SNAPSHOT_SESSION_ID="$(basename "$SNAPSHOT_FILE" | sed -E 's/.*-([0-9a-f-]{36})\.jsonl/\1/')"
[[ "$SNAPSHOT_SESSION_ID" != "$SESSION_ID" ]] || die "fork did not create a new session id"

SNAPSHOT_TEXT="$(cat "$SNAPSHOT_FILE")"
printf '%s' "$SNAPSHOT_TEXT" | rg -F "$PROMPT" >/dev/null || die "snapshot session did not record the checkpoint prompt"
SNAPSHOT_TAIL_CHECK="$(
    python3 - "$PROMPT" "$SNAPSHOT_FILE" <<'PY'
import sys
from pathlib import Path

prompt = sys.argv[1]
session_file = Path(sys.argv[2])
text = session_file.read_text()
idx = text.rfind(prompt)
if idx == -1:
    print("missing-prompt")
    sys.exit(0)
tail = text[idx:]
bad_tokens = [
    '"command_execution"',
    '"function_call"',
    '"function_call_output"',
    '"tool_call"',
]
if any(token in tail for token in bad_tokens):
    print("executed-after-prompt")
    sys.exit(0)
if "preserved fork point" in tail:
    print("ok-ack")
    sys.exit(0)
print("missing-terminal-ack")
PY
)"
case "$SNAPSHOT_TAIL_CHECK" in
    ok-ack)
        ;;
    executed-after-prompt)
        die "snapshot session executed commands after the checkpoint prompt"
        ;;
    missing-terminal-ack)
        die "snapshot session did not end in a verified checkpoint acknowledgement"
        ;;
    *)
        die "snapshot session tail verification failed: $SNAPSHOT_TAIL_CHECK"
        ;;
esac

POST_ACTIVE_MATCHES="$(ps -eo pid,cmd | rg -F "$WORKTREE_ABS" | rg -v 'checkpoint-state.sh|rg -F' || true)"
[[ -z "$POST_ACTIVE_MATCHES" ]] || die "worktree still has an active process after snapshot creation"

TIMESTAMP_UTC="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
STAMP_FILE="$(date -u +%Y%m%dT%H%M%SZ)"
LABEL_SAFE="$(sanitize_label "$LABEL")"
MANIFEST_BASENAME="${STAMP_FILE}-${LABEL_SAFE}.yaml"
MANIFEST_REL="$MANIFEST_DIR_REL/$MANIFEST_BASENAME"
MANIFEST_ABS="$MANIFEST_DIR_ABS/$MANIFEST_BASENAME"

DISPATCH_HASH="$(sha256_file "$LEDGER_ABS")"
if [[ -f "$WORKTREE_AGENTS_ABS" ]]; then
    AGENTS_HASH="$(sha256_file "$WORKTREE_AGENTS_ABS")"
else
    AGENTS_HASH="$(sha256_file "$AGENTS_ABS")"
fi
if [[ -f "$WORKTREE_STATE_ABS" ]]; then
    STATE_HASH="$(sha256_file "$WORKTREE_STATE_ABS")"
else
    STATE_HASH="$(sha256_file "$STATE_ABS")"
fi
if [[ -f "$WORKTREE_LEDGER_ABS" ]]; then
    DISPATCH_HASH="$(sha256_file "$WORKTREE_LEDGER_ABS")"
fi
SESSION_HASH="$(sha256_file "$SNAPSHOT_FILE")"
STATUS_HASH="$(sha256_text "$TRACKED_STATUS")"

STATUS_PATHS="$(git -C "$WORKTREE_ABS" status --short --untracked-files=all | awk '{sub(/^.. /, ""); print}' | sort)"
STATIC_LAUNCH_REPORTS="$(find "$WORKTREE_ABS/status/launch" -maxdepth 1 -type f -name '*.md' 2>/dev/null | sed "s#^$WORKTREE_ABS/##" | sort || true)"
ALLOWLIST_REPORTS="$(printf '%s\n%s\n' "$TRACKED_ALLOWED_FILES" "$UNTRACKED_ALLOWED_FILES" | awk '/^(status\/workers\/.*\.md|status\/launch\/.*\.md)$/' | sort -u)"
REPORTS_LIST="$(printf '%s\n%s\n' "$STATIC_LAUNCH_REPORTS" "$ALLOWLIST_REPORTS" | awk 'NF {print}' | sort -u)"

{
    printf 'label: %s\n' "$LABEL"
    printf 'created_at_utc: %s\n' "$TIMESTAMP_UTC"
    printf 'parent_session: %s\n' "$SESSION_ID"
    printf 'snapshot_session: %s\n' "$SNAPSHOT_SESSION_ID"
    printf 'snapshot_session_file: %s\n' "$SNAPSHOT_FILE"
    printf 'codex_home: %s\n' "$CODEX_HOME_ABS"
    printf 'repo: %s\n' "$REPO_ROOT"
    printf 'worktree: %s\n' "$WORKTREE_ABS"
    printf 'branch: %s\n' "$BRANCH_NAME"
    printf 'head_commit: %s\n' "$HEAD_COMMIT"
    printf 'ledger: %s\n' "$LEDGER_REL"
    printf 'dispatch_state_hash: %s\n' "$DISPATCH_HASH"
    printf 'agents_hash: %s\n' "$AGENTS_HASH"
    printf 'state_hash: %s\n' "$STATE_HASH"
    printf 'snapshot_session_hash: %s\n' "$SESSION_HASH"
    printf 'tracked_status_hash: %s\n' "$STATUS_HASH"
    printf 'tracked_diff_hash: %s\n' "$TRACKED_DIFF_HASH"
    printf 'untracked_hash: %s\n' "$UNTRACKED_HASH"
    if [[ ${#ALLOW_PREFIXES[@]} -gt 0 ]]; then
        printf 'allow_prefixes:\n'
        for prefix in "${ALLOW_PREFIXES[@]}"; do
            printf '  - %s\n' "$prefix"
        done
    else
        printf 'allow_prefixes: []\n'
    fi
    printf 'strict_untracked: %s\n' "$STRICT_UNTRACKED"
    printf 'ledger_files:\n'
    printf '  - path: AGENTS.md\n'
    printf '    sha256: %s\n' "$AGENTS_HASH"
    printf '  - path: .memory/STATE.md\n'
    printf '    sha256: %s\n' "$STATE_HASH"
    printf '  - path: %s\n' "$LEDGER_REL"
    printf '    sha256: %s\n' "$DISPATCH_HASH"
    if [[ -n "$TRACKED_ALLOWED_FILES" ]]; then
        printf 'tracked_dirty_files:\n'
        while IFS= read -r line; do
            [[ -n "$line" ]] && printf '  - %s\n' "$line"
        done <<< "$TRACKED_ALLOWED_FILES"
    else
        printf 'tracked_dirty_files: []\n'
    fi
    if [[ -n "$TRACKED_OUTSIDE_FILES" ]]; then
        printf 'tracked_outside_allowlist:\n'
        while IFS= read -r line; do
            [[ -n "$line" ]] && printf '  - %s\n' "$line"
        done <<< "$TRACKED_OUTSIDE_FILES"
    else
        printf 'tracked_outside_allowlist: []\n'
    fi
    if [[ -n "$UNTRACKED_LIST" ]]; then
        printf 'untracked_files:\n'
        while IFS= read -r line; do
            [[ -n "$line" ]] && printf '  - %s\n' "$line"
        done <<< "$UNTRACKED_LIST"
    else
        printf 'untracked_files: []\n'
    fi
    if [[ -n "$UNTRACKED_OUTSIDE_FILES" ]]; then
        printf 'untracked_outside_allowlist:\n'
        while IFS= read -r line; do
            [[ -n "$line" ]] && printf '  - %s\n' "$line"
        done <<< "$UNTRACKED_OUTSIDE_FILES"
    else
        printf 'untracked_outside_allowlist: []\n'
    fi
    if [[ -n "$REPORTS_LIST" ]]; then
        printf 'reports:\n'
        while IFS= read -r line; do
            [[ -n "$line" ]] && printf '  - %s\n' "$line"
        done <<< "$REPORTS_LIST"
    else
        printf 'reports: []\n'
    fi
    printf 'restore:\n'
    printf '  resume: cd %s && env CODEX_HOME=%s codex resume %s\n' "$WORKTREE_ABS" "$CODEX_HOME_ABS" "$SNAPSHOT_SESSION_ID"
    printf '  exec_resume_template: cd %s && env CODEX_HOME=%s codex exec resume %s \"<prompt>\"\n' "$WORKTREE_ABS" "$CODEX_HOME_ABS" "$SNAPSHOT_SESSION_ID"
    printf '  worktree: %s\n' "$WORKTREE_ABS"
} > "$MANIFEST_ABS"

if [[ ! -f "$CHECKPOINT_LEDGER_ABS" ]]; then
    {
        printf '# Orchestration checkpoints\n'
        printf '# Appended by scripts/checkpoint-state.sh\n'
    } > "$CHECKPOINT_LEDGER_ABS"
fi

{
    printf -- '- label: %s\n' "$LABEL"
    printf '  created_at_utc: %s\n' "$TIMESTAMP_UTC"
    printf '  parent_session: %s\n' "$SESSION_ID"
    printf '  snapshot_session: %s\n' "$SNAPSHOT_SESSION_ID"
    printf '  worktree: %s\n' "$WORKTREE_ABS"
    printf '  branch: %s\n' "$BRANCH_NAME"
    printf '  head_commit: %s\n' "$HEAD_COMMIT"
    printf '  manifest: %s\n' "$MANIFEST_REL"
} >> "$CHECKPOINT_LEDGER_ABS"

printf 'checkpoint created\n'
printf '  label: %s\n' "$LABEL"
printf '  snapshot_session: %s\n' "$SNAPSHOT_SESSION_ID"
printf '  manifest: %s\n' "$MANIFEST_REL"
