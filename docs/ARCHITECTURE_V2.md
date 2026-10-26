# VeriCrypt v2 — Evidence Engine Architecture Update

Status: implemented and tested in this patch (13 engine tests pass; CBOM validated against the official CycloneDX 1.7 schema; signing/verification, tamper detection, drift diff, Merkle proofs, TLS probe and license gate exercised end-to-end).

## 1. Why v2 exists (gaps found in v1 by reading and running the code)

| v1 behaviour (as built from the public repo) | v2 replacement |
|---|---|
| Did not compile: duplicated `span:` field, `seedc/src/parser.rs:124` | one-line fix |
| Headline "quantum-vulnerable" count printed the number of failed proofs (0 for a folder of RSA) | every count computed from the asset list |
| Ed25519 → "Unknown"; PKCS#8 private keys → "Generic / not vulnerable" | OID registry + PKCS#8/PKCS#1/SEC1 parsing; Ed25519/Ed448/X25519/X448/DSA/DH classified |
| Key sizes taken from public-key byte length | RSA modulus bits / EC curve parsed properly |
| "Compliance proofs" ran ASL axioms that set `rsa_ok = 1` and never read the inventory; all four failed at runtime ("stack underflow") | deterministic policy engine; every rule reads the inventory and returns pass/fail plus evidence |
| "SLH-DSA signature" was a BLAKE3 hash anyone could recompute | real SLH-DSA-SHAKE-128f (FIPS 205) via RustCrypto `slh-dsa`; offline `verify`; key pinning |
| "Merkle root" was a single hash of the CBOM | real binary Merkle tree over every asset and finding, with inclusion proofs |
| CBOM used an invalid asset type (`key`) | CycloneDX 1.7 CBOM, schema-validated: algorithm, related-crypto-material, certificate components + dependency graph |
| Exposure model inputs were constants | transparent rules + Mosca timing check with user-supplied inputs |
| License state trusted an expiry in `license.json` | gated features re-verify the PASETO token signature on every use |

## 2. Pipeline

```
 files / TLS endpoints
        │
   discover.rs   parse PEM/DER certs, PKCS#8/PKCS#1/SEC1/SPKI keys; record everything skipped (Coverage)
        │
   classify.rs   OID registry → family, classical bits, quantum status, NIST PQC level, IR 8547 dates
        │
   policy.rs     rule packs (hygiene, cabf, pqc) → Findings with evidence + references
   lifecycle.rs  CA/B Forum 398→200→100→47-day schedule → renewal-burden forecast
        │
   engine/mod.rs summary + Mosca check + assumptions + limitations → Body
        │
   merkle.rs     BLAKE3 Merkle root over every asset and finding (canonical JSON leaves)
   attest.rs     SLH-DSA signature over canonical Body (optional, gated)
        │
   report.json  cbom.json  report.html (gated)          verify / diff / prove (offline)
```

Module map: `model.rs` types · `classify.rs` registry · `discover.rs` parsing/probing · `policy.rs` rules · `lifecycle.rs` forecast · `merkle.rs` tree/proofs · `attest.rs` signing · `cbom.rs` CycloneDX · `report_html.rs` report · `diff.rs` drift · `mod.rs` orchestration + CLI.

## 3. Report format and trust model

`report.json = { format: "vericrypt-report/2", body: {...}, attestation: {...}|null }`

* `body` contains only strings, integers, arrays and objects (no floats) so it canonicalises exactly.
* Canonical form: recursively key-sorted compact JSON (own implementation, not dependent on serde_json map ordering).
* Merkle leaves: `H(0x00 ‖ kind ‖ 0x00 ‖ id ‖ 0x00 ‖ canonical(item))`; nodes `H(0x01 ‖ l ‖ r)`; odd nodes are carried up, never duplicated.
* Attestation: SLH-DSA-SHAKE-128f, context string `vericrypt-report-v2`, hedged signing, over canonical `body` (which contains `merkle_root`).
* `verify` semantics: (1) recompute Merkle root from the items; (2) verify the signature; (3) compare the signer to a key you pin with `--pubkey` / `--key-id`. A report with no signature **fails** verification unless `--allow-unsigned` is passed (stripping a signature must not look like success). A signature without a pinned key proves integrity against the embedded key only, and says so.
* `prove` / `verify-proof`: inclusion proof that one asset or finding is committed by a root.

## 4. Policy packs (all outcomes derived from inventory data)

| Rule | Severity | Fires when | Reference |
|---|---|---|---|
| HYG-EXPIRED / HYG-EXPIRING-7D / HYG-EXPIRING-30D | high / high / medium | certificate expired / ≤7 / ≤30 days left | DORA RTS (EU) 2024/1774 Art. 7 (certificate register; renewal before expiry) |
| HYG-WEAK-KEY | critical | classical strength < 112 bits (e.g. RSA-1024) | — |
| HYG-WEAK-SIGNATURE | high | MD5/SHA-1 signature (not self-signed roots) | — |
| HYG-PRIVKEY-PLAINTEXT | high | unencrypted private key on disk | DORA RTS Art. 7 (protect keys against unauthorised access) |
| CABF-LIFETIME | high (declared public) / info (undeclared) | validity exceeds the cap at its issue date | CA/Browser Forum Baseline Requirements schedule |
| PQC-IR8547 | high (≤112-bit) / medium | quantum-vulnerable algorithm | NIST IR 8547 ipd (draft): ≤112-bit deprecated after 2030; all disallowed after 2035; DORA RTS Art. 6 |
| PQC-OUTLIVES-2035 | high | vulnerable-algorithm certificate valid beyond 2035 | NIST IR 8547 ipd |
| PQC-UNKNOWN-ALGORITHM | low | OID not in registry | — |
| PQC-READY | info | ML-KEM / ML-DSA / SLH-DSA | FIPS 203/204/205 |

Portfolio check: **Mosca** `X + Y > Z` using `--shelf-life-years`, `--migration-years`, `--crqc-years` (default Z=14 is an explicit assumption, shown in the report).

## 5. CLI

```
vericrypt keygen  [--out key.json]                      # SLH-DSA key pair (+ key.pub.json)
vericrypt analyze -i DIR [-i DIR] [--tls host:port] --out DIR
                  [--public-issuers "A,B"] [--private-issuers "X"] [--sign-key key.json]
                  [--as-of RFC3339] [--fail-on high] [--no-html]
vericrypt verify  report.json [--pubkey key.pub.json | --key-id ID] [--allow-unsigned]
vericrypt diff    old/report.json new/report.json [--fail-on high]
vericrypt prove   report.json --id <asset-or-finding-id>
vericrypt verify-proof proof.json --root <hex>
```
Exit codes: 0 ok · 1 verification failure/internal · 2 usage/IO · 3 `--fail-on` threshold hit · 4 license required.

## 6. Editions

Build with `--features license-gate` for the commercial edition. Free: scan, findings, forecast, CBOM, JSON, `verify`, `verify-proof`. Licensed: signed attestation, HTML report, drift `diff`. Because the source is public, gating is a convenience for honest customers, not a hard barrier; durable value comes from signed policy-pack updates, hosted services and support.

## 7. Honest limits (also printed in every report)

* TLS probe returns the leaf certificate only: no chain, protocol versions, cipher suites or key-exchange groups.
* PKCS#12 / JKS not opened; encrypted private keys inventoried but algorithm not visible.
* No revocation (OCSP/CRL) checking; no chain building.
* HSM/KMS keys, keys in source code and application-level crypto are out of scope.
* `slh-dsa` (RustCrypto) is not independently audited. `signature` is pinned to `2.3.0-pre.4` because `slh-dsa 0.1.0` does not compile against `pre.7`.
* NIST IR 8547 is an initial public draft; dates may move.
* Debug-build timings only; no release benchmarks yet.

## 8. Competitive position (from the research in this conversation)

Open-source and vendor tools I found concentrate on discovery and CBOM generation (e.g. Trivy's crypto scanner, IBM CBOM tooling, Encryption Consulting, Entrust). In what I found, none ship: PQ-signed reports verifiable offline with key pinning, a Merkle commitment with per-item proofs, signed drift diffing, and an explicit coverage/limitations ledger in every report. Competitors are ahead on: TLS protocol/cipher enumeration (testssl/sslyze), source-code crypto scanning, HSM/KMS integration and continuous monitoring. Closing those is the roadmap.

## 9. Roadmap (in value order)

1. TLS protocol/cipher/group enumeration (the biggest functional gap).
2. Chain building + OCSP/CRL revocation.
3. PKCS#12/JKS with supplied passphrase; SSH keys; cloud-KMS inventory connectors.
4. Source-code crypto scanning (so the CBOM covers code, not just files).
5. Signed policy-pack updates (ADR-017) so rule updates ship without a new binary.
6. GitHub Action / CI mode using `analyze --fail-on` and `diff`.
7. Re-express rule semantics in Lean/ASL and prove them against the Rust evaluator (only then call anything "proved").
