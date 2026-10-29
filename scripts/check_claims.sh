#!/usr/bin/env bash
# Claims check (workstream W8). Run from the repo root:  scripts/check_claims.sh [path-to-vericrypt-binary]
# 1. Every test named in CLAIMS.md section A must exist.
# 2. README.md and the CLI help must not use phrases that CLAIMS.md section C says are not claimable.
set -uo pipefail
BIN="${1:-target/release/vericrypt}"
fail=0

tests=$(cargo test -p vericrypt --lib --test engine_pipeline --test engine_cli -- --list 2>/dev/null)
if [ -z "$tests" ]; then echo "FAIL: could not list tests"; exit 1; fi

# Section A = lines between "## A." and "## B."; the last table column holds the test names in backticks.
names=$(sed -n '/^## A\./,/^## B\./p' CLAIMS.md | grep '^|' | awk -F'|' 'NF>=4 {print $3}' | grep -o '`[^`]*`' | tr -d '`')
count=0
for n in $names; do
  count=$((count+1))
  case "$n" in
    *'::*') key="${n%\*}" ;;   # wildcard like engine::classify::tests::*  -> prefix match
    *)      key="$n" ;;
  esac
  if ! printf '%s\n' "$tests" | grep -F -q -- "$key"; then
    echo "FAIL: CLAIMS.md names test '$n' but no such test exists"; fail=1
  fi
done
echo "checked $count test references from CLAIMS.md section A"

# Phrases that must not appear affirmatively in public text.
banned='Lean 4|formal(ly)? verif|mathematical(ly)? proof|\bTEE\b|\bSGX\b|vericrypt scan\b|compliance proof'
public="$(cat README.md; if [ -x "$BIN" ]; then "$BIN" --help; for c in analyze keygen verify diff prove verify-proof activate; do "$BIN" "$c" --help; done; fi)"
hits=$(printf '%s\n' "$public" | grep -n -i -E "$banned" || true)
if [ -n "$hits" ]; then echo "FAIL: banned claim wording found:"; echo "$hits"; fail=1; fi

[ "$fail" -eq 0 ] && echo "claims check passed"
exit "$fail"
