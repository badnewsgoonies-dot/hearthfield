#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
TARGET_SCRIPT="$REPO_ROOT/scripts/restore-checkpoint.sh"

TMP_ROOT="$(mktemp -d /tmp/restore-checkpoint-test.XXXXXX)"
cleanup() {
    rm -rf "$TMP_ROOT"
}
trap cleanup EXIT

REPO="$TMP_ROOT/repo"
WORKTREE="$TMP_ROOT/worktree"
CODEX_HOME="$TMP_ROOT/codex-home"
MANIFEST_DIR="$REPO/status/checkpoints"
LEDGER_DIR="$REPO/status/foreman"
TEST_SCRIPT_DIR="$REPO/scripts"
SESSION_ID="00000000-0000-0000-0000-000000000123"
SESSION_FILE="$CODEX_HOME/sessions/2026/03/13/rollout-2026-03-13T00-00-00-${SESSION_ID}.jsonl"

mkdir -p "$REPO/.memory" "$LEDGER_DIR" "$MANIFEST_DIR" "$CODEX_HOME/sessions/2026/03/13" "$TEST_SCRIPT_DIR"
cp "$TARGET_SCRIPT" "$TEST_SCRIPT_DIR/restore-checkpoint.sh"
chmod +x "$TEST_SCRIPT_DIR/restore-checkpoint.sh"

cat > "$REPO/AGENTS.md" <<'EOF'
# test agents
EOF

cat > "$REPO/.memory/STATE.md" <<'EOF'
# test state
EOF

cat > "$LEDGER_DIR/dispatch-state.yaml" <<'EOF'
phase: test
EOF

cat > "$SESSION_FILE" <<EOF
{"timestamp":"2026-03-13T00:00:00Z","type":"session_meta","payload":{"id":"${SESSION_ID}"}}
EOF

git -C "$TMP_ROOT" init -q repo
git -C "$REPO" config user.email test@example.com
git -C "$REPO" config user.name "Restore Checkpoint Test"
git -C "$REPO" add AGENTS.md .memory/STATE.md status/foreman/dispatch-state.yaml
git -C "$REPO" commit -m "init" >/dev/null
git -C "$REPO" worktree add "$WORKTREE" >/dev/null

cp "$REPO/AGENTS.md" "$WORKTREE/AGENTS.md"
mkdir -p "$WORKTREE/.memory" "$WORKTREE/status/foreman"
cp "$REPO/.memory/STATE.md" "$WORKTREE/.memory/STATE.md"
cp "$REPO/status/foreman/dispatch-state.yaml" "$WORKTREE/status/foreman/dispatch-state.yaml"

tracked_status="$(git -C "$WORKTREE" status --short --untracked-files=all)"
tracked_status_hash="$(printf '%s' "$tracked_status" | shasum -a 256 | awk '{print $1}')"
tracked_diff_hash="$(git -C "$WORKTREE" diff --binary HEAD | shasum -a 256 | awk '{print $1}')"
untracked_list="$(git -C "$WORKTREE" status --short --untracked-files=all | awk '/^\?\?/ {sub(/^\?\? /, ""); print}' | sort)"
untracked_hash="$(printf '%s' "$untracked_list" | shasum -a 256 | awk '{print $1}')"
agents_hash="$(shasum -a 256 "$WORKTREE/AGENTS.md" | awk '{print $1}')"
state_hash="$(shasum -a 256 "$WORKTREE/.memory/STATE.md" | awk '{print $1}')"
dispatch_hash="$(shasum -a 256 "$WORKTREE/status/foreman/dispatch-state.yaml" | awk '{print $1}')"
session_hash="$(shasum -a 256 "$SESSION_FILE" | awk '{print $1}')"
head_commit="$(git -C "$WORKTREE" rev-parse --verify HEAD)"
branch="$(git -C "$WORKTREE" rev-parse --abbrev-ref HEAD)"

cat > "$MANIFEST_DIR/test.yaml" <<EOF
label: test
created_at_utc: 2026-03-13T00:00:00Z
parent_session: parent
snapshot_session: ${SESSION_ID}
snapshot_session_file: ${SESSION_FILE}
codex_home: ${CODEX_HOME}
repo: ${REPO}
worktree: ${WORKTREE}
branch: ${branch}
head_commit: ${head_commit}
ledger: status/foreman/dispatch-state.yaml
dispatch_state_hash: ${dispatch_hash}
agents_hash: ${agents_hash}
state_hash: ${state_hash}
snapshot_session_hash: ${session_hash}
tracked_status_hash: ${tracked_status_hash}
tracked_diff_hash: ${tracked_diff_hash}
untracked_hash: ${untracked_hash}
EOF

cat > "$LEDGER_DIR/checkpoints.yaml" <<EOF
- label: test
  manifest: status/checkpoints/test.yaml
EOF

echo "test: verify by manifest"
bash "$TEST_SCRIPT_DIR/restore-checkpoint.sh" --manifest "$MANIFEST_DIR/test.yaml" >/dev/null

echo "test: verify by label"
(
    cd "$REPO"
    bash "$TEST_SCRIPT_DIR/restore-checkpoint.sh" --label test >/dev/null
)

echo "test: mismatch fails without force"
printf 'mutated\n' > "$WORKTREE/AGENTS.md"
if bash "$TEST_SCRIPT_DIR/restore-checkpoint.sh" --manifest "$MANIFEST_DIR/test.yaml" >/dev/null 2>&1; then
    echo "expected mismatch verification to fail" >&2
    exit 1
fi

echo "test: mismatch passes with force"
bash "$TEST_SCRIPT_DIR/restore-checkpoint.sh" --manifest "$MANIFEST_DIR/test.yaml" --force >/dev/null

echo "restore-checkpoint smoke tests: PASS"
