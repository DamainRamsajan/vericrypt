# VeriCrypt

Cryptographic inventory, post-quantum exposure findings and **offline-verifiable evidence**.

VeriCrypt scans certificates and keys in files (and can probe the leaf certificate of a live TLS endpoint), classifies them, applies policy rules (hygiene, CA/Browser Forum lifetimes, NIST IR 8547 draft post-quantum transition), forecasts certificate-renewal burden, and writes a signed report plus a CycloneDX 1.7 cryptographic bill of materials (CBOM). A report can be verified offline with the signer's public key.

> Status: **v2 evidence engine.** `CLAIMS.md` lists exactly what is tested and what is not implemented. Anything not listed there is not claimed here.

## Quick start
```bash
cargo build --release -p vericrypt
./target/release/vericrypt keygen --out my-key.json
./target/release/vericrypt analyze -i crates/vericrypt/tests/fixtures/pki --out out \
    --public-issuers "Acme Public Test CA" --sign-key my-key.json
./target/release/vericrypt verify out/report.json --pubkey my-key.pub.json
```
Full instructions: `docs/INSTALL_V2.md`.

## What it does today
- Classifies RSA, EC, Ed25519 and ML-DSA keys (RSA-2048 is 112-bit classical security and quantum-vulnerable).
- Findings: weak keys, SHA-1 signatures, plaintext private keys, over-long certificate lifetimes, certificates that outlive 2035, quantum-vulnerable algorithms.
- Reports are signed with SLH-DSA-SHAKE-128f (the FIPS 205 algorithm). A Merkle root commits to every asset and finding. Verification can pin the signer's key. An unsigned report fails verification unless you pass `--allow-unsigned`.
- The same inputs, as-of time and parameters give the same Merkle root.
- `analyze --store <dir>` saves what a scan saw. `reevaluate --store <dir> --scan <id>` re-judges that saved scan at another date or with other parameters. A saved batch that has been changed is refused.
- `diff` compares two reports and can fail a CI job on new findings (exit code 3). `prove` and `verify-proof` give and check a Merkle inclusion proof for one asset or finding.

## What it does not do yet
Deep TLS analysis (protocol versions, ciphers, key-exchange groups, including post-quantum groups), revocation and certificate-chain checks, Java/PKCS#12 keystores, SSH, code scanning, dashboards, connectors, hardware attestation. It is **not** a compliance certification, and it makes no formal-proof claims. Only tiny test inputs have been run, so performance at scale is unknown. The legacy v1 `scan` command has been removed.

## Documentation
- `CLAIMS.md`: the claims registry
- `docs/PROJECT_STATUS.md`: status and roadmap
- `docs/architecture/`: arc42 architecture (v4 approved, v5 draft), competitive analysis, users and use cases

Licence: proprietary. Copyright (c) 2026 Intellica AI Inc. All rights reserved. See `LICENSE`. Third-party components: `THIRD_PARTY_NOTICES.md`.
