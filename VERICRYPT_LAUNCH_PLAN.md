What the ARC42 and Addendums Promise vs. What We Have
Binary: We have vericrypt compiling. But we need both vericrypt and vericrypt-verify as separate artifacts with SHA256 checksums for each.

License system: ADR-006 specifies PASETO v4 capability tokens scoped to binary hash. The code has the activate function but it accepts any string. We need real token generation, signing, and verification.

PKI hierarchy: Addendum 3 specifies Root Verity Authority Key → Customer License Certificate → Report Signing Key. The code has build_certificate_chain() returning a hardcoded entry.

Offline revocation: ADR-015 specifies offline revocation bundles distributed with each binary release. The code has is_certificate_revoked() returning false unconditionally.

TEE attestation: Section 3.10 specifies Intel TDX and AMD SEV-SNP with firmware version tracking and CVE checking. The code collects attestation but doesn't track firmware versions or known CVEs.

Inventory confidence: Addendum 2 specifies dynamic confidence scoring based on endpoint coverage, subnet coverage, cert transparency correlation, etc. The code calls compute_inventory_confidence with hardcoded zeros.

Stage timing: Addendum 3 specifies per-stage elapsed time reporting. The code records timings but the output doesn't display them.

Compliance confidence display: Addendum 3 specifies displaying P × I × R in the CLI output. The code computes it but the CLI only shows violations count.

Three-phase deployment: Addendum 2 specifies Shadow, Parallel, Primary modes. The CLI has the flag but the output doesn't change behavior based on mode.

CMAP/PQCMM dual scoring: Addendum 2 specifies both maturity models in the roadmap. The code generates phases but doesn't include the scores.

Theorem pack import: ADR-017 specifies --load-theorems for signed axiom packs. The CLI has --load-bytecode instead.

STH export: ADR-018 specifies VeriChain anchoring with --publish-sth. The code has this.

Self-CBOM: The CI should run VeriCrypt on its own binary to generate a CBOM for supply chain verification.

Reproducible builds: ADR-011 specifies deterministic builds with SLSA provenance. We have the Cargo config but no provenance generation.

Evidence retention policy: Addendum 3 specifies documented retention. We have the EVIDENCE_RETENTION.md file.

Regulatory mapping documentation: We have REGULATORY_MAPPING.md.

CI/CD pipeline: We have a minimal build workflow. We need the full pipeline from the ARC42 Section 5.3: cross-compilation for x86_64 and ARM64 musl, minisign signing (replaced by SLH-DSA per ADR-011), and release publishing.

Web presence: The Verity site has landing, features, pricing, download, docs. VeriCrypt needs the same.

Business plan: We wrote it. It needs to be published as part of the site.

The Complete Path to Launch
Phase 1: Complete the CI/CD pipeline.

Add vericrypt-verify to the build and upload

Add SHA256 checksum generation for both binaries

Add cross-compilation for x86_64 and ARM64 musl

Add self-CBOM generation step

Add release publishing to GitHub Releases with changelog

Phase 2: Close the remaining code gaps.

Wire inventory confidence to actual scan statistics

Display compliance confidence in CLI output

Display stage timings in verbose mode

Add CMAP/PQCMM scores to roadmap output

Implement three-phase deployment mode behavior (Shadow mode marks reports as not for submission)

Rename --load-bytecode to --load-theorems per the spec

Implement PKI certificate chain properly (even if root key is embedded for v0.1.0)

Implement offline revocation bundle parsing

Add TEE firmware version and CVE tracking to attestation

Phase 3: License management.

Build the PASETO v4 token generation service (same pattern as Verity)

Token includes: binary_hash, expiry, customer_id, tier

Email delivery with download instructions

Activation flow tested end-to-end

Phase 4: Web presence.

Landing page with hero, architecture, trust signals

Features page with competitive matrix

Pricing page with three tiers

Download page with checksums and license activation

Implementation manual

User manual

Business plan page

Partner/investor page

Phase 5: Pre-launch verification.

Run the full conformance checklist (C-01 through C-49)

Test with real certificate data from a bank

Verify regulator workflow end-to-end (scan → report → verify)

Penetration test the binary

Third-party security audit

Phase 6: Launch.

Publish binaries on vericrypt.io

Activate license management

Send first customer email