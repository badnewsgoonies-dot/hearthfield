#!/usr/bin/env bash
set -euo pipefail

# Hearthfield — Unified Gate Validation Pipeline
# Run after every worker completion to verify correctness.
# All gates must pass. Non-negotiable.

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

FAIL=0

echo "══════════════════════════════════════════════════"
echo "  Hearthfield Gate Validation"
echo "══════════════════════════════════════════════════"
echo ""

# ── Gate 1: Contract Integrity (expanded) ─────────────
echo "== Gate 1: Contract Integrity =="
if shasum -a 256 -c .contract.sha256; then
    echo "  ✓ Contract checksum matches"
else
    echo "  ✗ CONTRACT DRIFT DETECTED — stop and restore"
    FAIL=1
fi
# Check re-exported modules are also covered
if [[ -f .contract-deps.sha256 ]]; then
    if shasum -a 256 -c .contract-deps.sha256; then
        echo "  ✓ Contract dependencies checksum matches"
    else
        echo "  ✗ CONTRACT DEPENDENCY DRIFT — re-exported module changed without checksum update"
        FAIL=1
    fi
fi
echo ""

# ── Gate 2: Type Check ──────────────────────────────
echo "== Gate 2: Type Check (cargo check) =="
if cargo check 2>&1; then
    echo "  ✓ Type check passed"
else
    echo "  ✗ Type check FAILED"
    FAIL=1
fi
echo ""

# ── Gate 3: Integration Tests ───────────────────────
echo "== Gate 3: Integration Tests (cargo test --test headless) =="
if cargo test --test headless 2>&1; then
    echo "  ✓ Integration tests passed"
else
    echo "  ✗ Integration tests FAILED"
    FAIL=1
fi
echo ""

# ── Gate 4: Lint Gate ───────────────────────────────
echo "== Gate 4: Lint Gate (cargo clippy) =="
if cargo clippy -- -D warnings 2>&1; then
    echo "  ✓ Clippy passed (zero warnings)"
else
    echo "  ✗ Clippy FAILED"
    FAIL=1
fi
echo ""

# ── Gate 5: Connectivity Check (auto-discovered) ───
echo "== Gate 5: Connectivity Check (no hermetic domains) =="
HERMETIC=0
# Auto-discover domains: all directories under src/ except shared/ itself
for d in src/*/; do
    domain=$(basename "$d")
    # shared is the contract, not a consumer
    [[ "$domain" == "shared" ]] && continue
    if [ -d "$d" ]; then
        if ! grep -r --include="*.rs" -q "crate::shared" "$d"; then
            echo "  ✗ HERMETIC: $d has no shared contract import"
            HERMETIC=1
        fi
    fi
done
if [ "$HERMETIC" -eq 0 ]; then
    echo "  ✓ All domains import from shared contract"
else
    echo "  ✗ Connectivity check FAILED: hermetic domains detected"
    FAIL=1
fi
echo ""

# ── Gate 6: STATE.md Freshness ──────────────────────
echo "== Gate 6: STATE.md Freshness =="
STATE_STALE=0
if [[ -f .memory/STATE.md ]]; then
    # Get HEAD commit hash
    CURRENT_HEAD=$(git rev-parse --short HEAD)
    # Get the HEAD recorded in STATE.md
    STATE_HEAD=$(grep -o 'HEAD:\*\* [a-f0-9]*' .memory/STATE.md 2>/dev/null | grep -o '[a-f0-9]\{7,\}' || echo "MISSING")

    if [[ "$STATE_HEAD" == "MISSING" ]]; then
        echo "  ⚠ STATE.md has no HEAD reference"
        STATE_STALE=1
    else
        # Check if STATE_HEAD is an ancestor of current HEAD (and not HEAD itself)
        if git merge-base --is-ancestor "$STATE_HEAD" HEAD 2>/dev/null; then
            COMMITS_BEHIND=$(git rev-list --count "${STATE_HEAD}..HEAD" 2>/dev/null || echo "?")
            if [[ "$COMMITS_BEHIND" -gt 0 ]]; then
                echo "  ⚠ STATE.md is $COMMITS_BEHIND commit(s) behind HEAD ($CURRENT_HEAD)"
                STATE_STALE=1
            else
                echo "  ✓ STATE.md references current HEAD"
            fi
        else
            echo "  ⚠ STATE.md HEAD ($STATE_HEAD) is not an ancestor of current HEAD ($CURRENT_HEAD)"
            STATE_STALE=1
        fi
    fi

else
    echo "  ✗ STATE.md not found"
    STATE_STALE=1
fi

if [ "$STATE_STALE" -eq 1 ]; then
    echo "  ⚠ STATE.md is stale (warning — does not block gate)"
fi
echo ""

# ── Gate 7: Artifact Source Refs (spot check) ──────
echo "== Gate 7: Artifact Source Refs =="
ARTIFACT_WARN_FILE=$(mktemp)
echo "0" > "$ARTIFACT_WARN_FILE"
for artifact in .memory/*.yaml; do
    [[ -f "$artifact" ]] || continue
    grep 'file:' "$artifact" 2>/dev/null | while IFS= read -r line; do
        filepath=$(echo "$line" | sed -n 's/.*@\([^:"]*\).*/\1/p')
        if [[ -n "$filepath" ]] && [[ ! -f "$filepath" ]]; then
            echo "  ⚠ $(basename "$artifact"): '$filepath' not found"
            echo "1" > "$ARTIFACT_WARN_FILE"
        fi
    done || true
done
ARTIFACT_WARN=$(cat "$ARTIFACT_WARN_FILE")
rm -f "$ARTIFACT_WARN_FILE"
if [ "$ARTIFACT_WARN" -eq 0 ]; then
    echo "  ✓ All artifact file refs resolve"
else
    echo "  ⚠ Some artifact refs broken (warning only)"
fi
echo ""

# ── Gate 8: STATE.md Claim Verification (spot check) ─
echo "== Gate 8: STATE.md Claim Verification =="
CLAIM_WARN=0
if [[ -f scripts/verify-state-claims.sh ]]; then
    # Run claim verifier, capture output but don't block the gate
    CLAIM_OUTPUT=$(bash scripts/verify-state-claims.sh 2>&1 || true)
    CLAIM_FAILURES=$(echo "$CLAIM_OUTPUT" | grep -c '✗' || true)
    if [[ "$CLAIM_FAILURES" -gt 0 ]]; then
        echo "$CLAIM_OUTPUT" | grep -E '(✗|✓|⚠|Verified:)' | head -20
        echo "  ⚠ $CLAIM_FAILURES claim(s) failed verification (warning — does not block gate)"
        CLAIM_WARN=1
    else
        echo "  ✓ All STATE.md claims verified against code"
    fi
else
    echo "  ⚠ verify-state-claims.sh not found"
    CLAIM_WARN=1
fi
echo ""

# ── Summary ─────────────────────────────────────────
echo "══════════════════════════════════════════════════"
if [ "$FAIL" -eq 0 ]; then
    echo "  ALL GATES PASSED ✓"
    if [ "$STATE_STALE" -eq 1 ] || [ "$ARTIFACT_WARN" -eq 1 ] || [ "$CLAIM_WARN" -eq 1 ]; then
        echo "  (warnings present — review before shipping)"
    fi
else
    echo "  GATES FAILED ✗ — fix before proceeding"
    exit 1
fi
echo "══════════════════════════════════════════════════"
