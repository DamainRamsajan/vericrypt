//! VeriCrypt v2 evidence engine.
//!
//! Pipeline:  discover -> classify -> evaluate (policy packs) -> forecast -> Merkle-commit -> attest -> report
//! Every number in the output is computed from the scanned inventory; nothing is a constant.
pub mod attest;
pub mod cbom;
pub mod classify;
pub mod diff;
pub mod discover;
pub mod lifecycle;
pub mod merkle;
pub mod model;
pub mod observe;
pub mod policy;
pub mod report_html;
pub mod store;

use chrono::{DateTime, SecondsFormat, Utc};
use clap::Args;
use model::*;
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::path::Path;
use std::time::Duration;

pub type CliResult = Result<(), (i32, String)>;

pub const REPORT_FORMAT: &str = "vericrypt-report/2";
pub const BODY_FORMAT: &str = "vericrypt-body/2";

#[derive(Debug, Args)]
pub struct AnalyzeArgs {
    /// Directory or file to scan (repeatable)
    #[arg(long = "input", short = 'i')]
    pub inputs: Vec<String>,
    /// TLS endpoint to probe, host:port (repeatable)
    #[arg(long = "tls")]
    pub tls: Vec<String>,
    /// Output directory
    #[arg(long, default_value = "./vericrypt-out")]
    pub out: String,
    /// Key file from `vericrypt keygen`; signs the report with SLH-DSA (FIPS 205)
    #[arg(long)]
    pub sign_key: Option<String>,
    /// Issuer substrings that are publicly trusted CAs (comma separated)
    #[arg(long, value_delimiter = ',')]
    pub public_issuers: Vec<String>,
    /// Issuer substrings that are private/internal CAs (comma separated)
    #[arg(long, value_delimiter = ',')]
    pub private_issuers: Vec<String>,
    /// Mosca X: years the protected data must stay confidential
    #[arg(long, default_value_t = 10)]
    pub shelf_life_years: u32,
    /// Mosca Y: years your migration will take
    #[arg(long, default_value_t = 5)]
    pub migration_years: u32,
    /// Mosca Z: assumed years until a cryptographically relevant quantum computer (an assumption, not a prediction)
    #[arg(long, default_value_t = 14)]
    pub crqc_years: u32,
    /// Evaluate as of this RFC 3339 time instead of now (makes reports reproducible)
    #[arg(long)]
    pub as_of: Option<String>,
    /// Exit with code 3 if any finding is at or above this severity (info|low|medium|high|critical)
    #[arg(long)]
    pub fail_on: Option<String>,
    /// Skip the HTML report
    #[arg(long)]
    pub no_html: bool,
    /// Save what the scan saw into this store directory, so it can be re-judged later with `reevaluate`
    #[arg(long)]
    pub store: Option<String>,
}

#[derive(Debug, Args)]
pub struct ReevaluateArgs {
    /// Store directory that holds the saved scan
    #[arg(long)]
    pub store: String,
    /// Scan id to re-judge (printed by `analyze --store`; also the file name in <store>/scans)
    #[arg(long)]
    pub scan: String,
    /// Output directory
    #[arg(long, default_value = "./vericrypt-out")]
    pub out: String,
    /// Key file from `vericrypt keygen`; signs the report with SLH-DSA (FIPS 205)
    #[arg(long)]
    pub sign_key: Option<String>,
    /// Issuer substrings that are publicly trusted CAs (comma separated)
    #[arg(long, value_delimiter = ',')]
    pub public_issuers: Vec<String>,
    /// Issuer substrings that are private/internal CAs (comma separated)
    #[arg(long, value_delimiter = ',')]
    pub private_issuers: Vec<String>,
    /// Mosca X: years the protected data must stay confidential
    #[arg(long, default_value_t = 10)]
    pub shelf_life_years: u32,
    /// Mosca Y: years your migration will take
    #[arg(long, default_value_t = 5)]
    pub migration_years: u32,
    /// Mosca Z: assumed years until a cryptographically relevant quantum computer (an assumption, not a prediction)
    #[arg(long, default_value_t = 14)]
    pub crqc_years: u32,
    /// Judge as of this RFC 3339 time instead of now
    #[arg(long)]
    pub as_of: Option<String>,
    /// Exit with code 3 if any finding is at or above this severity (info|low|medium|high|critical)
    #[arg(long)]
    pub fail_on: Option<String>,
    /// Skip the HTML report
    #[arg(long)]
    pub no_html: bool,
}

#[derive(Debug, Args)]
pub struct KeygenArgs {
    /// Where to write the secret key file (a .pub.json with the public key is written beside it)
    #[arg(long, default_value = "vericrypt-signing-key.json")]
    pub out: String,
}

#[derive(Debug, Args)]
pub struct VerifyArgs {
    /// report.json produced by `analyze`
    pub report: String,
    /// Trusted public key file (.pub.json or key file); pins who signed the report
    #[arg(long)]
    pub pubkey: Option<String>,
    /// Trusted key id (alternative to --pubkey)
    #[arg(long)]
    pub key_id: Option<String>,
    /// Accept an unsigned report (integrity of items only; proves nothing about who produced it)
    #[arg(long)]
    pub allow_unsigned: bool,
}

#[derive(Debug, Args)]
pub struct ProveArgs {
    /// report.json produced by `analyze`
    pub report: String,
    /// Asset or finding id to prove is part of the committed report
    #[arg(long)]
    pub id: String,
}

#[derive(Debug, Args)]
pub struct VerifyProofArgs {
    /// Proof file produced by `prove`
    pub proof: String,
    /// Expected Merkle root (hex); take it from a signed report you trust
    #[arg(long)]
    pub root: String,
}

#[derive(Debug, Args)]
pub struct DiffArgs {
    /// Older report.json (baseline)
    pub old: String,
    /// Newer report.json
    pub new: String,
    /// Exit with code 3 if there is any NEW finding at or above this severity
    #[arg(long)]
    pub fail_on: Option<String>,
}

// ---------------------------------------------------------------------------------------------
// canonical JSON + helpers
// ---------------------------------------------------------------------------------------------

pub fn canonical(v: &Value) -> Vec<u8> {
    fn w(v: &Value, out: &mut Vec<u8>) {
        match v {
            Value::Object(m) => {
                let mut keys: Vec<&String> = m.keys().collect();
                keys.sort();
                out.push(b'{');
                for (i, k) in keys.iter().enumerate() {
                    if i > 0 {
                        out.push(b',');
                    }
                    out.extend(serde_json::to_vec(k).unwrap());
                    out.push(b':');
                    w(&m[*k], out);
                }
                out.push(b'}');
            }
            Value::Array(a) => {
                out.push(b'[');
                for (i, x) in a.iter().enumerate() {
                    if i > 0 {
                        out.push(b',');
                    }
                    w(x, out);
                }
                out.push(b']');
            }
            other => out.extend(serde_json::to_vec(other).unwrap()),
        }
    }
    let mut out = Vec::new();
    w(v, &mut out);
    out
}

fn leaves_from_values(assets: &[Value], findings: &[Value]) -> Vec<(String, String, merkle::Hash)> {
    let mut items: Vec<(String, String, merkle::Hash)> = Vec::new();
    let mut a: Vec<&Value> = assets.iter().collect();
    a.sort_by(|x, y| x["id"].as_str().cmp(&y["id"].as_str()));
    for v in a {
        let id = v["id"].as_str().unwrap_or("").to_string();
        items.push(("asset".into(), id.clone(), merkle::leaf("asset", &id, &canonical(v))));
    }
    let mut f: Vec<&Value> = findings.iter().collect();
    f.sort_by(|x, y| x["id"].as_str().cmp(&y["id"].as_str()));
    for v in f {
        let id = v["id"].as_str().unwrap_or("").to_string();
        items.push(("finding".into(), id.clone(), merkle::leaf("finding", &id, &canonical(v))));
    }
    items
}

pub fn merkle_root_hex(assets: &[Value], findings: &[Value]) -> String {
    let leaves: Vec<merkle::Hash> = leaves_from_values(assets, findings).into_iter().map(|t| t.2).collect();
    hex::encode(merkle::root(&leaves))
}

// ---------------------------------------------------------------------------------------------
// pipeline
// ---------------------------------------------------------------------------------------------

pub struct Inputs {
    pub dirs: Vec<String>,
    pub tls: Vec<String>,
    pub params: Params,
}

fn summarize(assets: &[Asset], findings: &[Finding], p: &Params) -> Summary {
    let mut by: BTreeMap<String, u64> = BTreeMap::new();
    for s in ["critical", "high", "medium", "low", "info"] {
        by.insert(s.to_string(), 0);
    }
    for f in findings {
        *by.entry(f.severity.label().to_string()).or_insert(0) += 1;
    }
    let count = |k: AssetKind| assets.iter().filter(|a| a.kind == k).count() as u64;
    let sum = p.shelf_life_years + p.migration_years;
    let at_risk = sum > p.crqc_years;
    Summary {
        total_assets: assets.len() as u64,
        certificates: count(AssetKind::Certificate),
        private_keys: count(AssetKind::PrivateKey),
        public_keys: count(AssetKind::PublicKey),
        quantum_vulnerable: assets.iter().filter(|a| a.quantum_status == QuantumStatus::Vulnerable).count() as u64,
        quantum_resistant: assets.iter().filter(|a| a.quantum_status == QuantumStatus::Resistant).count() as u64,
        unknown_algorithm: assets.iter().filter(|a| a.quantum_status == QuantumStatus::Unknown).count() as u64,
        findings_by_severity: by,
        mosca: Mosca {
            shelf_life_years: p.shelf_life_years,
            migration_years: p.migration_years,
            crqc_years: p.crqc_years,
            sum_years: sum,
            at_risk,
            note: if at_risk {
                format!("Data that must stay confidential for {} years plus a {}-year migration ({} years) outlasts the assumed {}-year quantum horizon. Migration should already be under way.", p.shelf_life_years, p.migration_years, sum, p.crqc_years)
            } else {
                format!("{} years of data shelf life plus a {}-year migration ({} years) fits inside the assumed {}-year quantum horizon. Re-check as the horizon estimate changes.", p.shelf_life_years, p.migration_years, sum, p.crqc_years)
            },
        },
    }
}

pub fn assumptions(p: &Params) -> Vec<String> {
    vec![
        format!("All date arithmetic uses the as-of time {}.", p.as_of),
        "Quantum status is decided by algorithm family (Shor's algorithm breaks RSA, finite-field DH/DSA and elliptic-curve schemes). The only timing assumption is the Mosca check, which uses the horizon you supplied; the default is an assumption, not a prediction.".to_string(),
        "NIST IR 8547 is an initial public draft (Nov 2024); its dates may change before final publication.".to_string(),
        "CA/Browser Forum lifetime caps (398 -> 200 -> 100 -> 47 days) bind publicly trusted TLS certificates only. Scope is taken from the issuers you declare; undeclared issuers are reported as 'applies only if publicly trusted'.".to_string(),
    ]
}

pub fn limitations() -> Vec<String> {
    vec![
        "Only the files and endpoints you pointed the tool at were examined; see Coverage for everything skipped and why.".to_string(),
        "PKCS#12 and Java keystores are not opened. Encrypted private keys are inventoried but their algorithm is not visible.".to_string(),
        "A TLS probe retrieves the leaf certificate only: not the chain, protocol versions, cipher suites or key-exchange groups.".to_string(),
        "Keys held in HSMs or cloud KMS, keys embedded in source code, and application-level cryptography are out of scope for this version.".to_string(),
        "This is an inventory-based assessment, not a compliance certification.".to_string(),
    ]
}

/// Step 1: look at the world and record what was seen. Reads files and the network; makes no judgement.
pub fn observe(inp: &Inputs, observed_at: DateTime<Utc>) -> observe::ObservationSet {
    let started_at = observed_at.to_rfc3339_opts(SecondsFormat::Secs, true);
    let version = env!("CARGO_PKG_VERSION").to_string();
    let scan_id = observe::scan_id_for(&started_at, &inp.dirs, &inp.tls, &version);
    let mut d = discover::Discovery::new();
    d.observed_at = started_at.clone();
    d.scan_id = scan_id.clone();
    d.tool_version = version;
    for dir in &inp.dirs {
        d.scan_dir(Path::new(dir));
    }
    for t in &inp.tls {
        d.probe_tls(t, Duration::from_secs(8));
    }
    observe::ObservationSet {
        format: observe::OBSERVATIONS_FORMAT.to_string(),
        scan_id,
        started_at,
        targets: observe::ScanTargets { dirs: inp.dirs.clone(), tls: inp.tls.clone() },
        observations: d.observations,
        coverage: d.coverage,
    }
}

/// Step 2: judge the observations. Pure: reads no files and no clock. The same observations,
/// parameters and `now` always give the same body and Merkle root.
pub fn evaluate(set: &observe::ObservationSet, params: &Params, now: DateTime<Utc>) -> Body {
    let mut merged: BTreeMap<String, Asset> = BTreeMap::new();
    for o in &set.observations {
        discover::merge_asset(&mut merged, o.attributes.clone());
    }
    let mut assets: Vec<Asset> = merged.into_values().collect();
    policy::apply_scope(&mut assets, &params.public_issuers, &params.private_issuers);
    let findings = policy::evaluate(&assets, now);
    let forecast = lifecycle::build(&assets, now);
    let summary = summarize(&assets, &findings, params);

    let av: Vec<Value> = assets.iter().map(|a| serde_json::to_value(a).unwrap()).collect();
    let fv: Vec<Value> = findings.iter().map(|f| serde_json::to_value(f).unwrap()).collect();
    let root = merkle_root_hex(&av, &fv);
    Body {
        format: BODY_FORMAT.to_string(),
        tool: ToolInfo { name: "vericrypt".into(), version: env!("CARGO_PKG_VERSION").to_string() },
        generated_at: params.as_of.clone(),
        parameters: params.clone(),
        coverage: set.coverage.clone(),
        summary,
        forecast,
        assumptions: assumptions(params),
        limitations: limitations(),
        assets,
        findings,
        merkle_root: root,
    }
}

pub fn run_pipeline(inp: &Inputs, now: DateTime<Utc>) -> Body {
    let set = observe(inp, now);
    evaluate(&set, &inp.params, now)
}

// ---------------------------------------------------------------------------------------------
// license gate (paid features). Compiled in with `--features license-gate`.
// ---------------------------------------------------------------------------------------------

fn require_license(feature: &str) -> CliResult {
    #[cfg(feature = "license-gate")]
    {
        let ok = crate::license::stored_token().map(|t| crate::license::activate(&t).is_ok()).unwrap_or(false) && crate::license::is_licensed();
        if !ok {
            return Err((4, format!("{} requires a VeriCrypt license. Run `vericrypt activate --key <token>`.", feature)));
        }
    }
    let _ = feature;
    Ok(())
}

fn err<E: std::fmt::Display>(code: i32) -> impl Fn(E) -> (i32, String) {
    move |e| (code, e.to_string())
}

fn write_secret(path: &str, contents: &str) -> std::io::Result<()> {
    std::fs::write(path, contents)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))?;
    }
    Ok(())
}

fn load_keyfile(path: &str) -> Result<attest::KeyFile, (i32, String)> {
    let s = std::fs::read_to_string(path).map_err(err(2))?;
    serde_json::from_str(&s).map_err(err(2))
}

// ---------------------------------------------------------------------------------------------
// commands
// ---------------------------------------------------------------------------------------------

pub fn run_keygen(args: KeygenArgs) -> CliResult {
    let kf = attest::keygen();
    write_secret(&args.out, &serde_json::to_string_pretty(&kf).unwrap()).map_err(err(2))?;
    let pub_path = format!("{}.pub.json", args.out.trim_end_matches(".json"));
    let pubonly = json!({"alg": kf.alg, "key_id": kf.key_id, "public_key": kf.public_key});
    std::fs::write(&pub_path, serde_json::to_string_pretty(&pubonly).unwrap()).map_err(err(2))?;
    println!("Generated {} signing key", kf.alg);
    println!("  key id:       {}", kf.key_id);
    println!("  secret key:   {}  (keep private; mode 0600)", args.out);
    println!("  public key:   {}  (share with auditors)", pub_path);
    Ok(())
}

pub fn run_analyze(args: AnalyzeArgs) -> CliResult {
    if args.inputs.is_empty() && args.tls.is_empty() {
        return Err((2, "nothing to scan: pass --input <dir> and/or --tls host:port".into()));
    }
    let fail_on = match &args.fail_on {
        Some(s) => Some(Severity::parse(s).ok_or((2, format!("unknown severity '{}'", s)))?),
        None => None,
    };
    let now = match &args.as_of {
        Some(s) => DateTime::parse_from_rfc3339(s).map_err(err(2))?.with_timezone(&Utc),
        None => Utc::now(),
    };
    let as_of = now.to_rfc3339_opts(SecondsFormat::Secs, true);
    let inp = Inputs {
        dirs: args.inputs.clone(),
        tls: args.tls.clone(),
        params: Params {
            as_of,
            shelf_life_years: args.shelf_life_years,
            migration_years: args.migration_years,
            crqc_years: args.crqc_years,
            public_issuers: args.public_issuers.clone(),
            private_issuers: args.private_issuers.clone(),
        },
    };
    let set = observe(&inp, Utc::now());
    if let Some(dir) = &args.store {
        let id = store::write_scan(dir, &set).map_err(err(2))?;
        println!("Saved scan {} in {}", id, dir);
    }
    let body = evaluate(&set, &inp.params, now);
    emit(body, &args.out, args.sign_key.as_deref(), args.no_html, fail_on)
}

/// Re-judge a saved scan: same observations, new as-of time and/or parameters.
pub fn run_reevaluate(args: ReevaluateArgs) -> CliResult {
    let fail_on = match &args.fail_on {
        Some(s) => Some(Severity::parse(s).ok_or((2, format!("unknown severity '{}'", s)))?),
        None => None,
    };
    let now = match &args.as_of {
        Some(s) => DateTime::parse_from_rfc3339(s).map_err(err(2))?.with_timezone(&Utc),
        None => Utc::now(),
    };
    let set = store::read_scan(&args.store, &args.scan).map_err(err(2))?;
    let params = Params {
        as_of: now.to_rfc3339_opts(SecondsFormat::Secs, true),
        shelf_life_years: args.shelf_life_years,
        migration_years: args.migration_years,
        crqc_years: args.crqc_years,
        public_issuers: args.public_issuers.clone(),
        private_issuers: args.private_issuers.clone(),
    };
    println!("Re-judging saved scan {} (observed {}) as of {}", set.scan_id, set.started_at, params.as_of);
    let body = evaluate(&set, &params, now);
    emit(body, &args.out, args.sign_key.as_deref(), args.no_html, fail_on)
}

fn emit(body: Body, out: &str, sign_key: Option<&str>, no_html: bool, fail_on: Option<Severity>) -> CliResult {
    let body_v = serde_json::to_value(&body).map_err(err(1))?;
    let body_bytes = canonical(&body_v);

    let attestation = match sign_key {
        Some(path) => {
            require_license("Signed attestation")?;
            let kf = load_keyfile(path)?;
            Some(attest::sign(&body_bytes, &kf).map_err(err(1))?)
        }
        None => None,
    };
    let report = json!({"format": REPORT_FORMAT, "body": body_v, "attestation": attestation});

    std::fs::create_dir_all(&out).map_err(err(2))?;
    let rp = format!("{}/report.json", out);
    std::fs::write(&rp, serde_json::to_string_pretty(&report).unwrap()).map_err(err(2))?;
    let cp = format!("{}/cbom.json", out);
    std::fs::write(&cp, serde_json::to_string_pretty(&cbom::build(&body)).unwrap()).map_err(err(2))?;
    let mut html_path = None;
    if !no_html {
        require_license("HTML report")?;
        let hp = format!("{}/report.html", out);
        std::fs::write(&hp, report_html::render(&body, attestation.as_ref())).map_err(err(2))?;
        html_path = Some(hp);
    }

    print_summary(&body, attestation.as_ref());
    println!("\nWrote {}", rp);
    println!("Wrote {}", cp);
    if let Some(h) = html_path {
        println!("Wrote {}", h);
    }
    if let Some(th) = fail_on {
        let worst = body.findings.iter().map(|f| f.severity).max();
        if let Some(w) = worst {
            if w >= th {
                return Err((3, format!("findings at or above '{}' present (worst: {})", th.label(), w.label())));
            }
        }
    }
    Ok(())
}


pub fn print_summary(b: &Body, att: Option<&attest::Attestation>) {
    let s = &b.summary;
    println!("VeriCrypt {} — evidence report (as of {})", b.tool.version, b.generated_at);
    println!("──────────────────────────────────────────────────────────");
    println!("Assets: {} ({} certificates, {} private keys, {} public keys)", s.total_assets, s.certificates, s.private_keys, s.public_keys);
    println!("Quantum-vulnerable: {}   Post-quantum: {}   Unknown: {}", s.quantum_vulnerable, s.quantum_resistant, s.unknown_algorithm);
    let c = |k: &str| s.findings_by_severity.get(k).copied().unwrap_or(0);
    println!("Findings: {} critical, {} high, {} medium, {} low, {} info", c("critical"), c("high"), c("medium"), c("low"), c("info"));
    println!("Mosca check (X+Y vs Z): {}+{} = {} vs {} -> {}", s.mosca.shelf_life_years, s.mosca.migration_years, s.mosca.sum_years, s.mosca.crqc_years, if s.mosca.at_risk { "AT RISK" } else { "within horizon" });
    println!("TLS certificates in lifetime-rule scope: {}", b.forecast.in_scope_certificates);
    if b.forecast.in_scope_certificates > 0 {
        let row = |i: usize| &b.forecast.schedule[i];
        println!("  renewals/year at 200d cap: {}   at 100d (Mar 2027): {}   at 47d (Mar 2029): {}", row(1).total_renewals_per_year, row(2).total_renewals_per_year, row(3).total_renewals_per_year);
    }
    println!("Coverage: {} files examined, {} items parsed, {} skipped, {} TLS probes ({} failed)", b.coverage.files_examined, b.coverage.items_parsed, b.coverage.skipped.len(), b.coverage.tls_probes.len(), b.coverage.tls_probes.iter().filter(|p| !p.ok).count());
    println!("Merkle root: {}", b.merkle_root);
    match att {
        Some(a) => println!("Attestation: {} signed, key id {}", a.alg, a.key_id),
        None => println!("Attestation: UNSIGNED (use --sign-key to sign)"),
    }
    let top: Vec<&Finding> = b.findings.iter().filter(|f| f.severity >= Severity::High).take(8).collect();
    if !top.is_empty() {
        println!("\nTop findings:");
        for f in top {
            println!("  [{}] {} — {} ({})", f.severity.label().to_uppercase(), f.title, f.asset_id, f.rule_id);
        }
    }
}

pub fn load_report(path: &str) -> Result<Value, (i32, String)> {
    let s = std::fs::read_to_string(path).map_err(err(2))?;
    let v: Value = serde_json::from_str(&s).map_err(err(2))?;
    if v["format"] != REPORT_FORMAT {
        return Err((2, format!("{} is not a {} file", path, REPORT_FORMAT)));
    }
    Ok(v)
}

fn trusted_key_id(args: &VerifyArgs) -> Result<Option<String>, (i32, String)> {
    if let Some(id) = &args.key_id {
        return Ok(Some(id.clone()));
    }
    if let Some(p) = &args.pubkey {
        let v: Value = serde_json::from_str(&std::fs::read_to_string(p).map_err(err(2))?).map_err(err(2))?;
        let pk = v["public_key"].as_str().ok_or((2, "no public_key in key file".to_string()))?;
        return Ok(Some(attest::key_id(&hex::decode(pk).map_err(err(2))?)));
    }
    Ok(None)
}

pub fn run_verify(args: VerifyArgs) -> CliResult {
    let report = load_report(&args.report)?;
    let body = &report["body"];
    let assets = body["assets"].as_array().cloned().unwrap_or_default();
    let findings = body["findings"].as_array().cloned().unwrap_or_default();
    let mut ok = true;

    let recomputed = merkle_root_hex(&assets, &findings);
    let claimed = body["merkle_root"].as_str().unwrap_or("");
    if recomputed == claimed {
        println!("[PASS] Merkle root matches all {} assets and {} findings", assets.len(), findings.len());
    } else {
        println!("[FAIL] Merkle root mismatch: report says {}, recomputed {}", claimed, recomputed);
        ok = false;
    }

    match report["attestation"].as_object() {
        None => {
            if args.allow_unsigned && args.pubkey.is_none() && args.key_id.is_none() {
                println!("[WARN] Report is UNSIGNED (accepted via --allow-unsigned): nothing proves who produced it, and the Merkle root is not authenticated.");
            } else {
                println!("[FAIL] Report carries no signature. Anyone could have produced or edited it (stripping a signature must never look like success).");
                ok = false;
            }
        }
        Some(_) => {
            let att: attest::Attestation = serde_json::from_value(report["attestation"].clone()).map_err(err(2))?;
            match attest::verify(&canonical(body), &att) {
                Ok(()) => println!("[PASS] {} signature is valid (key id {})", att.alg, att.key_id),
                Err(e) => {
                    println!("[FAIL] Signature check failed: {}", e);
                    ok = false;
                }
            }
            match trusted_key_id(&args)? {
                Some(k) if k == att.key_id => println!("[PASS] Signing key matches the trusted key you supplied"),
                Some(k) => {
                    println!("[FAIL] Signed by key {}, not the trusted key {}", att.key_id, k);
                    ok = false;
                }
                None => println!("[WARN] No trusted key supplied: the signature proves integrity against the embedded key only. Pin it with --pubkey or --key-id."),
            }
        }
    }
    if ok {
        println!("\nRESULT: VERIFIED");
        Ok(())
    } else {
        Err((1, "RESULT: NOT VERIFIED".into()))
    }
}

pub fn run_prove(args: ProveArgs) -> CliResult {
    let report = load_report(&args.report)?;
    let body = &report["body"];
    let assets = body["assets"].as_array().cloned().unwrap_or_default();
    let findings = body["findings"].as_array().cloned().unwrap_or_default();
    let items = leaves_from_values(&assets, &findings);
    let idx = items.iter().position(|t| t.1 == args.id).ok_or((2, format!("no asset or finding with id {}", args.id)))?;
    let leaves: Vec<merkle::Hash> = items.iter().map(|t| t.2).collect();
    let steps = merkle::proof(&leaves, idx);
    let src = if items[idx].0 == "asset" { &assets } else { &findings };
    let item = src.iter().find(|v| v["id"] == args.id.as_str()).cloned().unwrap_or(Value::Null);
    let out = json!({
        "format": "vericrypt-proof/1",
        "item_kind": items[idx].0,
        "item_id": args.id,
        "item": item,
        "root": hex::encode(merkle::root(&leaves)),
        "steps": steps,
    });
    println!("{}", serde_json::to_string_pretty(&out).unwrap());
    Ok(())
}

pub fn run_verify_proof(args: VerifyProofArgs) -> CliResult {
    let v: Value = serde_json::from_str(&std::fs::read_to_string(&args.proof).map_err(err(2))?).map_err(err(2))?;
    let kind = v["item_kind"].as_str().unwrap_or("");
    let id = v["item_id"].as_str().unwrap_or("");
    let leaf = merkle::leaf(kind, id, &canonical(&v["item"]));
    let steps: Vec<merkle::Step> = serde_json::from_value(v["steps"].clone()).map_err(err(2))?;
    let root_bytes = hex::decode(&args.root).map_err(err(2))?;
    if root_bytes.len() != 32 {
        return Err((2, "root must be 32 bytes of hex".into()));
    }
    let mut root = [0u8; 32];
    root.copy_from_slice(&root_bytes);
    if merkle::verify_proof(&leaf, &steps, &root) {
        println!("[PASS] {} {} is committed by Merkle root {}", kind, id, args.root);
        Ok(())
    } else {
        Err((1, "[FAIL] proof does not match the supplied root".into()))
    }
}

pub fn run_diff(args: DiffArgs) -> CliResult {
    require_license("Drift diff")?;
    diff::run(args)
}
