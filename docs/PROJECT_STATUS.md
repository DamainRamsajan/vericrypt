# VeriCrypt: Project status (2 Nov 2026)

## 1. Where we are
| Area | State | Evidence |
|---|---|---|
| Evidence engine | Built and tested | 36 automated tests pass with a plain `cargo test -p vericrypt` |
| Rules | Follow the **final** NIST IR 8547 (27 Oct 2026) | `CLAIMS.md` section A |
| Signing, verification, pinning, Merkle proofs, drift diff, CBOM 1.7 | Tested | `CLAIMS.md` section A |
| Risk-decision register (waivers) | New, tested | `docs/examples/waivers.example.json` |
| Live TLS probing incl. STARTTLS (SMTP, IMAP, PostgreSQL) | New, tested against local servers | `tests/engine_tls.rs` |
| Portability | Legacy v1 code deleted; OpenSSL and the ASL build step removed; binary links only to the C library | `ldd` check (debug build) |
| Pilot licence flow | Fixed and tested end to end | `scripts/verify_licence_e2e.sh` |
| Release assurance | `sign-file` / `verify-file`, SBOM, product CBOM, release packager | local trial; CI step untested |
| Licence | Proprietary, all rights reserved (repo stays public for now) | `LICENSE` |
| Architecture | arc42 v4 (approved) and v5 with Amendments 1 and 2 | `docs/architecture/` |
| Research | Rounds 1 to 3, founder-view plan | `docs/architecture/RESEARCH_ROUND_*.md`, `FOUNDER_VIEW.md` |
| Public repo | Updates are delivered as one cumulative zip (no push connector on this account) | `UPLOAD_STEPS.txt` |

## 2. Known issues and limits
1. The TLS probe reads certificates only: no protocol-version, cipher-suite or post-quantum-group enumeration. It cannot tell whether a server supports hybrid key exchange.
2. STARTTLS for POP3 and FTP is implemented but untested; no MySQL.
3. No revocation or chain checks, no PKCS#12/JKS, no SSH, no code scanning.
4. The static musl build and a Windows build are **not verified**.
5. `max_assets` is not enforced; licences cannot be revoked remotely.
6. Signing key is a plaintext JSON file (mode 0600); the `slh-dsa` crate is unaudited; `signature` is pinned to a pre-release.
7. The Supabase licence function was tested in Node, not Deno.
8. CI workflow rewritten but not yet run on GitHub; the provenance step is untested.
9. Performance measured only on 3,000 synthetic certificates (about 0.5 to 1.2 s on one core); the release binary needs glibc 2.34 or newer, so RHEL 8 and Ubuntu 20.04 need the static build.
10. Product-name check and legal review of `LICENSE` not done.

## 3. Pilot target
Trinidad and Tobago regulated institutions: the Central Bank's cybersecurity Self-Assessment Return is due every **31 March** (next: 31 March 2027). See `FOUNDER_VIEW.md`.

## 4. Next actions
1. Apply the update in the Codespace; run `cargo test -p vericrypt`, `python3 scripts/check_claims.py`, `bash scripts/verify_licence_e2e.sh`; push; check the first GitHub Actions run.
2. Verify the static build; add Windows.
3. Deep TLS slice (versions, ciphers, hybrid groups) and the Central Bank evidence pack.
