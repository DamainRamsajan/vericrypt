# VeriCrypt

Cryptographic inventory, post-quantum exposure findings and **offline-verifiable evidence**.

VeriCrypt scans certificates and keys (files and live TLS endpoints), classifies them, applies policy rules (hygiene, CA/Browser Forum lifetimes, the final NIST IR 8547 transition dates), forecasts certificate-renewal burden, and writes a signed report plus a CycloneDX 1.7 cryptographic bill of materials. Anyone can verify the report offline.

> **Status: pilot stage.** `CLAIMS.md` lists exactly what is tested, what was checked by hand, and what is **not** implemented.
> **Licence: proprietary, all rights reserved** (see `LICENSE`). Evaluation licence keys are issued to pilot customers.

## Quick start
```bash
cargo build --release -p vericrypt
./target/release/vericrypt keygen --out my-key.json
./target/release/vericrypt analyze -i crates/vericrypt/tests/fixtures/pki --out out \
    --public-issuers "Acme Public Test CA" --sign-key my-key.json
./target/release/vericrypt verify out/report.json --pubkey my-key.pub.json
```
Full instructions: `docs/INSTALL_V2.md`. Pilot licences and release packaging: `docs/PILOT_LICENCE_GUIDE.md`.

## What it does today
- Parses PEM/DER certificates and PKCS#1/PKCS#8/SEC1/SPKI keys; records everything it skipped and why.
- Classifies RSA, EC, Ed25519/X25519, DSA, DH and NIST post-quantum algorithms.
- Findings: expiry, weak keys, SHA-1/MD5 signatures, plaintext private keys, over-long certificate lifetimes, quantum-vulnerable algorithms (deprecated after 2030, disallowed after 2035 per the final NIST IR 8547).
- Reports signed with SLH-DSA-SHAKE-128f (a FIPS 205 algorithm); Merkle commitment over every asset and finding; verification pins the signer; unsigned reports fail.
- `diff` between two scans with CI exit codes; `prove` / `verify-proof` for Merkle inclusion.
- `sign-file` / `verify-file`: sign and verify any file (for example a release binary) with the same post-quantum key.
- Live probing: reads the certificate a TLS server presents, including through STARTTLS for SMTP, IMAP and PostgreSQL.
- Risk-decision register (waivers): accept a finding with an owner, a reason and an expiry. The finding stays visible, expired decisions re-open it, and the decisions are part of the signed report.
- Release files for customers: signed binary, software bill of materials and a cryptographic bill of materials of the binary itself. One self-contained binary: no OpenSSL.

## What it does not do yet
Deep TLS analysis (protocol versions, cipher suites, post-quantum key-exchange groups), STARTTLS for POP3 and FTP (implemented, untested), MySQL, revocation and chain checks, keystores, SSH, code scanning, dashboards, connectors. It is **not** a compliance certification.

## Documentation
- `docs/architecture/` — arc42 architecture, competitive analysis, users and use cases, research rounds
- `docs/PROJECT_STATUS.md` — status and roadmap
- `CLAIMS.md` — claims registry (CI checks it)
