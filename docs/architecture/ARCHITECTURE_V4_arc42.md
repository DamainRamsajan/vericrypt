# VeriCrypt v4 — arc42 Architecture (DRAFT for joint review)

Status: **DRAFT. No v4 code exists.** This document is complete on its own and **supersedes the v3 addendum**. It rests on two companion documents: `COMPETITIVE_ANALYSIS_v4.md` (market, threats, moats) and `USERS_AND_USE_CASES_v4.md` (personas, requirements R-xx, use cases UC-xx).
Research cut-off: 22 Oct 2026. Evidence quality is rated in §14.

---

## 0. Review sheet

### 0.1 Thesis
VeriCrypt is the **verifiable evidence and decision layer** for cryptographic posture. It does not try to out-scan the discovery vendors. It *imports* what they find, adds its own sensors where it can be uniquely right (deep TLS, reconciliation), decides what matters per regulation and use case, and produces output that anyone can verify offline, for years.

### 0.2 What v4 changes from v3 (and why)
| # | Change | Reason (research) |
|---|---|---|
| 1 | **Event-sourced Observation Store** + pure evaluator → time-travel re-evaluation, incremental scans, sharding | Deadlines change monthly; "what would the new rule say about last year?" is only cheap if raw observations are kept |
| 2 | **Algorithm registry becomes a signed pack with lifecycle status** (final/draft/proposed/deprecated/disallowed) | FN-DSA (FIPS 206) and HQC are still drafts; OIDs for FN-DSA not yet registered; registry must update without a release |
| 3 | **Edge-vs-origin PQ discrimination** in the TLS sensor | ~44% of top sites negotiate hybrid key exchange mainly because CDNs enable it; only ~10% of origins are PQ-ready |
| 4 | **Completeness by corroboration**: declared-vs-observed reconciliation, multi-vantage confidence, coverage ledger | Confidence gap (CISOs 90% vs engineers 33%); zero-knowledge cannot prove inventory completeness |
| 5 | **Minimal independent verifier** (small crate, WASM, published spec + test vectors) | "Who verifies the verifier?"; invites third-party implementations |
| 6 | **Selective disclosure with absence proofs** (Merkle, no ZK in v4) | Assurance use cases (vendor, M&A, insurer); ZK adds cost without solving completeness |
| 7 | **Read-only MCP interface that returns evidence references** and treats scanned text as untrusted | GRC platforms ship MCP servers; MCP ecosystem has serious vulnerabilities; certificate fields are attacker-controlled text |
| 8 | **Extension contract** (out-of-process plugins, versioned schemas) | Specialist sensors (blockchain exposure, firmware) without bloating the core |
| 9 | **Longevity design**: evidence renewal, format compatibility policy, reproducible builds, VeriCrypt's own CBOM | "Stand the test of time" |
| 10 | **Claims registry**: every public claim maps to a passing test; CI blocks untested claims | Credibility; v1 claims outran code |
| 11 | **Scenario analysis without randomness** (histogram of Q-Day years) | Deterministic, reproducible, explainable |
| 12 | Lean kernel moved later (v4.3); trust features first | Reorders effort toward what buyers can use sooner |

### 0.3 Carried forward from v3, provisionally accepted unless you object
DSSE + in-toto envelope; hybrid Ed25519 + SLH-DSA signatures with verifier-pinned algorithm policy; compliance lead as first dashboard persona; frameworks PCI DSS 4.0.1, ISO 27001 A.8.24, DORA RTS Art. 6–7, NIST IR 8547, CA/B schedule, CBTT guideline (SOC 2 after assessor confirmation); remove legacy `scan`; no hub in v4.0; TLS probing notice plus conservative default rate.

### 0.4 New open points
| # | Open point | Recommendation |
|---|---|---|
| OP-1 | Positioning: "proof layer above discovery", import-first | Yes |
| OP-2 | Event-sourced observation store with time-travel from v4.0 | Yes: cheap now, very costly to retrofit |
| OP-3 | Separate minimal verifier + WASM verifier + published spec/test vectors | Yes |
| OP-4 | Selective disclosure with absence proofs in v4.2; defer zero-knowledge | Yes |
| OP-5 | Read-only MCP interface in v4.1 | Yes, with untrusted-text handling |
| OP-6 | First adjacent use cases to commit to | UC-11 reconciliation, UC-7 vendor attestation, UC-10 product CBOM, UC-8 diligence pack. Crypto-asset exposure (UC-16) as a plugin; your call given VeriChain |
| OP-7 | Pack authority model | Multi-root trust: you author the default root; partners/community/enterprises may add their own roots |
| OP-8 | Licence and edition model (repo is `UNLICENSED` yet public) | Must be decided before v4.0 ships |

### 0.5 Corrections to things I said earlier
- I called the US quantum action a *draft* executive order. A law-firm primer now refers to the **June 2026 Quantum Orders** as issued. I have not read the orders; details unverified.
- v2's "analysis plus signing in 1 second" was measured on a 7-file fixture. It is **not** a performance claim.
- v2 advice to use ML-DSA for all signatures is wrong for public TLS; v4 fixes this through the advisor pack.

---

## 1. Introduction and Goals

### 1.1 Requirements overview
Requirement IDs (R-F1…R-F26, R-Q1…R-Q9) are defined in `USERS_AND_USE_CASES_v4.md`. Summary of what the system must do:

1. **Sense** cryptography from files, TLS endpoints, SSH, imported CBOMs, declared inventories (R-F1, F2, F14).
2. **Decide** with deterministic, dated, jurisdiction-aware rules and scores (R-F3, F5, F6, F15).
3. **Prove**: signed, hash-chained, timestamped, offline-verifiable, selectively disclosable evidence (R-F7, F16).
4. **Act and connect**: alerts, CI gates, GRC push, OSCAL/SARIF/CBOM export, AI-assistant interface (R-F9–F13, F26).
5. **Serve adjacent uses**: vendor attestation, diligence packs, product CBOM, MSSP, specialist sensors (R-F17–F24).

### 1.2 Quality goals (ranked)
| Rank | Goal | Meaning |
|---|---|---|
| 1 | **Trust** | Every number computed from observations; every claim backed by a test; limits always shown |
| 2 | **Longevity** | Output verifiable and interpretable in 10+ years; algorithm and format agility |
| 3 | **Usefulness** | Auditor can follow evidence unaided; first result within 15 minutes; plain language |
| 4 | **Privacy and offline** | Local-first; redaction; no mandatory cloud |
| 5 | **Interoperability** | Import and export standard formats; neutral |

### 1.3 Stakeholders
See personas in `USERS_AND_USE_CASES_v4.md` §2. Architecturally decisive: **auditor/third-party verifier** (trust path), **compliance lead** (evidence), **engineer/DevOps** (CI, alerts), **MSSP** (multi-client), **maintainer** (small core, packs as data).

---

## 2. Architecture Constraints
| Type | Constraint |
|---|---|
| Technical | Rust; single binary core; Linux and macOS first (ARM, Windows later); offline default; deterministic output; no floats in committed data |
| Cryptography | SLH-DSA via RustCrypto `slh-dsa` (unaudited; `signature` pinned `2.3.0-pre.4`); Ed25519 for the classical leg; FIPS 205 *algorithm* but not a FIPS-validated module |
| Organisational | Solo maintainer; staged releases; heavy reliance on packs and imports to limit scope |
| Conventions | arc42 + ADRs; **claims registry** (§8.12); semantic versions for binary and for each schema |
| Legal | Repo `UNLICENSED` yet public (OP-8); probing third parties is the operator's responsibility |
| Non-goals | Own source/binary scanner; CA or CLM functionality; "proved" claims outside the Lean kernel |

---

## 3. Context and Scope

### 3.1 Business context
```
  Targets (files, TLS endpoints, SSH)      Other tools' CBOMs (CBOMkit, Trivy, vendors)
  Declared inventories (CMDB/CSV)          Context (owners, data class, shelf life)
                         │                                │
                         ▼                                ▼
   Signed packs ──►  ┌──────────────────────────────────────────┐
   (registry, rules, │               VeriCrypt v4               │ ──► Auditors, customers, insurers,
    advice, weights, │  sense → store → decide → prove → serve  │     regulators (verify offline)
    mappings)        └──────────────────────────────────────────┘ ──► GRC platforms (Vanta, Drata, OSCAL)
                         │                                       ──► CI (SARIF), alerts (webhook)
                         ▼                                       ──► AI assistants (read-only MCP)
        Optional online: RFC 3161 timestamp, CT logs
```

### 3.2 Technical context
| Interface | Format | Direction |
|---|---|---|
| Evidence | DSSE envelope, in-toto Statement v1, predicate `crypto-posture/v4` | out |
| Disclosure bundle | Subset of items + Merkle/absence proofs + original envelope | out |
| CBOM | CycloneDX 1.7 | out; in (1.6/1.7) |
| Declared inventory | CSV/JSON schema `declared-asset/1` | in |
| Context | YAML schema `context/1` | in |
| Packs | Signed JSON bundles | in |
| CI | SARIF 2.1.0 | out |
| GRC | OSCAL Assessment Results (1.1.x); REST adapters (Vanta, Drata) | out |
| Alerts | Webhook JSON | out |
| Assistant | MCP (stdio or loopback), read-only | out |
| Plugins | `vericrypt-plugin/1` JSON lines over stdio | in/out |
| Timestamp | RFC 3161 | in |

Out of scope for v4.x: hosted SaaS, hub, eBPF sensor, mainframe discovery, HSM administration.

---

## 4. Solution Strategy
| Goal | Strategy |
|---|---|
| Trust | Pure evaluator over stored observations; coverage ledger; corroboration; claims registry; differential testing; small independent verifier |
| Longevity | Algorithm agility in signing policy; evidence renewal; schema compatibility policy; spec + test vectors; reproducible builds |
| Usefulness | Control-mapped packs; ecosystem-aware advice; five-screen dashboard; plain-language findings |
| Privacy/offline | Single binary; loopback-only servers; redaction profiles; online sensors opt-in |
| Interoperability | Import-first; standard formats; plugin contract |
| Maintainability | Rules, registry, advice, weights, mappings as signed data (packs) |

---

## 5. Building Block View

### 5.1 Level 1
```
 CLI · local server · MCP · CI action
          │
 ┌────────┴──────────────────────────────────────────────────────────────┐
 │ SENSORS ──► OBSERVATION STORE ──► NORMALISER/GRAPH ──► EVALUATOR      │
 │ files, tls-deep, ssh,   (append-only,   (assets, edges,    (rules, scores,│
 │ cbom-import, declared,   content-        corroboration)     waivers,      │
 │ plugins                  addressed)                         controls)     │
 │                                                  │              │         │
 │                                                  ▼              ▼         │
 │                                         ADVISOR (recipes)   EVIDENCE      │
 │                                                           (Merkle, DSSE,  │
 │ PACKS (registry, rules,                                   timestamp,      │
 │  advice, weights, mappings)                               chain, renewal) │
 │                                                                  │        │
 │ REPORTERS: dashboard, evidence pack, CBOM, SARIF, OSCAL, disclosure      │
 │ CONNECTORS: GRC push, webhook         HISTORY: report chain + store      │
 └───────────────────────────────────────────────────────────────────────┘
 VERIFIER (separate minimal crate; WASM build) — shares only the format spec
```

| Block | Responsibility | New in v4? |
|---|---|---|
| Sensors | Emit observations with sensor identity, vantage, method, confidence | extended |
| Observation Store | Append-only, content-addressed batches; supports replay | **new** |
| Normaliser/Graph | Merge observations into assets; typed edges; corroboration count | new |
| Evaluator | Pure function: observations + packs + context + waivers + `as_of` → findings, scores, control results | extended |
| Advisor | Context-aware remediation from advice pack | new |
| Evidence | Merkle roots, DSSE signing, timestamps, chain, renewal, disclosure bundles | extended |
| Packs manager | Verify signatures/trust roots, load, version, record digests | new |
| Reporters | Dashboard, evidence pack, CBOM, SARIF, OSCAL, disclosure | extended |
| Connectors | GRC push, webhook | new |
| MCP server | Read-only queries with evidence references | new |
| Plugin host | Run sensors/reporters out of process | new |
| **Verifier** | Verify reports, disclosures, packs; no scanning code, minimal deps | **new, separate** |

### 5.2 Level 2 — Observation Store
Layout: `store/obs/YYYY/MM/<batch-hash>.jsonl` (immutable batches); `store/scans/<scan-id>.json` (scan record: tool build hash, parameters, targets, coverage ledger, timestamps); `store/reports/`; `store/packs/`. Batches are hashed; a scan record lists batch hashes; reports commit to scan records. Raw artifacts (certificate DER, handshake transcripts) are stored by hash when the operator allows. Local server may index into SQLite; JSONL remains the source of truth. Export to Parquet for analytics.

### 5.3 Level 2 — Sensors
| Sensor | Method | Phase |
|---|---|---|
| files | PEM/DER certs, keys (PKCS#1/8/SEC1/SPKI) | v2 exists |
| tls-deep | per-version, per-suite, per-key-exchange-group handshakes with rustls; raw ClientHello for TLS ≤1.1 and ServerHello group classification; chain; OCSP staple; server-preference probe (offer classical-first vs hybrid-first) | v4.0 |
| edge-origin | detect CDN-fronted endpoints (headers, issuer, address ranges) and probe declared origins; confidence shown, mapping overridable in context file | v4.0 |
| cbom-import | map CycloneDX CBOM → observations (`vantage=code/binary/container`, `method=imported`) | v4.1 |
| declared | CMDB/CSV import (`vantage=declared`) | v4.1 |
| ssh | host key and KEX offerings | v4.1 |
| keystores | PKCS#12/JKS with supplied passphrase | v4.1 |
| chain-exposure (plugin) | blockchain address reuse / exposed public keys | v4.3 |
| ct | certificate-transparency lookup (opt-in online) | v4.2 |

### 5.4 Level 2 — Evaluator pipeline
`observations → merge (identity by content hash) → graph → corroboration score → context join → packs: registry lookup → rules → waivers → scores → control results → findings`. Every stage is pure given inputs and `as_of`. Results are cacheable by input digest (incremental evaluation).

### 5.5 Level 2 — Packs
| Kind | Content |
|---|---|
| `algorithm-registry` | OID, name, family, status (final/draft/proposed/deprecated/disallowed), quantum status, security bits, NIST level, standard reference, transition dates with source and status |
| `jurisdiction` | Obligations, deadlines, scope, source URLs, status, `last_verified` (NIST IR 8547, CNSA 2.0, EU roadmap, UK NCSC, G7 financial, DORA, CBTT…) |
| `framework-mapping` | Control → tests → evidence rules (PCI DSS 4.0.1, ISO 27001 A.8.24, DORA RTS, others) |
| `ecosystem-advice` | Context-specific advice with ecosystem state (e.g. hybrid key exchange default in OpenSSL 3.5+, Go 1.24+, GnuTLS 3.8.9+; browser PQ-certificate path via MTC) |
| `scoring-weights` | Weights and thresholds for the four scores and priority matrix |
| `remediation-recipes` | Configuration snippets and upgrade steps |
| `questionnaire-map` | Vendor-questionnaire question → controls/tests (for UC-7) |

Packs are **data, not code**. Manifest: id, version, kind, status, `last_verified`, sources, trust-root id, signature. Reports record pack digests.

### 5.6 Level 2 — Evidence
Components: canonicaliser (for Merkle leaves only), Merkle builder (assets, findings, control results, attribute index), DSSE signer (hybrid), timestamp client (optional), chain linker, renewal tool, disclosure builder. The **verifier** reimplements verification from the spec.

### 5.7 Level 2 — Interfaces
CLI; local server (loopback) with dashboard, history and trends; MCP (read-only); GitHub Action; connectors. All interfaces call the same library surface.

---

## 6. Runtime View

**RV-1 Scan and evidence.** Load and verify packs → run sensors (rate-limited, concurrent) → append observations and scan record → evaluate → build Merkle roots → sign DSSE (hybrid) → optional timestamp → write reports and exports → link `previous_report_root`.

**RV-2 Deep TLS probe.** For each key-exchange group, one handshake offering *only* that group; for each version/suite, one restricted handshake; one full handshake for chain and OCSP; legacy raw ClientHello; preference probe; classify from ServerHello, not tool text; record transcript hashes; mark CDN-fronted endpoints and probe declared origins.

**RV-3 Reconciliation.** Import declared inventory → evaluator marks declared-not-observed (stale or unreachable), observed-not-declared (shadow), attribute mismatches → inventory confidence per asset = number of independent vantages × freshness.

**RV-4 Time-travel.** `reevaluate --scan <id> --packs <new>` replays stored observations under new packs → produces a new report that cites old scan and new pack digests → `diff` shows what the new rules change.

**RV-5 Auditor verification.** Verifier loads report, policy file (accepted algorithms, required signers, max age, timestamp requirement, trusted pack roots) → checks Merkle roots, signatures, timestamp, chain link, pack digests → prints PASS/FAIL per check.

**RV-6 Disclosure.** Operator selects claims (e.g. "no RSA keys", "no certificates expiring in 30 days", "findings for asset X") → bundle contains revealed items with proofs, absence proofs from the sorted attribute index, original envelope, and the signed coverage ledger → verifier validates without seeing other assets.

**RV-7 Assistant query.** MCP request → server loads latest *verified* report → returns structured answer with `evidence{report_id, root, item_ids, proof_ref}` → scanned strings flagged `untrusted_text`.

**RV-8 Scheduled drift.** Timer → RV-1 → `diff` vs last → notify owners (new findings, expiries, expired waivers).

---

## 7. Deployment View
| Variant | Notes |
|---|---|
| CLI binary | laptop, jump host, air-gapped server |
| Container image | CI and scheduled jobs |
| GitHub Action | SARIF output, ratchet gate |
| Local server | `serve` on 127.0.0.1; static assets only; strict CSP |
| Browser verifier | WASM build of the verifier in a static page; no server |
| Plugins | separate processes; optional OS sandbox (seccomp/Landlock) |
| Hub (not v4) | stateless ingest of signed reports; tenant IDs reserved in schemas |

---

## 8. Cross-cutting Concepts

**8.1 Data model.** *Observation*: `obs_id, sensor{id,version,kind}, vantage(host|network|code|cloud|declared|chain), method, target, observed_at, subject, attributes, confidence(0–100), raw_ref, scan_id`. *Asset*: merged identity with list of observation IDs and vantage set. *Finding*: rule, pack version, evidence refs, waiver ref, score components. *ScanRecord*: build hash, parameters, targets, coverage ledger, timing. All committed data uses integers and strings only.

**8.2 Evidence and trust.** Signed object = DSSE envelope over an in-toto Statement; predicate contains `assets_root, findings_root, controls_root, attribute_index_root, scan_record_digest, pack_digests, previous_report_root`. Hybrid signatures (Ed25519 + SLH-DSA-SHAKE-128f). Verification uses a **verifier-pinned policy**, never algorithms declared in the file. RFC 3161 timestamps prove existence at a time; otherwise time is self-asserted and the verifier says so. A report is trustworthy only relative to keys the verifier pinned.

**8.3 Selective disclosure.** An **attribute index** is a second Merkle tree over sorted tuples `(attribute, value, asset_id)`. Revealing an item uses inclusion proofs; proving absence (e.g. no `algorithm=RSA`) uses a non-membership proof from adjacent leaves. Limitation stated in every bundle: absence is relative to the committed inventory; inventory completeness is a separate claim carried by the signed coverage ledger and corroboration data. Zero-knowledge proofs are deferred: they could hide the sorted-index structure but cannot establish completeness.

**8.4 Completeness by corroboration.** Per asset: independent vantages and freshness. Per estate: declared-vs-observed deltas. Optional **experimental** estimator (capture–recapture across two independent sensors) with explicit independence assumptions, flagged `experimental`, excluded from signed claims unless enabled. *This idea is mine and unvalidated in the literature I found.*

**8.5 Scoring (integers, deterministic).**
- **Quantum Risk — confidentiality (QR-C)**: key-establishment algorithm status, exposure, data shelf life, Mosca margin per use case. Not applicable to pure signature assets.
- **Quantum Risk — authenticity (QR-A)**: signature algorithm status, trust-anchor role (root > intermediate > leaf), validity versus horizon, forgery impact (code signing > TLS leaf).
- **Migration Time (MT)**: ecosystem readiness (advice pack), dependency depth (graph), vendor support (context), agility signals.
- **Crypto-Agility (CA)**: configurability, library support (from imported SBOM/CBOM), central management.
Priority class (start now / plan / monitor) from thresholds in `scoring-weights`. **Scenario analysis**: input a histogram of Q-Day years (user or pack supplied); output is the weighted share of scenarios in which each asset class violates the Mosca inequality; plain arithmetic, no random numbers.

**8.6 Time-travel.** Because evaluation is pure and observations are retained, any past scan can be replayed under any pack set and `as_of`. Reports always cite scan and pack digests.

**8.7 Longevity.** (a) Signing-policy registry: Ed25519, SLH-DSA variants now; ML-DSA after crate review; FN-DSA after FIPS 206 is final and OIDs are registered. (b) `renew` re-timestamps and re-signs the *envelope* with newer algorithms, forming a renewal chain. (c) Schema compatibility: each schema semver'd; readers support N-1 majors for at least five years; `migrate` provided. (d) Spec and test vectors live in the repository independent of code. (e) Reproducible builds, signed releases, VeriCrypt's own CBOM and SBOM published. (f) Verifier kept tiny.

**8.8 Extension contract.** Plugins speak `vericrypt-plugin/1` (JSON lines over stdio): `hello` (capabilities, schema versions) → `scan` requests → observation stream. Plugins cannot sign reports or write the store directly. Observations carry sensor identity and may be signed by the plugin key. WASM components considered later behind the same schema.

**8.9 AI-assistant safety.** MCP tools are read-only (`list_findings`, `get_asset`, `get_control_status`, `explain_finding`, `diff`, `prove`). No scan or write tools. Loopback/stdio only; no network egress; schema-validated inputs; size-limited outputs; every free-text field from scanned data (subjects, SANs, file paths) is escaped, length-limited and flagged `untrusted_text` to reduce prompt-injection; every answer carries evidence references; server refuses to answer from a report that fails verification.

**8.10 Threat model (summary).**
| Threat | Control |
|---|---|
| Malicious certificate/file exploits parser or exhausts resources | Memory-safe Rust, size/time limits, fuzzing, no `unsafe` in engine |
| Terminal/HTML/prompt injection through certificate text | Escaping, `untrusted_text`, CSP |
| Report tampering or signature stripping | DSSE, pinned verifier policy, strict mode fails unsigned reports |
| Signing-key theft | Encrypted key files; PKCS#11/HSM signer (v4.1); rotation |
| Malicious pack | Packs are signed data validated against schema; trust roots pinned; no code execution |
| Malicious plugin | Out-of-process, no signing rights, optional sandbox |
| Supply chain | Pinned lockfile, `cargo-deny`, reproducible builds, second SLH-DSA implementation in tests |
| Scanning without permission | Notice, conservative rate limits, documentation |

**8.11 Privacy.** Inventories reveal topology. Redaction profiles (hash hostnames, drop subjects), disclosure bundles, loopback-only servers, no telemetry.

**8.12 Claims registry.** `CLAIMS.json` maps each public statement (README, CLI help, data room, packs) to one or more test IDs. CI fails if a claim has no passing test. README capability sections are generated from it.

**8.13 Testing.** Unit and golden tests; differential tests of classification against OpenSSL/zlint/Trivy on a corpus; PQ-TLS oracle tests against local servers (OpenSSL 3.5+, rustls, Go) with known group preferences; schema validation of CBOM/SARIF/OSCAL; property tests (Merkle, canonicalisation, proofs); fuzzing; mutation testing on the evaluator; later Lean-model differential random testing; published conformance vectors for the verifier.

**8.14 Editions.** Capability flags per feature; token verified on every use. Honest limit: the repo is public, so flags deter casual copying only (OP-8). Verification must always remain free so evidence stays checkable.

---

## 9. Architecture Decisions (all PROPOSED)

| ID | Decision | Rationale / consequence | Supersedes |
|---|---|---|---|
| ADR-01 | Event-sourced Observation Store + pure evaluator | Time-travel, incremental scans, sharding, reproducibility; more storage; must design schema carefully | new |
| ADR-02 | DSSE + in-toto Statement `crypto-posture/v4`; hybrid Ed25519 + SLH-DSA; verifier-pinned policy; optional RFC 3161 | Standard envelope; downgrade-resistant | v3 ADR-01/02 |
| ADR-03 | Separate minimal verifier crate + WASM + spec + test vectors | Independent trust path; third-party implementations | new |
| ADR-04 | TLS sensor: rustls with explicit groups/versions + raw ClientHello module; per-group probing; server-preference probe; replace `native-tls` | Group control; avoids tool-output false positives | v3 ADR-03 |
| ADR-05 | Edge-vs-origin discrimination with visible confidence and overridable mapping | Avoids false "PQ-ready" from CDN | new |
| ADR-06 | Import-first discovery for code/binary/container | Others do it well | v3 ADR-04 |
| ADR-07 | Declared-vs-observed reconciliation and multi-vantage confidence; experimental capture–recapture estimator | Addresses confidence gap | new |
| ADR-08 | Packs as signed data with multi-root trust; seven kinds incl. algorithm registry with lifecycle | Update without release; partner/enterprise roots | v3 ADR-05 (extended) |
| ADR-09 | Four integer scores + histogram scenario analysis | Deterministic; matches research | v3 ADR-06 |
| ADR-10 | Ecosystem-aware advisor with dated state | Fixes v2 flaw | v3 ADR-07 |
| ADR-11 | Evidence packs mapped to controls; OSCAL/SARIF/GRC push | Where compliance money is spent | v3 ADR-08, 10 |
| ADR-12 | Selective disclosure with attribute index and absence proofs; defer ZK | Assurance use cases without ZK cost | new |
| ADR-13 | Read-only MCP with evidence references and untrusted-text handling | AI interface without new attack surface | new |
| ADR-14 | Out-of-process plugin contract `vericrypt-plugin/1` | Specialist sensors; stable ABI | new |
| ADR-15 | Longevity policy: renewal chain, schema compatibility window, reproducible builds, own CBOM | Stand the test of time | new |
| ADR-16 | Claims registry enforced in CI | Credibility | new |
| ADR-17 | Waivers as signed expiring documents | Enterprise governance | v3 ADR-11 |
| ADR-18 | Report hash-chain | Tamper-evident history | v3 ADR-12 |
| ADR-19 | Lean 4 kernel of evaluator semantics + differential random testing (v4.3) | Truthful "proved" for the kernel | v3 ADR-13 |
| ADR-20 | Remove legacy v1 `scan` and ASL proof stage | Prevents misleading output | v3 ADR-15 |
| ADR-21 | Capability flags with per-use token verification; verification always free | Honest editioning | v3 ADR-14 |

---

## 10. Quality Requirements
| ID | Scenario | Target | Status |
|---|---|---|---|
| Q-1 | Change one byte of a signed report | verify fails | demonstrated in v2 |
| Q-2 | Same inputs, `as_of`, packs | identical roots | demonstrated in v2 |
| Q-3 | Classification vs OpenSSL/zlint | zero unexplained disagreements on corpus | to build |
| Q-4 | PQ-TLS probe vs known local servers (OpenSSL 3.5+, rustls, Go) | 100% correct group classification, including HRR and preference | to build |
| Q-5 | Scan 10,000 certificate files | < 60 s on 4 cores | to measure |
| Q-6 | Probe 1,000 endpoints at conservative rate | < 10 min | to measure |
| Q-7 | Re-evaluate a 12-month history under a new pack | < 5 min for 100k observations | to measure |
| Q-8 | Install to first signed report | ≤ 15 min | v2 build + run verified |
| Q-9 | Verifier unaided by a non-technical auditor | completes verification with the guide | to test (user validation step 5) |
| Q-10 | Verifier size and dependencies | independent crate, no network or scanner code | to build |
| Q-11 | Offline | core run with network disabled | testable |
| Q-12 | Schema validation | CBOM (shown in v2), SARIF, OSCAL validate in CI | partly shown |
| Q-13 | New jurisdiction | one pack, no code, < 1 day | to test |
| Q-14 | Parser robustness | 24 h fuzz without crash before release | to build |
| Q-15 | Claims | every public claim has a passing test | to build |
| Q-16 | Longevity | v2/v3 reports verify under v4 verifier via `migrate` | to build |

---

## 11. Risks and Technical Debt
| ID | Risk | Mitigation |
|---|---|---|
| R-1 | `slh-dsa` unaudited | Hybrid signing; second implementation in tests; follow audits |
| R-2 | `signature` pre-release pin | Pin; track upstream; move when stable |
| R-3 | PQ-TLS probing errors or server quirks (e.g. group-selection behaviour) | Per-group probing, oracle tests, documented quirks |
| R-4 | Draft standards and national orders change (IR 8547 draft; FIPS 206/HQC drafts; June 2026 US orders unread) | Pack status and `last_verified`; reports cite pack versions |
| R-5 | Control mappings from advisory sources | Assessor review before claims (validation step 4) |
| R-6 | GRC API/plan changes | Isolated adapters; non-fatal failures |
| R-7 | Public source weakens gating | Value in packs, integrations, support (OP-8) |
| R-8 | Legacy v1 code and conflicting public claims | ADR-20, ADR-16 |
| R-9 | Scope creep for solo maintainer | Import-first; plugin contract; phasing |
| R-10 | Probing without permission | Notice, rate limits |
| R-11 | WebPKI path still moving | Dated advice pack |
| R-12 | Timestamp authority needs network | Optional; state limitation |
| R-13 | Dashboard/MCP data exposure | Loopback only, redaction, untrusted-text handling |
| R-14 | Pack curation burden | Multi-root authorship; source-monitoring automation |
| R-15 | Procurement demands vendor assurances (SOC 2/ISO) | Plan certification; dogfood evidence; reproducible builds |
| R-16 | Platform bundling and GRC competition (see competitive analysis) | Neutral proof layer; integrate not fight |
| R-17 | Capture–recapture estimator misleads | Experimental flag; excluded from signed claims by default |
| R-18 | Desk research not validated with users | Validation plan before scope lock |

**Technical debt to retire:** v1 `scan` pipeline (TEE/PKI/revocation stubs, ASL proofs), `native-tls`, `static mut` in `license.rs`, committed build artifacts and batch scripts, stale README/data-room statements.

---

## 12. Roadmap and exit criteria
| Release | Contents | Exit criteria |
|---|---|---|
| **v4.0 Trustworthy core** | Observation store; pure evaluator; packs engine (registry, jurisdiction, mapping, advice, weights); tls-deep + edge-origin; evidence layer (DSSE hybrid, chain, optional timestamp); minimal verifier with spec and vectors; evidence packs (PCI, ISO, DORA, NIST IR 8547, CA/B, CBTT); static dashboard; alerts; SARIF; claims registry; legacy removal | Q-1…Q-4, Q-8, Q-10, Q-11, Q-15 pass; three real inventories processed |
| **v4.1 Connected and corroborated** | Declared inventory + reconciliation; CBOM import; GRC push (Vanta, Drata); MCP read-only; time-travel CLI; waivers; local server; SSH; keystores; vendor attestation v1 | Validation steps 1–4 complete; connector tests against sandboxes |
| **v4.2 Assurance** | Selective disclosure + absence proofs; diligence pack; product CBOM; MSSP workspaces; scenario analysis; browser verifier; CT sensor | Q-9 passes with a real auditor |
| **v4.3 Breadth and proof** | Supervisor packs; chain-exposure plugin; Lean kernel + differential testing; evidence renewal; plugin SDK; ARM/Windows builds; hub readiness | Q-16; Lean kernel agrees with Rust on 1M random cases |

---

## 13. Traceability (requirements → decisions)
| Requirement | Decisions / blocks |
|---|---|
| R-F1, F2 | ADR-04, 05, 06; Sensors |
| R-F3, F12 | ADR-08, 11; Packs; Reporters; Connectors |
| R-F4, F10 | Evaluator; alerts; ADR-01 |
| R-F5, F6 | ADR-09, 10; Advisor |
| R-F7 | ADR-02, 03, 18; Evidence; Verifier |
| R-F8, F14 | ADR-07; Context; Reconciliation |
| R-F9 | SARIF reporter; CI action |
| R-F13 | Dashboard (static → server) |
| R-F15 | ADR-01 |
| R-F16, F17, F18 | ADR-12; Evidence disclosure |
| R-F25 | ADR-17 |
| R-F26 | ADR-13 |
| R-F21, F23 | ADR-14 (plugins) |
| R-Q1 | ADR-16, 19; testing |
| R-Q7 | ADR-15 |
| R-Q8 | §8.11; ADR-13 |

---

## 14. Glossary, sources, gaps

### 14.1 Glossary (additions to v3)
**Observation** — one sensed fact with sensor, vantage, method, confidence. **Vantage** — where a fact was seen from (host, network, code, cloud, declared, chain). **Corroboration** — the same asset seen from independent vantages. **Attribute index** — sorted Merkle tree enabling absence proofs. **Disclosure bundle** — revealed items plus proofs. **Edge/origin** — CDN front end vs the server behind it. **MTC** — Merkle Tree Certificate. **QR-C / QR-A** — quantum risk for confidentiality / authenticity. **Renewal chain** — successive re-timestamps/re-signatures of an envelope. **Claims registry** — mapping of public claims to tests.

### 14.2 Source register (selected)
| Topic | Source | Quality |
|---|---|---|
| Market structure, consolidation, pricing signals | Axelspire, PKI Insights, Akeyless, FixMyCert, Mordor | Vendor/analyst-style, indicative |
| Competitor capabilities | SandboxAQ pages/release notes, CrowdStrike listing, QCecuring comparison | Vendor; one is a rival's view (biased) |
| Confidence gap and blockers | Axiad (Jul 2026), Entrust/Ponemon 2026, Keyfactor survey (via NHIMG) | Vendor surveys |
| Compliance spend | Security Boulevard, Hyperproof | Vendor |
| PCI/ISO/DORA mappings | Schellman, HALOCK, Infisical; DORA RTS summaries | Assessor/vendor writeups |
| PQ-TLS blindness and probing | arXiv 2609.07849; IACR ePrint 2026/834 | Preprints |
| PQ adoption | Scott Helme top-1M analysis (13 Jun 2026); Cloudflare Radar blog (27 Feb 2026); Cloudflare notes OpenSSL 3.5+, GnuTLS 3.8.9+, Go 1.24+ default hybrid | Primary data / vendor blog |
| Runtime discovery | IACR ePrint 2026/2345; DynamicCBOM | Preprint / academic |
| Prioritisation research | Europol-coordinated framework (via Help Net), ePrint 2026/1467 | Press / preprint |
| WebPKI PQ path | postquantum.com, Help Net, heise | Secondary press |
| Deadlines | postquantum.com, Quantum Insider, CSA note; law-firm primer on June 2026 US orders | Secondary |
| Algorithm lifecycle | Encryption Consulting (FN-DSA/HQC status), NIST presentation (FIPS 206), IETF LAMPS draft | Vendor + primary |
| Selective disclosure limits | postquantum.com "Proving your crypto inventory is complete…" (cites Petra, USENIX Security 2026) | Secondary |
| DSSE, in-toto, RFC 3161, Sigstore | Sigstore blog and package docs | Primary/community |
| Lean + differential testing | AWS Cedar paper and blog | Primary |
| MCP in GRC; MCP risk | Vanta/Secureframe docs; SoftwareSeni assessment (Sep 2026) | Vendor docs / secondary |
| Assurance use cases | Freshfields, CyberCube via Help Net, AUSCERT questionnaire, SafeLogic, Project Eleven/Relm | Law firm / industry |
| EU CRA | noze.it, finitestate, Docker, NCSC Ireland, ETSI reports | Secondary / government |
| Blockchain exposure | Glassnode via Decrypt; Project Eleven; BIP-360 text | Industry data / primary draft |
| GRC integration | Vanta developer docs; Drata help centre; ServiceNow release notes | Primary vendor docs |

### 14.3 Known gaps in this research
No user interviews; no hands-on competitor trials; no analyst-report access; SOC 2 mapping unconfirmed; GRC plan constraints unchecked; June 2026 US orders unread; CRA's treatment of cryptography in technical documentation unverified; hyperscaler native crypto inventory not researched; capture–recapture idea unvalidated; no measured performance numbers beyond a 7-file fixture; Windows/macOS/ARM build behaviour untested.
