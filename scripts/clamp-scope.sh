#!/usr/bin/env bash
set -euo pipefail

# Usage: clamp-scope.sh <allowed_prefix1> [allowed_prefix2] ...
# Reverts all tracked changes and removes untracked files outside allowed prefixes.
# Verifies clamping succeeded — fails loudly on any error.

if [ $# -eq 0 ]; then
    echo "Usage: $0 <allowed_prefix1> [allowed_prefix2] ..."
    exit 1
fi

PREFIXES=("$@")
ERRORS=0

is_allowed() {
    local f="$1"
    shift
    for prefix in "$@"; do
        [[ "$f" == "${prefix}"* ]] && return 0
    done
    return 1
}

# Collect files into arrays first (avoids subshell pipe problem)
mapfile -d '' UNSTAGED < <(git diff --name-only -z 2>/dev/null || true)
mapfile -d '' STAGED < <(git diff --name-only -z --cached 2>/dev/null || true)
mapfile -d '' UNTRACKED < <(git ls-files --others --exclude-standard -z 2>/dev/null || true)

# Revert tracked unstaged changes outside scope
for f in "${UNSTAGED[@]}"; do
    [[ -z "$f" ]] && continue
    if ! is_allowed "$f" "${PREFIXES[@]}"; then
        if ! git restore --worktree -- "$f"; then
            echo "ERROR: failed to restore unstaged file: $f" >&2
            ERRORS=$((ERRORS + 1))
        fi
    fi
done

# Revert tracked staged changes outside scope
for f in "${STAGED[@]}"; do
    [[ -z "$f" ]] && continue
    if ! is_allowed "$f" "${PREFIXES[@]}"; then
        if ! git restore --staged --worktree -- "$f"; then
            echo "ERROR: failed to restore staged file: $f" >&2
            ERRORS=$((ERRORS + 1))
        fi
    fi
done

# Remove untracked files outside scope
for f in "${UNTRACKED[@]}"; do
    [[ -z "$f" ]] && continue
    if ! is_allowed "$f" "${PREFIXES[@]}"; then
        if ! rm -rf -- "$f"; then
            echo "ERROR: failed to remove untracked file: $f" >&2
            ERRORS=$((ERRORS + 1))
        fi
    fi
done

# Post-clamp verification: check nothing remains outside scope
LEAKED=0
for f in $(git diff --name-only 2>/dev/null) $(git diff --name-only --cached 2>/dev/null); do
    if ! is_allowed "$f" "${PREFIXES[@]}"; then
        echo "LEAK: $f still modified after clamp" >&2
        LEAKED=$((LEAKED + 1))
    fi
done
for f in $(git ls-files --others --exclude-standard 2>/dev/null); do
    if ! is_allowed "$f" "${PREFIXES[@]}"; then
        echo "LEAK: $f still untracked after clamp" >&2
        LEAKED=$((LEAKED + 1))
    fi
done

if [ "$ERRORS" -gt 0 ] || [ "$LEAKED" -gt 0 ]; then
    echo "CLAMP FAILED: $ERRORS errors, $LEAKED leaks" >&2
    exit 1
fi

echo "Scope clamped to: ${PREFIXES[*]} (verified clean)"
