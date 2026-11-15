#!/usr/bin/env bash
# Build a licence-gated release and sign it with your post-quantum key.
# Usage:  bash scripts/make_release.sh /path/to/your-signing-key.json
# Output: dist/  (vericrypt, SHA256SUMS, vericrypt.sbom.cdx.json (software bill of materials), vericrypt.cbom.cdx.json
#                 (cryptographic bill of materials of the binary) and a *.sig.json for each)
# Options (environment): PROFILE=debug for a quick trial build; TARGET=x86_64-unknown-linux-musl for a static build.
set -euo pipefail
KEY="${1:?usage: bash scripts/make_release.sh /path/to/signing-key.json}"
PROFILE="${PROFILE:-release}"
FLAGS=(-p vericrypt --features license-gate --bin vericrypt)
[ "$PROFILE" = "release" ] && FLAGS+=(--release)
[ -n "${TARGET:-}" ] && FLAGS+=(--target "$TARGET")
cargo build "${FLAGS[@]}"
OUT="target/${TARGET:+$TARGET/}$([ "$PROFILE" = release ] && echo release || echo debug)"
rm -rf dist && mkdir dist
cp "$OUT/vericrypt" dist/vericrypt
python3 scripts/make_sbom.py > dist/vericrypt.sbom.cdx.json
python3 scripts/make_self_cbom.py > dist/vericrypt.cbom.cdx.json
(cd dist && sha256sum vericrypt vericrypt.sbom.cdx.json vericrypt.cbom.cdx.json > SHA256SUMS)
for f in vericrypt vericrypt.sbom.cdx.json vericrypt.cbom.cdx.json SHA256SUMS; do
  "$OUT/vericrypt" sign-file "dist/$f" --key "$KEY" >/dev/null
done
echo "Release files in dist/:"; ls -1 dist
echo "Give customers the files in dist/ and your PUBLIC key file (the .pub.json next to your signing key)."
echo "They check it with:  vericrypt verify-file vericrypt --pubkey your-key.pub.json"
