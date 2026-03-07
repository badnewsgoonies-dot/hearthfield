#!/usr/bin/env bash
set -euo pipefail
ALLOW_PREFIX="${1:?Usage: clamp-scope.sh src/domains/cases/}"

cd "$(dirname "$0")/.."

# Revert tracked unstaged changes outside scope
git diff --name-only -z | while IFS= read -r -d '' f; do
  [[ "$f" == "${ALLOW_PREFIX}"* ]] && continue
  git restore --worktree -- "$f"
done

# Revert tracked staged changes outside scope
git diff --name-only -z --cached | while IFS= read -r -d '' f; do
  [[ "$f" == "${ALLOW_PREFIX}"* ]] && continue
  git restore --staged --worktree -- "$f"
done

# Remove untracked files outside scope
git ls-files --others --exclude-standard -z | while IFS= read -r -d '' f; do
  [[ "$f" == "${ALLOW_PREFIX}"* ]] && continue
  rm -rf -- "$f"
done

echo "Scope clamped to: ${ALLOW_PREFIX}"
