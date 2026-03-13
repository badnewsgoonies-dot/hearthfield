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

# Post-checkout hook: warns about STATE.md staleness after branch switch
cat > "$HOOKS_DIR/post-checkout" << 'HOOK'
#!/bin/bash
# Post-checkout hook: STATE.md freshness + checkpoint availability
REPO_ROOT="$(git rev-parse --show-toplevel)"

# Only run on branch switches (flag=1), not file checkouts (flag=0)
if [[ "${3:-0}" != "1" ]]; then
    exit 0
fi

new_head=$(git rev-parse --short HEAD)
branch=$(git rev-parse --abbrev-ref HEAD)

# Check STATE.md freshness
if [[ -f "$REPO_ROOT/.memory/STATE.md" ]]; then
    state_head=$(grep -oP '\*\*HEAD:\*\*\s*\K\w+' "$REPO_ROOT/.memory/STATE.md" 2>/dev/null || echo "")
    if [[ -n "$state_head" && "$state_head" != "$new_head"* && "$new_head" != "$state_head"* ]]; then
        drift=$(git rev-list --count "${state_head}..HEAD" 2>/dev/null || echo "?")
        echo "⚠ STATE.md HEAD ($state_head) doesn't match checkout ($new_head), drift: $drift commits" >&2
    fi
fi

# Check if a checkpoint exists for this branch
if [[ -f "$REPO_ROOT/status/foreman/checkpoints.yaml" ]]; then
    if grep -q "branch:.*$branch" "$REPO_ROOT/status/foreman/checkpoints.yaml" 2>/dev/null; then
        echo "✓ Checkpoint available for branch $branch" >&2
        echo "  Restore with: bash scripts/restore-checkpoint.sh --label <name>" >&2
    fi
fi

exit 0
HOOK
chmod +x "$HOOKS_DIR/post-checkout"
echo "  ✓ post-checkout hook installed (STATE.md freshness + checkpoint notice)"

echo ""
echo "Hooks installed to: $HOOKS_DIR"
echo "Claude Code hooks (PostToolUse/PreToolUse) are configured separately in .claude/settings.json"
