# VeriCrypt: Research round 2 (1 Nov 2026)

Method: web search of news, standards pages, preprints and vendor pages dated up to 30 Oct 2026. **Source quality is stated for each item.** Items marked ⚠ rest on secondary sources only and must be checked against primary texts.

## 1. NIST IR 8547 is final (27 Oct 2026): biggest correction this round
- **What:** NIST's own page lists the draft as withdrawn on 27 Oct 2026 and the final as published (primary source). Secondary write-ups (Quantum Computing Report, postquantum.com, SITG Consulting) agree that the **final deprecates all RSA and elliptic-curve public-key algorithms after 2030 and disallows them after 2035**, at every strength. The 2024 draft only deprecated the 112-bit strengths after 2030.
- **Impact on us:** our v2 rules used the draft. **Fixed** in this update: every quantum-vulnerable algorithm is now "deprecated after 2030, disallowed after 2035"; a new rule flags certificates that outlive 2030; tests and the assumptions text updated.
- **Lesson (ADR-40):** a regulation changed within days of our code being written. Dates belong in data packs, not code.
- ⚠ Secondary sources also tie the final to US executive order 14412 (June 2026) and OMB memo M-26-15 (agency PQC plans due 22 Oct 2026), and report a German federal migration guideline (12 Oct 2026, end-2030 for high-risk systems). Primary texts not read.

## 2. Competitor moves in the last three weeks
| Item | What the sources say | Quality | What it means for us |
|---|---|---|---|
| **IBM Quantum Safe suite (27 Oct)** | Cryptography Manager (discovery, posture, migration tracking), Explorer (code scanning and CBOM), Proxy (sits between legacy and post-quantum clients and servers). IBM cites its own estate: about 47 million lines of code, 5,815 repositories, more than 3,900 cryptographic issues found. A Futurum analyst note says the cryptographic inventory will never be complete | IBM announcement plus analyst commentary | A full suite from the largest competitor. Our answer is evidence quality and honest coverage, plus treating gateways as remediation targets (ADR-42) |
| **DuoKey free probing (about 24 Oct)** | Free live probing of post-quantum key exchange; covers STARTTLS and databases; CycloneDX 1.7 CBOM; migration planning. DuoKey's own comparison says IBM is strong in air-gapped environments, Keyfactor in breadth, SandboxAQ in TLS traffic analysis | Press report, vendor claims | A free tool overlaps our TLS and CBOM. Add STARTTLS and database protocols (ADR-41). "Offline" alone is not a differentiator against IBM |
| **IonQ acquired Elisity (about 12 Oct)** | Quantum-hardware company moving into security posture | postquantum.com analysis | New entrants buy their way in. Watch for more |
| **Encryption Consulting CBOM Secure** | Version 1.1 with 20+ sensors; Frost & Sullivan report (27 Oct) | Press release | Paid analyst validation exists for a CBOM-first product |
| **SandboxAQ** | Five-year agreement with the US Department of War CIO for AQtive Guard (Dec 2025) | Press | Government channel is taken |
| **Palo Alto Networks** | Quantum-safe app building an inventory from firewall logs; closed its CyberArk acquisition in Feb 2026 | Vendor | Network-side inventory is bundled into a platform |

## 3. TLS key-exchange group codepoints (for the deep TLS sensor and the algorithm pack)
From IETF drafts (hybrid draft `draft-ietf-tls-ecdhe-mlkem-05`, May 2026; pure ML-KEM draft `draft-ietf-tls-mlkem-11`, Sept 2026). **Drafts, not final RFCs; codepoints may still change.**
| Group | Codepoint |
|---|---|
| X25519MLKEM768 | 0x11EC |
| SecP256r1MLKEM768 | 0x11EB |
| SecP384r1MLKEM1024 | 0x11ED |
| MLKEM512 / MLKEM768 / MLKEM1024 (pure) | 0x0200 / 0x0201 / 0x0202 |
| X25519Kyber768Draft00, SecP256r1Kyber768Draft00 (obsolete pre-standard) | 0x6399, 0x639A |

## 4. What pilot banks will ask of a vendor binary
Advisory and vendor write-ups on DORA third-party risk and software supply chains consistently point to: a software bill of materials per release, signed artifacts and build provenance, vulnerability-notification terms, audit and exit rights in the contract, and security evidence for the vendor itself. (Quality: advisory and vendor blogs, indicative. Not checked against DORA article text beyond the RTS Art. 6-7 work in earlier rounds.)
- **Done this round:** `sign-file` / `verify-file` (SLH-DSA), `scripts/make_sbom.py` (CycloneDX 1.6, validated against the official schema; 288 components), `scripts/make_release.sh`, optional CI provenance step (untested).
- **Still to do:** pilot agreement and security-contact documents; vulnerability-disclosure policy; later SOC 2 or ISO 27001.

## 5. Offline licensing practice
Licensing vendors (LicenseSpring, Nalpeiron, others) describe the common patterns for air-gapped customers: pre-issued signed licence files verified offline; offline activation tied to a machine code (hardware binding), which is standard but brittle when hardware changes; signed revocation lists with a freshness limit; version ceilings for maintenance periods; grace periods. (Quality: vendor documentation.)
- **Our choice (ADR-38):** signed token bound to organisation, expiry and features; no hardware binding, no activation server in pilots. **Deferred:** signed revocation list, version ceiling, enforcement of `max_assets`.

## 6. Bugs found in our own code by this round's testing
| Bug | Effect | Status |
|---|---|---|
| Licence public key never embedded (build script looked in the wrong folder) | Every activation failed with "Invalid public key" | Fixed, tested |
| Licence generator produced a non-standard token and missed required claims | No generated key could ever verify | Fixed, tested in Node, not in Supabase's runtime |
| Binary-hash claim could never match | Activation refused even for correct tokens | Check removed, tested |
| Legacy tests did not compile | `cargo test` impossible | Parked in `tests/legacy_v1/` |
| Rules assumed the IR 8547 draft | Wrong deprecation dates | Fixed, tested |

## 7. Next research queue (to keep digging each round)
1. Read primary texts: EO 14412, OMB M-26-15, the German guideline, NIST IR 8547 final (what exactly it says about hybrid and about "deprecated" status).
2. STARTTLS and database protocol support for post-quantum key exchange in common servers (Postfix, Exim, Dovecot, PostgreSQL, MySQL).
3. IBM Proxy architecture and what it implies for remediation recipes.
4. Musl static builds with post-quantum C dependencies; Windows and macOS packaging and code signing for bank environments.
5. Customer and regulator expectations in Trinidad and Tobago and the wider Caribbean (Central Bank guideline due dates).
6. CNSA 2.0 vendor-attestation expectations (January 2027 acquisition gate).
7. in-toto predicate for CycloneDX BOMs and whether a crypto-posture predicate exists.
8. Hybrid TLS server behaviours that break clients (large ClientHello, middleboxes), with test servers for the oracle tests.
