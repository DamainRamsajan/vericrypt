# VeriCrypt — Users, Requirements and Use Cases (v4 input)

As of 22 Oct 2026. **This is desk research, not user research.** No interviews have been done. Section 6 is the plan to validate everything here with real people before building.

---

## 1. Users in six clusters

| Cluster | Personas | Core job |
|---|---|---|
| **Operate** | Security/PKI engineer · DevOps/SRE · Software developer | Know what exists, fix what is broken, never get surprised by expiry |
| **Comply** | Compliance/GRC lead · CISO · Risk manager · Board/audit committee | Prove to auditors and regulators; decide where money goes |
| **Assure others** | Auditor/QSA · Customer or vendor-risk analyst · Cyber insurer · M&A diligence team · Regulator/supervisor | Verify a claim someone else makes, without trusting them |
| **Serve many** | MSSP / consultant | Repeat the work for many clients with consistent quality |
| **Specialist** | Crypto-asset custodian/exchange · Product manufacturer (EU CRA) · Government/defence supplier · Telecom/OT/IoT operator | Meet sector-specific rules |
| **Public interest** | National CERT / sector observatory / researchers | Measure and improve posture across many organisations |

---

## 2. Persona detail: needs, pains, requirements

IDs reference the requirements catalogue in §3.

| Persona | What they need to do | Pain (evidence) | Must-have requirements | Success looks like |
|---|---|---|---|---|
| **Security / PKI engineer** | Find all certs/keys, get a prioritised fix list, get alerts | Managing keys, certificates and secrets is "very or extremely difficult" for 68% (Entrust/Ponemon 2026); only a third of engineers say the inventory is current (Axiad) | R-F1, F2, F4, F6, F8, Q3 | Fix list in minutes; zero expiry surprises |
| **DevOps / SRE** | Automate renewals before 100/47-day caps; gate CI | Outages from expired certificates: >1/3 had one in a year (DigiCert survey) | R-F4, F9, F10, Q4 | CI fails on new vulnerable crypto; renewal burden known |
| **Software developer** | Know which crypto calls/libraries are risky; stay crypto-agile | Skills shortage in PQC testing and deployment (TCG) | R-F9, F11, Q4 | Crypto linting in the PR |
| **Compliance / GRC lead** | Produce crypto evidence for PCI, ISO, SOC 2, DORA each year | Evidence collection is the biggest compliance cost (staff time 30–40% of budgets) | R-F3, F7, F12, Q2 | One-click evidence pack; pushed into the GRC platform |
| **CISO** | Know the real posture; justify budget; report to board | Confidence gap: 90% of CISOs/CIOs believe inventory is current vs 33% of engineers | R-F5, F13, F14, Q1 | One page that cannot be inflated; trend; deadline countdown |
| **Risk manager** | Quantify exposure and timing; scenarios | Only 30–38% budget/prepare; unsure how long they have | R-F5, F15 | Scenario table with stated assumptions |
| **Board / audit committee** | Understand exposure in plain words | Boards must show technical literacy (ASIC outlook cited by AUSCERT) | R-F13, Q6 | Three-slide summary, no jargon |
| **Auditor / QSA** | Verify evidence without redoing the work | Evidence arrives as screenshots and spreadsheets | R-F7, F16, Q1, Q2 | Verifies offline in minutes; traces control → test → evidence |
| **Vendor-risk / customer analyst** | Assess suppliers' PQC readiness | Questionnaires are unverified; weakest dependency may be several tiers down | R-F17, F16 | Receives a signed, scoped attestation, not a PDF |
| **Cyber insurer / broker** | Price readiness and vendor exposure | Industry still at education/scenario stage, not pricing | R-F16, F18 | Standard indicators, verifiable |
| **M&A diligence team** | Assess a target's crypto posture and HNDL exposure | "Quantum-infused" diligence now recommended by law firms | R-F18, F16 | Point-in-time signed pack, selective disclosure, no data room leak |
| **Regulator / supervisor** | Collect comparable readiness data from regulated firms | Fragmented templates; self-reported | R-F19, F16 | Machine-readable, verifiable submissions |
| **MSSP / consultant** | Run it for many clients, brand it | Rebuilding spreadsheets per client | R-F20, Q5 | One engagement template; consistent evidence |
| **Crypto custodian / exchange** | Know key and address exposure (reuse, exposed public keys) | Exchanges hold ~40% of operationally exposed BTC (Glassnode via Decrypt) | R-F21 | Exposure report and hygiene plan |
| **Product manufacturer (CRA)** | Include crypto transparency in technical documentation | CRA reporting live since 11 Sep 2026; SBOM duties from 11 Dec 2027; no mandated format | R-F22 | CBOM bundled with SBOM per product release |
| **Government / defence supplier** | Meet CNSA 2.0 acquisition expectations from Jan 2027 | Vendors must show algorithm support and roadmap | R-F16, F23 | Verifiable capability statement |
| **Telecom / OT / IoT operator** | Firmware-signing and long-lived device crypto | Long lifecycles; constrained devices | R-F23 (later) | Fleet-level exposure view |
| **National CERT / observatory** | Measure public TLS posture of a country's domains | Need baselines and trends | R-F24 | Periodic public-data report, ethical guardrails |

---

## 3. Requirements catalogue (traceable)

### 3.1 Functional

| ID | Requirement | Primary users |
|---|---|---|
| R-F1 | Discover certificates and keys from files, TLS endpoints, SSH; import CBOMs from other tools | Engineer, GRC |
| R-F2 | Deep TLS: versions, cipher suites, key-exchange groups, chain, OCSP; edge-vs-origin discrimination | Engineer, GRC, auditor |
| R-F3 | Control-mapped evidence packs (PCI 4.0.1, ISO A.8.24, DORA, others via packs) | GRC, auditor |
| R-F4 | Expiry and renewal-burden forecasting (200/100/47 days) with owners and alerts | Engineer, SRE |
| R-F5 | Deterministic scoring: confidentiality vs authenticity risk, migration time, crypto-agility; scenario analysis | CISO, risk |
| R-F6 | Ecosystem-aware remediation advice (public WebPKI vs private PKI vs code signing vs S/MIME), recipes | Engineer |
| R-F7 | Verifiable evidence: signed, hash-chained, timestamped, offline-verifiable | Auditor, GRC |
| R-F8 | Context file: owner, data class, shelf life, criticality | Engineer, CISO |
| R-F9 | CI mode: gates, SARIF, "ratchet" (no new vulnerable crypto) | DevOps, developer |
| R-F10 | Scheduled scans with drift diff and notifications | SRE |
| R-F11 | Library/dependency crypto-agility signals (from imported CBOM/SBOM) | Developer |
| R-F12 | GRC push (Vanta, Drata) and OSCAL export | GRC |
| R-F13 | Dashboard (Overview, Inventory, Calendar, Evidence, Changes) plus board summary | CISO, board |
| R-F14 | Declared-vs-observed reconciliation (CMDB/asset-list import) and coverage ledger | CISO, GRC |
| R-F15 | Time-travel: re-evaluate any past inventory under new rule packs | Risk, auditor |
| R-F16 | Selective-disclosure bundles with absence proofs; redaction profiles | Vendors, M&A, insurer |
| R-F17 | Vendor attestation: issue and verify signed scoped statements; map to questionnaires | Vendor-risk |
| R-F18 | Assurance packs for diligence/insurance | M&A, insurer |
| R-F19 | Supervisor submission formats via jurisdiction packs | Regulator |
| R-F20 | Multi-client workspaces and white-label reports | MSSP |
| R-F21 | Crypto-asset exposure sensor (address reuse, exposed public keys) | Custodian |
| R-F22 | Product CBOM per release (bundled with SBOM) | Manufacturer |
| R-F23 | Firmware/code-signing inventory (LMS/XMSS/ML-DSA) | Gov supplier, OT |
| R-F24 | Public-data observatory mode with guardrails | CERT |
| R-F25 | Waivers (signed, owned, expiring) | GRC, CISO |
| R-F26 | AI-assistant interface (read-only MCP) returning answers with evidence references | All |

### 3.2 Quality

| ID | Requirement |
|---|---|
| R-Q1 | **Trust:** every number computed from observations; every claim backed by a test; coverage and limits always shown |
| R-Q2 | **Auditor usability:** control → test → result → evidence traceable without help |
| R-Q3 | **Speed to value:** first useful result within 15 minutes of download |
| R-Q4 | **Fits workflows:** works in CI, cron, air-gapped hosts, standard formats |
| R-Q5 | **Scale:** thousands of assets per scan; many clients per operator |
| R-Q6 | **Plain language:** every finding explains why it matters and how to fix it; board summary without jargon |
| R-Q7 | **Longevity:** reports remain verifiable for 10+ years (algorithm agility, evidence renewal, stable formats) |
| R-Q8 | **Privacy:** inventories are sensitive; redaction and local-first by default |
| R-Q9 | **Accessibility:** WCAG-conformant dashboard, keyboard navigation, print-friendly |

### 3.3 Constraints and non-goals
Offline-first core; no mandatory cloud; single binary; no source-code scanner of our own; not a CA or CLM; no claim of "proof" beyond the formal kernel.

---

## 4. Use-case catalogue

Scored 1–5 on **Value** (pain evidence), **Fit** (reuses the planned architecture), **Effort** (5 = small), **Evidence** (quality of demand evidence). Priority = judgement, not arithmetic.

| ID | Use case | V | F | E | Ev | Phase |
|---|---|---|---|---|---|---|
| UC-1 | Annual audit evidence pack (PCI, ISO, DORA) | 5 | 5 | 4 | 4 | v4.0 |
| UC-2 | Expiry and 47-day readiness with owners | 5 | 5 | 5 | 4 | v4.0 |
| UC-3 | PQ-TLS posture: edge vs origin, per-group truth | 4 | 4 | 3 | 5 | v4.0 |
| UC-4 | Migration prioritisation (risk × migration time) | 4 | 4 | 3 | 4 | v4.0 |
| UC-5 | CI gate / crypto "ratchet" | 4 | 5 | 4 | 3 | v4.0 |
| UC-6 | Verifiable report shared with a third party | 5 | 5 | 4 | 4 | v4.0 |
| UC-11 | Declared-vs-observed reconciliation (shadow crypto) | 5 | 4 | 3 | 4 | v4.1 |
| UC-7 | Vendor/customer PQC attestation + questionnaire answers | 4 | 4 | 3 | 4 | v4.1 |
| UC-14 | AI assistant queries with verifiable answers (MCP) | 4 | 4 | 4 | 3 | v4.1 |
| UC-12 | Retroactive re-evaluation when rules change | 4 | 5 | 4 | 3 | v4.1 |
| UC-8 | M&A / insurance diligence pack with selective disclosure | 4 | 4 | 3 | 3 | v4.2 |
| UC-10 | Product CBOM for EU CRA technical documentation | 4 | 4 | 3 | 3 | v4.2 |
| UC-15 | MSSP multi-client operation | 3 | 3 | 3 | 3 | v4.2 |
| UC-13 | Scenario modelling (Mosca distributions, "if Q-Day is year X") | 3 | 4 | 4 | 3 | v4.2 |
| UC-9 | Supervisor submission via jurisdiction pack | 3 | 4 | 3 | 3 | v4.3 |
| UC-16 | Crypto-asset exposure (address reuse, exposed keys) | 3 | 3 | 3 | 4 | v4.3 (plugin) |
| UC-17 | NHI / AI-agent identity crypto inventory | 3 | 3 | 2 | 3 | v4.3+ |
| UC-18 | Firmware / code-signing inventory | 3 | 3 | 2 | 3 | later |
| UC-19 | National/sector observatory (public data only) | 3 | 4 | 3 | 2 | later |
| UC-20 | Tabletop and training simulation | 2 | 3 | 3 | 2 | later |
| UC-21 | Procurement checker (verify supplier algorithm claims, e.g. CNSA 2.0) | 3 | 4 | 3 | 3 | later |

**Notes on weak spots:** insurance pricing use is still exploratory per CyberCube, so UC-8 is a diligence pack first. CRA mandates SBOMs, not CBOMs: UC-10 is an opportunity, not a legal requirement (to be validated). UC-19 touches ethics and law (scanning third parties): public data only, rate limits, published guidelines.

---

## 5. Key user journeys

**J-1 Audit season (compliance lead).** Pull last scan → `analyze` with context → open Evidence screen → see PCI 4.2.1.1 and 12.3.3 results with gaps flagged → export pack → push to GRC → auditor verifies signature offline → next year: re-run, diff, show improvement.

**J-2 Migration planning (CISO + engineer).** Load context (data shelf life, owners) → scores place assets in start-now / plan / monitor → calendar shows regulator dates plus certificate caps → advisor gives public-TLS vs private-PKI steps → ratchet gate stops new vulnerable crypto → quarterly diff shows progress.

**J-3 Third-party assurance (vendor and customer).** Vendor runs scan → generates scoped attestation (selective disclosure, absence proofs) → customer's tool verifies offline and maps to its questionnaire → mismatches against declared answers are flagged.

**J-4 Day-in-the-life (engineer).** Morning alert: three certificates expire in 21 days (owners named) → fix → ask the assistant "what changed since Monday?" → answer cites report IDs.

---

## 6. Validation plan (before we commit scope)

| Step | Action | Output |
|---|---|---|
| 1 | 12 interviews: 3 compliance leads, 3 PKI/security engineers, 2 auditors/QSAs, 2 vendor-risk analysts, 1 insurer/broker, 1 M&A/legal | Ranked pains; confirm or kill UC-7/8/10 |
| 2 | Show a clickable mock of the five dashboard screens | Which screen is the home page for each persona |
| 3 | Take 3 real inventories (consented, anonymised) through v2 and compare to their own spreadsheets | Evidence for the confidence-gap thesis; find parser gaps |
| 4 | Ask an assessor to review the PCI/ISO/SOC 2 mapping | Confirm or correct control mapping |
| 5 | Test the verifier with a non-technical auditor | Can they verify unaided? |

**Key assumptions to test:** (A1) compliance leads are the best first buyers; (A2) engineers will accept a CLI-first workflow; (A3) auditors value offline verification; (A4) vendor-attestation exchange has a counterparty willing to consume it; (A5) mid-market and regulated organisations prefer a lightweight tool over a platform.
