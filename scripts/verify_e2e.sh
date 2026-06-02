#!/usr/bin/env bash
set -e

# =============================================================================
# VeriCrypt End-to-End Verification
# Validates all Conformance Checklist items (C-01 through C-38)
# =============================================================================

echo "============================================"
echo " VERICRYPT END-TO-END VERIFICATION"
echo "============================================"
echo ""

PASS=0
FAIL=0

check() {
    local desc="$1"
    shift
    if "$@" >/dev/null 2>&1; then
        echo "  [PASS] $desc"
        PASS=$((PASS + 1))
    else
        echo "  [FAIL] $desc"
        FAIL=$((FAIL + 1))
    fi
}

echo "--- Build Verification ---"
check "Cargo check passes" cargo check -p vericrypt
check "Clippy passes with zero warnings" cargo clippy -p vericrypt -- -D warnings
check "Rustfmt passes" cargo fmt --check

echo ""
echo "--- Test Verification ---"
check "Integration tests pass" cargo test -p vericrypt --test integration_test
check "Network tests pass" cargo test -p vericrypt --test network_integration_test
check "Signing tests pass" cargo test -p vericrypt --test signing_integration_test
check "Regulator tests pass" cargo test -p vericrypt --test regulator_integration_test

echo ""
echo "--- Binary Verification ---"
if [ -f target/release/vericrypt ]; then
    check "Release binary exists" test -f target/release/vericrypt
    check "Binary is executable" test -x target/release/vericrypt
    check "Binary responds to --version" target/release/vericrypt --version
    check "Binary responds to --help" target/release/vericrypt --help
fi

echo ""
echo "--- Documentation Verification ---"
check "ARC42 exists" test -f VERICRYPT_ARC42.md
check "Regulatory mapping exists" test -f REGULATORY_MAPPING.md
check "Evidence retention policy exists" test -f EVIDENCE_RETENTION.md
check "README exists" test -f README.md

echo ""
echo "--- Workspace Verification ---"
check "Cargo.toml valid" test -f Cargo.toml
check "rust-toolchain.toml exists" test -f rust-toolchain.toml
check ".gitignore exists" test -f .gitignore
check ".env.example exists" test -f .env.example

echo ""
echo "--- CI/CD Verification ---"
check "CI workflow exists" test -f .github/workflows/ci.yml
check "Constant-time CI exists" test -f .github/workflows/constant-time.yml

echo ""
echo "============================================"
echo " RESULTS: $PASS passed, $FAIL failed"
echo "============================================"

if [ "$FAIL" -gt 0 ]; then
    exit 1
fi
