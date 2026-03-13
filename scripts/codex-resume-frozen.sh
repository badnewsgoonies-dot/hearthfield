#!/usr/bin/env bash
set -euo pipefail

usage() {
    cat <<'EOF'
Usage: scripts/codex-resume-frozen.sh <session-id>

Resume a Codex session while holding an exclusive local lock for that session ID.
This prevents a second terminal from attaching to the same resumed chat at the
same time, which reduces cross-talk from shared local session state.
EOF
}

if [[ $# -ne 1 ]]; then
    usage >&2
    exit 1
fi

if [[ ${1:-} == "-h" || ${1:-} == "--help" ]]; then
    usage
    exit 0
fi

session_id=$1
lock_root=${CODEX_RESUME_LOCK_ROOT:-/tmp/codex-resume-locks}
lock_dir="$lock_root/$session_id"
meta_file="$lock_dir/owner.env"

mkdir -p "$lock_root"

print_lock_owner() {
    local owner_pid owner_started_at owner_cwd owner_tty
    owner_pid=""
    owner_started_at="unknown"
    owner_cwd="unknown"
    owner_tty="unknown"

    if [[ -f "$meta_file" ]]; then
        # shellcheck disable=SC1090
        source "$meta_file"
    fi

    printf 'Session %s is already resumed elsewhere.\n' "$session_id" >&2
    printf 'PID: %s\n' "${owner_pid:-unknown}" >&2
    printf 'Started: %s\n' "${owner_started_at:-unknown}" >&2
    printf 'TTY: %s\n' "${owner_tty:-unknown}" >&2
    printf 'CWD: %s\n' "${owner_cwd:-unknown}" >&2
}

reap_stale_lock_if_needed() {
    local owner_pid

    [[ -d "$lock_dir" ]] || return 0

    owner_pid=""
    if [[ -f "$meta_file" ]]; then
        # shellcheck disable=SC1090
        source "$meta_file"
    fi

    if [[ -n "${owner_pid:-}" ]] && kill -0 "$owner_pid" 2>/dev/null; then
        print_lock_owner
        exit 2
    fi

    rm -rf "$lock_dir"
}

cleanup() {
    rm -rf "$lock_dir"
}

reap_stale_lock_if_needed

if ! mkdir "$lock_dir" 2>/dev/null; then
    print_lock_owner
    exit 2
fi

owner_pid=$$
owner_started_at=$(date -Iseconds)
owner_cwd=$PWD
owner_tty=$(tty 2>/dev/null || printf 'not-a-tty')

trap cleanup EXIT HUP INT TERM

cat >"$meta_file" <<EOF
owner_pid=$owner_pid
owner_started_at=$owner_started_at
owner_cwd=$owner_cwd
owner_tty=$owner_tty
EOF

printf 'Resuming locked session %s\n' "$session_id" >&2
codex resume "$session_id"
