//! File-level behaviour of the v2 engine commands: signing, pinning, tamper detection,
//! unsigned handling, CI gating, drift diff and CBOM structure.
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use vericrypt::engine::{self, AnalyzeArgs, DiffArgs, KeygenArgs, SignFileArgs, VerifyArgs, VerifyFileArgs};

const FIXTURES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/pki");
const AS_OF: &str = "2026-11-02T00:00:00Z";

fn analyze(input: &Path, out: &Path, sign_key: Option<&Path>, fail_on: Option<&str>) -> engine::CliResult {
    analyze_with(input, out, sign_key, fail_on, None)
}

fn analyze_with(input: &Path, out: &Path, sign_key: Option<&Path>, fail_on: Option<&str>, waivers: Option<&Path>) -> engine::CliResult {
    engine::run_analyze(AnalyzeArgs {
        waivers: waivers.map(|p| p.display().to_string()),
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

#[test]
fn file_signatures_roundtrip_pin_and_detect_changes() {
    let d = tempfile::tempdir().unwrap();
    let (k1, p1) = keygen(d.path(), "k1");
    let (_, p2) = keygen(d.path(), "k2");
    let bin = d.path().join("vericrypt-release.bin");
    std::fs::write(&bin, vec![7u8; 4096]).unwrap();
    let sig = d.path().join("release.sig.json");
    engine::run_sign_file(SignFileArgs { path: bin.display().to_string(), key: k1.display().to_string(), out: Some(sig.display().to_string()) }).unwrap();
    let check = |pubkey: Option<&Path>| {
        engine::run_verify_file(VerifyFileArgs {
            path: bin.display().to_string(),
            sig: Some(sig.display().to_string()),
            pubkey: pubkey.map(|p| p.display().to_string()),
            key_id: None,
        })
    };
    assert_eq!(code(check(Some(&p1))), 0, "right key");
    assert_eq!(code(check(Some(&p2))), 1, "wrong pinned key");
    // change one byte of the file
    let mut data = std::fs::read(&bin).unwrap();
    data[100] ^= 1;
    std::fs::write(&bin, &data).unwrap();
    assert_eq!(code(check(Some(&p1))), 1, "modified file");
    // restore the file but edit the signature file's claimed hash
    data[100] ^= 1;
    std::fs::write(&bin, &data).unwrap();
    assert_eq!(code(check(Some(&p1))), 0);
    let mut doc: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&sig).unwrap()).unwrap();
    doc["size"] = serde_json::json!(1);
    std::fs::write(&sig, serde_json::to_string(&doc).unwrap()).unwrap();
    assert_eq!(code(check(Some(&p1))), 1, "edited signature file");
}

fn waiver_file(dir: &Path, name: &str, expires: &str) -> PathBuf {
    let p = dir.join(name);
    std::fs::write(
        &p,
        serde_json::json!([{
            "rule_id": "HYG-WEAK-KEY", "asset_id": "*", "owner": "Head of PKI", "reason": "legacy partner endpoint, decommission scheduled",
            "expires": expires, "approved_by": "CISO", "reference": "CHG-1042"
        }])
        .to_string(),
    )
    .unwrap();
    p
}

#[test]
fn active_waiver_accepts_the_finding_without_hiding_it() {
    let d = tempfile::tempdir().unwrap();
    let w = waiver_file(d.path(), "w.json", "2027-12-31T00:00:00Z");
    let out = d.path().join("out");
    // the only critical finding in the fixtures is the RSA-1024 weak key: with a waiver the CI gate passes ...
    assert_eq!(code(analyze_with(Path::new(FIXTURES), &out, None, Some("critical"), Some(&w))), 0);
    // ... but the finding is still in the report, marked as accepted, and the decision is recorded
    let r: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(out.join("report.json")).unwrap()).unwrap();
    let f = r["body"]["findings"].as_array().unwrap().iter().find(|f| f["rule_id"] == "HYG-WEAK-KEY").expect("finding still present");
    assert_eq!(f["accepted_risk"]["owner"], "Head of PKI");
    assert_eq!(r["body"]["summary"]["accepted_risk"], 1);
    assert_eq!(r["body"]["summary"]["findings_by_severity"]["critical"], 0, "open critical count excludes accepted findings");
    assert_eq!(r["body"]["waivers"][0]["status"], "active");
}

#[test]
fn expired_waiver_reopens_the_finding_and_is_reported() {
    let d = tempfile::tempdir().unwrap();
    let w = waiver_file(d.path(), "w.json", "2026-01-01T00:00:00Z"); // already expired at the pinned as-of date
    let out = d.path().join("out");
    assert_eq!(code(analyze_with(Path::new(FIXTURES), &out, None, Some("critical"), Some(&w))), 3, "expired waiver must not pass the gate");
    let r: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(out.join("report.json")).unwrap()).unwrap();
    assert!(r["body"]["findings"].as_array().unwrap().iter().any(|f| f["rule_id"] == "GOV-WAIVER-EXPIRED"));
    assert_eq!(r["body"]["waivers"][0]["status"], "expired");
}

#[test]
fn waivers_are_part_of_the_signed_report() {
    let d = tempfile::tempdir().unwrap();
    let (key, pubkey) = keygen(d.path(), "k1");
    let w = waiver_file(d.path(), "w.json", "2027-12-31T00:00:00Z");
    let out = d.path().join("out");
    analyze_with(Path::new(FIXTURES), &out, Some(&key), None, Some(&w)).unwrap();
    let report = out.join("report.json");
    assert_eq!(code(verify(&report, Some(&pubkey), false)), 0);
    // quietly removing the waiver record from the signed report must be detected
    let mut v: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&report).unwrap()).unwrap();
    v["body"]["waivers"] = serde_json::json!([]);
    let bad = d.path().join("bad.json");
    std::fs::write(&bad, serde_json::to_string(&v).unwrap()).unwrap();
    assert_eq!(code(verify(&bad, Some(&pubkey), false)), 1);
}
