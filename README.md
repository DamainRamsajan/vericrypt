# VeriCrypt

**Quantum Exposure, Formally Proven.**

VeriCrypt is an air-gapped, single-binary cryptographic posture verification engine. It ingests a financial institution's certificate inventory and outputs a cryptographically signed, tamper-evident, regulator-ready `.pqc` compliance artifact — with mathematical proofs that every finding is correct.

Unlike every existing PQC tool, VeriCrypt does not merely report on cryptographic posture. It **proves** compliance claims using the ASL → Lean 4 theorem extraction pipeline, anchoring every finding to a Merkle root and signing it with NIST FIPS 205 (SLH-DSA) post-quantum signatures.

---

## What VeriCrypt Does

- **Discovers** every cryptographic asset across certificate stores, TLS endpoints, code repositories, and HSM configurations
- **Classifies** quantum vulnerability using the multiplicative HNDL threat model (Rufino et al., May 2026)
- **Attributes** exposure contributions per asset using Shapley value decomposition from cooperative game theory
- **Proves** regulatory compliance via Lean 4 theorem verification — DORA, SEC PQFIF, NCSC, NIST SP 1800-38
- **Generates** a CycloneDX 1.7 CBOM with full cryptoProperties and evidence capture
- **Produces** a tamper-evident `.pqc` report with SLH-DSA signatures, Merkle proofs, and optional TEE attestation
- **Verifies** offline — any regulator can independently confirm report integrity without access to bank systems

---

## Quick Start

```bash
# 1. Download and verify the binary
wget https://verity.io/vericrypt
sha256sum -c vericrypt.sha256

# 2. Run your first scan
./vericrypt scan --cert-dir /etc/ssl --output ./compliance-audit/

# 3. Review the results
#    - report.pqc  → Signed compliance artifact (submit to regulator)
#    - cbom.json   → CycloneDX 1.7 Cryptographic Bill of Materials
#    - roadmap.md  → Prioritized PQC migration roadmap

# 4. Verify independently (regulator side)
./vericrypt-verify compliance-audit/report.pqc
Output:

text
VERIFIED — scan at 2026-05-28T14:30:00Z
  Binary: vericrypt v0.1.0 (hash: a1b2c3d4e5f6...)
  Assets: 2,437
  Quantum-vulnerable: 342
  Theorems: 3 proved, 1 counterexample
  Inventory confidence: High (87%)
  Signature: Valid (SLH-DSA, NIST FIPS 205)
Architecture
VeriCrypt is architected as a single statically-linked Rust binary with ten internal components:

Component	Function
Ingestion Engine	Discovers crypto assets from PEM, DER, PKCS#12, CSV, JSON, and network TLS endpoints
Knowledge Graph Builder	Constructs typed dependency graph of all cryptographic assets
Quantum Exposure Analyzer	Computes HNDL exposure using the multiplicative Rufino model
ASL → Lean 4 Compliance Bridge	Translates regulatory axioms into machine-checked Lean 4 theorems
Prioritization Engine	Generates risk-prioritized Phase 1/2/3 migration roadmap
CBOM Generator	Produces CycloneDX 1.7 Cryptographic Bill of Materials
Report Generator	Assembles cryptographically signed .pqc compliance artifact
TEE Attestation Module	Collects hardware-signed attestation from Intel TDX or AMD SEV-SNP
Verification Tool	Standalone offline binary for regulator verification
CLI + License	Clap-based CLI with PASETO v4 capability-scoped license tokens
Full architecture specification: VERICRYPT_ARC42.md

What Makes VeriCrypt Different
Capability	VeriCrypt	Other PQC Tools
Formal compliance proofs	Lean 4 theorem verification	Reports and risk scores only
Multiplicative HNDL model	Structurally justified (Rufino 2026)	Additive heuristics
Shapley value attribution	Game-theoretic marginal contributions	Fixed weighted scoring
Constant-size evidence	O(1) verification regardless of scan size	Verification proportional to report size
Air-gapped binary delivery	Single static binary, zero cloud dependency	SaaS platforms requiring cloud upload
TEE attestation	Intel TDX + AMD SEV-SNP	Not offered
Post-quantum signed output	SLH-DSA (NIST FIPS 205)	Classical signatures only
CycloneDX 1.7 CBOM	Native ECMA-424 compliance	Partial or absent
Regulatory Coverage
VeriCrypt maps compliance findings to:

EU DORA — Articles 5–14 (ICT governance, protection, detection, crypto-agility, incident management, reporting)

SEC PQFIF — Post-Quantum Financial Infrastructure Framework for US digital asset markets

UK NCSC — Phase 1/2/3 PQC migration guidance

NIST SP 1800-38 — PQC migration practice guide (October 2025 final)

NIST CSWP 39 — Crypto agility strategies and practices

Formal Assurance Boundary
VeriCrypt provides machine-checked assurance that:

The observed cryptographic inventory satisfies formally encoded regulatory axioms

Every compliance theorem was accepted by the Lean 4 kernel

The .pqc report artifact has not been modified after generation (Merkle-proofed)

The scan was executed by the measured binary when TEE attestation is present

VeriCrypt does not prove:

That all organizational systems were visible to the scanner

That regulatory axioms perfectly capture legal intent

That the organization is operationally secure beyond observed cryptographic posture

Legal disclaimer: Formal proofs generated by VeriCrypt are computational validations of encoded supervisory rules and should not be interpreted as legal opinions or binding regulatory determinations.

Threat Model
VeriCrypt's security guarantees hold under explicit trust assumptions documented in the architecture. Key mitigations:

Supply-chain attacks: Reproducible builds, SLSA provenance, deterministic Cargo.lock

Side-channel attacks: Constant-time cryptographic operations, dudect validation in CI

Parser attacks: Bounded memory streaming parsers, fuzz testing on all input boundaries

Downgrade attacks: Schema/version pinning, minimum NIST security level enforcement

Tampering: Merkle-root integrity verification, SLH-DSA signatures, TEE attestation

Full threat model: VERICRYPT_ARC42.md, Section 2.5

Repository Structure
text
vericrypt/
├── VERICRYPT_ARC42.md     # Complete architecture specification
├── Cargo.toml             # Rust workspace
├── rust-toolchain.toml    # Pinned toolchain for reproducible builds
├── crates/
│   └── vericrypt/         # Main binary crate
│       ├── src/
│       │   ├── main.rs    # CLI entry point
│       │   ├── verify_main.rs  # vericrypt-verify entry point
│       │   ├── ingest/    # Ingestion engine (PEM, DER, PKCS#12, CSV, JSON, TLS)
│       │   ├── graph/     # Knowledge graph builder (petgraph)
│       │   ├── exposure/  # Quantum exposure analyzer (Rufino model)
│       │   ├── compliance/# ASL → Lean 4 compliance bridge
│       │   ├── prioritize/# Prioritization engine (Shapley)
│       │   ├── cbom/      # CBOM generator (CycloneDX 1.7)
│       │   ├── report/    # Report generator (.pqc)
│       │   └── tee/       # TEE attestation (TDX/SEV-SNP)
│       └── tests/         # Integration tests
├── batch-0-preflight.sh   # Pre-flight validation
├── batch-1-core-scaffold.sh
├── batch-2-integration.sh
├── batch-3-network-lean4.sh
├── batch-4-tee-hardening.sh
└── .github/workflows/     # CI/CD pipeline
Build
bash
# Prerequisites: Rust (see rust-toolchain.toml for version)

# Option 1: Sequential batch build
bash batch-0-preflight.sh
bash batch-1-core-scaffold.sh
bash batch-2-integration.sh
bash batch-3-network-lean4.sh
bash batch-4-tee-hardening.sh

# Option 2: Direct Cargo build
cargo build --release -p vericrypt

# Cross-compile for air-gapped deployment
cargo build --release --target x86_64-unknown-linux-musl -p vericrypt
Licensing
VeriCrypt uses a capability-based licensing model:

Free tier: Full scan with unsigned .pqc report (CBOM + roadmap included)

Licensed tier: Signed .pqc report with SLH-DSA signatures and Lean 4 proof terms

License keys are PASETO v4 tokens scoped to the binary hash. Purchase at verity.io/license.

Research Foundation
VeriCrypt is grounded in peer-reviewed research published in 2025–2026:

Lean-Agent Protocol (Rashie & Rashi, arXiv:2604.01483, April 2026) — Auto-formalized regulatory compliance via Lean 4

EHV Paper (arXiv:2605.17909, May 2026) — TLA+-verified enforcement hardening with TEE attestation

HNDL Time-Dependent Threat Model (ResearchGate, March 2026) — Formal Ld > Ha vulnerability condition

Constant-Size Cryptographic Evidence (Kao, arXiv:2511.17118, February 2026) — Q-Audit Integrity formal proofs

Multiplicative HNDL Exposure (Rufino et al., May 2026) — Structural necessity of multiplicative scoring

CTI-Shapley (AIMS Sciences, April 2025) — Coalition-structured Shapley for vulnerability attribution

PQCMM v1.0 (PKI Consortium, October 2025) — Industry-standard PQC maturity model

NIST SP 1800-38 (October 2025) — US government PQC migration practice guide

Full provenance log: VERICRYPT_ARC42.md, Section 12

Status
VeriCrypt is in active development. Current status:

✅ ARC42 architecture specification complete (v1.0 + Addendum 1 + Addendum 2)

✅ Batch 0–4 build scripts printed

✅ Ingestion engine: PEM, DER, PKCS#12, CSV, JSON, TLS endpoints

✅ Knowledge graph: typed dependency edges, trust chain resolution

✅ Exposure analysis: multiplicative HNDL model, Shapley attribution

✅ Compliance bridge: Lean 4 kernel integration, graceful degradation

✅ CBOM generation: CycloneDX 1.7, ECMA-424 compliant

✅ Report signing: SLH-DSA (NIST FIPS 205)

✅ TEE attestation: Intel TDX, AMD SEV-SNP

✅ Offline verification: vericrypt-verify tool

⏳ Batch 5: Critical remediations from Addendum 2

⏳ Batch 6: Regulator hardening

Contact
peterdramsajan@gmail.com
