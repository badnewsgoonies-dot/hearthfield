#!/usr/bin/env bash
set -euo pipefail

# verify-state-claims.sh — Mechanical verification of STATE.md numeric claims
# against actual code/test/git state. Catches the exact class of bugs that
# Trial F and Trial G identified: numeric drift in cached artifacts.
#
# Exit codes:
#   0 = all claims verified
#   1 = one or more claims failed verification
#   2 = STATE.md not found

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

STATE=".memory/STATE.md"
if [[ ! -f "$STATE" ]]; then
    echo "ERROR: $STATE not found" >&2
    exit 2
fi

FAIL=0
WARN=0
CHECKED=0
PASSED=0

check() {
    local label="$1" expected="$2" actual="$3"
    CHECKED=$((CHECKED + 1))
    if [[ "$expected" == "$actual" ]]; then
        echo "  ✓ $label: $actual"
        PASSED=$((PASSED + 1))
    else
        echo "  ✗ $label: STATE says '$expected', actual is '$actual'"
        FAIL=1
    fi
}

warn() {
    local label="$1" msg="$2"
    echo "  ⚠ $label: $msg"
    WARN=$((WARN + 1))
}

echo "══════════════════════════════════════════════════"
echo "  STATE.md Claim Verification"
echo "══════════════════════════════════════════════════"
echo ""

# ── 1. HEAD reference ────────────────────────────────
echo "== HEAD Reference =="
STATE_HEAD=$(grep -oP '\*\*HEAD:\*\*\s*\K\w+' "$STATE" 2>/dev/null || echo "MISSING")
ACTUAL_HEAD=$(git rev-parse --short HEAD 2>/dev/null || echo "UNKNOWN")
DRIFT=$(git rev-list --count "${STATE_HEAD}..HEAD" 2>/dev/null || echo "?")
check "HEAD commit" "$STATE_HEAD" "$ACTUAL_HEAD"
if [[ "$DRIFT" != "0" && "$DRIFT" != "?" ]]; then
    warn "HEAD drift" "${DRIFT} commit(s) behind"
fi
echo ""

# ── 2. Working tree status ───────────────────────────
echo "== Working Tree =="
STATE_TREE=$(grep -oP '\*\*Working tree:\*\*\s*\K\w+' "$STATE" 2>/dev/null || echo "MISSING")
if git diff --quiet && git diff --cached --quiet; then
    ACTUAL_TREE="clean"
else
    ACTUAL_TREE="dirty"
fi
check "Working tree" "$STATE_TREE" "$ACTUAL_TREE"
echo ""

# ── 3. Test count ────────────────────────────────────
echo "== Test Count =="
STATE_TESTS=$(grep -oP 'Gate 3.*?:\s*\K\d+(?=\s+headless)' "$STATE" 2>/dev/null || echo "MISSING")
if [[ -f tests/headless.rs ]]; then
    ACTUAL_TESTS=$(grep -c '#\[test\]' tests/headless.rs 2>/dev/null || echo "0")
    check "Headless test count" "$STATE_TESTS" "$ACTUAL_TESTS"
else
    warn "Test count" "tests/headless.rs not found"
fi
echo ""

# ── 4. GoldChangeEvent producer count ────────────────
echo "== GoldChangeEvent Producers =="
STATE_PRODUCERS=$(grep -oP '(\d+)\s+producers' "$STATE" 2>/dev/null | head -1 | grep -oP '\d+' || echo "MISSING")
if [[ "$STATE_PRODUCERS" != "MISSING" ]]; then
    # Count actual GoldChangeEvent { construction sites (producers), excluding the struct definition
    ACTUAL_PRODUCERS=$(grep -r 'GoldChangeEvent\s*{' src/ 2>/dev/null | \
        grep -v 'pub struct GoldChangeEvent' | \
        wc -l | tr -d ' ')
    check "GoldChangeEvent producer call sites" "$STATE_PRODUCERS" "$ACTUAL_PRODUCERS"
fi
echo ""

# ── 5. Domain count ──────────────────────────────────
echo "== Domain Count =="
ACTUAL_DOMAINS=$(find src/ -mindepth 1 -maxdepth 1 -type d | wc -l | tr -d ' ')
echo "  ℹ Domain directories (src/*/): $ACTUAL_DOMAINS"
echo ""

# ── 6. Map count ─────────────────────────────────────
echo "== Map Files =="
ACTUAL_MAPS=$(find assets/maps/ -name '*.ron' 2>/dev/null | wc -l | tr -d ' ')
echo "  ℹ Map .ron files on disk: $ACTUAL_MAPS"
echo ""

# ── 7. Branch name ───────────────────────────────────
echo "== Branch =="
STATE_BRANCH=$(grep -oP '\*\*Branch:\*\*\s*\K\S+' "$STATE" 2>/dev/null || echo "MISSING")
ACTUAL_BRANCH=$(git branch --show-current 2>/dev/null || echo "UNKNOWN")
check "Branch" "$STATE_BRANCH" "$ACTUAL_BRANCH"
echo ""

# ── 8. Coverage Manifest consistency ─────────────────
echo "== Coverage Manifest Spot Check =="
# Check that domains listed as "covered" actually have src/ directories
COVERED_LINE=$(grep -A1 'Covered domains' "$STATE" 2>/dev/null | tail -1 || echo "")
if [[ -n "$COVERED_LINE" ]]; then
    # Extract domain names from the comma-separated list
    # Known aliases map to actual src/ directories
    MISSING_DOMAINS=""
    for domain in $(echo "$COVERED_LINE" | tr ',' '\n' | tr '/' '\n' | sed 's/[^a-z_]//g' | sort -u); do
        [[ -z "$domain" ]] && continue
        # Skip sub-features and qualifiers that aren't top-level directories
        case "$domain" in
            tools|maps|shops|partial|day|season|load|loadpartial) continue ;;
            sailing) continue ;;  # part of world/player, not its own src/ dir
            calendarday|saveload) continue ;;  # compound parse artifacts
        esac
        if [[ ! -d "src/$domain" ]]; then
            MISSING_DOMAINS="$MISSING_DOMAINS $domain"
        fi
    done
    if [[ -z "$MISSING_DOMAINS" ]]; then
        echo "  ✓ All covered domains have src/ directories"
        CHECKED=$((CHECKED + 1))
        PASSED=$((PASSED + 1))
    else
        echo "  ⚠ Covered domains without src/ directories:$MISSING_DOMAINS"
        WARN=$((WARN + 1))
    fi
else
    warn "Coverage" "Could not parse covered domains from STATE.md"
fi
echo ""

# ── 9. Artifact count ────────────────────────────────
echo "== Memory Artifacts =="
ARTIFACT_COUNT=$(find .memory/ -name '*.yaml' 2>/dev/null | wc -l | tr -d ' ')
echo "  ℹ Active .yaml artifacts: $ARTIFACT_COUNT"
# Check all have required fields
BAD_ARTIFACTS=0
for f in .memory/*.yaml; do
    [[ -f "$f" ]] || continue
    if ! grep -q '^id:' "$f" || ! grep -q '^type:' "$f" || ! grep -q '^evidence:' "$f"; then
        echo "  ⚠ Malformed artifact: $(basename "$f") (missing id/type/evidence)"
        BAD_ARTIFACTS=$((BAD_ARTIFACTS + 1))
    fi
done
if [[ "$BAD_ARTIFACTS" -eq 0 ]]; then
    echo "  ✓ All artifacts have required schema fields"
    CHECKED=$((CHECKED + 1))
    PASSED=$((PASSED + 1))
fi
echo ""

# ── 10. Contract checksum freshness ──────────────────
echo "== Contract Checksums =="
if [[ -f .contract.sha256 ]]; then
    if shasum -a 256 -c .contract.sha256 >/dev/null 2>&1; then
        echo "  ✓ Contract checksum valid"
        CHECKED=$((CHECKED + 1))
        PASSED=$((PASSED + 1))
    else
        echo "  ✗ Contract checksum MISMATCH"
        FAIL=1
        CHECKED=$((CHECKED + 1))
    fi
fi
if [[ -f .contract-deps.sha256 ]]; then
    if shasum -a 256 -c .contract-deps.sha256 >/dev/null 2>&1; then
        echo "  ✓ Contract deps checksum valid"
        CHECKED=$((CHECKED + 1))
        PASSED=$((PASSED + 1))
    else
        echo "  ✗ Contract deps checksum MISMATCH"
        FAIL=1
        CHECKED=$((CHECKED + 1))
    fi
fi
echo ""

# ── Summary ──────────────────────────────────────────
echo "══════════════════════════════════════════════════"
echo "  Verified: $PASSED/$CHECKED claims passed"
if [[ "$WARN" -gt 0 ]]; then
    echo "  Warnings: $WARN"
fi
if [[ "$FAIL" -ne 0 ]]; then
    echo "  RESULT: CLAIMS FAILED ✗"
    echo "  STATE.md contains stale or incorrect numeric claims."
    echo "  Update STATE.md before relying on it for decisions."
    exit 1
else
    echo "  RESULT: ALL CLAIMS VERIFIED ✓"
fi
echo "══════════════════════════════════════════════════"
