//! File-level behaviour of the v2 engine commands: signing, pinning, tamper detection,
//! unsigned handling, CI gating, drift diff and CBOM structure.
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use vericrypt::engine::{self, AnalyzeArgs, DiffArgs, KeygenArgs, ReevaluateArgs, VerifyArgs};

const FIXTURES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/pki");
const AS_OF: &str = "2026-11-01T00:00:00Z";

fn analyze(input: &Path, out: &Path, sign_key: Option<&Path>, fail_on: Option<&str>) -> engine::CliResult {
    engine::run_analyze(AnalyzeArgs {
        inputs: vec![input.display().to_string()],
        tls: vec![],
        out: out.display().to_string(),
        sign_key: sign_key.map(|p| p.display().to_string()),
        public_issuers: vec!["Acme Public Test CA".into()],
        private_issuers: vec![],
        shelf_life_years: 10,
        migration_years: 5,
        crqc_years: 14,
        as_of: Some(AS_OF.into()),
        fail_on: fail_on.map(|s| s.to_string()),
        no_html: true,
        store: None,
    })
}

fn keygen(dir: &Path, name: &str) -> (PathBuf, PathBuf) {
    let k = dir.join(format!("{}.json", name));
    engine::run_keygen(KeygenArgs { out: k.display().to_string() }).unwrap();
    (k, dir.join(format!("{}.pub.json", name)))
}

fn verify(report: &Path, pubkey: Option<&Path>, allow_unsigned: bool) -> engine::CliResult {
    engine::run_verify(VerifyArgs {
        report: report.display().to_string(),
        pubkey: pubkey.map(|p| p.display().to_string()),
        key_id: None,
        allow_unsigned,
    })
}

fn code(r: engine::CliResult) -> i32 {
    match r {
        Ok(()) => 0,
        Err((c, _)) => c,
    }
}

#[test]
fn signed_report_verifies_and_key_pinning_works() {
    let d = tempfile::tempdir().unwrap();
    let (key, pubkey) = keygen(d.path(), "k1");
    let (_, other_pub) = keygen(d.path(), "k2");
    let out = d.path().join("out");
    assert_eq!(code(analyze(Path::new(FIXTURES), &out, Some(&key), None)), 0);
    let report = out.join("report.json");
    assert_eq!(code(verify(&report, Some(&pubkey), false)), 0, "right key must verify");
    assert_eq!(code(verify(&report, Some(&other_pub), false)), 1, "wrong pinned key must fail");
}

#[test]
fn unsigned_report_fails_unless_explicitly_allowed() {
    let d = tempfile::tempdir().unwrap();
    let out = d.path().join("out");
    assert_eq!(code(analyze(Path::new(FIXTURES), &out, None, None)), 0);
    let report = out.join("report.json");
    assert_eq!(code(verify(&report, None, false)), 1, "stripping or omitting a signature must not look like success");
    assert_eq!(code(verify(&report, None, true)), 0);
}

#[test]
fn tampering_with_a_signed_report_is_detected() {
    let d = tempfile::tempdir().unwrap();
    let (key, pubkey) = keygen(d.path(), "k1");
    let out = d.path().join("out");
    analyze(Path::new(FIXTURES), &out, Some(&key), None).unwrap();
    let report = out.join("report.json");
    let mut v: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&report).unwrap()).unwrap();
    v["body"]["summary"]["quantum_vulnerable"] = serde_json::json!(0);
    let bad = d.path().join("tampered.json");
    std::fs::write(&bad, serde_json::to_string(&v).unwrap()).unwrap();
    assert_eq!(code(verify(&bad, Some(&pubkey), false)), 1);
}

#[test]
fn fail_on_threshold_returns_exit_code_3() {
    let d = tempfile::tempdir().unwrap();
    // fixtures contain an RSA-1024 certificate => a critical finding
    assert_eq!(code(analyze(Path::new(FIXTURES), &d.path().join("o1"), None, Some("critical"))), 3);
    assert_eq!(code(analyze(Path::new(FIXTURES), &d.path().join("o2"), None, None)), 0);
}

#[test]
fn drift_diff_flags_new_findings() {
    let d = tempfile::tempdir().unwrap();
    let a_in = d.path().join("a_in");
    std::fs::create_dir_all(&a_in).unwrap();
    for e in std::fs::read_dir(FIXTURES).unwrap() {
        let p = e.unwrap().path();
        if p.file_name().unwrap() != "legacy.pem" && p.is_file() {
            std::fs::copy(&p, a_in.join(p.file_name().unwrap())).unwrap();
        }
    }
    let (a_out, b_out) = (d.path().join("a_out"), d.path().join("b_out"));
    analyze(&a_in, &a_out, None, None).unwrap();
    analyze(Path::new(FIXTURES), &b_out, None, None).unwrap();
    let (ra, rb) = (a_out.join("report.json"), b_out.join("report.json"));
    let diff = |o: &Path, n: &Path, f: Option<&str>| {
        engine::run_diff(DiffArgs { old: o.display().to_string(), new: n.display().to_string(), fail_on: f.map(|s| s.to_string()) })
    };
    assert_eq!(code(diff(&ra, &rb, Some("high"))), 3, "adding the legacy certificate introduces new high/critical findings");
    assert_eq!(code(diff(&ra, &ra, Some("high"))), 0, "identical reports have no new findings");
}

#[test]
fn cbom_is_structurally_consistent() {
    let d = tempfile::tempdir().unwrap();
    let out = d.path().join("out");
    analyze(Path::new(FIXTURES), &out, None, None).unwrap();
    let cbom: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(out.join("cbom.json")).unwrap()).unwrap();
    assert_eq!(cbom["bomFormat"], "CycloneDX");
    assert_eq!(cbom["specVersion"], "1.7");
    let comps = cbom["components"].as_array().unwrap();
    assert!(!comps.is_empty());
    let allowed: HashSet<&str> = ["algorithm", "certificate", "related-crypto-material", "protocol"].into_iter().collect();
    let refs: HashSet<String> = comps.iter().map(|c| c["bom-ref"].as_str().unwrap().to_string()).collect();
    assert_eq!(refs.len(), comps.len(), "bom-refs must be unique");
    for c in comps {
        let cp = &c["cryptoProperties"];
        assert!(allowed.contains(cp["assetType"].as_str().unwrap()), "invalid assetType");
        if cp["assetType"] == "certificate" {
            let props = &cp["certificateProperties"];
            for k in ["signatureAlgorithmRef", "subjectPublicKeyRef"] {
                if let Some(r) = props[k].as_str() {
                    assert!(refs.contains(r), "{} points at a missing component", k);
                }
            }
        }
    }
}

// ---- saved scans: analyze --store, then reevaluate ----

fn analyze_into_store(store: &Path, out: &Path) -> String {
    engine::run_analyze(AnalyzeArgs {
        inputs: vec![FIXTURES.to_string()],
        tls: vec![],
        out: out.display().to_string(),
        sign_key: None,
        public_issuers: vec!["Acme Public Test CA".into()],
        private_issuers: vec![],
        shelf_life_years: 10,
        migration_years: 5,
        crqc_years: 14,
        as_of: Some(AS_OF.into()),
        fail_on: None,
        no_html: true,
        store: Some(store.display().to_string()),
    })
    .unwrap();
    let mut ids: Vec<String> = std::fs::read_dir(store.join("scans"))
        .unwrap()
        .map(|e| e.unwrap().path().file_stem().unwrap().to_string_lossy().to_string())
        .collect();
    assert_eq!(ids.len(), 1);
    ids.remove(0)
}

fn reevaluate(store: &Path, scan: &str, out: &Path, as_of: &str) -> engine::CliResult {
    engine::run_reevaluate(ReevaluateArgs {
        store: store.display().to_string(),
        scan: scan.to_string(),
        out: out.display().to_string(),
        sign_key: None,
        public_issuers: vec!["Acme Public Test CA".into()],
        private_issuers: vec![],
        shelf_life_years: 10,
        migration_years: 5,
        crqc_years: 14,
        as_of: Some(as_of.into()),
        fail_on: None,
        no_html: true,
    })
}

fn body_of(out: &Path) -> serde_json::Value {
    let v: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(out.join("report.json")).unwrap()).unwrap();
    v["body"].clone()
}

fn count_rule(body: &serde_json::Value, rule: &str) -> usize {
    body["findings"].as_array().unwrap().iter().filter(|f| f["rule_id"] == rule).count()
}

#[test]
fn saved_scan_re_evaluates_to_the_same_report_root() {
    let d = tempfile::tempdir().unwrap();
    let store = d.path().join("store");
    let first = d.path().join("first");
    let again = d.path().join("again");
    let id = analyze_into_store(&store, &first);
    reevaluate(&store, &id, &again, AS_OF).unwrap();
    assert_eq!(body_of(&first)["merkle_root"], body_of(&again)["merkle_root"]);
}

#[test]
fn re_evaluating_a_saved_scan_later_changes_findings_not_assets() {
    let d = tempfile::tempdir().unwrap();
    let store = d.path().join("store");
    let first = d.path().join("first");
    let later = d.path().join("later");
    let id = analyze_into_store(&store, &first);
    reevaluate(&store, &id, &later, "2040-01-01T00:00:00Z").unwrap();
    let (a, b) = (body_of(&first), body_of(&later));
    assert_eq!(a["assets"].as_array().unwrap().len(), b["assets"].as_array().unwrap().len());
    assert_ne!(a["merkle_root"], b["merkle_root"]);
    assert!(count_rule(&b, "HYG-EXPIRED") > count_rule(&a, "HYG-EXPIRED"));
}

#[test]
fn a_changed_batch_file_is_refused() {
    let d = tempfile::tempdir().unwrap();
    let store = d.path().join("store");
    let id = analyze_into_store(&store, &d.path().join("first"));
    // find the one batch file and change a byte
    let mut batch = None;
    for y in std::fs::read_dir(store.join("obs")).unwrap() {
        for m in std::fs::read_dir(y.unwrap().path()).unwrap() {
            for f in std::fs::read_dir(m.unwrap().path()).unwrap() {
                batch = Some(f.unwrap().path());
            }
        }
    }
    let batch = batch.expect("a batch file was written");
    let mut bytes = std::fs::read(&batch).unwrap();
    bytes[10] = bytes[10].wrapping_add(1);
    std::fs::write(&batch, bytes).unwrap();
    let r = reevaluate(&store, &id, &d.path().join("out"), AS_OF);
    match r {
        Err((2, msg)) => assert!(msg.contains("hash no longer matches"), "unexpected message: {}", msg),
        other => panic!("expected exit 2 for a changed batch, got {:?}", other.map(|_| ())),
    }
}
