//! Drift detection between two reports: what appeared, what disappeared, what got worse.
use super::model::Severity;
use super::{load_report, CliResult, DiffArgs};
use serde_json::Value;
use std::collections::BTreeMap;

fn index<'a>(v: &'a Value, key: &str) -> BTreeMap<String, &'a Value> {
    v["body"][key]
        .as_array()
        .map(|a| a.iter().filter_map(|x| x["id"].as_str().map(|id| (id.to_string(), x))).collect())
        .unwrap_or_default()
}

fn sev(v: &Value) -> Severity {
    Severity::parse(v["severity"].as_str().unwrap_or("info")).unwrap_or(Severity::Info)
}

pub fn run(args: DiffArgs) -> CliResult {
    let threshold = match &args.fail_on {
        Some(s) => Some(Severity::parse(s).ok_or((2, format!("unknown severity '{}'", s)))?),
        None => None,
    };
    let old = load_report(&args.old)?;
    let new = load_report(&args.new)?;
    let (oa, na) = (index(&old, "assets"), index(&new, "assets"));
    let (of, nf) = (index(&old, "findings"), index(&new, "findings"));

    println!("Baseline: {} (as of {})", args.old, old["body"]["generated_at"].as_str().unwrap_or("?"));
    println!("Current:  {} (as of {})", args.new, new["body"]["generated_at"].as_str().unwrap_or("?"));
    for (name, r) in [("baseline", &old), ("current", &new)] {
        if r["attestation"].is_null() {
            println!("  note: {} report is unsigned; run `vericrypt verify` before trusting it", name);
        }
    }

    let added: Vec<&&Value> = na.iter().filter(|(k, _)| !oa.contains_key(*k)).map(|(_, v)| v).collect();
    let removed: Vec<&&Value> = oa.iter().filter(|(k, _)| !na.contains_key(*k)).map(|(_, v)| v).collect();
    println!("\nAssets: +{} added, -{} removed", added.len(), removed.len());
    let desc = |v: &Value| format!("{} {} [{}]", v["kind"].as_str().unwrap_or("?"), v["algorithm"].as_str().unwrap_or("?"), v["locations"][0].as_str().unwrap_or("?"));
    for a in &added {
        println!("  + {}  {}", a["id"].as_str().unwrap_or(""), desc(a));
    }
    for a in &removed {
        println!("  - {}  {}", a["id"].as_str().unwrap_or(""), desc(a));
    }

    let mut new_f: Vec<&Value> = nf.iter().filter(|(k, _)| !of.contains_key(*k)).map(|(_, v)| *v).collect();
    new_f.sort_by(|a, b| sev(b).cmp(&sev(a)));
    let mut resolved: Vec<&Value> = of.iter().filter(|(k, _)| !nf.contains_key(*k)).map(|(_, v)| *v).collect();
    resolved.sort_by(|a, b| sev(b).cmp(&sev(a)));
    println!("\nFindings: {} new, {} resolved", new_f.len(), resolved.len());
    for f in &new_f {
        println!("  NEW      [{}] {} ({})", sev(f).label().to_uppercase(), f["title"].as_str().unwrap_or(""), f["id"].as_str().unwrap_or(""));
    }
    for f in &resolved {
        println!("  RESOLVED [{}] {} ({})", sev(f).label().to_uppercase(), f["title"].as_str().unwrap_or(""), f["id"].as_str().unwrap_or(""));
    }

    if let Some(t) = threshold {
        if let Some(worst) = new_f.iter().map(|f| sev(f)).max() {
            if worst >= t {
                return Err((3, format!("new findings at or above '{}' (worst: {})", t.label(), worst.label())));
            }
        }
    }
    Ok(())
}
