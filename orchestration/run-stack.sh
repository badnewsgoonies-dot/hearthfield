#!/usr/bin/env bash
set -euo pipefail

###############################################################################
# 4-LAYER CODEX STACK — One-shot launcher
#
# Usage:  bash orchestration/run-stack.sh "your request here"
# Example: bash orchestration/run-stack.sh "make the fishing system more fun"
#
# Stack:
#   Layer 1: Codex "Geni"    — explores codebase, writes creative directions
#   Layer 2: Codex "Claude"  — reads code, writes exact-value prompts
#   Layer 3: Codex "Foreman" — creates worktrees, dispatches workers
#   Layer 4: Codex Workers   — single-file code changes
#
# All results cherry-picked to master and pushed.
###############################################################################

REQUEST="${1:?Usage: bash orchestration/run-stack.sh \"your request\"}"
REPO="$(cd "$(dirname "$0")/.." && pwd)"
RUN_ID="stack-$(date +%s)"
RESULTS="/tmp/${RUN_ID}"
PROMPTS="${RESULTS}/prompts"

mkdir -p "$RESULTS" "$PROMPTS"

echo "╔══════════════════════════════════════════════╗"
echo "║  4-LAYER CODEX STACK                         ║"
echo "╚══════════════════════════════════════════════╝"
echo "Request: ${REQUEST}"
echo "Run ID:  ${RUN_ID}"
echo "Repo:    ${REPO}"
echo ""

cd "$REPO"

# ─────────────────────────────────────────────────────────────────────────────
# LAYER 1: Codex "Geni" — explore and direct
# ─────────────────────────────────────────────────────────────────────────────
echo "━━━ LAYER 1: Geni (explore + direct) ━━━"

timeout 300 codex exec --dangerously-bypass-approvals-and-sandbox -m gpt-5.4 \
  -C "$REPO" \
  "You are Geni, the game director for Hearthfield. Read orchestration/GENI_AGENT.md.

The human's request is: \"${REQUEST}\"

Execute:
1. Explore the codebase (read key src/ files relevant to the request)
2. Based on the request, identify exactly 5 concrete improvements
3. For each, write a 2-4 sentence creative direction describing what the PLAYER should feel
4. Write all 5 directions to ${RESULTS}/geni-directions.txt in this format:

DIRECTION 1: [category A-E]
[your direction, 2-4 sentences about player feel]
TARGET FILES: [the source file(s) involved]

DIRECTION 2: ...
(repeat for all 5)

Write the file and stop. Do NOT dispatch anything." \
  > "${RESULTS}/layer1-output.txt" 2>&1

if [ ! -f "${RESULTS}/geni-directions.txt" ]; then
  echo "FAIL: Geni did not write directions"
  exit 1
fi
DIRECTION_COUNT=$(grep -c "^DIRECTION" "${RESULTS}/geni-directions.txt")
echo "✓ Geni wrote ${DIRECTION_COUNT} directions"
echo ""

# ─────────────────────────────────────────────────────────────────────────────
# LAYER 2: Codex "Claude" — translate directions to exact specs
# ─────────────────────────────────────────────────────────────────────────────
echo "━━━ LAYER 2: Claude (read code + write specs) ━━━"

timeout 300 codex exec --dangerously-bypass-approvals-and-sandbox -m gpt-5.4 \
  -C "$REPO" \
  "You are Claude, the orchestrator. Read orchestration/CLAUDE_AGENT.md.

Read ${RESULTS}/geni-directions.txt — these are Geni's creative directions.

For EACH direction (up to 5):
1. Read the TARGET FILE to find CURRENT values (exact numbers, strings, parameters)
2. Decide what the NEW values should be to achieve Geni's vision
3. Write an exact-value worker prompt to ${PROMPTS}/task-N.txt

Use this template for every prompt:
HF|SCOPE:[exact_file.rs] ONLY

[What to change — 1 sentence]

Exact changes:
- [parameter] currently [old] → change to [new]
- [parameter] currently [old] → change to [new]

Change ONLY [filename]. Nothing else.
- DON'T [constraint 1]
- DON'T [constraint 2]

Write all prompts and stop. Do NOT dispatch workers." \
  > "${RESULTS}/layer2-output.txt" 2>&1

PROMPT_COUNT=$(ls "${PROMPTS}"/task-*.txt 2>/dev/null | wc -l)
if [ "$PROMPT_COUNT" -eq 0 ]; then
  echo "FAIL: Claude wrote no prompts"
  exit 1
fi
echo "✓ Claude wrote ${PROMPT_COUNT} prompts"
echo ""

# ─────────────────────────────────────────────────────────────────────────────
# LAYER 3: Codex "Foreman" — create worktrees, dispatch workers
# ─────────────────────────────────────────────────────────────────────────────
echo "━━━ LAYER 3: Foreman (dispatch workers) ━━━"

# Pre-create worktrees (faster than letting foreman do it)
for i in $(seq 1 "$PROMPT_COUNT"); do
  git worktree remove "/tmp/${RUN_ID}-lane-${i}" 2>/dev/null || true
  git branch -D "${RUN_ID}/task-${i}" 2>/dev/null || true
  git worktree add "/tmp/${RUN_ID}-lane-${i}" -b "${RUN_ID}/task-${i}" HEAD 2>/dev/null
done

# Build the dispatch script for the foreman
DISPATCH_SCRIPT="#!/bin/bash\n"
for i in $(seq 1 "$PROMPT_COUNT"); do
  DISPATCH_SCRIPT+="(\n"
  DISPATCH_SCRIPT+="  timeout 180 codex exec --dangerously-bypass-approvals-and-sandbox -m gpt-5.4 \\\\\n"
  DISPATCH_SCRIPT+="    -C /tmp/${RUN_ID}-lane-${i} \\\\\n"
  DISPATCH_SCRIPT+="    \"\$(cat ${PROMPTS}/task-${i}.txt)\" \\\\\n"
  DISPATCH_SCRIPT+="    > ${RESULTS}/worker-${i}.txt 2>&1\n"
  DISPATCH_SCRIPT+="  cd /tmp/${RUN_ID}-lane-${i}\n"
  DISPATCH_SCRIPT+="  git add -A\n"
  DISPATCH_SCRIPT+="  git commit -m '${RUN_ID}: task ${i}' --allow-empty\n"
  DISPATCH_SCRIPT+="  echo WORKER_${i}_DONE\n"
  DISPATCH_SCRIPT+=") &\n"
  DISPATCH_SCRIPT+="sleep 3\n"
done
DISPATCH_SCRIPT+="wait\n"

printf "%b" "$DISPATCH_SCRIPT" > "${RESULTS}/dispatch.sh"
chmod +x "${RESULTS}/dispatch.sh"

timeout 420 codex exec --dangerously-bypass-approvals-and-sandbox -m gpt-5.4 \
  -C "$REPO" \
  "You are the Foreman. Your ONLY job: run this dispatch script and report results.

Run: bash ${RESULTS}/dispatch.sh

Then check each worktree:
for i in $(seq 1 $PROMPT_COUNT); do
  cd /tmp/${RUN_ID}-lane-\$i
  echo \"Task \$i: \$(git diff --stat HEAD~1 2>/dev/null | tail -1)\"
done

Write results to ${RESULTS}/foreman-report.txt.
Go." \
  > "${RESULTS}/layer3-output.txt" 2>&1

echo "✓ Foreman dispatched ${PROMPT_COUNT} workers"
echo ""

# ─────────────────────────────────────────────────────────────────────────────
# COLLECT RESULTS
# ─────────────────────────────────────────────────────────────────────────────
echo "━━━ RESULTS ━━━"

SHIPPED=0
EMPTY=0
for i in $(seq 1 "$PROMPT_COUNT"); do
  WT="/tmp/${RUN_ID}-lane-${i}"
  if [ -d "$WT" ]; then
    DIFF=$(cd "$WT" && git diff --stat HEAD~1 2>/dev/null | tail -1)
    if [ -n "$DIFF" ] && echo "$DIFF" | grep -q "changed"; then
      echo "✓ Task ${i}: ${DIFF}"
      SHIPPED=$((SHIPPED + 1))
    else
      echo "✗ Task ${i}: empty"
      EMPTY=$((EMPTY + 1))
    fi
  else
    echo "✗ Task ${i}: no worktree"
    EMPTY=$((EMPTY + 1))
  fi
done

echo ""
echo "Ship rate: ${SHIPPED}/${PROMPT_COUNT}"
echo ""

# ─────────────────────────────────────────────────────────────────────────────
# MERGE TO MASTER
# ─────────────────────────────────────────────────────────────────────────────
if [ "$SHIPPED" -gt 0 ]; then
  echo "━━━ MERGING TO MASTER ━━━"
  git checkout master 2>/dev/null

  for i in $(seq 1 "$PROMPT_COUNT"); do
    WT="/tmp/${RUN_ID}-lane-${i}"
    [ ! -d "$WT" ] && continue
    DIFF=$(cd "$WT" && git diff --stat HEAD~1 2>/dev/null | tail -1)
    echo "$DIFF" | grep -q "changed" || continue

    SHA=$(cd "$WT" && git log --format=%H -1)
    if git cherry-pick "$SHA" --no-edit 2>/dev/null; then
      echo "  ✓ Merged task ${i}"
    else
      CONFLICTED=$(git diff --name-only --diff-filter=U 2>/dev/null)
      if [ -n "$CONFLICTED" ]; then
        git checkout --theirs $CONFLICTED 2>/dev/null
        git add $CONFLICTED 2>/dev/null
        GIT_EDITOR=true git cherry-pick --continue 2>/dev/null
        echo "  ⚠ Merged task ${i} (conflict resolved)"
      else
        git cherry-pick --abort 2>/dev/null
        echo "  ✗ Task ${i} merge failed"
      fi
    fi
  done

  git push origin master 2>&1 | tail -2
  echo ""
fi

# ─────────────────────────────────────────────────────────────────────────────
# CLEANUP
# ─────────────────────────────────────────────────────────────────────────────
for i in $(seq 1 "$PROMPT_COUNT"); do
  git worktree remove "/tmp/${RUN_ID}-lane-${i}" 2>/dev/null || true
done

echo "╔══════════════════════════════════════════════╗"
echo "║  DONE — ${SHIPPED}/${PROMPT_COUNT} shipped to master              ║"
echo "╚══════════════════════════════════════════════╝"
echo ""
echo "Artifacts: ${RESULTS}/"
echo "  geni-directions.txt  — creative directions"
echo "  prompts/task-*.txt   — exact specs"
echo "  worker-*.txt         — worker outputs"
echo "  foreman-report.txt   — dispatch results"
