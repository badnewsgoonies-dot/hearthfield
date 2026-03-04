#!/usr/bin/env bash
set -euo pipefail

# Hearthfield — Unified Gate Validation Pipeline
# Run after every worker completion to verify correctness.
# All four gates must pass. Non-negotiable.

FAIL=0

echo "══════════════════════════════════════════════════"
echo "  Hearthfield Gate Validation"
echo "══════════════════════════════════════════════════"
echo ""

# ── Gate 1: Contract Integrity ──────────────────────
echo "== Gate 1: Contract Integrity =="
if shasum -a 256 -c .contract.sha256; then
    echo "  ✓ Contract checksum matches"
else
    echo "  ✗ CONTRACT DRIFT DETECTED — stop and restore"
    FAIL=1
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

# ── Gate 5: Connectivity Check ──────────────────────
echo "== Gate 5: Connectivity Check (no hermetic domains) =="
HERMETIC=0
for d in src/animals/ src/calendar/ src/crafting/ src/data/ src/economy/ \
         src/farming/ src/fishing/ src/input/ src/mining/ src/npcs/ \
         src/player/ src/save/ src/ui/ src/world/; do
    if [ -d "$d" ]; then
        # Check that at least one .rs file imports from crate::shared
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

# ── Summary ─────────────────────────────────────────
echo "══════════════════════════════════════════════════"
if [ "$FAIL" -eq 0 ]; then
    echo "  ALL GATES PASSED ✓"
else
    echo "  GATES FAILED ✗ — fix before proceeding"
    exit 1
fi
echo "══════════════════════════════════════════════════"
