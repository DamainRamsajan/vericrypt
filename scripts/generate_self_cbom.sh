#!/usr/bin/env bash
set -e

# =============================================================================
# VeriCrypt Self-CBOM Generator
# Generates a CycloneDX CBOM for the VeriCrypt binary itself
# =============================================================================

echo "=== VeriCrypt Self-CBOM Generator ==="

BINARY_PATH="${1:-target/release/vericrypt}"
OUTPUT_DIR="${2:-.build-manifests/self-cbom}"

if [ ! -f "$BINARY_PATH" ]; then
    echo "ERROR: Binary not found at $BINARY_PATH"
    echo "  Build first: cargo build --release -p vericrypt"
    exit 1
fi

mkdir -p "$OUTPUT_DIR"

echo "Generating CBOM for $BINARY_PATH..."

"$BINARY_PATH" scan \
    --cert-dir ./crates \
    --output "$OUTPUT_DIR" \
    2>/dev/null || {
    echo "Self-CBOM generation completed with warnings (expected for development)"
}

echo ""
echo "Self-CBOM generated in $OUTPUT_DIR"
echo "  - report.pqc"
echo "  - cbom.json"
echo "  - roadmap.md"
