# Reference

## Commands

| Command | What it does |
| --- | --- |
| `vericrypt keygen --out <file>` | Create an SLH-DSA signing key |
| `vericrypt analyze -i <dir> --out <dir>` | Scan certificates and keys, write the report, CBOM and HTML |
| `vericrypt analyze --tls host:port` | Probe a live TLS endpoint (leaf certificate only) |
| `vericrypt diff <old> <new> --fail-on <level>` | Compare two reports; exit 3 on new findings at or above the level |
| `vericrypt verify <report> --pubkey <file>` | Verify a signed report. Unsigned reports fail unless `--allow-unsigned` |
| `vericrypt-verify` | Standalone verifier binary |

## What is covered

The project keeps a claims registry (`CLAIMS.md`): a capability is described only if an automated test covers it or it was checked by hand. Covered today:

- Key classification (RSA, EC, Ed25519, ML-DSA) and findings on real properties
- NIST IR 8547 (draft) transition years and the CA/Browser Forum lifetime schedule (398, 200, 100, 47 days)
- Deterministic Merkle root, canonical JSON, inclusion proofs
- SLH-DSA-SHAKE-128f signing, key pinning, tamper detection
- `diff` and `--fail-on`
- CycloneDX 1.7 CBOM, validated against the official schema

## Not available yet

These are on the roadmap and not implemented: formal compliance proofs, hardware (TEE) attestation, revocation checking, chain building, TLS version and cipher enumeration, PKCS#12 and Java keystores, SSH, code scanning, hybrid signatures, trusted timestamps, dashboards and connectors. VeriCrypt produces an observation-based inventory and findings, not a compliance certification.
