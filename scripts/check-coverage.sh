#!/usr/bin/env bash
# check-coverage.sh — Compare STATE.md mentioned domains against actual src/ domains
# Flags domains that exist in code but have no [Observed] or [Inferred] entry in STATE.md
set -euo pipefail

STATE=".memory/STATE.md"
SRC="src"

if [ ! -f "$STATE" ]; then
  echo "ERROR: $STATE not found" >&2
  exit 1
fi

echo "== Coverage Gap Check =="
echo ""

# Get all domain directories
domains=$(ls -d "$SRC"/*/ 2>/dev/null | xargs -n1 basename | sort)

covered=0
uncovered=0
uncovered_list=""

for domain in $domains; do
  # Check if domain is mentioned in STATE.md with any evidence tag
  if grep -qi "\b${domain}\b" "$STATE" 2>/dev/null; then
    covered=$((covered + 1))
  else
    uncovered=$((uncovered + 1))
    uncovered_list="${uncovered_list}  - ${domain}/\n"
  fi
done

total=$((covered + uncovered))
pct=$((covered * 100 / total))

echo "Domains in src/:     $total"
echo "Mentioned in STATE:  $covered"
echo "NOT mentioned:       $uncovered"
echo "Coverage:            ${pct}%"
echo ""

if [ "$uncovered" -gt 0 ]; then
  echo "⚠ Uncovered domains (silence ≠ working):"
  echo -e "$uncovered_list"
fi

# Check for surfaces table completeness
surfaces=$(grep -c '^\|' "$STATE" 2>/dev/null || echo 0)
echo "Runtime surfaces documented: $((surfaces - 2))"  # subtract header rows

echo ""
echo "== Done =="
