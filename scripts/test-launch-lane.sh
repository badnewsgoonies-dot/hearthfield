#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
TARGET_SCRIPT="$REPO_ROOT/scripts/launch-lane.sh"

TMP_ROOT="$(mktemp -d /tmp/launch-lane-test.XXXXXX)"
cleanup() {
    if [[ -n "${LAUNCHED_PGID:-}" ]]; then
        kill -"${LAUNCHED_PGID}" 2>/dev/null || true
        sleep 0.5
    fi
    if [[ -n "${LAUNCHED_PID:-}" ]]; then
        kill "${LAUNCHED_PID}" 2>/dev/null || true
        wait "${LAUNCHED_PID}" 2>/dev/null || true
    fi
    rm -rf "$TMP_ROOT" || true
}
trap cleanup EXIT

FAKE_HOME="$TMP_ROOT/home"
FAKE_REPO="$TMP_ROOT/repo"
WORKTREE="$TMP_ROOT/worktree"
CODEX_HOME="$TMP_ROOT/codex-home"
OBJECTIVE="$FAKE_REPO/objectives/launch/test-objective.md"
BRANCH_NAME="launch/test-lane-$$"

mkdir -p "$FAKE_HOME/.codex/skills" "$FAKE_REPO/objectives/launch" "$FAKE_REPO/status" "$FAKE_REPO/docs" "$FAKE_REPO/tests"

cat > "$FAKE_HOME/.codex/auth.json" <<'EOF'
{"test":true}
EOF
cat > "$FAKE_HOME/.codex/config.toml" <<'EOF'
model = "gpt-5.4"
EOF
cat > "$FAKE_HOME/.codex/version.json" <<'EOF'
{"version":"test"}
EOF
cat > "$FAKE_REPO/AGENTS.md" <<'EOF'
# test agents
EOF
cat > "$FAKE_REPO/objectives/TEMPLATE.md" <<'EOF'
# template
EOF
cat > "$OBJECTIVE" <<'EOF'
# test objective
EOF
cat > "$FAKE_REPO/tests/headless.rs" <<'EOF'
// test
EOF

git -C "$TMP_ROOT" init -q repo
git -C "$FAKE_REPO" config user.email test@example.com
git -C "$FAKE_REPO" config user.name "Launch Lane Test"
git -C "$FAKE_REPO" add .
git -C "$FAKE_REPO" commit -m "init" >/dev/null

export HOME="$FAKE_HOME"

echo "test: launch wrapper creates isolated worktree and codex home"
OUTPUT="$(
    bash "$TARGET_SCRIPT" \
        --lane-id test_lane \
        --branch "$BRANCH_NAME" \
        --worktree "$WORKTREE" \
        --codex-home "$CODEX_HOME" \
        --objective "$OBJECTIVE"
)"

printf '%s\n' "$OUTPUT" | rg "lane launched" >/dev/null
LAUNCHED_PID="$(printf '%s\n' "$OUTPUT" | awk '/^  pid: / {print $2}')"
LAUNCHED_PGID="$(printf '%s\n' "$OUTPUT" | awk '/^  pgid: / {print $2}')"
[[ -n "$LAUNCHED_PID" ]]
[[ -n "$LAUNCHED_PGID" ]]
[[ -d "$WORKTREE" ]]
[[ -d "$CODEX_HOME" ]]
[[ -f "$WORKTREE/AGENTS.md" ]]
[[ -f "$CODEX_HOME/auth.json" ]]
[[ -d "$CODEX_HOME/skills" ]]
[[ -d "$CODEX_HOME/run" ]]
[[ -f "$OBJECTIVE" ]]

echo "launch-lane smoke tests: PASS"
