# VeriCrypt — Competitive Analysis (v4 input)

As of 22 Oct 2026. Desk research only: public pages, vendor and rival comparisons, press, preprints. **No hands-on trials of competitor products, no verified pricing except where cited, no analyst reports (Gartner/Forrester) accessed.** Treat every competitor capability below as "as far as I found".

---

## 1. Market map (five segments, one gap between them)

| Segment | Representative players | What they sell | Signals |
|---|---|---|---|
| **A. Crypto discovery / posture (CPM)** | SandboxAQ AQtive Guard; IBM Guardium Quantum Safe (+ Quantum Safe Explorer); Keyfactor (AgileSec via InfoSec Global and CipherInsights, 2025); Entrust Cryptographic Security Platform; Encryption Consulting CBOM Secure; QCecuring CBOM; Fortanix | Find cryptography across network, hosts, code; CBOM; prioritisation; some remediation workflows | AQtive Guard uses three analyzers and host sensors deployed via CrowdStrike Falcon; it is also expanding into AI security posture. A market report says SandboxAQ got a USD 500M US Commerce commitment in June 2026 and is positioned as the federal discovery tool (claim not independently verified). Entrust and IBM Consulting announced a joint migration offering in April 2026 |
| **B. Certificate lifecycle / PKI (CLM)** | Venafi → CyberArk (Oct 2024, $1.54B) → Palo Alto Networks (closed 11 Feb 2026); Keyfactor; DigiCert; Sectigo; AppViewX; Entrust PKI Hub; Akeyless; HashiCorp Vault | Issue, renew, automate certificates and machine identities | Heavy consolidation. One comparison puts Keyfactor mid-market at roughly $75k–$200k a year; DigiCert moved to seat-based licensing in Oct 2025; buyers cite per-identity pricing and post-acquisition roadmap uncertainty |
| **C. GRC / trust platforms** | Vanta, Drata, Secureframe, ServiceNow GRC | Collect evidence, map controls, run audits | Accept pushed evidence (Vanta custom resources/tests; Drata custom connections), all shipping **MCP servers** for AI assistants; ServiceNow imports/exports OSCAL Assessment Results |
| **D. Services / migration programmes** | Accenture, Deloitte, EY, IBM Consulting, boutique firms | Inventory projects, roadmaps, DORA+PQC gap assessments | Large banks buy this as a project; deliverables are documents, not verifiable data |
| **E. Open source / DIY** | PQCA **CBOMkit**; **Trivy** crypto scanning; Sonar Cryptography plugin; **CBOM-Lens**; CryptoBOM Forge; PQ-TLS probes (pqprobe, pqready, pqcprobe, certmonitor); classic TLS auditors (testssl.sh, sslyze, sslscan); Cloudflare Radar PQ test | Free discovery for code, containers, filesystems, ports, TLS | Discovery is commoditising here; the PQ-TLS probes are small single-purpose tools |

**The gap between segments:** A finds, B renews, C collects, D advises, E scans. **Nobody I found makes the output independently verifiable, correct about the *current* migration path, usable offline, and portable across all five.**

---

## 2. What the market is telling us (demand-side evidence)

| Signal | Evidence | Implication |
|---|---|---|
| **Confidence gap** | Axiad (July 2026, n=315, US 500+ employees): 75% claim a continuously updated crypto inventory, but 90% of CISOs/CIOs vs 33% of security architects/PKI engineers say so; 22% contradicted themselves. Keyfactor: 92% claim visibility, 47% actively monitor | Leaders believe more than practitioners know. A tool that *proves* its claims and flags stale inventory is valuable to both |
| **Visibility is the top blocker** | Entrust/Ponemon 2026: 41% name inability to inventory as their biggest challenge; only 43% report full visibility; only 38% are preparing (down from 41%) | Inventory still unsolved for most; budgets lag |
| **Compliance spend is evidence labour** | Staff time 30–40% of compliance budgets; platforms 20–25%; audits 15–20% (Security Boulevard); GRC budgets: headcount 28%, software 24% (Hyperproof) | Tools that cut evidence-gathering hours win budget |
| **Recurring crypto audit requirements** | PCI DSS 4.0.1 4.2.1.1 and 12.3.3 (since 31 Mar 2025); ISO 27001 A.8.24; DORA RTS 2024/1774 Art. 6–7 | Annual demand independent of quantum timelines |
| **Outage cost** | DigiCert 2026 survey (vendor): >1/3 had an expired-certificate outage in the past year; ~1 in 4 worst incident >$250k; ~10% automated | Expiry/47-day automation is a pain with a price tag |
| **Deadlines fragmenting** | EU roadmap (inventories by end-2026, 2030, 2035); G7 financial roadmap; UK 2028/2031/2035; CNSA 2.0 Jan 2027; US orders in June 2026 (law-firm primer; details unverified) | Rules change per jurisdiction and quickly: need rules-as-data |
| **WebPKI moving to a new model** | Chrome favours Merkle Tree Certificates (quantum-resistant root store targeted Q3 2027), Let's Encrypt and Apple following | Generic "replace with ML-DSA" advice is wrong for public TLS |
| **Edge ≠ origin** | Top-1M crawl (13 Jun 2026): ~44% of responding sites negotiate X25519MLKEM768, driven by Cloudflare/Google defaults; Cloudflare says only about 10% of origins could benefit from PQ-preferred key agreement | Customers think they are PQ-ready because the CDN is; need edge/origin discrimination |
| **Assurance use cases emerging** | M&A diligence (Freshfields, Feb 2026); cyber insurers exploring readiness indicators (CyberCube: education/scenarios, *not yet pricing*); vendor PQC questionnaires (AUSCERT, SafeLogic); EU CRA reporting live since 11 Sep 2026, SBOM duties from 11 Dec 2027 | Others will ask for proof; today they send questionnaires |
| **AI-assistant interfaces are standard** | Vanta, Secureframe ship MCP servers; MCP ecosystem security is weak (30+ CVEs in 60 days early 2026) | Offer an AI interface, but a hardened, read-only, evidence-citing one |

---

## 3. Capability comparison (as far as I found)

Legend: ● strong · ◐ partial/limited · ○ not found · ? unknown

| Capability | Enterprise CPM (SandboxAQ, IBM, Keyfactor) | OSS CBOM (CBOMkit, Trivy, Sonar) | OSS PQ-TLS probes | GRC platforms | **VeriCrypt v2 (built)** | **VeriCrypt v4 (planned)** |
|---|---|---|---|---|---|---|
| File/cert/key discovery | ● | ◐ | ○ | ○ | ◐ | ● |
| Deep TLS incl. PQ groups, chain, OCSP | ◐ | ◐ | ● (narrow) | ○ | ○ (leaf only) | ● |
| Edge vs origin PQ discrimination | ? | ○ | ○ | ○ | ○ | ● |
| Source/binary/container scanning | ● (IBM, Sonar) | ● | ○ | ○ | ○ | ◐ via import |
| Passive network / host sensors at scale | ● (SandboxAQ) | ◐ | ○ | ○ | ○ | ◐ later |
| Runtime (eBPF) discovery | ? | ○ (academic only) | ○ | ○ | ○ | ◐ later |
| Cloud KMS/HSM inventory | ◐ | ○ | ○ | ◐ | ○ | ◐ read-only |
| CBOM export (CycloneDX) | ◐ (some proprietary) | ● | ○ | ○ | ● (schema-valid 1.7) | ● |
| CBOM import from other tools | ? | ◐ | ○ | ○ | ○ | ● |
| Remediation orchestration | ● (claims) | ○ | ○ | ◐ | ○ | ◐ recipes |
| Dated, ecosystem-aware advice | ? | ○ | ○ | ○ | ○ (v2 advice flawed) | ● |
| Rules/jurisdictions as signed data | ? | ○ | ○ | ◐ | ○ | ● |
| Scoring that separates confidentiality vs authenticity | ? | ○ | ○ | ○ | ○ | ● |
| GRC push / OSCAL / SARIF | ◐ | ◐ (SARIF) | ○ | ● | ○ | ● |
| Offline / air-gapped | ◐ | ● | ● | ○ | ● | ● |
| **Independently verifiable signed evidence (PQ)** | ○ found | ○ | ○ | ○ | ● | ● |
| Selective disclosure / absence proofs | ○ | ○ | ○ | ○ | ◐ (Merkle proofs) | ● |
| Time-travel re-evaluation | ○ | ○ | ○ | ○ | ○ | ● |
| Declared-vs-observed reconciliation | ? | ○ | ○ | ◐ | ○ | ● |
| Verifiable AI-assistant (MCP) answers | ○ | ○ | ○ | ◐ (unverified) | ○ | ● |
| Footprint / price | heavy / six figures (rival claim) | free | free | per-seat/org | single binary | single binary; tiered |

---

## 4. Where incumbents are genuinely ahead (do not fight here first)

1. **Scale and passive discovery** across thousands of hosts (SandboxAQ sensors via EDR platforms).
2. **Mainframe and enterprise-code estates** (IBM).
3. **Certificate automation at fleet scale** (ACME/CLM integrations, 200+ integrations at Venafi).
4. **Services partnerships** (Accenture, Deloitte, EY with SandboxAQ; Entrust with IBM Consulting).
5. **HSM/PKI hardware ecosystems** (Entrust, Thales, Fortanix).

**Strategic response:** *import* their discovery output (CBOMs), do not replicate it; compete on proof, correctness, dated advice, portability and footprint.

---

## 5. White space we can own (ranked by evidence strength)

| ID | White space | Why it is defensible | Evidence strength |
|---|---|---|---|
| W1 | **Verifiable evidence**: PQ-signed, hash-chained, timestamped, offline-verifiable, selectively disclosable | Needs cryptographic engineering plus trust discipline; incumbents sell dashboards, not proofs | Strong (confidence-gap data) |
| W2 | **PQ-TLS truth**: edge vs origin, per-group probing, legacy versions, tied to inventory and CBOM | Major legacy auditors blind to PQ groups (Sept 2026 preprint); small tools lack integration | Strong |
| W3 | **Dated, ecosystem-aware advice** (WebPKI/MTC vs private PKI vs code signing) | Requires continuous curation; competitors' generic advice goes stale | Medium-strong |
| W4 | **Rules-as-data with signed packs** per jurisdiction and standard | Deadlines fragment monthly; packs compound over time | Strong |
| W5 | **Neutral hub**: import everything, export everything | Platform vendors favour their own ecosystems | Medium |
| W6 | **Completeness by corroboration** (declared vs observed, cross-sensor) | Zero-knowledge cannot prove completeness; corroboration can raise confidence | Medium (design is novel; unvalidated) |
| W7 | **Time-travel**: re-evaluate yesterday's inventory under today's rules | Needs event-sourced design from day one; hard to retrofit | Medium (architecture-driven) |
| W8 | **Assurance packs** for M&A, insurance, vendor questionnaires | Others are asking; today answers are PDFs | Medium (demand emerging, not yet priced) |
| W9 | **Verifiable AI interface** | Everyone adds MCP; few add proofs | Medium |
| W10 | **Lightweight, offline, mid-market/regulated price point** | Incumbents are consolidating upmarket | Medium |

---

## 6. Threats and responses

| ID | Threat | Likelihood / impact | Response |
|---|---|---|---|
| T1 | Platform bundling (Palo Alto/CyberArk, Keyfactor, IBM) makes "good enough" discovery + evidence part of existing suites | High / High | Be the neutral proof layer that *ingests* their output; compete where bundles are weak (offline, verification, small estates) |
| T2 | GRC platforms add native crypto tests | Medium / High | Integrate rather than compete: push evidence in, supply deeper tests than they will build |
| T3 | Open source commoditises discovery | High / Medium | Do not build scanners we can import; own evidence + decision layers |
| T4 | Hyperscalers add native crypto inventory (not researched) | Medium / Medium | Watch; stay multi-cloud and neutral |
| T5 | WebPKI/PQ ecosystem shifts again | High / Medium | Ecosystem state as dated pack; quarterly review |
| T6 | Regulatory drafts change or contradict | High / Medium | Pack status (draft/final), `last_verified`, reports cite pack versions |
| T7 | Procurement requires vendor assurances (SOC 2, ISO 27001, insurance) a solo vendor lacks | Medium / High | Plan certification path; dogfood evidence; reproducible builds; public claims registry |
| T8 | Pack curation burden outgrows one maintainer | High / Medium | Multi-root pack authorship (partners/community), automation for monitoring sources |
| T9 | Claims outrun implementation (credibility loss) | Medium / Very high | Claims registry tied to tests; CI blocks untested claims |

---

## 7. Moat analysis ("years ahead" only if several compound)

| Moat | Durability | Copyability | Notes |
|---|---|---|---|
| Evidence integrity + verification ecosystem (small verifier, test vectors, browser verifier) | High | Medium: standards are open, trust takes time | Publishing test vectors invites third-party verifiers: strengthens the standard we author |
| Signed packs (dated knowledge) | High if maintained | Medium | Value compounds with history and multi-root trust |
| Event-sourced observation history (time-travel) | High | Low-Medium | Only possible if designed in from v4.0 |
| Interop breadth | Medium | High | Table stakes, not a moat |
| Formal kernel (Lean + differential testing) | High | Low (rare skill) | Credibility moat; scope strictly to the evaluator kernel |
| Offline single binary | Medium | Medium | Valuable in regulated/gov/OT segments |
| Brand/trust | Slow | Low | Earned by honest claims and public verification |

**Conclusion:** no single feature stays ahead for years. The combination of *verifiable + correct + portable + maintained* does, provided the claims discipline holds.

---

## 8. What we should not do
1. Build our own source/binary scanner (import CBOMs instead).
2. Compete on passive network analysis at enterprise scale.
3. Become a CLM/CA (renewal automation is a partner/integration play).
4. Market zero-knowledge as solving completeness (it cannot).
5. Claim "proved" outside the small Lean kernel.

## 9. Competitive intelligence cadence
Quarterly refresh of §1–§3; monthly watch on: NIST FIPS 206/HQC, IETF LAMPS/PLANTS, Chrome/Let's Encrypt/Apple MTC policy, PCI SSC, EU CRA harmonised standards, G7 CEG/EU/NCSC updates, CBOMkit/Trivy releases, Vanta/Drata API changes.

## 10. Source register
Competitor/market: Axelspire, PKI Insights, Akeyless, FixMyCert (Venafi chain), Mordor Intelligence (PQC services market), QCecuring (competitor-authored comparison, biased), SandboxAQ pages and release notes, CrowdStrike marketplace listing, IBM/SourceForge listing. Demand: Entrust/Ponemon 2026, Axiad 2026, Keyfactor survey via NHIMG, Security Boulevard, Hyperproof, DigiCert survey, PCI/Schellman/HALOCK, Freshfields, CyberCube via Help Net, AUSCERT, Insurance Business. Technical: arXiv 2609.07849, ePrint 2026/834, 2026/2345, Scott Helme top-1M analysis (13 Jun 2026), Cloudflare Radar blog (27 Feb 2026), Project Eleven/Glassnode data via press. Quality rating: vendor/press = indicative; preprints = unreviewed; competitor-authored = biased.
