#!/usr/bin/env bash
set -euo pipefail

# Mechanical scope enforcement helper for DLC lanes.
#
# Usage:
#   ./dlc/city/tools/scope_guard.sh <allowed_prefix1> [allowed_prefix2 ...]
#   ./dlc/city/tools/scope_guard.sh --allow-file dlc/city/TASKS.md <lane_id>
#
# --allow-file mode extracts markdown backtick paths from the given file and uses
# them as allowed prefixes. If an optional lane id is provided, extraction starts
# at the first heading that contains the lane id and stops at the next heading of
# same or higher level.

ALLOW_FILE=""
LANE_ID=""

if [[ $# -ge 2 && "$1" == "--allow-file" ]]; then
    ALLOW_FILE="$2"
    shift 2
    if [[ $# -ge 1 ]]; then
        LANE_ID="$1"
        shift
    fi
fi

PREFIXES=()

extract_prefixes_from_file() {
    local file="$1"
    local lane="$2"
    local in_lane=1

    if [[ -n "$lane" ]]; then
        in_lane=0
    fi

    while IFS= read -r line; do
        if [[ -n "$lane" ]]; then
            if [[ "$line" =~ ^###?[[:space:]].*${lane} ]]; then
                in_lane=1
            elif [[ "$line" =~ ^##[[:space:]] && $in_lane -eq 1 ]]; then
                break
            fi
        fi

        [[ $in_lane -eq 1 ]] || continue

        while [[ "$line" =~ \`([^\`]*)\` ]]; do
            local path="${BASH_REMATCH[1]}"
            PREFIXES+=("$path")
            line="${line#*\`$path\`}"
        done
    done < "$file"
}

if [[ -n "$ALLOW_FILE" ]]; then
    if [[ ! -f "$ALLOW_FILE" ]]; then
        echo "allow-file not found: $ALLOW_FILE" >&2
        exit 1
    fi
    extract_prefixes_from_file "$ALLOW_FILE" "$LANE_ID"
fi

if [[ $# -gt 0 ]]; then
    PREFIXES+=("$@")
fi

if [[ ${#PREFIXES[@]} -eq 0 ]]; then
    echo "Usage: $0 <allowed_prefix...>" >&2
    echo "   or: $0 --allow-file <file.md> [lane_id]" >&2
    exit 1
fi

# De-duplicate.
mapfile -t PREFIXES < <(printf '%s\n' "${PREFIXES[@]}" | sed '/^$/d' | sort -u)

echo "Scope guard prefixes:"
printf '  - %s\n' "${PREFIXES[@]}"

is_allowed() {
    local f="$1"
    for prefix in "${PREFIXES[@]}"; do
        [[ "$f" == "$prefix"* ]] && return 0
    done
    return 1
}

# Revert tracked unstaged changes outside scope.
git diff --name-only -z | while IFS= read -r -d '' f; do
    if ! is_allowed "$f"; then
        git restore --worktree -- "$f"
    fi
done

# Revert tracked staged changes outside scope.
git diff --name-only -z --cached | while IFS= read -r -d '' f; do
    if ! is_allowed "$f"; then
        git restore --staged --worktree -- "$f"
    fi
done

# Remove untracked files outside scope.
git ls-files --others --exclude-standard -z | while IFS= read -r -d '' f; do
    if ! is_allowed "$f"; then
        rm -rf -- "$f"
    fi
done

echo "Scope enforcement complete."
