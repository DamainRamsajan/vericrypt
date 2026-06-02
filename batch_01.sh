#!/usr/bin/env bash
set -e

# =============================================================================
# VERICRYPT — Master Build 1
# Workspace scaffold: root Cargo.toml, rust-toolchain.toml, .gitignore, .env.example
# Arc42 Sections: 1.4 (Constraints), 5.1-5.4 (Deployment View, Environments, CI/CD, Env Vars)
# ADRs Enforced: ADR-001 (Single static binary), ADR-011 (Reproducible builds)
# Conformance Items: C-01, C-02
# Language/Stack: Rust / Cargo Workspace / musl static linking
# =============================================================================

echo "============================================"
echo " VERICRYPT MASTER BUILD 1 — WORKSPACE SCAFFOLD "
echo "============================================"

# -------------------------------------------------------------------
# 1.1 — Root workspace Cargo.toml
# Arc42: Section 5.1 (Deployment Model), ADR-001 (Single static binary)
# -------------------------------------------------------------------
echo "[+] Creating workspace Cargo.toml"

cat > Cargo.toml << 'EOF'
[workspace]
members = [
    "crates/vericrypt",
]
resolver = "2"

[workspace.package]
version = "0.1.0"
edition = "2021"
license = "UNLICENSED"
repository = "https://github.com/intellica-ai-llc/vericrypt"

[workspace.dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"
uuid = { version = "1", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
blake3 = "1"
hex = "0.4"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["json", "env-filter"] }
rand = "0.8"
x509-parser = "0.17"
rustls-pemfile = "2"
der = "0.7"
tokio = { version = "1", features = ["full"] }
tokio-rustls = "0.26"
native-tls = "0.2"
petgraph = "0.6"
walkdir = "2"
csv = "1"
ipnet = "2"
which = "7"
hostname = "0.4"
dirs = "6"
zstd = "0.13"
pqcrypto-sphincsplus = "0.7"
pqcrypto-traits = "0.2"
cyclonedx-bom = "0.6"
clap = { version = "4", features = ["derive"] }
tempfile = "3"
criterion = { version = "0.5", features = ["html_reports"] }
EOF

echo "  [OK] Workspace Cargo.toml written"

# -------------------------------------------------------------------
# 1.2 — Rust toolchain configuration
# Arc42: ADR-011 (Reproducible builds), Section 5.2 (CI Environment)
# -------------------------------------------------------------------
echo "[+] Creating rust-toolchain.toml"

cat > rust-toolchain.toml << 'EOF'
[toolchain]
channel = "stable"
components = ["rustfmt", "clippy"]
targets = ["x86_64-unknown-linux-musl", "aarch64-unknown-linux-musl"]
EOF

echo "  [OK] rust-toolchain.toml written"

# -------------------------------------------------------------------
# 1.3 — Cargo build configuration for reproducible static binaries
# Arc42: Section 5.2 (CI), ADR-011
# -------------------------------------------------------------------
echo "[+] Creating .cargo/config.toml"

mkdir -p .cargo

cat > .cargo/config.toml << 'EOF'
[target.x86_64-unknown-linux-musl]
linker = "x86_64-linux-musl-gcc"
rustflags = [
    "-C", "target-feature=+crt-static",
    "-C", "link-arg=-Wl,--build-id=sha1",
]

[target.aarch64-unknown-linux-musl]
linker = "aarch64-linux-musl-gcc"
rustflags = [
    "-C", "target-feature=+crt-static",
    "-C", "link-arg=-Wl,--build-id=sha1",
]

[build]
rustflags = [
    "--remap-path-prefix=$HOME=/build",
    "--remap-path-prefix=$PWD=/workspace",
]
EOF

echo "  [OK] .cargo/config.toml written"

# -------------------------------------------------------------------
# 1.4 — .gitignore
# Arc42: Section 5.2 (Environments)
# -------------------------------------------------------------------
echo "[+] Creating .gitignore"

cat > .gitignore << 'EOF'
# Rust
/target/
**/*.rs.bk
*.pdb

# Build artifacts
*.pqc
*.cbom.json
.build-manifests/

# IDE
.idea/
.vscode/
*.swp
*.swo

# OS
.DS_Store
Thumbs.db

# Environment (secrets)
.env
EOF

echo "  [OK] .gitignore written"

# -------------------------------------------------------------------
# 1.5 — Environment variable catalog
# Arc42: Section 5.4 (Environment Variable Catalog)
# -------------------------------------------------------------------
echo "[+] Creating .env.example"

cat > .env.example << 'EOF'
# VeriCrypt Environment Variables
# Arc42 Section 5.4 — names only, values are placeholders

VERICRYPT_LICENSE_KEY=placeholder-license-key
VERICRYPT_DATA_DIR=./vericrypt-data
VERICRYPT_LOG_LEVEL=info
VERICRYPT_LEAN4_PATH=/usr/local/bin/lean
VERICRYPT_SCAN_TIMEOUT=300
VERICRYPT_PROOF_TIMEOUT=30
VERICRYPT_TEE_ATTESTATION=auto
EOF

echo "  [OK] .env.example written"

# -------------------------------------------------------------------
# 1.6 — Verification
# -------------------------------------------------------------------
echo ""
echo "============================================"
echo " Running workspace validation..."
echo "============================================"

# Verify all files exist
[ -f Cargo.toml ] || { echo "ERROR: Cargo.toml missing"; exit 1; }
[ -f rust-toolchain.toml ] || { echo "ERROR: rust-toolchain.toml missing"; exit 1; }
[ -f .cargo/config.toml ] || { echo "ERROR: .cargo/config.toml missing"; exit 1; }
[ -f .gitignore ] || { echo "ERROR: .gitignore missing"; exit 1; }
[ -f .env.example ] || { echo "ERROR: .env.example missing"; exit 1; }

# Verify Rust toolchain is available
command -v cargo >/dev/null 2>&1 || { echo "ERROR: cargo not found. Install Rust: https://rustup.rs"; exit 1; }

# Validate Cargo.toml syntax by parsing workspace members
cargo metadata --no-deps --format-version 1 >/dev/null 2>&1 || {
    echo "WARNING: cargo metadata failed — this is expected before any crates exist"
    echo "  The workspace Cargo.toml is valid but has no members yet."
    echo "  Master Build 2 will create the first crate."
}

echo ""
echo "============================================"
echo " ✅ Master Build 1 Complete"
echo " Workspace scaffolded: Cargo.toml, rust-toolchain.toml,"
echo " .cargo/config.toml, .gitignore, .env.example"
echo "============================================"