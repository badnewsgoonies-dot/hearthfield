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
TMPDIR_CLAMP=$(mktemp -d)
trap 'rm -rf "$TMPDIR_CLAMP"' EXIT

is_allowed() {
    local f="$1"
    shift
    for prefix in "$@"; do
        [[ "$f" == "${prefix}"* ]] && return 0
    done
    return 1
}

# Collect file lists into temp files (avoids subshell pipe problem)
git diff --name-only -z 2>/dev/null > "$TMPDIR_CLAMP/unstaged" || true
git diff --name-only -z --cached 2>/dev/null > "$TMPDIR_CLAMP/staged" || true
git ls-files --others --exclude-standard -z 2>/dev/null > "$TMPDIR_CLAMP/untracked" || true

# Revert tracked unstaged changes outside scope
while IFS= read -r -d '' f; do
    [[ -z "$f" ]] && continue
    if ! is_allowed "$f" "${PREFIXES[@]}"; then
        if ! git restore --worktree -- "$f"; then
            echo "ERROR: failed to restore unstaged file: $f" >&2
            ERRORS=$((ERRORS + 1))
        fi
    fi
done < "$TMPDIR_CLAMP/unstaged"

# Revert tracked staged changes outside scope
while IFS= read -r -d '' f; do
    [[ -z "$f" ]] && continue
    if ! is_allowed "$f" "${PREFIXES[@]}"; then
        if ! git restore --staged --worktree -- "$f"; then
            echo "ERROR: failed to restore staged file: $f" >&2
            ERRORS=$((ERRORS + 1))
        fi
    fi
done < "$TMPDIR_CLAMP/staged"

# Remove untracked files outside scope
while IFS= read -r -d '' f; do
    [[ -z "$f" ]] && continue
    if ! is_allowed "$f" "${PREFIXES[@]}"; then
        if ! rm -rf -- "$f"; then
            echo "ERROR: failed to remove untracked file: $f" >&2
            ERRORS=$((ERRORS + 1))
        fi
    fi
done < "$TMPDIR_CLAMP/untracked"

# Post-clamp verification: check nothing remains outside scope
LEAKED=0
git diff --name-only -z 2>/dev/null > "$TMPDIR_CLAMP/post_unstaged" || true
git diff --name-only -z --cached 2>/dev/null > "$TMPDIR_CLAMP/post_staged" || true
git ls-files --others --exclude-standard -z 2>/dev/null > "$TMPDIR_CLAMP/post_untracked" || true

for src in post_unstaged post_staged post_untracked; do
    while IFS= read -r -d '' f; do
        [[ -z "$f" ]] && continue
        if ! is_allowed "$f" "${PREFIXES[@]}"; then
            echo "LEAK: $f still present after clamp" >&2
            LEAKED=$((LEAKED + 1))
        fi
    done < "$TMPDIR_CLAMP/$src"
done

if [ "$ERRORS" -gt 0 ] || [ "$LEAKED" -gt 0 ]; then
    echo "CLAMP FAILED: $ERRORS errors, $LEAKED leaks" >&2
    exit 1
fi

echo "Scope clamped to: ${PREFIXES[*]} (verified clean)"
