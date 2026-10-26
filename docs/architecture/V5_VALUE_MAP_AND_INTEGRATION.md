# VeriCrypt v5 — Value Map: the bigger picture

As of 22 Oct 2026. Builds on `COMPETITIVE_ANALYSIS_v4.md` and `USERS_AND_USE_CASES_v4.md`. Desk research; vendor and preprint sources are rated in §9. **Where I give a number I cite it; where I could not source one I say so.**

---

## 1. The holistic problem, in plain words

An organisation facing cryptographic change has to do six jobs. Today each job is a different tool, team or consultant, and nothing connects them.

| Job | Question | Who does it today |
|---|---|---|
| **Know** | What cryptography do we have? | Discovery tools, spreadsheets |
| **Decide** | What matters first, and what will it cost? | Consultants, workshops |
| **Fix** | How do we change it safely? | PKI/CLM tools, engineers, increasingly AI coding agents |
| **Prove** | Can we show an auditor, customer or insurer it is true? | GRC platforms, PDFs, screenshots |
| **Share** | Can we answer our customers' questionnaires without weeks of work? | Trust-centre and questionnaire tools |
| **Keep up** | Rules and ecosystems change monthly. Are we still right? | Whoever reads the news |

Evidence the pieces do not connect:
- Entrust and IBM Consulting announced a joint migration offer in April 2026, pairing a cryptographic security platform with a migration orchestrator (market report). My reading is that this reflects how hard the pieces are to deliver separately; that is an inference, not their stated reason.
- One organisation reportedly bought a $150K discovery tool and then found it had no asset inventory to scan against (postquantum.com migration-cost guide; anecdotal).
- Only 33% of security architects and PKI engineers say their inventory is continuously updated, against 90% of CISOs and CIOs (Axiad survey, July 2026).

**v5 thesis:** VeriCrypt becomes the **closed loop** across all six jobs. It does not replace the tools people already own. It *connects* them, **verifies** what they claim, and **proves** the result.

```
        ┌────────────── KEEP UP (packs, radar) ──────────────┐
        ▼                                                    │
  KNOW ──► DECIDE ──► FIX ──► PROVE ──► SHARE ──► (new evidence feeds KNOW)
 sensors   scores,    recipes,  signed,    trust page,
 imports   cost,      receipts, verifiable attestations,
           advice     verify    disclosure  questionnaires
```

---

## 2. How v5 absorbs competitor value (integrate, do not rebuild)

| Competitor group | The value they provide | What v5 does with it |
|---|---|---|
| Discovery (SandboxAQ, IBM, Keyfactor, Entrust, CBOMkit, Trivy, Sonar) | Find crypto in code, hosts, networks | **Import** CBOMs and APIs as observations; cross-check against our own sensors; show *which vantage saw what* |
| Certificate lifecycle (CyberArk/Venafi, Keyfactor, DigiCert, AppViewX, Akeyless, cert-manager, ACME) | Issue and renew at fleet scale | **Orchestrate through connectors** (ACME, vendor APIs); we verify the result and issue a receipt. We never become a CA |
| GRC (Vanta, Drata, Secureframe, ServiceNow) | Collect evidence, map controls | **Push** control results and evidence; **read** control context; OSCAL export |
| Trust centres and questionnaires (SafeBase, Conveyor, Whistic, Wolfia) | Share posture, auto-answer questionnaires | **Publish** a verifiable trust page; **answer crypto questions only from verified evidence**, refuse otherwise |
| NHI / machine-identity (CyberArk, Oasis, Aembit, SPIFFE/SPIRE) | Govern machine and AI-agent identities | **Import** identities; add the **cryptographic facet** (algorithm, lifetime, rotation, PQ-readiness) |
| Consultancies and migration orchestrators (IBM Consulting, Accenture, Deloitte) | Roadmaps, gap analysis, cost-to-delay | **Generate their deliverables as data** (roadmap, gap analysis, cost scenarios) so a consultant starts from evidence |
| TLS auditors and PQ probes (sslyze, testssl, sslscan, pqprobe, Cloudflare Radar test) | Handshake and group testing | **Use as test oracles** for our own sensor; embed best methods |
| AI coding agents (Claude Code and others) | Write the code changes | **Verify, don't write**: independent contract tests plus a signed receipt |
| SIEM / ITSM (Splunk, ServiceNow, Jira) | Alerts and tickets | Webhooks and ticket connectors |

---

## 3. Frontier innovations (what could put us years ahead)

Each: what it is, why now, how it fits, effort (S/M/L), verdict.

### F1 Closed-loop remediation with signed receipts, and "AI proposes, VeriCrypt proves"
- **What:** a remediation is a *change plus its proof*: before-scan, change (by recipe, human, or AI agent), independent verification, after-scan, signed **remediation receipt**. Auditors get evidence that the control *operated*, not just that a policy exists.
- **Why now:** A coding-agent study on post-quantum migration (arXiv 2512.12989) found 12 patches that passed local verification but failed external requirements, and optional access to a checker did not raise completion. Local tests are not enough; independent verification is. Separately, a fine-tuned model reportedly reached 92.5% functional correctness on isolated code fragments (news summary of a 2026 preprint), but LLMs fall short on secure, compilable migrations that need dependency and context awareness (arXiv 2506.00790, mobile apps).
- **Design:** a **verification harness** with a *contract* (interoperability against a second implementation, negative tests for tampered inputs and wrong keys, rescans) that any patch must pass; agents interact through read-only MCP plus patch proposals; the harness runs in a sandbox.
- **Effort:** M (harness and receipts), L (broad stack coverage). **Verdict: build; it is the strongest differentiator against remediation claims.**

### F2 PQ path preflight and per-hop coverage
- **What:** before enabling hybrid key exchange, test the *whole path* (load balancers, inspection appliances, split ClientHello) and report coverage **per hop**, with key-exchange coverage and authentication coverage as separate numbers.
- **Why now:** Practitioner guides (Encryption Consulting, postquantum.com) say load balancers and inspection appliances with hardcoded buffer sizes or outdated group lists are a common source of production incidents. The same guides note that upgrading only the public edge leaves downstream hops on classical key exchange, and that a link should count as protected only when telemetry proves a successful hybrid negotiation. Hybrid key exchange addresses harvest-now-decrypt-later confidentiality, not signature forgery, so it should not be labelled "fully quantum-safe TLS". A July 2026 study (arXiv 2607.29005) measured the gap between PQ policy and observed reality at scale.
- **Design:** `preflight` command and a hop graph from the context file plus sensors run from several vantage points. **Effort: M. Verdict: build early; it builds directly on the v4 TLS sensor.**

### F3 Protocol-facet sensors beyond TLS (cheap breadth)
- **What:** JWKS/OIDC discovery (key algorithms, key age, rotation), SAML metadata certificates, DNSSEC algorithms, SSH host keys; plus an **NHI/AI-agent facet** importing identities from NHI platforms and adding cryptographic attributes.
- **Why now:** One 2026 report (via ITWeb) puts machine identities at 109 per human, with roughly 79 of those being AI agents. Figures range from 45:1 to 109:1 across sources, so treat the ratio as direction, not fact. Certificates are only a slice of non-human identity.
- **Design:** each facet is a small plugin over public endpoints. **Effort: S each. Verdict: build; high breadth for low cost.** DNSSEC post-quantum path: not researched, to verify.

### F4 Verifiable trust page and attestation exchange
- **What:** a one-click static page showing posture with a **Verify** button (browser verifier) and scoped signed attestations that a customer's tool can verify and map to its questionnaire. Crypto questions are answered *only* from verified evidence.
- **Why now:** Questionnaire tools (SafeBase, Conveyor, Wolfia and others; comparisons are competitor-authored) draft answers from policies and past responses, not from cryptographic evidence. Vendor PQC questionnaires already exist (AUSCERT, SafeLogic) and are unverified.
- **Design:** reuses v4 selective disclosure and the WASM verifier. **Effort: S–M. Verdict: build; also the main distribution channel, because every shared page teaches a counterparty to verify.**

### F5 Cost and ROI planner, with calibration
- **What:** deterministic low/central/high migration cost scenarios from inventory counts times effort coefficients that the *user* calibrates from their own actuals; sequencing to avoid emergency migration.
- **Why now:** A quantum-budgeting commentary says CFOs ask "what will this cost?" and nobody can answer, because cryptography has never been a separate budget line. The postquantum.com cost guide recommends presenting low, central and high estimates. Consultancies already sell "cost-to-delay scenarios" and ROI assessments as deliverables.
- **Caution:** I found no trustworthy public unit-cost benchmarks. A vendor claims 3–5× for emergency versus planned migration and $840K+ per migration cycle for a 200-application institution; both are unverified vendor figures, so **ship the planner with empty, clearly flagged coefficients**, not invented ones. **Effort: S. Verdict: build.**

### F6 Evidence-grounded AI interface (extends v4 MCP)
- **What:** an assistant can draft answers, plans and board summaries, but every statement cites verified evidence and the tool refuses to answer without it.
- **Why now:** GRC platforms ship MCP servers, and MCP servers have had serious vulnerabilities; an assessment (SoftwareSeni, Sept 2026) counts more than 30 CVEs against MCP servers in 60 days in early 2026. **Effort: S on top of v4. Verdict: build.**

### F7 Horizon radar and pack factory
- **What:** monitors standards, regulators and ecosystem sources; when something changes, it drafts a pack diff for human review and, using time-travel, shows *which of your assets the change affects and what it would cost*.
- **Why now:** deadlines are fragmenting by jurisdiction, and algorithms are mid-flight (FN-DSA's final standard is expected late 2026 or early 2027, per vendor commentary). It also cuts the maintainer's own curation burden. **Effort: M. Verdict: build the internal pipeline first; expose to users later.**

### F8 Crypto policy-as-code
- **What:** organisations write their own crypto standard (for example "TLS 1.3 only, hybrid key exchange required, no RSA below 3072") as a user-authored pack; VeriCrypt checks reality against it and enforces it as a CI ratchet.
- **Design:** packs already allow this (v4 ADR-08); add an authoring guide and linter. **Effort: S. Verdict: build.**

### F9 Migration simulator on the dependency graph
- **What:** "if algorithm X is disallowed on date D, which services must change, in what order, at what cost?" computed from the v4 graph and F5.
- **Effort: M. Verdict: build after F5; deterministic, no machine learning.**

### F10 Optional public anchoring of evidence roots
- **What:** publish a report's root to a transparency log for non-repudiation. VeriChain could be one anchoring backend.
- **Verdict: watch/optional. Must never be required; verification stays offline.**

### F11 Capture–recapture completeness estimate (carried from v4)
- Experimental; excluded from signed claims by default. **Verdict: prototype only after real data exists.**

---

## 4. Productivity and profitability (for the user)

**Principle: measure, don't assume.** v5 includes a **Value Ledger** that records what the tool automated and multiplies by coefficients the user sets, so every claimed saving is traceable and adjustable.

| Lever | What changes | How it is measured |
|---|---|---|
| Evidence labour | Hours spent collecting crypto evidence for audits fall. Internal time and coordination are 30–40% of compliance budgets (Security Boulevard). | Evidence packs generated × user-set hours per pack |
| Questionnaire labour | Crypto questions answered from verified evidence | Answers served × user-set minutes |
| Renewal toil and outages | Expiry alerts and automated renewals. More than a third of surveyed firms had an expired-certificate outage in a year (DigiCert survey), vendor survey | Alerts acted on; incidents avoided (user-recorded) |
| Migration cost | Sequencing and unit-cost planning | F5 scenarios vs actuals |
| Deal velocity | Verifiable trust page, attestations | Reviews closed faster (user-recorded) |
| Insurance and diligence | Ready-made signed packs | Requests answered from packs |

**Profitability for the user** comes from avoided incidents, avoided emergency migration, faster sales cycles and lower audit effort. I cannot put reliable numbers on these today; the ledger lets the *user* do it with their own data.

---

## 5. Revenue surfaces designed into the architecture (hooks only, no pricing)

| Surface | Architecture hook | Customer |
|---|---|---|
| Signed content packs (jurisdictions, mappings, advice, recipes, questionnaire maps) | Pack kinds, multi-root trust (v4) | Everyone |
| Connectors (GRC, CLM, ITSM, cloud KMS, trust centres) | Connector framework, plugin contract | Mid-market, enterprise |
| Attestation exchange and trust page | v4 disclosure + F4 | Vendors and their customers |
| Remediation harness and receipts | F1 | Engineering teams, MSSPs |
| MSSP / consultant workspaces and white-label | Workspaces (v4.2) | Service providers |
| Hub (history, users, API, benchmarks) | Hub-ready formats; designed in v5, built later | Teams |
| Support, certification-readiness and training | Docs, claims registry | Regulated buyers |

Verification and the verifier stay **free and open** so evidence remains independently checkable. That is the trust anchor the paid surfaces depend on.

---

## 6. Low-hanging fruit (do these early, in order)

| # | Item | Why it is cheap | Evidence |
|---|---|---|---|
| 1 | **Per-stack remediation recipes** as a data pack | Mostly content. Cloudflare's support matrix (developer docs) shows hybrid key exchange default in OpenSSL 3.5.0+, Go 1.24+, GnuTLS 3.8.8+, Botan 3.7.0+, Node 24.5.0+ and 22.20.0+, Caddy 2.10.0+, nginx built with OpenSSL 3.5+, rustls 0.23.27+, and in Java 27+. Verify exact versions before publishing recipes. A practitioner write-up says OpenSSH 10.0 defaults to the hybrid mlkem768x25519 key exchange but, as of mid-2026, has no PQ algorithms for user authentication. For many shops "upgrade the library" is the fix | Primary docs plus practitioner post |
| 2 | **Verifiable trust page** (static HTML + browser verifier) | Reuses v4 evidence and WASM verifier | Competitor gap (§3 F4) |
| 3 | **Separate KEX coverage and auth coverage** in every score and dashboard | Display and definition change | F2 sources |
| 4 | **Evidence-grounded answers for crypto questions** | Mapping table plus refusal logic | Questionnaire market |
| 5 | **JWKS/OIDC and DNSSEC facet sensors** | HTTP/DNS fetch and parse | F3 |
| 6 | **Cost planner skeleton** with empty coefficients | Arithmetic over counts | F5 |
| 7 | **Value Ledger** | Counters and user coefficients | §4 |
| 8 | **Policy-as-code authoring guide + linter** | Packs already exist | F8 |
| 9 | **Pack factory watchers** (internal) | Scripts that diff known source pages | F7 |
| 10 | **Cross-vendor oracle tests** (OpenSSL 3.5, Go, rustls, Caddy servers with known group preferences) | Local containers | Quality scenario Q-4 |

---

## 7. What not to build
1. A source/binary scanner, a CA, or a CLM (integrate).
2. Our own code-writing AI (verify others').
3. Zero-knowledge proofs of completeness (impossible); keep them optional and late.
4. Mainframe discovery, eBPF runtime sensor, WASM plugins in v5.0 (watch list).
5. Any marketing claim without a passing test (claims registry).

## 8. Scope discipline for a solo builder
v5 is an *architecture target*, not a build plan. The ordering rule: **v4.0 core first**, then v5 slices ranked by value ÷ effort (§6). Each v5 capability must be deliverable as a data pack, a thin connector or a small plugin; anything that needs a new core subsystem waits.

## 9. Source quality and research gaps
| Topic | Source type |
|---|---|
| Market fragmentation, partnerships | Market report (Mordor) |
| Confidence gap | Axiad vendor survey |
| Remediation by AI agents | arXiv preprints and news summaries; peer review unknown |
| PQ deployment problems | Vendor/advisory blogs, arXiv 2607.29005 |
| Ecosystem support matrix | Cloudflare developer docs (primary), practitioner blog |
| NHI ratios | Vendor reports, wide range |
| Questionnaire market | Competitor-authored comparisons (biased) |
| Cost evidence | Vendor claims only |

**Gaps:** no user interviews; no unit-cost benchmarks; no measurement of how much time evidence packs really save; DNSSEC PQ status unchecked; GRC and trust-centre integration terms unchecked; F1 harness effort untested.
