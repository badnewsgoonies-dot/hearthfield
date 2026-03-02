#!/usr/bin/env bash
set -euo pipefail

# Usage: clamp-scope.sh <allowed_prefix1> [allowed_prefix2] ...
# Reverts all tracked changes and removes untracked files outside allowed prefixes.

if [ $# -eq 0 ]; then
    echo "Usage: $0 <allowed_prefix1> [allowed_prefix2] ..."
    exit 1
fi

is_allowed() {
    local f="$1"
    for prefix in "$@"; do
        [[ "$f" == "$prefix"* ]] && return 0
    done
    return 1
}

PREFIXES=("$@")

# Revert tracked unstaged changes outside scope
git diff --name-only -z | while IFS= read -r -d '' f; do
    allowed=false
    for prefix in "${PREFIXES[@]}"; do
        [[ "$f" == "${prefix}"* ]] && allowed=true && break
    done
    $allowed || git restore --worktree -- "$f"
done

# Revert tracked staged changes outside scope
git diff --name-only -z --cached | while IFS= read -r -d '' f; do
    allowed=false
    for prefix in "${PREFIXES[@]}"; do
        [[ "$f" == "${prefix}"* ]] && allowed=true && break
    done
    $allowed || git restore --staged --worktree -- "$f"
done

# Remove untracked files outside scope
git ls-files --others --exclude-standard -z | while IFS= read -r -d '' f; do
    allowed=false
    for prefix in "${PREFIXES[@]}"; do
        [[ "$f" == "${prefix}"* ]] && allowed=true && break
    done
    $allowed || rm -rf -- "$f"
done

echo "Scope clamped to: ${PREFIXES[*]}"
