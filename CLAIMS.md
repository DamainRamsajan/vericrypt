# VeriCrypt claims registry

Rule: **a capability may only be described in the README, CLI help, data room or marketing if it appears in section A or B below.** Section C lists what must NOT be claimed yet. Update this file in the same change that alters a capability. `python3 scripts/check_claims.py` fails if a test named in section A does not exist (CI runs it).

Last reviewed: 2 Nov 2026.

## A. Verified by automated tests (run all: `cargo test -p vericrypt`)

| Claim | Test |
|---|---|
| RSA/EC/Ed25519/ML-DSA keys are classified correctly (RSA-2048 = 112-bit classical and vulnerable; Ed25519 not missed; ML-DSA resistant) | `engine::classify::tests::*` |
| Transition dates follow the FINAL NIST IR 8547 (27 Oct 2026): every quantum-vulnerable public-key algorithm is deprecated after 2030 and disallowed after 2035 | `ir8547_final_deprecates_every_strength_after_2030` |
| Certificates valid beyond 2030 / beyond 2035 with a vulnerable algorithm are flagged | `rules_fire_on_real_properties` |
| CA/Browser Forum validity caps follow the schedule 398 → 200 → 100 → 47 days; renewal burden is counted for in-scope TLS certificates only | `caps_follow_schedule`, `forecast_counts_only_in_scope_leaf_certificates` |
| Asset counts and quantum-vulnerable counts are computed from the scanned files | `counts_are_computed_from_the_inventory` |
| Findings fire on real properties (weak key, SHA-1 signature, plaintext private key, over-long lifetime) | `rules_fire_on_real_properties` |
| Same inputs, as-of time and parameters give an identical Merkle root | `merkle_root_is_deterministic_and_covers_content` |
| Canonical JSON ignores key order | `canonical_json_ignores_key_order` |
| Merkle inclusion proofs verify for every leaf (1–9 leaves); a tampered leaf fails | `engine::merkle::tests::*` |
| Reports can be signed with SLH-DSA-SHAKE-128f (FIPS 205 algorithm); tampering and wrong keys fail | `signature_roundtrip_and_tamper_detection` |
| A signed report verifies against a pinned key; a different pinned key fails | `signed_report_verifies_and_key_pinning_works` |
| A report with no signature FAILS verification unless `--allow-unsigned` is passed | `unsigned_report_fails_unless_explicitly_allowed` |
| Editing a signed report file is detected | `tampering_with_a_signed_report_is_detected` |
| `--fail-on` returns exit code 3 when findings meet the threshold | `fail_on_threshold_returns_exit_code_3` |
| `diff` reports new findings and can gate on them (exit code 3) | `drift_diff_flags_new_findings` |
| The CBOM is internally consistent (CycloneDX 1.7 structure, unique bom-refs, valid asset types, certificate references resolve) | `cbom_is_structurally_consistent` |
| Any file (for example a release binary) can be signed with SLH-DSA and verified offline; a changed file, an edited signature file or a wrong pinned key is detected | `file_signatures_roundtrip_pin_and_detect_changes` |
| The TLS probe reads the certificate a live TLS server presents, records where it came from, and a refused connection is reported without failing the scan | `live_tls_server_certificate_is_inventoried`, `refused_connection_is_recorded_not_fatal` |
| STARTTLS upgrades work for SMTP, IMAP and PostgreSQL (the probe speaks the protocol, upgrades to TLS and inventories the certificate); a server that does not offer STARTTLS gives a clear protocol error | `smtp_starttls_certificate_is_inventoried`, `imap_starttls_certificate_is_inventoried`, `postgres_sslrequest_certificate_is_inventoried`, `server_that_refuses_starttls_is_reported_clearly` |
| A documented risk decision (waiver) accepts a finding without hiding it; the open counts and the CI gate exclude accepted findings; an expired decision re-opens the finding and adds a governance finding | `active_waiver_accepts_the_finding_without_hiding_it`, `expired_waiver_reopens_the_finding_and_is_reported` |
| Waivers are part of the signed report: removing one from a signed report is detected | `waivers_are_part_of_the_signed_report` |
| Licence keys in the standard PASETO v4.public format verify; expired, tampered, other-key, empty-key and old-generator-format tokens are refused; a `binary_hash` claim is not required | `license::tests::*` |

## B. Verified manually (no automated test yet)

| Claim | How it was checked | To automate |
|---|---|---|
| The CBOM, the SBOM and the product CBOM (`scripts/make_self_cbom.py`) validate against the official CycloneDX JSON schema | `python3 scripts/validate_cbom.py out/cbom.json` and `python3 scripts/make_sbom.py > sbom.json` then validate; both reported VALID | add to CI (the workflow does for the SBOM) |
| Full licence flow on a licence-gated binary: signing refused without a licence; free features still work; expired, other-key and forged `license.json` refused; genuine licence activates; signed report verifies | `bash scripts/verify_licence_e2e.sh` (8 checks pass) | add to CI |
| The TypeScript licence signer (`supabase/functions/generate-license/paseto.ts`) produces tokens the binary accepts | Ran the exact file in Node 22 and verified with `vericrypt-license inspect`. **Not run in Deno / Supabase** | deploy and test once |
| Speed and size (release build, Ubuntu 24.04, **one CPU core**): 3,000 synthetic certificates analysed, signed and written as JSON, CBOM and HTML in about 0.5 to 1.2 s; verification 0.15 s; peak memory about 124 MB; report.json about 7 MB; binary 6.4 MB; needs glibc 2.34 or newer (Ubuntu 22.04+, RHEL 9+; **not** RHEL 8 or Ubuntu 20.04) | Timed by hand on generated EC certificates; larger and real-world inventories not measured | add a benchmark to CI |
| The binary has no OpenSSL dependency | `ldd target/release/vericrypt` lists only libc, libm and libgcc_s (release build checked) | CI step |
| Clean build from the source zip, quickstart and tests pass (Ubuntu 24.04, Rust 1.91) | Run by hand | CI matrix |

## C. NOT implemented. Do not claim

- "Mathematical proofs of compliance", "Lean 4 verification", "formal verification" of anything in this repository.
- TEE / hardware attestation; certificate revocation checking; certificate chain building; trust-store validation.
- TLS protocol-version, cipher-suite or key-exchange-group enumeration (including post-quantum groups): the probe reads certificates only and cannot tell whether a server supports hybrid post-quantum key exchange; edge-versus-origin detection.
- STARTTLS for POP3 and FTP (implemented, not tested); MySQL and other database protocols.
- Individually signed waiver files (waivers are committed into the signed report, not signed on their own).
- PKCS#12 / Java keystore reading; SSH; code or binary scanning; cloud KMS/HSM inventory.
- DSSE / in-toto envelopes, hybrid (classical + PQ) signatures, trusted timestamps, selective disclosure, MCP interface, dashboard server, connectors (Vanta, Drata, ITSM), questionnaire answers, trust pages.
- Licence tier features `tee_attestation`, `verichain_sth`, `custom_axioms`, `white_label_verifier` (removed from the generator). Only `signed_reports`, `html_report` and `drift_diff` are gated.
- Enforcement of `max_assets` from the licence (recorded, not checked). Licence revocation (licences can only expire).
- A static (musl) Linux binary built and tested by me. (The binary no longer needs OpenSSL, but it is still linked dynamically to glibc.)
- Build provenance attestation in CI (the workflow step is untested).
- Compliance certification of any kind. VeriCrypt produces an observation-based inventory and findings.
- Performance on estates larger than a few thousand certificates, or on real-world data (only 3,000 synthetic certificates have been timed).
- The legacy v1 `scan` command: removed from the CLI. Its code and old tests remain in the repository (`tests/legacy_v1/`, not built) until the v1 modules are deleted.
- Support for macOS, native Windows or ARM builds (untested).
