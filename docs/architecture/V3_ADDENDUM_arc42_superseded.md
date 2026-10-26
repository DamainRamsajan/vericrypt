# VeriCrypt v3 — arc42 Architecture Addendum (DRAFT for joint review)

Status: **DRAFT — no v3 code exists yet.** This addendum describes the *delta* from v2 (the evidence engine delivered in `vericrypt-v2`). Sections follow arc42. Items marked **[OPEN]** need your decision before code is written (listed in §0).
Date of research cut-off: 22 Oct 2026. Evidence quality is rated in the Source Register (§13).

---

## 0. Review sheet — decisions needed before we print code

| # | Open point | My recommendation | Why |
|---|---|---|---|
| OP-1 | Report envelope: keep v2's custom JSON, or move to **DSSE + in-toto Statement** | DSSE + in-toto | Standard format; works with Sigstore/SLSA tooling; supports several signatures; algorithm is chosen by the verifier, not read from the file |
| OP-2 | Signatures on a report | **Two:** Ed25519 *and* SLH-DSA; strict mode requires both | Old tools can check Ed25519 today; SLH-DSA protects long-lived evidence |
| OP-3 | Who is the first dashboard screen for | Compliance lead (audit evidence) | Recurring annual spend sits there |
| OP-4 | Frameworks in v3.0 | PCI DSS 4.0.1, ISO 27001:2022 A.8.24, DORA RTS 2024/1774 Art. 6–7, NIST IR 8547, CA/B Forum schedule, CBTT guideline. SOC 2 only after an assessor confirms the mapping | SOC 2 mapping is from vendor writeups only |
| OP-5 | Remove the legacy v1 `scan` command | Yes, remove in v3.0 (keep only reusable parsers) | Its proof stage and counts are unreliable; README/data-room claims conflict with it |
| OP-6 | Repository licence and edition model (the repo currently declares `UNLICENSED` while being public) | Decide before v3.0 ships | Affects what can be gated and how packs are distributed |
| OP-7 | Hub (multi-user server) in v3 scope | No. Keep formats hub-ready; ship local dashboard first | Keeps the core offline-first and small |
| OP-8 | TLS probing safeguards | Notice + conservative default rate; no hard gate | Orgs must scan their own public endpoints; abuse risk handled by rate limits and docs |

---

## 1. Introduction and Goals

### 1.1 Requirements overview
v2 is a *verifiable inventory*. v3 becomes a *verifiable decision engine for cryptographic compliance and migration*: it senses cryptography, decides what matters for each regulation and use case, proves its output, and fits into the workflows where compliance money is already spent.

| ID | Requirement |
|---|---|
| FR-1 | **Audit evidence packs** mapped to control sets (PCI DSS 4.0.1 4.2.1.1 and 12.3.3; ISO 27001:2022 A.8.24; DORA RTS 2024/1774 Art. 6–7; others as packs). Each control shows pass/fail/not-assessed with linked evidence. |
| FR-2 | **Deep TLS assessment:** protocol versions, cipher suites, key-exchange groups including hybrid `X25519MLKEM768` (0x11EC), certificate chain, OCSP stapling. |
| FR-3 | **Dashboard:** five screens (Overview, Inventory, Calendar, Evidence, Changes). Static single-file first, local server next. |
| FR-4 | **Alerts:** expiry and drift notifications to named owners (webhook/email/Slack-compatible JSON). |
| FR-5 | **Ecosystem-aware advice:** remediation differs for public WebPKI TLS, private PKI, code signing and S/MIME. |
| FR-6 | **Scoring:** deterministic Quantum Risk (confidentiality vs authenticity), Migration Time and Crypto-Agility scores; weights live in packs. |
| FR-7 | **Evidence integrity:** DSSE/in-toto envelope, hybrid signatures, optional RFC 3161 timestamp, key pinning, report hash-chain. |
| FR-8 | **Integrations:** push to GRC platforms (Vanta, Drata), import CBOMs (CycloneDX 1.6/1.7), export SARIF 2.1.0, OSCAL Assessment Results, CBOM 1.7. |
| FR-9 | **Waivers:** signed, owned, expiring risk acceptances, always visible in the report. |
| FR-10 | **Context file:** per-asset owner, data class, data shelf life, business criticality. |

### 1.2 Quality goals (ranked)
1. **Trustworthy output** — every number computed from observed data; every claim testable; reports verifiable offline.
2. **Auditor usability** — an assessor can follow control → test → evidence without help.
3. **Offline-first** — core works air-gapped; online sensors are opt-in.
4. **Interoperability** — standard formats in and out.
5. **Maintainability** — regulatory change handled as signed data, not code releases.

### 1.3 Stakeholders
| Role | Expectation |
|---|---|
| Compliance lead | One-click evidence for the yearly audit; trend over time |
| Security / PKI engineer | Fix list, expiry alerts, CI gates |
| CISO / board | One-page posture, deadline countdown |
| Auditor / assessor / customer | Evidence they can verify without trusting the producer |
| Consultant / MSSP | Repeatable multi-client runs, branded reports |
| Maintainer (you) | Small core, rules as data, claims discipline |

---

## 2. Architecture Constraints
| Type | Constraint |
|---|---|
| Technical | Rust; single binary for the core; Linux and macOS first; offline by default; deterministic output (no floats in signed data) |
| Cryptography | SLH-DSA via RustCrypto `slh-dsa` (**unaudited**); `signature` crate pinned to `2.3.0-pre.4` for it; FIPS 205 *algorithm*, not a FIPS-validated module |
| Organisational | Solo maintainer; staged releases (v3.0/3.1/3.2) |
| Conventions | arc42 + ADRs; **claims discipline:** no capability is described in docs, README or data room unless a test demonstrates it |
| Legal | Repo currently `UNLICENSED` and public (**OP-6**); scanning third parties without permission is the user's responsibility |

---

## 3. Context and Scope

### 3.1 Business context
```
 Targets: files / TLS endpoints / SSH / CBOM files from other tools
                              │
 Context file (owners, data) ─┤
 Signed packs (rules, jurisdictions, advice) ─┤
                              ▼
                      ┌──────────────┐        ┌────────────────────────┐
                      │  VeriCrypt   │───────►│ Auditor / customer     │ verifies offline
                      │     v3       │        └────────────────────────┘
                      └──────────────┘───────►  GRC platforms (Vanta, Drata, ServiceNow via OSCAL)
                              │      ───────►  CI (SARIF), webhooks (alerts)
                              ▼
                 Optional online: timestamp authority, CT logs
```

### 3.2 Technical context (interfaces)
| Interface | Format | Direction |
|---|---|---|
| Evidence report | DSSE envelope, in-toto Statement v1, custom predicate `crypto-posture/v1` | out |
| CBOM | CycloneDX 1.7 | out and in (1.6/1.7) |
| CI findings | SARIF 2.1.0 | out |
| GRC assessment results | OSCAL Assessment Results (target 1.1.x; 1.2 exists) | out |
| GRC push | REST/JSON: Vanta custom resources, Drata custom connections | out |
| Alerts | Webhook JSON | out |
| Timestamp | RFC 3161 token | in |
| Packs | Signed JSON/YAML | in |

**Out of scope for v3:** hosted SaaS, HSM/KMS connectors beyond read-only metadata (v3.1+), eBPF runtime sensor, WASM plugins.

---

## 4. Solution Strategy
| Goal | Approach |
|---|---|
| Trustworthy | Pure deterministic core (`observations → findings`), coverage ledger, differential tests against OpenSSL/zlint/Trivy; small Lean model later (ADR-13) |
| Auditor usability | Control-mapped evidence packs (ADR-08), plain-language findings with evidence |
| Offline-first | One binary; online sensors (CT, timestamp) opt-in |
| Interoperability | Import-first for code/container discovery (ADR-04); standard envelopes (ADR-01) |
| Maintainability | Rules, jurisdictions, advice as **signed data packs** (ADR-05) |
| Correct advice | Ecosystem-aware advisor with dated ecosystem state (ADR-07) |

---

## 5. Building Block View

### 5.1 Level 1
```
 CLI / local server
        │
 ┌──────┴───────────────────────────────────────────────────────────┐
 │ Sensors ──► Normaliser/Graph ──► Evaluator ──► Advisor            │
 │ (files, tls-deep, ssh,          (rules, scores,  (remediation)    │
 │  cbom-import, ct*)               waivers, context)                │
 │                                      │                            │
 │                                      ▼                            │
 │                               Evidence (Merkle, DSSE, timestamp,  │
 │                                        hash-chain)                │
 │                                      │                            │
 │ Reporters: dashboard, evidence pack, CBOM, SARIF, OSCAL, alerts   │
 │ Connectors: Vanta, Drata, webhook        Pack manager  History    │
 └───────────────────────────────────────────────────────────────────┘
 (* opt-in online)
```

| Block | Responsibility | v2 status |
|---|---|---|
| Sensors | Produce `Observation{asset, method, confidence, source}` | files + TLS leaf exist; rest new |
| Normaliser/Graph | Merge observations into assets; typed edges (issued-by, uses-key, terminates-at) | new (v1 had a graph, v2 dropped it) |
| Context | Join owner/data class/shelf life/criticality by selector | new |
| Evaluator | Rules + scores + waivers → findings | rules exist; scores/waivers new |
| Advisor | Context-aware remediation | new (fixes v2 advice bug) |
| Evidence | Merkle commitment, DSSE signing, timestamp, hash-chain, verify, proofs | Merkle/sign/verify exist; DSSE/timestamp/chain new |
| Reporters | Dashboard, packs, CBOM, SARIF, OSCAL | HTML/CBOM exist; rest new |
| Connectors | Push evidence to GRC/webhooks | new |
| Pack manager | Verify, load, version signed packs | new |
| History | Append-only report folder with chain links | new |

### 5.2 Level 2 — Sensors
| Sensor | Method | Notes |
|---|---|---|
| files | parse PEM/DER certs and keys | exists (v2) |
| tls-deep | per-endpoint probes: protocol versions, cipher suites, key-exchange groups, chain, OCSP | ADR-03 |
| ssh | read host key/KEX offerings | v3.1 |
| cbom-import | map CycloneDX CBOM → observations (`method=imported`) | ADR-04 |
| ct | certificate-transparency lookup for a domain | opt-in online, v3.1 |
| keystores | PKCS#12/JKS with user-supplied passphrase | v3.1 |

### 5.3 Level 2 — Evaluator pipeline
`observations → merge → graph → context join → rules (packs) → waivers → scores → findings`. Each stage is a pure function of its inputs plus `as_of`, enabling reproducible reports and later differential testing.

---

## 6. Runtime View

**RV-1 Scan and evidence pack.** (1) load packs, verify pack signatures; (2) run sensors with rate limits; (3) merge, build graph, join context; (4) evaluate with `as_of`; (5) build Merkle root over assets, findings, control results; (6) sign DSSE with configured keys; (7) optional RFC 3161 timestamp; (8) write report, CBOM, evidence pack, dashboard; (9) link `previous_report_root`.

**RV-2 Deep TLS probe of one endpoint.** (1) for each offered key-exchange group, one handshake offering *only* that group (avoids key-share prediction effects); (2) for each protocol version and cipher suite, one handshake restricted to it; (3) full handshake to capture chain and OCSP staple; (4) a raw legacy ClientHello for TLS ≤1.1; (5) classify results; false positives are checked against the ServerHello group, not a tool's text output.

**RV-3 Auditor verification.** Auditor receives report + envelope; runs `verify` with a pinned key set and algorithm policy; checks Merkle root, signatures, timestamp, chain link; optionally requests a Merkle proof for one finding.

**RV-4 Scheduled drift.** Timer runs `analyze` → `diff` against last report → alert owners for new findings, expiries within threshold, expired waivers.

**RV-5 Dashboard.** Static: open `report.html`. Server: `vericrypt serve` reads the history folder on 127.0.0.1 only and renders trends.

**RV-6 GRC push.** After a verified report, adapters send control results and evidence references to Vanta/Drata; failures never alter the signed report.

---

## 7. Deployment View
| Variant | Where | Notes |
|---|---|---|
| CLI | laptop, jump host, air-gapped server | single binary |
| CI | GitHub Action / generic CI | SARIF + exit codes |
| Scheduled | cron, systemd timer, Kubernetes CronJob | alerts via webhook |
| Local dashboard | `serve` on 127.0.0.1 | no external assets, strict CSP |
| Hub (**OP-7: later**) | server ingesting signed reports | stable formats keep this open; not built in v3 |

---

## 8. Cross-cutting Concepts

**8.1 Evidence and trust model.** Signed object = DSSE envelope over an in-toto Statement whose predicate is the report body. Signatures are verified under a **verifier-pinned algorithm policy** (the file does not choose its own acceptable algorithms). Timestamps (RFC 3161) prove the report existed at a time; the time inside the report is otherwise self-asserted. Reports link to the previous report root (hash-chain). A report is trustworthy only relative to the keys the verifier pinned.

**8.2 Determinism.** Same inputs + `as_of` + pack versions ⇒ same Merkle root. No floats in committed data; sorted keys for Merkle leaves; stable IDs from content hashes.

**8.3 Packs (rules as data).** Signed manifest: `id`, `version`, `kind` (rules | jurisdiction | advice | scoring-weights), `status` (draft/final/proposed), `last_verified`, `sources`, `signature`. Reports record pack IDs and versions. Unknown pack kinds are rejected, not ignored.

**8.4 Scoring.** *Quantum Risk* separates **confidentiality** (harvest-now-decrypt-later; urgent for key establishment) from **authenticity** (forgery needs a quantum computer to exist; later). Inputs: algorithm status, data shelf life, exposure, Mosca per use case. *Migration Time* reflects solution availability (e.g. hybrid key exchange available vs PQ certificates not yet trusted in browsers), dependency chain length and vendor support. *Crypto-Agility* reflects whether the algorithm is configurable and the library supports PQ. Weights are pack data; the result is a deterministic 2-D priority (start now / plan / monitor).

**8.5 Coverage ledger.** Every report lists what was examined, skipped (with reason), failed, and what cannot be seen. This stays mandatory.

**8.6 Security of the tool.** Parsers fuzzed before release; no `unsafe` in the engine; secrets zeroised; signing keys stored encrypted or via PKCS#11 (v3.1); `static mut` licence state removed; dependency policy (`cargo-deny`, pinned lockfile, second SLH-DSA implementation used only in tests as a cross-check).

**8.7 Privacy and redaction.** Inventories reveal hostnames and topology. Provide redaction profiles and selective-disclosure bundles (Merkle proofs) for sharing with auditors or customers; dashboard server binds to loopback only.

**8.8 Testing.** Unit and golden tests; differential tests of classification against OpenSSL and zlint; CBOM/SARIF/OSCAL schema validation in CI; property tests for Merkle/DSSE; fuzzing; later Lean-model differential random testing (precedent: AWS Cedar).

**8.9 Versioning.** Semantic versions for the binary; independent schema versions for report, pack, context file. `vericrypt upgrade` converts v2 reports to v3 envelopes without re-scanning.

**8.10 Editions.** Capability flags (signing, HTML, diff, connectors). Token verified on every use. Because the source is public, flags deter casual copying only; durable value comes from signed packs, integrations and support (**OP-6**).

**8.11 Claims discipline.** README, data room and CLI help may only state capabilities covered by a passing test; each pack states its source and `last_verified` date.

---

## 9. Architecture Decisions (all PROPOSED)

| ID | Decision | Rationale and consequences | Alternatives rejected |
|---|---|---|---|
| ADR-01 | DSSE + in-toto Statement v1; predicate `crypto-posture/v1` | Standard; multiple signatures; algorithm-agnostic; ecosystem tooling. Replaces v2 envelope; `upgrade` command provided | Keep custom JSON (no interop) |
| ADR-02 | Hybrid signatures Ed25519 + SLH-DSA-SHAKE-128f; verifier policy pinned; strict mode = all required | Classical verifiers work today; PQ protects archives; avoids algorithm-downgrade by file | SLH-DSA only (tooling gap); ML-DSA (crate maturity to be reviewed) |
| ADR-03 | Deep TLS sensor: **rustls** with explicit key-exchange groups and versions, plus a small raw ClientHello module for legacy versions and ServerHello group classification; replaces `native-tls` | rustls lets us restrict key-exchange groups per handshake and read chain/OCSP; raw records detect TLS ≤1.1 and avoid tool-output false positives | Shell out to OpenSSL (false-positive mode documented); `native-tls` (cannot select groups) |
| ADR-04 | Import-first for code/binary/container discovery via CycloneDX CBOMs | Others already do this; we focus on evidence and decisions | Build our own source scanner (large, duplicative) |
| ADR-05 | Rules, jurisdictions, advice and scoring weights as signed packs | Update without releases; auditable provenance | Hard-coded rules (v2) |
| ADR-06 | Three deterministic scores (Quantum Risk, Migration Time, Crypto-Agility) | Matches current prioritisation research; fixes confidentiality/authenticity conflation | Single severity (v2) |
| ADR-07 | Ecosystem-aware advisor | Public TLS: hybrid key exchange now, short-lived automated certs, MTC readiness; private PKI: ML-DSA/composite trials; code signing: per CNSA 2.0 where applicable. State held in a dated pack | One-size "use ML-DSA" (v2 flaw) |
| ADR-08 | Evidence pack = framework → control → test → result → evidence refs | Reuses findings; lands in audits | Free-form report only |
| ADR-09 | Dashboard in three steps: single-file HTML → `serve` (loopback) → hub (later) | Keeps offline strength | Web SaaS first |
| ADR-10 | Integrations: SARIF 2.1.0, OSCAL AR, GRC push adapters, webhooks | Meets users where compliance spend occurs | Proprietary exports only |
| ADR-11 | Waivers are signed documents with owner and expiry, listed in every report | Enterprise requirement; no silent suppression | Config-file ignore lists |
| ADR-12 | Report hash-chain via `previous_report_root` | Tamper-evident history | Unlinked reports |
| ADR-13 | Small Lean 4 model of evaluator semantics + differential random testing against Rust (phase v3.2) | Precedent exists (Cedar); gives truthful use of "proved" for that kernel only | Prove the whole product (unrealistic) |
| ADR-14 | Capability flags with per-use token verification | Honest editioning | Trust expiry stored on disk (v2 flaw) |
| ADR-15 | Remove legacy v1 `scan` and its proof stage (**OP-5**) | Prevents misleading output | Keep with warning (v2 interim) |

---

## 10. Quality Requirements

### 10.1 Quality scenarios (targets, not yet measured unless stated)
| ID | Scenario | Target |
|---|---|---|
| Q-1 Verifiability | Any single-byte change to a signed report | verify fails (**demonstrated in v2 tests**) |
| Q-2 Reproducibility | Same inputs, `as_of`, packs | identical Merkle root (**demonstrated in v2**) |
| Q-3 Correctness | Classification vs OpenSSL/zlint on a corpus | zero unexplained disagreements |
| Q-4 Performance | Scan 10,000 certificate files | < 60 s on 4 cores (to measure) |
| Q-5 Scale | Probe 1,000 endpoints with concurrency | < 10 min at conservative rate (to measure) |
| Q-6 Usability | Install to first signed report | ≤ 15 min (v2 guide: build ~5–15 min, run ~1 s) |
| Q-7 Offline | Core run with network disabled | full function excluding opt-in sensors |
| Q-8 Interop | CBOM, SARIF, OSCAL outputs | validate against official schemas in CI (CBOM **demonstrated in v2**) |
| Q-9 Maintainability | New jurisdiction | one pack, no code change, < 1 day |
| Q-10 Security | Parsers before release | 24 h fuzzing without crashes |

---

## 11. Risks and Technical Debt

| ID | Risk | Impact | Mitigation |
|---|---|---|---|
| R-1 | `slh-dsa` crate unaudited | Signature assurance | Hybrid signing; second implementation in tests; track audits |
| R-2 | `signature` pre-release pin | Build breakage | Pin; upstream watch; plan to move when stable |
| R-3 | PQ TLS probing false positives | Wrong findings | Per-group handshakes; ServerHello classification; OpenSSL oracle tests; documented server quirks (e.g. group-selection bugs) |
| R-4 | Regulatory drafts change (IR 8547 is a draft; some national orders are drafts) | Stale advice | Pack `status` and `last_verified`; reports show pack versions |
| R-5 | Mapping evidence mostly vendor/advisory sources (PCI, ISO, SOC 2) | Over-claiming coverage | Assessor confirmation before marketing; SOC 2 deferred |
| R-6 | GRC APIs, plans and schemas change | Broken connectors | Adapters isolated; failures non-fatal |
| R-7 | Public source weakens licence gating | Revenue leakage | Value in packs/integrations/support (**OP-6**) |
| R-8 | Legacy v1 code and conflicting public claims | Credibility | ADR-15; align README/data room (**claims discipline**) |
| R-9 | Scope creep for a solo maintainer | Delays | Phase plan; import-first strategy |
| R-10 | Probing endpoints without permission | Legal/ethical | Notice, rate limits, documentation (**OP-8**) |
| R-11 | WebPKI PQ path still moving (MTC programmes by browsers/CAs) | Advice goes stale | Dated ecosystem pack |
| R-12 | Timestamp authority needs network | Offline gap | Optional; document the self-asserted-time limitation |
| R-13 | Dashboard exposes sensitive inventory | Data leak | Loopback only, no external assets, redaction profiles |

**Technical debt to retire:** v1 `scan` pipeline (TEE stubs, PKI/revocation stubs, ASL "proof" stage); `native-tls`; `static mut` in `license.rs`; committed build artifacts and batch scripts; stale README/data-room statements.

---

## 12. Phasing
| Release | Contents |
|---|---|
| **v3.0 "Audit-ready"** | ADR-01/02 (DSSE, hybrid signatures), ADR-03 (deep TLS), ADR-05–08 (packs, scores, advisor, evidence packs), dashboard step 1, expiry alerts, context file, ADR-15 |
| **v3.1 "Connected"** | GRC push, CBOM import, SSH, keystores, CT sensor, waivers, SARIF/OSCAL, dashboard `serve`, key custody (PKCS#11) |
| **v3.2 "Provable at scale"** | Lean model + differential testing, sharded scans with hierarchical Merkle roots, timestamps by default for packs, hub readiness |

---

## 13. Glossary and Source Register

### 13.1 Glossary
- **CBOM** — cryptographic bill of materials (CycloneDX).
- **HNDL** — harvest now, decrypt later.
- **Mosca inequality** — data shelf life + migration time > time to a cryptographically relevant quantum computer ⇒ act now.
- **ML-KEM / ML-DSA / SLH-DSA** — NIST FIPS 203 / 204 / 205.
- **X25519MLKEM768** — hybrid TLS key exchange, group 0x11EC.
- **MTC** — Merkle Tree Certificate, the proposed compact PQ certificate format for the web.
- **DSSE** — Dead Simple Signing Envelope. **in-toto Statement** — standard attestation wrapper.
- **RFC 3161** — trusted timestamp protocol.
- **SARIF** — standard static-analysis results format. **OSCAL** — NIST machine-readable compliance format.
- **Pack** — signed data bundle (rules, jurisdictions, advice, weights).
- **Observation** — one sensed fact with method and confidence.
- **Waiver** — signed, expiring risk acceptance.

### 13.2 Source register (what each claim rests on)
| Topic | Source | Quality |
|---|---|---|
| Legacy TLS auditors blind to PQ hybrid groups | arXiv 2609.07849 "Nothing Breaks…" (Sept 2026) | Preprint |
| Raw-record PQ detection and probe false positives | IACR ePrint 2026/834 | Preprint |
| Runtime crypto discovery via eBPF | IACR ePrint 2026/2345; DynamicCBOM (SEG-UNIBE) | Preprint / academic repo |
| Prioritisation (Quantum Risk Score + Migration Time Score) | Europol-coordinated framework (Help Net Security, Jan 2026) | Secondary press |
| Crypto-agility readiness score | IACR ePrint 2026/1467 | Preprint |
| WebPKI PQ path (MTC; Chrome, Let's Encrypt, Apple) | postquantum.com articles; Help Net Security; heise | Secondary press |
| Migration deadlines across jurisdictions | postquantum.com timelines; The Quantum Insider; CSA note | Secondary/aggregator |
| PCI DSS 4.0.1 4.2.1.1, 12.3.3 | Schellman, HALOCK, Infisical writeups | Assessor/vendor writeups |
| DORA RTS 2024/1774 Art. 6–7 | Regulation text summaries | Secondary |
| Compliance spend split | Security Boulevard, Hyperproof, SOC 2 cost guides | Vendor |
| Certificate outage costs | DigiCert 2026 survey (via ITWire etc.) | Vendor survey |
| GRC push APIs | Vanta developer docs; Drata help centre | Primary (vendor docs) |
| DSSE/in-toto/Sigstore/RFC 3161 | Sigstore blog and package docs | Primary/community |
| Lean + differential testing precedent | AWS Cedar paper arXiv 2407.01688; AWS blog | Primary |
| rustls provider key-exchange group control | docs.rs for rustls providers | Primary (docs) |
| OSCAL Assessment Results adoption | NIST OSCAL material; ServiceNow release notes | Primary/vendor |
| CBOM tool landscape | CyberNX, QCecuring, O3 Security, QNu Labs comparisons | Vendor/marketing — treat with caution |

**Known gaps in this research:** SOC 2 control mapping not confirmed from primary text; GRC plan/pricing constraints; real-world PQ TLS adoption percentages not verified; no measured performance numbers for v2 at scale.
