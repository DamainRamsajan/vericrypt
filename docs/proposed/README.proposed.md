# VeriCrypt

Cryptographic inventory, post-quantum exposure findings and **offline-verifiable evidence**.

VeriCrypt scans certificates and keys (files and live TLS endpoints), classifies them, applies policy rules (hygiene, CA/Browser Forum lifetimes, NIST IR 8547 post-quantum transition), forecasts certificate-renewal burden, and writes a signed report plus a CycloneDX 1.7 cryptographic bill of materials. Anyone can verify the report offline.

> Status: **v2 evidence engine.** See `CLAIMS.md` for exactly what is tested and what is not implemented.

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
- Parses PEM/DER certificates and PKCS#1/PKCS#8/SEC1/SPKI keys; records everything it skipped and why.
- Classifies RSA, EC, Ed25519/X25519, DSA, DH and NIST post-quantum algorithms.
- Findings: expiry, weak keys, SHA-1/MD5 signatures, plaintext private keys, over-long certificate lifetimes, quantum-vulnerable algorithms.
- Reports signed with SLH-DSA-SHAKE-128f (FIPS 205 algorithm); Merkle commitment over every asset and finding; verification pins the signer; unsigned reports fail.
- `diff` between two scans with CI exit codes; `prove` / `verify-proof` for Merkle inclusion.

## What it does not do yet
Deep TLS analysis (protocols, ciphers, post-quantum groups), revocation and chain checks, keystores, SSH, code scanning, dashboards, connectors. It is **not** a compliance certification. Legacy `scan` is deprecated.

## Documentation
- `docs/architecture/` — arc42 architecture (v4 approved, v5 draft), competitive analysis, users and use cases
- `docs/PROJECT_STATUS.md` — status and roadmap
- `CLAIMS.md` — claims registry

Licence: to be decided.
