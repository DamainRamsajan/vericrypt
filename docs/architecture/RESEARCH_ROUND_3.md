# VeriCrypt: Research round 3 (2 Nov 2026)

Method as before: web search and page reads; source quality stated; ⚠ marks claims resting on secondary sources only. This round closed most of the open questions from round 2.

## 1. Questions from round 2, answered
| Question | Answer | Source quality |
|---|---|---|
| Is US executive order 14412 real, and what does it say? | **Yes.** Signed 22 June 2026 (Federal Register 25 June). Federal high-value and high-impact systems: post-quantum key establishment by **31 Dec 2030**, signatures by **31 Dec 2031**. It orders public guidance on the **minimum elements of a cryptographic bill of materials** (CISA with NIST, within 270 days, about **19 March 2027**), "to enable the automated assessment of the cryptographic assets" in a hardware or software element. A proposed FAR rule for contractors is due about December 2026 | Order text as reproduced by Embedded Computing Design; confirmed by Cloudflare, Keyfactor, postquantum.com and OpsMx write-ups. The White House page itself was not read |
| OMB M-26-15? | Agency PQC migration plans were due **22 Oct 2026**; roadmap runs 2026 to 2035; agencies are expected to use automated tools for inventory; TLS 1.3 adoption target **2 Jan 2030** | postquantum.com, Boston Identity glossary ⚠ (memo itself not read) |
| What exactly changed in NIST IR 8547 final? | Deprecates RSA and ECDSA at 112 bits and above, EdDSA, and DH/MQV/RSA key establishment: deprecated **after 2030**, disallowed **after 2035**. P-256 and Ed25519 are covered. "Deprecated" means usable, but the owner must examine the risk and decide, so a **recorded risk decision** is needed. Hybrid key establishment stays acceptable beyond 2035 if it includes an approved post-quantum component. Experts point to RFC 4998 evidence records for long-term signed evidence | postquantum.com analysis citing the NIST page (primary page confirmed final in round 2) |
| Are the TLS hybrid groups final? | **Yes, correction to round 2.** RFC 10024 (Aug 2026) finalised the hybrid ML-KEM groups (X25519MLKEM768 0x11EC, SecP256r1MLKEM768 0x11EB, SecP384r1MLKEM1024 0x11ED). Pure ML-KEM groups (0x0200-0x0202) are still an Internet-Draft | postquantum.com; IETF datatracker shows the hybrid draft as already published |
| Can servers other than web servers do hybrid key exchange? | PostgreSQL 18 can configure hybrid groups (`ssl_groups`); Postfix 3.10 with OpenSSL 3.5 can offer them and a later release is expected to enable them by default; MySQL supports post-quantum TLS with OpenSSL 3.5 or later | PostgreSQL mailing list, Oracle blog, Arch wiki / maintainers: primary-ish |
| Do sources agree on language defaults? | **No.** Cloudflare's matrix says Go 1.24+ enables the hybrid by default; one vendor blog says it is off by default. Recipes must cite the primary release notes, never a summary | Conflict found; unresolved here |
| When is the Trinidad and Tobago return due? | **31 March each year**, with an action plan; first submission was 31 March 2024. Next: **31 March 2027**. There is also a mandatory cyber-incident reporting circular (July 2025) | **Primary:** Central Bank of Trinidad and Tobago circular, July 2025 |

## 2. What this changes
- **A concrete pilot window:** regulated institutions in Trinidad and Tobago must file by 31 March 2027. An evidence pack mapped to the guideline's 20 requirements can be offered in time for that return.
- **CBOM guidance is coming (about March 2027):** our CBOM output is already machine-readable with algorithm parameters; we will compare it with the official minimum elements when published. We now also publish a CBOM **of the VeriCrypt binary itself**.
- **Risk-decision register matters:** from 2031 every classical-only use needs a documented decision. Waivers (owner, reason, expiry, shown in the signed report) are now in the product.
- **Hybrid key exchange is a final standard:** the deep TLS sensor can rely on those codepoints.
- **STARTTLS and PostgreSQL are in scope:** the competitor probe covers them; ours now does too (SMTP, IMAP, PostgreSQL tested; POP3 and FTP implemented, untested).

## 3. Built this round
Removed the legacy v1 code (about 1,300 lines), OpenSSL and the ASL build step; moved the TLS probe to rustls; added STARTTLS probing; added waivers; added the product CBOM and wired it into the release; upgraded the claims checker. The binary now depends only on the C library.

## 4. Next research queue
1. Read the primary White House, OMB and Federal Register texts; read NIST IR 8547 final in full for the exact algorithm tables.
2. CBTT guideline: map its 20 requirements to evidence we can produce (needs the guideline text, not a summary).
3. Go, Java, Node, .NET release notes for hybrid defaults (primary sources) to build trustworthy remediation recipes.
4. IBM Proxy architecture; what a PQ gateway needs from an inventory tool.
5. Musl static build with `ring`; Windows build and code signing.
6. Mid-size Caribbean and Latin American regulators with comparable return dates.
7. Pilot paperwork templates (agreement, NDA, data-processing terms) and local legal review.
