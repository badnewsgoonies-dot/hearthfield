#!/usr/bin/env bash
set -euo pipefail

# Installs Hearthfield git hooks and Claude Code hooks.
# Run once after cloning or when hooks are updated.

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
HOOKS_DIR="$REPO_ROOT/.git/hooks"

echo "Installing Hearthfield hooks..."

# Pre-commit hook: runs contract integrity check + scope verification
cat > "$HOOKS_DIR/pre-commit" << 'HOOK'
#!/bin/bash
set -euo pipefail
REPO_ROOT="$(git rev-parse --show-toplevel)"

# Gate 1: Contract integrity
if [[ -f "$REPO_ROOT/.contract.sha256" ]]; then
    if ! shasum -a 256 -c "$REPO_ROOT/.contract.sha256" >/dev/null 2>&1; then
        echo "PRE-COMMIT BLOCKED: Contract checksum mismatch (src/shared/mod.rs)" >&2
        echo "If this is intentional, update .contract.sha256 first." >&2
        exit 1
    fi
fi

# Gate 1b: Contract dependency integrity
if [[ -f "$REPO_ROOT/.contract-deps.sha256" ]]; then
    if ! shasum -a 256 -c "$REPO_ROOT/.contract-deps.sha256" >/dev/null 2>&1; then
        echo "PRE-COMMIT BLOCKED: Contract dependency checksum mismatch" >&2
        echo "A re-exported module in src/shared/ changed without updating .contract-deps.sha256" >&2
        exit 1
    fi
fi

exit 0
HOOK
chmod +x "$HOOKS_DIR/pre-commit"
echo "  ✓ pre-commit hook installed (contract integrity)"

# Pre-push hook: runs full gate suite
cat > "$HOOKS_DIR/pre-push" << 'HOOK'
#!/bin/bash
set -euo pipefail
REPO_ROOT="$(git rev-parse --show-toplevel)"

echo "Running gate validation before push..."
if [[ -x "$REPO_ROOT/scripts/run-gates.sh" ]]; then
    "$REPO_ROOT/scripts/run-gates.sh"
else
    echo "WARNING: run-gates.sh not found or not executable" >&2
fi
HOOK
chmod +x "$HOOKS_DIR/pre-push"
echo "  ✓ pre-push hook installed (full gate suite)"

echo ""
echo "Hooks installed to: $HOOKS_DIR"
echo "Claude Code hooks (PostToolUse/PreToolUse) are configured separately in .claude/settings.json"
