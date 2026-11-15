#!/usr/bin/env bash
# End-to-end check of the licence flow with a throw-away key pair. Run from the repository root:  bash scripts/verify_licence_e2e.sh
set -u
fail=0; pass(){ echo "PASS  $1"; }; bad(){ echo "FAIL  $1"; fail=1; }
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
export VERICRYPT_DATA_DIR="$T/data"
cargo build -q -p vericrypt --features issuer --bin vericrypt-license || { echo "cannot build issuer"; exit 2; }
L=target/debug/vericrypt-license
$L keygen --out "$T/k" >/dev/null && $L keygen --out "$T/other" >/dev/null
VERICRYPT_PUBKEY_FILE="$T/k/paseto_public_key.hex" cargo build -q -p vericrypt --features license-gate --bin vericrypt || { echo "cannot build gated binary"; exit 2; }
V=target/debug/vericrypt; FIX=crates/vericrypt/tests/fixtures/pki
GOOD=$($L issue --private-key "$T/k/paseto_private_key.hex" --org Pilot --email p@x.io 2>/dev/null)
OLD=$($L issue --private-key "$T/k/paseto_private_key.hex" --org Old --email o@x.io --days=-2 2>/dev/null)
OTHER=$($L issue --private-key "$T/other/paseto_private_key.hex" --org Evil --email e@x.io 2>/dev/null)
$V keygen --out "$T/sign.json" >/dev/null
$V analyze -i $FIX --out "$T/a" --no-html --sign-key "$T/sign.json" >/dev/null 2>&1; [ $? -eq 4 ] && pass "signing refused without a licence" || bad "signing refused without a licence"
$V analyze -i $FIX --out "$T/b" --no-html >/dev/null 2>&1; [ $? -eq 0 ] && pass "free features work without a licence" || bad "free features work without a licence"
$V activate --key "$OLD" >/dev/null 2>&1; [ $? -ne 0 ] && pass "expired licence refused" || bad "expired licence refused"
$V activate --key "$OTHER" >/dev/null 2>&1; [ $? -ne 0 ] && pass "licence from another key refused" || bad "licence from another key refused"
mkdir -p "$T/data/license"; echo '{"token":"v4.public.FAKE","expiry":"2099-01-01T00:00:00Z"}' > "$T/data/license/license.json"
$V analyze -i $FIX --out "$T/c" --no-html --sign-key "$T/sign.json" >/dev/null 2>&1; [ $? -eq 4 ] && pass "forged license.json does not unlock" || bad "forged license.json does not unlock"
rm -rf "$T/data"
$V activate --key "$GOOD" >/dev/null 2>&1; [ $? -eq 0 ] && pass "genuine licence activates" || bad "genuine licence activates"
$V analyze -i $FIX --out "$T/d" --sign-key "$T/sign.json" >/dev/null 2>&1; [ $? -eq 0 ] && pass "licensed: signed report and HTML written" || bad "licensed: signed report and HTML written"
$V verify "$T/d/report.json" --pubkey "$T/sign.pub.json" >/dev/null 2>&1; [ $? -eq 0 ] && pass "signed report verifies with pinned key" || bad "signed report verifies with pinned key"
[ $fail -eq 0 ] && echo "ALL LICENCE CHECKS PASSED" || echo "SOME CHECKS FAILED"; exit $fail
