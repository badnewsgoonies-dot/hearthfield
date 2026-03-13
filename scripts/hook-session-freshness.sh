#!/bin/bash
# SessionStart hook: STATE.md freshness check
# Compares STATE.md HEAD reference with actual HEAD.
# Warns about stale numerics, confirms structural facts likely valid.
# Non-blocking (exit 0) — informational only.

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

# Run once per session — use repo-specific lock
REPO_HASH=$(echo "$REPO_ROOT" | shasum -a 256 | cut -c1-8)
LOCK="/tmp/.hearthfield-freshness-${REPO_HASH}"
if [[ -f "$LOCK" ]]; then
  exit 0
fi
touch "$LOCK"

# Skip if no STATE.md
if [[ ! -f .memory/STATE.md ]]; then
  exit 0
fi

state_head=$(grep -oP '\*\*HEAD:\*\*\s*\K\w+' .memory/STATE.md 2>/dev/null || echo "")
actual_head=$(git rev-parse --short HEAD 2>/dev/null || echo "")

if [[ -z "$state_head" || -z "$actual_head" ]]; then
  exit 0
fi

# Check if state_head is a prefix of actual_head or vice versa
if [[ "$actual_head" == "$state_head"* || "$state_head" == "$actual_head"* ]]; then
  exit 0
fi

# Compute drift
drift=$(git rev-list --count "${state_head}..HEAD" 2>/dev/null || echo "?")

if [[ "$drift" == "0" || "$drift" == "?" ]]; then
  exit 0
fi

echo "════════════════════════════════════════" >&2
echo "  STATE.md FRESHNESS CHECK" >&2
echo "════════════════════════════════════════" >&2
echo "  STATE HEAD: ${state_head}" >&2
echo "  Actual HEAD: ${actual_head}" >&2
echo "  Drift: ${drift} commit(s) behind" >&2
echo "" >&2

if [[ "$drift" -le 3 ]]; then
  echo "  LOW DRIFT: Structural + numeric claims likely valid." >&2
elif [[ "$drift" -le 7 ]]; then
  echo "  MODERATE DRIFT: Structural claims valid. Numerics suspect." >&2
  echo "  Verify: test count, HEAD, producer counts against code." >&2
else
  echo "  HIGH DRIFT: Structural claims probably valid. Numerics stale." >&2
  echo "  Strongly recommend: update .memory/STATE.md before proceeding." >&2
fi

# Check test count specifically (known high-drift numeric)
state_tests=$(grep -oP 'Gate 3.*?:\s*\K\d+(?=\s+headless)' .memory/STATE.md 2>/dev/null || echo "")
if [[ -n "$state_tests" && -f tests/headless.rs ]]; then
  actual_tests=$(grep -c '#\[test\]' tests/headless.rs 2>/dev/null || echo "?")
  if [[ "$state_tests" != "$actual_tests" ]]; then
    echo "  ⚠ Test count: STATE says ${state_tests}, actual is ${actual_tests}" >&2
  fi
fi

echo "════════════════════════════════════════" >&2
exit 0
