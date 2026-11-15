# VeriCrypt v5 — arc42 Architecture (DRAFT for joint review)

Status: **DRAFT. No v4 or v5 code exists yet.** v5 **extends** `ARCHITECTURE_V4_arc42.md` (all v4 decisions you approved are locked as the baseline). Read this document together with:
- `ARCHITECTURE_V4_arc42.md` — the core (sensing, store, evaluator, evidence, verifier, packs)
- `V5_VALUE_MAP_AND_INTEGRATION.md` — why v5 exists, competitor value, frontier ideas, low-hanging fruit
- `COMPETITIVE_ANALYSIS_v4.md` and `USERS_AND_USE_CASES_v4.md` — market and users

**Build order rule:** v4.0 is built first. v5 is an architecture *target*; each v5 capability must arrive as a data pack, a thin connector or a small plugin, never as a new core subsystem.

---

## Amendment 2: 2 Nov 2026 (newest; wins over Amendment 1 and the text below)

**Research round 3 closed most open questions** (`RESEARCH_ROUND_3.md`): EO 14412 and OMB M-26-15 are corroborated by several sources; the US order requires government guidance on the **minimum elements of a CBOM (about March 2027)**; **RFC 10024 made the hybrid TLS groups final** (August 2026); the Trinidad and Tobago Self-Assessment Return is due **every 31 March** (primary source), the next on **31 March 2027**.

**Built:** legacy v1 code deleted (about 1,300 lines) and with it OpenSSL and the ASL build step; the binary needs only the C library. TLS probe now uses rustls. STARTTLS probing for SMTP, IMAP and PostgreSQL (POP3 and FTP implemented, untested). **Waivers** (documented risk decisions). Product CBOM for the binary. All with tests (36 pass).

**Delivery rule:** until you tell me you have uploaded, every zip is cumulative against GitHub commit `95aa48f`, so the latest zip is the only one to upload. Once you confirm an upload, that state becomes the new base.

**Pilot target (decided in principle):** Trinidad and Tobago regulated institutions, aiming to give them evidence for the 31 March 2027 return.

**Pilot-ready checklist changes:** OpenSSL requirement removed; STARTTLS covered for mail and PostgreSQL; risk-decision register present; product CBOM present. Still open: static musl build verified, Windows build, first real inventory, pilot paperwork, product-name check, legal review of LICENSE, deep TLS (versions, ciphers, hybrid groups).

## Amendment 1 — 1 Nov 2026 (this section wins where it conflicts with text further down)

**What changed in the world (research round 2; details and source quality in `RESEARCH_ROUND_2.md`)**
- **NIST IR 8547 became final on 27 Oct 2026.** Every quantum-vulnerable public-key algorithm is deprecated after 2030 and disallowed after 2035, at every strength. The v2 rule code assumed the 2024 draft and had to be changed.
- **IBM launched a Quantum Safe suite (27 Oct):** Cryptography Manager (discovery, posture, migration tracking), Explorer (code scanning and CBOM) and a PQ Proxy for legacy-to-PQ traffic. An analyst commenting on it says the cryptographic inventory will never be complete, which supports our evidence-and-coverage position.
- **DuoKey offers free live probing** of post-quantum key exchange, covering STARTTLS and databases, with CycloneDX 1.7 output. A free competitor now overlaps our TLS and CBOM work.
- US executive order 14412 (June 2026) and OMB memo M-26-15 (agency PQC plans due 22 Oct 2026), and a German federal migration guideline (12 Oct 2026, end-2030 for high-risk systems), are reported by secondary sources only. Primary texts are unread.

**What changed in the project**
- **Licence decided (28 Oct): proprietary, all rights reserved.** The repository stays public for now (your decision). The open-core recommendation in §0.3 item 6 is withdrawn.
- **Goal: a binary with a licence key for pilot customers, working without bugs.** The licence system was found broken three ways (public key never embedded, generator produced a non-standard token format, hash check could never match). All fixed and tested.
- **Legacy `scan` removed from the CLI; v1 tests parked** in `tests/legacy_v1/` so a plain `cargo test -p vericrypt` now runs and passes (31 tests).
- **New:** `sign-file` / `verify-file`, SBOM generator, release packager, claims checker in CI, README and LICENSE aligned with `CLAIMS.md`.
- **Stance: functionality over process.** Code ships first. Architecture changes are recorded as dated amendments (this section), not new document versions, until the structure itself changes (ADR-44).

**Pilot-ready checklist**
| Item | State |
|---|---|
| Licence keys activate end to end; expired, forged and other-key refused | Done, tested |
| Release signed with our own SLH-DSA key, SBOM and checksums, offline verification by the customer | Done, tested |
| README, LICENSE and CLAIMS agree; CI checks claims | Done (CI itself not yet run on GitHub) |
| Static (musl) binary that runs without OpenSSL 3 | Not verified |
| First real customer inventory run through the tool | Not done |
| Pilot agreement, NDA, privacy and security-contact documents | Not done |
| Product-name and trademark check; legal review of LICENSE | Not done |
| Windows and macOS builds (banks often run Windows) | Not decided |

## 0. Review sheet

### 0.1 Locked from v4 (you said yes)
Proof-layer positioning (import-first) · event-sourced observation store with time-travel · separate minimal verifier with browser build and published spec/test vectors · selective disclosure with absence proofs (zero-knowledge deferred) · read-only AI interface · first adjacent uses: declared-vs-observed check, vendor attestation, product CBOM, diligence pack · multi-root pack signing · plus the v3 carry-overs (standard signing wrapper, two signatures, compliance lead first, remove old `scan`, no hub in v4.0, TLS probing notice).

### 0.2 What v5 adds, in plain words
v4 *knows, decides and proves*. v5 closes the loop so the same system can also **fix, share and plan**:

| Job | v5 capability |
|---|---|
| **Fix** | Per-stack remediation recipes; a safe "plan → approve → apply → verify" loop; **verification harness** that independently tests any change (made by a person, a script or an AI agent); signed **remediation receipts** |
| **Know (more)** | **Preflight** test of the whole network path before turning on hybrid key exchange; key-exchange coverage and authentication coverage reported separately per hop; **facet sensors** for JWKS/OIDC, SAML, DNSSEC, SSH, and a cryptographic facet for non-human and AI-agent identities |
| **Share** | **Verifiable trust page**; scoped **attestation exchange**; questionnaire answers drawn only from verified evidence |
| **Plan** | **Cost and ROI planner** (empty, user-calibrated coefficients); migration **simulator** on the dependency graph; **horizon radar** that shows which of your assets a rule change affects |
| **Everywhere** | **Value Ledger** (what was automated, valued with the user's own numbers); **policy-as-code** kit for an organisation's own crypto standard |

### 0.3 Decisions needed from you
1. **Closed-loop thesis.** "Integrate, don't rebuild; verify, don't write." We never become a certificate authority, a scanner vendor or a code-writing AI. *Recommendation: yes.*
2. **Safety model for any automatic change.** Every action defaults to a dry run, needs a named human approval, uses narrowly scoped credentials, is reversible where possible, and is recorded in a signed audit log. No unattended changes to production in any v5.x release. *Recommendation: yes.*
3. **First v5 slice after v4.0.** "Fix-lite and Share-lite": recipes, preflight, coverage split, trust page, JWKS/DNSSEC sensors, value ledger, policy kit, cost-planner skeleton. *Recommendation: yes.*
4. **Cost planner ships with empty coefficients.** I found no trustworthy public unit-cost benchmarks, so users enter their own. *Recommendation: yes.*
5. **Wording of trust pages and attestations.** They state *observed posture at a time under stated coverage*, never "certified" or "quantum-safe". *Recommendation: yes.*
6. **Licence: DECIDED 28 Oct 2026.** Proprietary, all rights reserved; repository stays public for now. See Amendment 1.
7. **Hub.** Design the file and API contracts now; build the server only after real demand. *Recommendation: yes.*
8. **VeriChain anchoring.** Optionally publish report roots to a transparency log (VeriChain could be one backend). Never required; verification stays offline. *Your call.*

### 0.4 Corrections and cautions
- In my v5 value map I first left literal citation markup and invented index numbers in the text. I fixed it; every figure there now names its source in plain words.
- Figures on machine-identity ratios vary from 45:1 to 109:1 across sources. Treat as direction, not fact.
- Unit-cost claims in vendor material (for example 3–5× for emergency migration, $840K per cycle) are unverified. The planner does not use them.
- I could not push to GitHub from this chat (no connector, no credentials). See `GITHUB_CONNECT_AND_PUSH_GUIDE.md`.

---

## 1. Introduction and Goals

### 1.1 Added requirements (extends R-F1…R-F26, R-Q1…R-Q9)
| ID | Requirement |
|---|---|
| R-F27 | Per-stack remediation recipes as data, mapped to detected library/product versions |
| R-F28 | Plan → approve → apply → verify loop with signed receipts and audit log |
| R-F29 | Verification harness: independent contract tests for any change, whoever made it |
| R-F30 | PQ path preflight with per-hop key-exchange and authentication coverage |
| R-F31 | Facet sensors: JWKS/OIDC, SAML metadata, DNSSEC, SSH; non-human/AI-agent cryptographic facet via import |
| R-F32 | Verifiable trust page and attestation exchange |
| R-F33 | Evidence-grounded questionnaire answers; refuse when evidence is missing |
| R-F34 | Cost/ROI planner with user-calibrated coefficients; scenario table |
| R-F35 | Migration simulator on the dependency graph |
| R-F36 | Horizon radar: change detection, pack-diff drafting, impact analysis via time-travel |
| R-F37 | Value Ledger with user-set coefficients |
| R-F38 | Policy-as-code authoring kit and linter for organisation-specific standards |
| R-Q10 | **Action safety:** no write action without dry run, approval, scoped credential and audit entry |
| R-Q11 | **Honest wording:** shared artefacts never claim certification or full quantum safety |

### 1.2 Quality goals (v4 ranking plus one)
1 Trust · 2 Longevity · 3 **Safety of action (new)** · 4 Usefulness · 5 Privacy/offline · 6 Interoperability. Safety ranks third because v5 is the first release that can change things in the world.

### 1.3 Stakeholders added
Platform/SRE teams (apply changes), change-advisory boards (approve), AI coding agents (propose changes; treated as untrusted producers), trust-centre and questionnaire tools (consume attestations), counterparties (verify them).

---

## 2. Constraints (additions)
- Write actions only through connectors with scoped credentials; never through ad-hoc shell.
- Receipts and audit logs use the v4 evidence layer (hybrid signatures, chain).
- Harness runs in a sandbox (container or OS sandbox) with no production credentials.
- No default cost coefficients and no default savings claims.

---

## 3. Context and Scope (delta)

```
 Sensors + imports (v4) ─► STORE ─► EVALUATOR ─► PLAN ─► ACT ──────────► VERIFY ─► RECEIPT
                                      │            │     (connectors)     (harness)   │
                                      │            │                                  ▼
                                      │            └─► Cost / simulator          EVIDENCE (v4)
                                      ▼                                                │
                               ADVISOR + recipes                                       ▼
                                                                 SHARE: trust page, attestations,
                                                                        questionnaire answers
 Connectors out:  CLM/ACME · ITSM (Jira, ServiceNow) · GRC (Vanta, Drata) · cloud KMS (read-only)
                  trust centres · SIEM/webhook · AI agents (read-only MCP + patch proposals in)
 Connectors in:   discovery CBOMs · NHI platforms · declared inventories
```

| New interface | Format | Direction |
|---|---|---|
| Remediation plan | JSON `remediation-plan/1` | internal, signed on approval |
| Receipt | in-toto Statement, predicate `remediation-receipt/1` | out |
| Connector contract | `vericrypt-connector/1` (JSON lines over stdio, declared scopes, dry-run flag) | in/out |
| Attestation | in-toto Statement, predicate `posture-attestation/1` (scoped, with coverage ledger) | out |
| Trust page | static HTML + WASM verifier + attestation bundle | out |
| Questionnaire map | pack kind `questionnaire-map` | in |
| Patch proposal | unified diff + declared contract | in (from agents/humans) |

Out of scope for v5: acting as a CA or CLM, writing code with our own model, unattended production changes, mainframe and eBPF discovery.

---

## 4. Solution Strategy (additions)
| Goal | Strategy |
|---|---|
| Close the loop without scope explosion | Thin connectors; recipes and mappings as packs; reuse v4 evidence layer for receipts and attestations |
| Safe action | Dry-run default, approval step, scoped credentials, signed audit log, reversibility, blast-radius limits |
| Trustworthy AI help | **AI proposes, VeriCrypt proves:** agents submit patches; the harness verifies against an independent contract |
| Honest sharing | Attestations carry coverage and time; wording rules enforced by templates |
| Planning without invented numbers | Planner structure ships; coefficients come from the user |

---

## 5. Building Block View (delta to v4 §5)

### 5.1 New blocks
| Block | Responsibility |
|---|---|
| **Recipe engine** | Match findings and detected versions to recipes (from `remediation-recipes` pack); produce a plan |
| **Orchestrator** | Run the plan through connectors: dry-run → approval → apply → verify; write audit log |
| **Verification harness** | Sandbox that runs a **contract**: rescan, interoperability against a second implementation, negative tests (tampered inputs, wrong keys), regression checks; returns pass/fail with evidence |
| **Receipt builder** | Before/after scans + change description + harness result → signed receipt |
| **Preflight sensor + path graph** | Test full path with hybrid ClientHello variants; per-hop coverage; vantage points |
| **Facet sensors** | JWKS/OIDC, SAML metadata, DNSSEC, SSH |
| **Identity importer** | Import non-human/agent identities; attach cryptographic attributes |
| **Attestation service** | Create/verify scoped attestations; build trust page bundle |
| **Answer service** | Map questionnaire items to controls/tests; answer only from verified evidence; cite proofs |
| **Planner** | Cost scenarios, scenario analysis, simulator on the graph |
| **Radar / pack factory** | Watch sources, draft pack diffs, run impact analysis through time-travel |
| **Value Ledger** | Count automated actions and apply user coefficients |
| **Policy kit** | Authoring guide, schema, linter for organisation packs |
| **Connector framework** | Contract, scopes, dry-run, retries, idempotency |
| **Hub contracts** | Stable ingest/API schemas; no server in v5.0 |

### 5.2 Level 2 — Remediation lifecycle
`finding → recipe match → plan (steps, scopes, rollback) → dry run (diff shown) → approval (named person, signed) → apply (connector) → verify (harness contract) → rescan → receipt (signed) → audit log entry → dashboard "Fixed" with proof`.
Plans are data and are signed at approval so what was approved is what runs.

### 5.3 Level 2 — Verification harness contract
Declares: *subject* (service/artifact), *claims* (for example "negotiates X25519MLKEM768 and still accepts classical clients"), *checks* (positive, negative, interop, regression), *oracles* (second implementation such as OpenSSL/rustls/Go), *evidence* (transcript hashes). Rationale: local tests alone are not enough; a study of coding agents on post-quantum migration found patches that passed local verification but failed external requirements.

### 5.4 Level 2 — Preflight and coverage
Per hop (edge → load balancer → origin → internal service): **key-exchange coverage** (hybrid negotiated: yes/no/untested) and **authentication coverage** (PQ or classical certificate chain) are separate fields. A hop counts as covered only when a probe or telemetry shows a *successful* negotiation. Preflight sends hybrid ClientHello variants (normal, large, fragmented) and records resets, timeouts, alerts and fallback.

### 5.5 Level 2 — Planner
Inputs: inventory counts by class, dependency graph, user coefficients (effort per asset class, unit costs, rates), scenario histogram of Q-Day years. Outputs: low/central/high cost scenarios, critical-path ordering, cost-of-delay table. Everything is integer arithmetic over stated inputs; uncalibrated outputs are labelled **UNCALIBRATED**.

### 5.6 Level 2 — Attestation and trust page
An attestation is a signed, **scoped** statement: what was assessed, when, by which sensors, the coverage ledger, the selected claims with proofs. The trust page is a static bundle (HTML + browser verifier + attestation). Templates forbid "certified", "compliant" and "quantum-safe" unless a control result literally says so under a named framework.

---

## 6. Runtime View (additions)

**RV-9 Fix loop.** Finding → plan → dry run → approval → apply → harness → rescan → receipt → ledger entry.

**RV-10 Preflight before enabling hybrid key exchange.** Select service + vantage points → run variants → report per-hop results and failures → only then offer the recipe.

**RV-11 Publish trust page.** Choose claims → build disclosure bundle → render page with browser verifier → publish to the user's own static hosting. Visitors verify locally.

**RV-12 Questionnaire.** Import questions → map to controls/tests → for each item return answer + evidence reference, or **"no verified evidence"** → human reviews → export to the trust-centre tool.

**RV-13 Horizon radar.** Source change detected → draft pack diff → human review and signature → time-travel impact analysis over stored scans → "N assets affected; cost range X–Y (uncalibrated)".

**RV-14 AI-agent patch.** Agent reads findings via read-only MCP → proposes diff + contract → harness verifies in sandbox → pass: ready for human approval; fail: report which check failed.

**RV-15 Cost scenario.** Load coefficients → compute scenarios → show assumptions → export board summary.

---

## 7. Deployment View (additions)
- **Connector credentials:** held in the operator's secret store; passed to connectors per run; never written into reports or the store.
- **Sandbox:** container or OS sandbox for the harness; no production network by default; explicit allowlist for oracle servers.
- **Trust page hosting:** the user's own static hosting; no VeriCrypt-run service required.
- **Hub (not built in v5.0):** stateless ingest, attestation registry, optional benchmarks; contracts frozen early.

---

## 8. Cross-cutting Concepts (additions)

**8.1 Action safety model.**
| Control | Rule |
|---|---|
| Dry run first | Every action shows exact changes before execution |
| Approval | Named human; two-person rule configurable; approval is signed over the plan hash |
| Scoped credentials | Least privilege per connector and per plan; short-lived where the target supports it |
| Reversibility | Plans declare rollback; irreversible steps are flagged and need explicit approval |
| Blast-radius limits | Maximum targets per run; staged rollout; stop-on-failure |
| Audit | Signed, chained audit log of who approved, what ran, results |
| No unattended production changes | Out of scope for all v5.x |

**8.2 Receipts.** Statement subjects: before-scan and after-scan digests; predicate: plan hash, approver, connector, harness contract and result, timestamps. Receipts are evidence that a control *operated*.

**8.3 Untrusted producers.** Patches from AI agents or contractors are untrusted input. The harness, not the author, decides pass/fail. All scanned text used in agent prompts is escaped and flagged `untrusted_text` (v4 §8.9).

**8.4 Calibration policy.** The planner, ledger and any ROI view require user-entered coefficients; outputs without them display **UNCALIBRATED**. We never ship default savings or cost numbers.

**8.5 Honest wording.** Shared artefacts include: observed-at time, coverage ledger summary, "observation-based; not a certification". Claims such as "PQ-protected link" require a recorded successful negotiation.

**8.6 Connectors.** Contract `vericrypt-connector/1`: declared scopes, dry-run support, idempotency keys, structured results, no direct store access. Reference connectors: ACME/cert-manager, ServiceNow/Jira, Vanta/Drata, cloud KMS (read-only first).

**8.7 Claims registry (extended).** Every capability claim, including remediation outcomes and questionnaire coverage, maps to tests; CI blocks releases otherwise.

---

## 9. Architecture Decisions (all PROPOSED; continue v4 numbering)

| ID | Decision | Rationale | Alternatives rejected |
|---|---|---|---|
| ADR-22 | Closed-loop thesis: integrate, don't rebuild; verify, don't write | Scope control; plays to strengths | Build scanner/CLM/code AI |
| ADR-23 | Action safety model (dry run, approval, scoped credentials, audit, no unattended production changes) | Safety of first write-capable release | Autonomous remediation |
| ADR-24 | Remediation recipes as a signed pack kind, matched to detected versions | Content not code; updates without release | Hard-coded advice |
| ADR-25 | Verification harness with independent contract and oracles | Local tests miss external requirements | Trust patch author's tests |
| ADR-26 | Signed remediation receipts using v4 evidence layer | Evidence that controls operated | Free-text change logs |
| ADR-27 | Preflight and per-hop coverage; key-exchange and authentication coverage reported separately | Avoids false "PQ-ready"; matches confidentiality/authenticity split | Single readiness percentage |
| ADR-28 | Facet sensors as plugins (JWKS/OIDC, SAML, DNSSEC, SSH); identity importer for NHI/agents | Cheap breadth; machine identities dominate | Build own NHI platform |
| ADR-29 | Scoped attestations and static trust page with browser verifier; wording templates | Distribution plus honesty | Hosted trust centre service |
| ADR-30 | Questionnaire answers only from verified evidence; refusal otherwise | Differentiates from policy-based AI answers | Generative answers |
| ADR-31 | Planner with user coefficients, UNCALIBRATED labelling, deterministic arithmetic | No invented numbers | Ship benchmark defaults |
| ADR-32 | Simulator over the v4 graph, deterministic | Sequencing and cost-of-delay without ML | Predictive models |
| ADR-33 | Horizon radar and pack factory with human sign-off; impact via time-travel | Maintenance burden and user value | Fully automated pack updates |
| ADR-34 | Value Ledger with user coefficients | Honest productivity evidence | Marketing claims |
| ADR-35 | Policy-as-code kit using the existing pack mechanism | Cheap, high value | New DSL |
| ADR-36 | Connector contract and framework; hub contracts frozen, server deferred | Ecosystem fit without a server | Build hub now |
| ADR-37 | Optional public anchoring of report roots; never required | Non-repudiation option, VeriChain-compatible | Mandatory anchoring |
| ADR-38 | Licence model: pre-issued, signed PASETO v4.public token bound to organisation, expiry and a feature list; verified offline against an embedded public key; no activation server and no hardware binding in pilots; revocation list and version ceiling deferred | Fits offline and air-gapped buyers; hardware binding is the usual alternative but brittle when machines change | Activation server; machine-fingerprint binding |
| ADR-39 | Release assurance: each release ships SHA256SUMS, a CycloneDX SBOM and detached SLH-DSA signatures (`sign-file`); customers verify offline (`verify-file`); CI build-provenance attestation is an optional extra | What bank third-party reviews ask of software vendors; reuses our own evidence stack | Unsigned binaries |
| ADR-40 | Regulatory dates live in data packs, not code; a minimal `jurisdiction` pack format moves into v4.0 slice 1 | NIST IR 8547 became final on 27 Oct 2026 and rule code had to change | Hard-coded dates |
| ADR-41 | Deep TLS sensor also covers STARTTLS (SMTP, IMAP, POP3, FTP) and common database protocols | A free competitor probe already covers them | TLS-only |
| ADR-42 | Position as the evidence layer next to suite vendors; treat PQ gateways (IBM Proxy and similar) as remediation targets, not something we build | IBM launched Cryptography Manager, Explorer and Proxy on 27 Oct 2026 | Build a gateway |
| ADR-43 | Claims registry enforced in CI (`scripts/check_claims.py`) | Already caught a stale claim during this round | Manual review |
| ADR-44 | Documentation policy: code first; arc42 changes recorded as dated amendments until the structure changes | Stated preference: functionality over process | New document version each round |
| ADR-45 | Waivers are a risk-decision register: each has an owner, reason and expiry; findings stay visible and marked accepted; an expired waiver re-opens the finding and raises a governance finding; waivers are committed into the signed report | From 2031 NIST IR 8547 expects a recorded risk decision for every deprecated use; auditors will ask for it | Config-file ignore lists; silently suppressing findings |
| ADR-46 | Standard release set: signed binary, SHA256SUMS, SBOM (CycloneDX 1.6) and a product CBOM (CycloneDX 1.7), all with detached SLH-DSA signatures | Bank vendor reviews and the coming US CBOM guidance | Binary only |
| ADR-47 | One dependency-light binary: no OpenSSL (rustls with ring), no ASL build step, legacy v1 code removed | Portability for customer machines; smaller attack surface; faster builds | Keep legacy modules "for later" |
| ADR-48 | First pilot segment: Trinidad and Tobago regulated institutions, evidence for the 31 March 2027 return | Primary-source deadline, home market | Broad launch |

---

## 10. Quality Requirements (additions)
| ID | Scenario | Target | Status |
|---|---|---|---|
| Q-17 | Dry run | zero writes; diff shown | to build |
| Q-18 | Approval | execution refuses without valid signed approval over the plan hash | to build |
| Q-19 | Harness | rejects a patch that passes its own tests but fails the independent interop contract | to build |
| Q-20 | Receipt | verifies offline; tampering fails | to build |
| Q-21 | Preflight | correctly flags failures on emulated middleboxes (hardcoded buffer, outdated group list) | to build |
| Q-22 | Coverage split | key-exchange and authentication coverage never merged in any output | to build |
| Q-23 | Trust page | verifies in a browser with no network in under 2 s for a typical bundle | to measure |
| Q-24 | Questionnaire | 100% of answers carry evidence or the "no verified evidence" marker | to build |
| Q-25 | Planner | all outputs labelled UNCALIBRATED until coefficients entered | to build |
| Q-26 | Honest wording | no shared artefact contains "certified" or "quantum-safe" unless a control result says so | to build |
| Q-27 | Radar | impact analysis of a new pack over 12 months of scans < 5 min | to measure |
| Q-28 | Licence | genuine key activates; expired, other-key and forged licences refused | demonstrated (`scripts/verify_licence_e2e.sh`) |
| Q-29 | Release verification | a customer verifies binary and SBOM offline | demonstrated (file-signature test) |
| Q-30 | Static build | musl binary runs on a host without OpenSSL 3 | to build and test |
| Q-31 | Pack-driven dates | changing a deprecation date needs no code change | to build |
| Q-32 | STARTTLS probe | correct against local SMTP and IMAP servers | to build |

---

## 11. Risks and Technical Debt (additions)
| ID | Risk | Mitigation |
|---|---|---|
| R-19 | AI-proposed changes are wrong or unsafe | Harness with independent contract; human approval; sandbox |
| R-20 | Connector actions cause outages | Dry run, staged rollout, blast-radius limits, rollback, stop-on-failure |
| R-21 | Liability for automated or recommended changes | Approval by named human; audit log; clear wording; legal review of terms |
| R-22 | Scope explosion for a solo builder | v4.0 first; data packs, thin connectors, plugins only; slice gating |
| R-23 | Attestations misread as certification | Wording templates, coverage ledger, "observation-based" statement |
| R-24 | Questionnaire answers create contractual exposure | Evidence-only answers; human review; refusal marker |
| R-25 | Cost planner misused as a forecast | UNCALIBRATED label; assumptions shown; no defaults |
| R-26 | NHI/agent identity data quality varies by source | Facet marked with source and confidence |
| R-27 | Competitors respond (platform bundling, GRC native tests, MCP-based AI helpers) | Neutral proof layer; verifier stays free and open; integrate not compete |
| R-28 | Trust pages leak topology | Redaction profiles; scoped claims; review before publish |
| R-29 | Horizon radar errors propagate into packs | Human sign-off; pack status and `last_verified` |
| R-30 | Ecosystem support matrix goes stale | Dated recipes; radar; verify versions before publishing |
| R-31 | Licence design flaws (generator and verifier could never agree) | Fixed; end-to-end script runs in CI |
| R-32 | Regulation moves faster than releases (IR 8547 changed within a week) | ADR-40 data packs; horizon radar |
| R-33 | Public repository with a proprietary licence: source is visible, enforcement rests on licence terms | Counsel review; revisit a private repo or a split (private core) later; current decision recorded as public |
| R-34 | Suite vendors (IBM and others) bundle inventory, posture and proxy | Be the neutral evidence layer; integrate |
| R-35 | EO 14412 and OMB M-26-15 are corroborated by several sources; the German guideline and the primary texts remain unread | Read primary texts next round (partly resolved in Amendment 2) |
| R-36 | LICENSE is a plain-language draft not reviewed by counsel; entity name "Intellica AI Inc." differs from the GitHub organisation name | Confirm the legal entity; legal review |
| R-37 | Pilot return deadline (31 March 2027) is five months away and the CBTT guideline text has not been mapped to our evidence | Map the 20 requirements next; start pilot conversations now |
| R-38 | Remediation recipes could repeat wrong language defaults (a source conflict was found) | Cite primary release notes only |

---

## 12. Roadmap

| Release | Content | Exit criteria |
|---|---|---|
| v4.0–v4.3 | As in v4 | As in v4 |
| **v5.0 Fix-lite and Share-lite** | Recipes pack; preflight; coverage split; trust page; JWKS/OIDC and DNSSEC facet sensors; policy kit; value ledger; cost-planner skeleton | Q-21, Q-22, Q-23, Q-25, Q-26 |
| **v5.1 Closed loop** | Orchestrator; harness; receipts; ACME and ITSM connectors | Q-17…Q-20 |
| **v5.2 Exchange** | Attestation exchange; questionnaire answer service; trust-centre connectors; SAML and SSH facets | Q-24 |
| **v5.3 Plan and radar** | Calibrated cost scenarios; simulator; horizon radar; NHI/agent importer | Q-27 |
| **v5.4 Hub (only if demanded)** | Ingest, history, registry | Demand evidence |

---

### 12.1 Pilot track (runs ahead of the v4.0 build order)
1. Static musl build verified on a machine without OpenSSL 3.
2. Run one real, consented customer inventory; fix whatever it exposes.
3. Pilot paperwork (agreement, NDA, privacy and security-contact documents) and the product-name check.
4. Deep TLS slice first (versions, cipher suites, post-quantum groups, STARTTLS), because banks will ask the PQ-readiness question immediately.
5. Pack format for regulatory dates (ADR-40) so the next rule change needs no release.

## 13. Traceability (v5 requirements)
| Requirement | Decisions |
|---|---|
| R-F27 | ADR-24 |
| R-F28, R-Q10 | ADR-23, 26 |
| R-F29 | ADR-25 |
| R-F30 | ADR-27 |
| R-F31 | ADR-28 |
| R-F32, R-Q11 | ADR-29 |
| R-F33 | ADR-30 |
| R-F34, F35 | ADR-31, 32 |
| R-F36 | ADR-33 |
| R-F37 | ADR-34 |
| R-F38 | ADR-35 |
| Connectors / hub | ADR-36, 37 |

---

## 14. Glossary, sources, gaps

**Glossary (additions).** *Recipe* — signed, data-driven remediation steps for a stack. *Contract* — the set of independent checks a change must pass. *Receipt* — signed proof a remediation operated. *Preflight* — pre-rollout path test. *Hop* — one network segment/termination point. *KEX coverage / auth coverage* — share of hops with successful hybrid key exchange / PQ authentication. *Facet* — a narrow sensor for one protocol or identity type. *Value Ledger* — record of automated work valued with user coefficients. *UNCALIBRATED* — label for outputs lacking user coefficients. *Horizon radar* — change-monitoring and impact feature.

**Sources.** See `V5_VALUE_MAP_AND_INTEGRATION.md` §9 and `ARCHITECTURE_V4_arc42.md` §14.2. Added for v5: Cloudflare PQC support matrix (developer docs); Encryption Consulting and postquantum.com deployment guides; arXiv 2607.29005 (policy vs reality in PQ TLS); arXiv 2512.12989 (coding agents and PQ migration); arXiv 2506.00790 (mobile apps); questionnaire-tool comparisons (competitor-authored); non-human-identity reports (CSA, vendor).

**Known gaps.** No user interviews; no measured effect of any v5 feature; harness effort and sandbox design unproven; connector terms of service unchecked; DNSSEC post-quantum status unchecked; support-matrix versions need verification before recipes ship; legal review of attestation and questionnaire wording not done; cost benchmarks absent by design.
