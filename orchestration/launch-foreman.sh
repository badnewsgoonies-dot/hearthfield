#!/usr/bin/env bash
set -euo pipefail

# Foreman Launcher — dispatches a foreman agent that reads the playbook
# and iteratively improves the game via sub-agent dispatch.
#
# Usage: bash orchestration/launch-foreman.sh [round_name]
#
# Example: bash orchestration/launch-foreman.sh visual-polish-1

ROUND="${1:-round-$(date +%s)}"
REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
RESULTS_DIR="/tmp/foreman-results-${ROUND}"
PROMPTS_DIR="/tmp/foreman-prompts"

mkdir -p "$RESULTS_DIR" "$PROMPTS_DIR"

echo "=== Foreman Launcher ==="
echo "Round: $ROUND"
echo "Repo:  $REPO_ROOT"
echo "Results: $RESULTS_DIR"
echo ""

# Ensure tokens are available
if [ -f ~/.env_tokens ]; then
  source ~/.env_tokens
fi

# Verify codex is available
if ! command -v codex &>/dev/null; then
  echo "ERROR: codex CLI not found"
  exit 1
fi

# Get file size reference for the playbook
echo "=== File sizes (for foreman reference) ==="
cd "$REPO_ROOT"
find src/ -name "*.rs" -exec wc -l {} + 2>/dev/null | sort -rn | head -25 > "$RESULTS_DIR/file-sizes.txt"
cat "$RESULTS_DIR/file-sizes.txt"
echo ""

# The foreman prompt: read the playbook, then execute it
FOREMAN_PROMPT="You are a foreman agent. Your job is to improve the Hearthfield game by dispatching sub-agents.

READ THIS FIRST: orchestration/FOREMAN_PLAYBOOK.md

Then execute all 7 phases in order:
1. Explore the codebase (read key files, understand structure)
2. Select 5 improvement targets
3. Write dispatch prompts to /tmp/foreman-prompts/task-1.txt through task-5.txt
4. Create worktrees and dispatch sub-agents in parallel
5. Evaluate results (check git diff in each worktree)
6. Push branches
7. Write report to /tmp/foreman-results-${ROUND}/REPORT.md

CRITICAL RULES:
- Read the playbook FIRST before doing anything
- Use the exact prompt template from the playbook
- Give EXACT values in every prompt (colors, multipliers, positions)
- One file per sub-agent prompt
- Stagger sub-agent launches by 3 seconds
- 180 second timeout per sub-agent
- After sub-agents finish, check what shipped and retry failures with more specificity

Your sub-agent dispatch command:
codex exec --dangerously-bypass-approvals-and-sandbox -m gpt-5.4 -C [worktree_path] \"[prompt]\"

Go."

echo "=== Dispatching foreman agent ==="
echo "Timeout: 600s (10 minutes for full cycle)"
echo ""

timeout 600 codex exec --dangerously-bypass-approvals-and-sandbox -m gpt-5.4 \
  -C "$REPO_ROOT" \
  "$FOREMAN_PROMPT" \
  > "$RESULTS_DIR/foreman-output.txt" 2>&1

EXIT_CODE=$?

echo ""
echo "=== Foreman completed (exit: $EXIT_CODE) ==="
echo "Output: $(wc -w < "$RESULTS_DIR/foreman-output.txt") words"
echo ""

# Check what was produced
echo "=== Prompts written ==="
ls -la "$PROMPTS_DIR"/task-*.txt 2>/dev/null || echo "(none)"

echo ""
echo "=== Worktrees created ==="
git worktree list 2>/dev/null | grep foreman || echo "(none)"

echo ""
echo "=== Branches ==="
git branch | grep foreman || echo "(none)"

echo ""
echo "=== Results ==="
for i in 1 2 3 4 5; do
  WT="/tmp/foreman-lane-$i"
  if [ -d "$WT" ]; then
    DIFF=$(cd "$WT" && git diff --stat HEAD~1 2>/dev/null | tail -1)
    echo "Task $i: ${DIFF:-no changes}"
  fi
done

echo ""
echo "=== Report ==="
if [ -f "$RESULTS_DIR/REPORT.md" ]; then
  cat "$RESULTS_DIR/REPORT.md"
else
  echo "(no report written — check foreman-output.txt)"
fi
