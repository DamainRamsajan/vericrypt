# VeriCrypt — Project status (23 Oct 2026)

## 1. Where we are

| Area | State | Evidence |
|---|---|---|
| v2 evidence engine (code) | **Built and tested** | 19 automated tests pass (7 unit, 6 pipeline, 6 CLI); clean release build from the source zip; quickstart run end-to-end |
| Signing and verification | Real SLH-DSA-SHAKE-128f signatures; key pinning; unsigned reports fail; tamper detection | `CLAIMS.md` section A |
| CBOM | CycloneDX 1.7, validated against the official schema | `scripts/validate_cbom.py` |
| Install guide | Verified on a clean copy | `docs/INSTALL_V2.md` |
| Architecture | v3 addendum (superseded), **v4 arc42** (approved), **v5 arc42** (draft, 8 decisions pending) | `docs/architecture/` |
| Research | Competitive analysis, users and use cases, v5 value map | `docs/architecture/` |
| Code for v4/v5 | **Not started** (by agreement: finalise architecture first) | — |
| Public repository | **Not updated yet.** GitHub push from chat is blocked (no connector on this account); this bundle is the manual route | — |

## 2. Known issues in v2 (honest list)
1. TLS probe returns the leaf certificate only (no protocol/cipher/group enumeration, no chain, no OCSP).
2. No revocation checking, no chain building, no PKCS#12/JKS, no SSH, no code scanning.
3. Signing key stored as a plaintext JSON file (mode 0600). Encrypted keys / PKCS#11 are planned.
4. `slh-dsa` crate is unaudited; `signature` is pinned to a pre-release (`2.3.0-pre.4`).
5. Legacy v1 `scan` command still present (deprecated warning only); old README and data-room statements overstate it.
6. `native-tls` and `static mut` licence state are technical debt.
7. Repository declares `UNLICENSED` while public.
8. Tracked junk in the repo: `target/` (3 files), `.wrangler/cache` (Cloudflare account files), `tatus`, 10 `batch_*.sh` scripts.
9. Performance only measured on tiny fixtures; macOS, Windows and ARM untested.

## 3. Decisions still owed
1. **Licence and edition model** (recommended: open-core — permissive core and verifier; commercial packs, connectors, hub).
2. The eight v5 decisions in `docs/architecture/ARCHITECTURE_V5_arc42.md` §0.3.
3. How to connect GitHub (or keep uploading manually): see `docs/architecture/GITHUB_CONNECT_AND_PUSH_GUIDE.md`.

## 4. What is left until launch

"Launch" here means **v4.0 "Trustworthy core"** is publicly released. Effort sizes are my rough estimates (S ≈ 1–3 days, M ≈ 1–2 weeks, L ≈ 2–4 weeks of focused work), not measurements.

### 4.1 Build workstreams (v4.0)
| # | Workstream | Size | Notes |
|---|---|---|---|
| W1 | Observation store + pure evaluator refactor (time-travel) | M | Foundation for everything else |
| W2 | Packs engine: algorithm registry (with lifecycle status), jurisdiction and framework-mapping packs | L | Content-heavy; needs sources checked |
| W3 | Deep TLS sensor (rustls with explicit groups/versions + raw ClientHello; edge-vs-origin; replace native-tls) | L | Biggest functional gap; oracle tests against OpenSSL 3.5, Go, rustls |
| W4 | Evidence layer v4 (DSSE/in-toto, hybrid Ed25519 + SLH-DSA, hash-chain, optional RFC 3161) | M | |
| W5 | Minimal verifier crate + spec + test vectors | M | Independent trust path |
| W6 | Evidence packs (PCI DSS 4.0.1, ISO 27001 A.8.24, DORA RTS, NIST IR 8547, CA/B, CBTT) | M | Needs assessor review |
| W7 | Static dashboard v1, expiry alerts, SARIF | M | |
| W8 | Claims registry in CI; remove legacy `scan`; hygiene (junk files, README) | S–M | |
| W9 | Quality: corpus and differential tests, fuzzing, performance measurements | M | |
| W10 | Packaging: release binaries, reproducible build, signed releases, docs | M | |

Rough total: about 8–16 weeks of focused solo work. This is an estimate; it depends on your hours and on external people (assessors, interviewees).

### 4.2 Validation (runs in parallel; depends on other people)
- 12 user interviews (compliance leads, PKI engineers, auditors, vendor-risk, insurer/broker, M&A counsel).
- Three real (consented, anonymised) inventories run through the tool.
- An assessor reviews the PCI/ISO/DORA mappings.
- A non-technical auditor verifies a report unaided.

### 4.3 Commercial and legal readiness (not started)
- Licence decision; terms of use; privacy statement; vulnerability-disclosure policy and `security.txt`.
- **Product-name check (trademark and domain) not done.**
- Support channel; issue templates; release process.
- Data-room and README alignment with `CLAIMS.md`.
- Later: SOC 2 / ISO 27001 plan (buyers may ask a vendor for it).

### 4.4 Launch gate (all must be true)
Quality scenarios Q-1…Q-4, Q-8, Q-10, Q-11, Q-15 pass · three real inventories processed · an auditor verifies unaided · licence decided · every public claim is in `CLAIMS.md` sections A or B · fuzzing run without crashes · release is reproducible and signed.

## 5. Next actions (in order)
1. Upload this bundle and merge the pull request (v2 + docs + tests + claims registry).
2. Answer the pending decisions (licence first).
3. Start W1 and W8 (foundation and hygiene), then W3 and W4 in parallel.
