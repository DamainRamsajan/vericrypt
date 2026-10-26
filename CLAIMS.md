# VeriCrypt claims registry

Rule: **a capability may only be described in the README, CLI help, data room or marketing if it appears in section A or B below.** Section C lists what must NOT be claimed yet. Update this file in the same pull request that changes a capability.

Last reviewed: 23 Oct 2026 (v2 evidence engine).

## A. Verified by automated tests (run: `cargo test -p vericrypt --lib engine::` and `cargo test -p vericrypt --test engine_pipeline --test engine_cli`)

| Claim | Test |
|---|---|
| RSA/EC/Ed25519/ML-DSA keys are classified correctly (RSA-2048 = 112-bit classical, vulnerable; Ed25519 not missed; ML-DSA resistant) | `engine::classify::tests::*` |
| NIST IR 8547 (draft) transition years are applied (≤112-bit deprecated after 2030; all disallowed after 2035) | `engine::classify::tests::ir8547_dates` |
| CA/Browser Forum validity caps follow the schedule 398 → 200 → 100 → 47 days | `engine::lifecycle::tests::caps_follow_schedule`, `forecast_counts_only_in_scope_leaf_certificates` |
| Asset counts and quantum-vulnerable counts are computed from the scanned files | `counts_are_computed_from_the_inventory` |
| Findings fire on real properties (weak key, SHA-1 signature, plaintext private key, over-long lifetime, certificate outliving 2035) | `rules_fire_on_real_properties` |
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

## B. Verified manually (no automated test yet)

| Claim | How it was checked | To automate |
|---|---|---|
| The CBOM validates against the official CycloneDX 1.7 JSON schema | `python3 scripts/validate_cbom.py out/cbom.json` → `VALID` (sample in `docs/sample/`) | add the script to CI |
| A TLS endpoint's leaf certificate can be probed | Probed a local `openssl s_server`; a refused port was recorded as a failed probe | add a CI test with a local server |
| With `--features license-gate`, signing, HTML report and `diff` refuse without a license, and a forged `license.json` does not unlock them | Run by hand on 23 Oct 2026 | add feature-gated test |
| Clean build from the source zip, quickstart, tests pass (Ubuntu 24.04, Rust 1.91) | Run by hand | CI matrix |

## C. NOT implemented. Do not claim

- "Mathematical proofs of compliance", "Lean 4 verification", "formal verification" of anything in this repository.
- TEE / hardware attestation; certificate revocation checking; certificate chain building; trust-store validation.
- TLS protocol-version, cipher-suite or key-exchange-group enumeration (including post-quantum groups); edge-versus-origin detection.
- PKCS#12 / Java keystore reading; SSH; code or binary scanning; cloud KMS/HSM inventory.
- DSSE / in-toto envelopes, hybrid (classical + PQ) signatures, trusted timestamps, selective disclosure, MCP interface, dashboard server, connectors (Vanta, Drata, ITSM), questionnaire answers, trust pages.
- Compliance certification of any kind. VeriCrypt produces an observation-based inventory and findings.
- Performance at scale (only tiny fixtures have been run).
- The legacy v1 `scan` command: its compliance-proof stage failed at runtime and several of its summary counts were wrong. It prints a deprecation warning and should be removed.
- Support for macOS, native Windows or ARM builds (untested).
