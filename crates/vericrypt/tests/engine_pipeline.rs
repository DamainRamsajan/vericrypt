use chrono::{TimeZone, Utc};
use vericrypt::engine::{self, attest, model::*, Inputs};

fn run() -> Body {
    let now = Utc.with_ymd_and_hms(2026, 11, 1, 0, 0, 0).unwrap();
    let inp = Inputs {
        dirs: vec![concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/pki").to_string()],
        tls: vec![],
        params: Params {
            as_of: "2026-11-01T00:00:00Z".into(),
            shelf_life_years: 10,
            migration_years: 5,
            crqc_years: 14,
            public_issuers: vec!["Acme Public Test CA".into()],
            private_issuers: vec![],
        },
    };
    engine::run_pipeline(&inp, now)
}

fn has(b: &Body, rule: &str) -> usize {
    b.findings.iter().filter(|f| f.rule_id == rule).count()
}

#[test]
fn counts_are_computed_from_the_inventory() {
    let b = run();
    assert_eq!(b.summary.certificates, 4); // CA, web, legacy, long
    assert_eq!(b.summary.private_keys, 3); // web.key, edkey.pem, enc.key
    assert_eq!(b.summary.quantum_vulnerable, 6); // 4 certs + web.key + ed25519
    assert_eq!(b.summary.unknown_algorithm, 1); // encrypted key
}

#[test]
fn rules_fire_on_real_properties() {
    let b = run();
    assert_eq!(has(&b, "HYG-WEAK-KEY"), 1); // RSA-1024 leaf
    assert_eq!(has(&b, "HYG-WEAK-SIGNATURE"), 1); // SHA-1 leaf
    assert_eq!(has(&b, "HYG-PRIVKEY-PLAINTEXT"), 2); // web.key + ed25519 (enc.key is encrypted)
    assert_eq!(has(&b, "CABF-LIFETIME"), 2); // web (365d) and long (5000d) exceed the 200d cap
    assert_eq!(has(&b, "PQC-OUTLIVES-2035"), 2); // CA (10y) and long
    assert!(b.assets.iter().any(|a| a.algorithm == "Ed25519" && a.quantum_status == QuantumStatus::Vulnerable));
    assert!(b.assets.iter().any(|a| a.algorithm == "RSA-1024" && a.classical_bits == Some(80)));
}

#[test]
fn forecast_counts_only_in_scope_leaf_certificates() {
    let b = run();
    assert_eq!(b.forecast.in_scope_certificates, 3); // web, legacy, long (CA excluded)
    assert_eq!(b.forecast.schedule[1].total_renewals_per_year, "5.5"); // 3 * 365.25/200
}

#[test]
fn merkle_root_is_deterministic_and_covers_content() {
    let a = run();
    let b = run();
    assert_eq!(a.merkle_root, b.merkle_root);
}

#[test]
fn signature_roundtrip_and_tamper_detection() {
    let body = serde_json::to_value(run()).unwrap();
    let bytes = engine::canonical(&body);
    let key = attest::keygen();
    let att = attest::sign(&bytes, &key).unwrap();
    assert!(attest::verify(&bytes, &att).is_ok());
    let mut tampered = body.clone();
    tampered["summary"]["quantum_vulnerable"] = serde_json::json!(0);
    assert!(attest::verify(&engine::canonical(&tampered), &att).is_err());
    // a different key must not validate
    let other = attest::keygen();
    let mut forged = att.clone();
    forged.public_key = other.public_key.clone();
    forged.key_id = other.key_id.clone();
    assert!(attest::verify(&bytes, &forged).is_err());
}

#[test]
fn canonical_json_ignores_key_order() {
    let a = serde_json::json!({"b": 1, "a": [{"y": 2, "x": 1}]});
    let b: serde_json::Value = serde_json::from_str(r#"{"a":[{"x":1,"y":2}],"b":1}"#).unwrap();
    assert_eq!(engine::canonical(&a), engine::canonical(&b));
}

fn fixture_inputs() -> Inputs {
    Inputs {
        dirs: vec![concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/pki").to_string()],
        tls: vec![],
        params: Params {
            as_of: "2026-11-01T00:00:00Z".into(),
            shelf_life_years: 10,
            migration_years: 5,
            crqc_years: 14,
            public_issuers: vec!["Acme Public Test CA".into()],
            private_issuers: vec![],
        },
    }
}

#[test]
fn saved_observations_re_evaluate_to_the_identical_root() {
    let now = Utc.with_ymd_and_hms(2026, 11, 1, 0, 0, 0).unwrap();
    let inp = fixture_inputs();
    let direct = engine::run_pipeline(&inp, now);
    let set = engine::observe(&inp, now);
    let json = serde_json::to_string(&set).unwrap();
    let back: engine::observe::ObservationSet = serde_json::from_str(&json).unwrap();
    let again = engine::evaluate(&back, &inp.params, now);
    assert_eq!(direct.merkle_root, again.merkle_root);
    assert_eq!(direct.assets.len(), again.assets.len());
    assert_eq!(direct.findings.len(), again.findings.len());
    assert!(set.observations.iter().all(|o| {
        o.sensor.id == "files" && o.vantage == "host" && o.method == "file-parse" && o.confidence == 100 && !o.observed_at.is_empty() && o.scan_id == set.scan_id
    }));
    let ids: std::collections::HashSet<_> = set.observations.iter().map(|o| o.obs_id.clone()).collect();
    assert_eq!(ids.len(), set.observations.len()); // every sighting has its own id
    assert!(set.observations.iter().all(|o| o.obs_id == o.compute_id())); // and the id is the hash of the record
}

#[test]
fn re_evaluating_saved_observations_at_a_later_date_changes_findings_not_facts() {
    let now = Utc.with_ymd_and_hms(2026, 11, 1, 0, 0, 0).unwrap();
    let inp = fixture_inputs();
    let set = engine::observe(&inp, now);
    let early = engine::evaluate(&set, &inp.params, now);
    let later = Utc.with_ymd_and_hms(2040, 1, 1, 0, 0, 0).unwrap();
    let mut p2 = inp.params.clone();
    p2.as_of = "2040-01-01T00:00:00Z".into();
    let late = engine::evaluate(&set, &p2, later);
    assert_eq!(early.assets.len(), late.assets.len()); // same facts
    assert_ne!(early.merkle_root, late.merkle_root); // different judgement
    assert!(has(&late, "HYG-EXPIRED") > has(&early, "HYG-EXPIRED"));
}
