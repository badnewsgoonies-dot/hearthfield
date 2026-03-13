#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
TARGET_SCRIPT="$REPO_ROOT/scripts/checkpoint-state.sh"

TMP_ROOT="$(mktemp -d /tmp/checkpoint-state-test.XXXXXX)"
cleanup() {
    rm -rf "$TMP_ROOT"
}
trap cleanup EXIT

MAIN_REPO="$TMP_ROOT/repo"
WORKTREE="$TMP_ROOT/worktree"
CODEX_HOME="$TMP_ROOT/codex-home"
TEST_SCRIPT_DIR="$MAIN_REPO/scripts"
SESSION_ID="00000000-0000-0000-0000-000000000123"

mkdir -p "$MAIN_REPO/.memory" "$MAIN_REPO/status/foreman" "$MAIN_REPO/status/workers" "$CODEX_HOME/sessions/2026/03/13" "$TEST_SCRIPT_DIR"
cp "$TARGET_SCRIPT" "$TEST_SCRIPT_DIR/checkpoint-state.sh"
chmod +x "$TEST_SCRIPT_DIR/checkpoint-state.sh"

cat > "$MAIN_REPO/AGENTS.md" <<'EOF'
# test agents
EOF

cat > "$MAIN_REPO/.memory/STATE.md" <<'EOF'
# test state
EOF

cat > "$MAIN_REPO/status/foreman/dispatch-state.yaml" <<'EOF'
phase: test
EOF

cat > "$CODEX_HOME/sessions/2026/03/13/rollout-2026-03-13T00-00-00-${SESSION_ID}.jsonl" <<EOF
{"timestamp":"2026-03-13T00:00:00Z","type":"session_meta","payload":{"id":"${SESSION_ID}"}}
EOF

git -C "$TMP_ROOT" init -q repo
git -C "$MAIN_REPO" config user.email test@example.com
git -C "$MAIN_REPO" config user.name "Checkpoint Test"
git -C "$MAIN_REPO" add AGENTS.md .memory/STATE.md status/foreman/dispatch-state.yaml
git -C "$MAIN_REPO" commit -m "init" >/dev/null
git -C "$MAIN_REPO" worktree add "$WORKTREE" >/dev/null

mkdir -p "$WORKTREE/src/farming" "$WORKTREE/docs" "$WORKTREE/status/workers"
printf 'dirty\n' > "$WORKTREE/src/farming/example.rs"
printf 'report\n' > "$WORKTREE/status/workers/tranche-test.md"
git -C "$WORKTREE" add src/farming/example.rs
git -C "$WORKTREE" commit -m "add example" >/dev/null
printf 'changed\n' > "$WORKTREE/src/farming/example.rs"
printf 'note\n' > "$WORKTREE/docs/outside.md"

echo "test: dry-run passes for in-scope tracked dirty file"
bash "$TEST_SCRIPT_DIR/checkpoint-state.sh" \
    --label test-pass \
    --session "$SESSION_ID" \
    --worktree "$WORKTREE" \
    --allow-prefix src/farming \
    --allow-prefix status/workers/tranche-test.md \
    --dry-run \
    --ledger status/foreman/dispatch-state.yaml \
    --codex-home "$CODEX_HOME" >/dev/null

echo "test: strict-untracked fails on outside allowlist file"
if bash "$TEST_SCRIPT_DIR/checkpoint-state.sh" \
    --label test-fail-untracked \
    --session "$SESSION_ID" \
    --worktree "$WORKTREE" \
    --allow-prefix src/farming \
    --allow-prefix status/workers/tranche-test.md \
    --strict-untracked \
    --dry-run \
    --ledger status/foreman/dispatch-state.yaml \
    --codex-home "$CODEX_HOME" >/dev/null 2>&1; then
    echo "expected strict-untracked preflight to fail" >&2
    exit 1
fi

echo "test: tracked outside allowlist fails"
printf 'mutated\n' > "$WORKTREE/AGENTS.md"
if bash "$TEST_SCRIPT_DIR/checkpoint-state.sh" \
    --label test-fail-tracked \
    --session "$SESSION_ID" \
    --worktree "$WORKTREE" \
    --allow-prefix src/farming \
    --allow-prefix status/workers/tranche-test.md \
    --dry-run \
    --ledger status/foreman/dispatch-state.yaml \
    --codex-home "$CODEX_HOME" >/dev/null 2>&1; then
    echo "expected tracked outside-allowlist preflight to fail" >&2
    exit 1
fi

echo "checkpoint-state smoke tests: PASS"
