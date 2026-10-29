# VeriCrypt claims registry

Rule: **a capability may only be described in the README, CLI help, data room or marketing if it appears in section A or B below.** Section C lists what must NOT be claimed yet. Update this file in the same pull request that changes a capability.

Last reviewed: 28 Oct 2026 (v2 evidence engine). CI runs `scripts/check_claims.sh`: it fails if a test named in section A does not exist, or if the README or CLI help uses wording from section C.

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
| A scan's raw observations can be saved and re-evaluated later to give the identical Merkle root | `saved_observations_re_evaluate_to_the_identical_root` |
| Re-evaluating the same saved observations at a later as-of date changes the findings (for example, newly expired certificates) but not the set of assets | `re_evaluating_saved_observations_at_a_later_date_changes_findings_not_facts` |
| `analyze --store` saves a scan (immutable batch files plus a scan record); `reevaluate` on the same date and parameters reproduces the same Merkle root | `saved_scan_re_evaluates_to_the_same_report_root` |
| `reevaluate` at a later date gives different findings (more expired certificates) over the same assets | `re_evaluating_a_saved_scan_later_changes_findings_not_assets` |
| A saved batch that has been changed is refused (its hash no longer matches the scan record) | `a_changed_batch_file_is_refused` |
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
| `prove` produces a Merkle inclusion proof for one finding and `verify-proof` accepts it against the report's root and rejects a wrong root | Run by hand on 28 Oct 2026 (Ubuntu 24.04, Rust 1.91.1, debug build) on the `pki` fixtures | add a CLI test |

## C. NOT implemented. Do not claim

- "Mathematical proofs of compliance", "Lean 4 verification", "formal verification" of anything in this repository.
- TEE / hardware attestation; certificate revocation checking; certificate chain building; trust-store validation.
- TLS protocol-version, cipher-suite or key-exchange-group enumeration (including post-quantum groups); edge-versus-origin detection.
- PKCS#12 / Java keystore reading; SSH; code or binary scanning; cloud KMS/HSM inventory.
- DSSE / in-toto envelopes, hybrid (classical + PQ) signatures, trusted timestamps, selective disclosure, MCP interface, dashboard server, connectors (Vanta, Drata, ITSM), questionnaire answers, trust pages.
- Compliance certification of any kind. VeriCrypt produces an observation-based inventory and findings.
- Performance at scale (only tiny fixtures have been run).
- The legacy v1 `scan` command: removed from the command line on 28 Oct 2026 (its compliance-proof stage failed at runtime and several summary counts were wrong). Its library code and old tests remain until deleted; none of it is claimed.
- Classification of DSA, DH and X25519 keys; the list of skipped files; MD5 and certificate-expiry findings. Code exists for these, but there is no test or manual check yet.
- Support for macOS, native Windows or ARM builds (untested).
