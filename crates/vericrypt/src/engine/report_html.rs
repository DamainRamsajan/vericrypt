//! Self-contained HTML report (no external assets) that prints cleanly to PDF.
use super::attest::Attestation;
use super::model::*;

fn esc(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

const CSS: &str = r#"
:root{--bg:#fff;--fg:#17202a;--mut:#5d6d7e;--line:#d6dbdf;--card:#f6f8fa;--crit:#b03a2e;--high:#d35400;--med:#b7950b;--low:#2874a6;--info:#7f8c8d;--ok:#1e8449}
@media (prefers-color-scheme:dark){:root{--bg:#10151a;--fg:#e8ecef;--mut:#9aa7b2;--line:#2a343d;--card:#18202a}}
*{box-sizing:border-box}body{margin:0;background:var(--bg);color:var(--fg);font:15px/1.55 system-ui,-apple-system,Segoe UI,Roboto,sans-serif}
main{max-width:1040px;margin:0 auto;padding:32px 20px 64px}
h1{margin:0 0 4px;font-size:26px}h2{margin:34px 0 10px;font-size:19px;border-bottom:1px solid var(--line);padding-bottom:6px}
.sub{color:var(--mut);margin:0 0 20px}.cards{display:grid;grid-template-columns:repeat(auto-fit,minmax(150px,1fr));gap:10px}
.card{background:var(--card);border:1px solid var(--line);border-radius:8px;padding:12px}.card b{display:block;font-size:24px}.card span{color:var(--mut);font-size:13px}
table{width:100%;border-collapse:collapse;font-size:13.5px}th,td{text-align:left;padding:7px 8px;border-bottom:1px solid var(--line);vertical-align:top}th{color:var(--mut);font-weight:600}
.wrap{overflow-x:auto}.tag{display:inline-block;padding:1px 8px;border-radius:10px;font-size:12px;font-weight:600;color:#fff}
.critical{background:var(--crit)}.high{background:var(--high)}.medium{background:var(--med)}.low{background:var(--low)}.info{background:var(--info)}
code{font:12.5px ui-monospace,Menlo,Consolas,monospace;word-break:break-all}.note{background:var(--card);border-left:4px solid var(--low);padding:10px 14px;border-radius:4px}
.warn{border-left-color:var(--high)}.good{border-left-color:var(--ok)}ul{margin:6px 0 0 20px;padding:0}
@media print{main{max-width:none;padding:0}h2{break-after:avoid}tr{break-inside:avoid}}
"#;

fn tag(s: Severity) -> String {
    format!("<span class=\"tag {0}\">{0}</span>", s.label())
}

pub fn render(b: &Body, att: Option<&Attestation>) -> String {
    let s = &b.summary;
    let mut h = String::new();
    h.push_str("<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\">");
    h.push_str(&format!("<title>VeriCrypt evidence report — {}</title><style>{}</style></head><body><main>", esc(&b.generated_at), CSS));
    h.push_str(&format!("<h1>Cryptographic &amp; Quantum Exposure Report</h1><p class=\"sub\">As of {} · VeriCrypt {} · Merkle root <code>{}</code></p>", esc(&b.generated_at), esc(&b.tool.version), esc(&b.merkle_root)));

    // cards
    let c = |k: &str| s.findings_by_severity.get(k).copied().unwrap_or(0);
    h.push_str("<div class=\"cards\">");
    for (v, l) in [
        (s.total_assets, "assets inventoried"),
        (s.certificates, "certificates"),
        (s.quantum_vulnerable, "quantum-vulnerable"),
        (s.quantum_resistant, "post-quantum"),
        (c("critical") + c("high"), "critical + high findings"),
        (b.forecast.in_scope_certificates, "TLS certs in lifetime scope"),
    ] {
        h.push_str(&format!("<div class=\"card\"><b>{}</b><span>{}</span></div>", v, l));
    }
    h.push_str("</div>");

    // mosca
    h.push_str(&format!(
        "<h2>Quantum timing check (Mosca)</h2><div class=\"note {}\"><b>{} + {} = {} years vs an assumed {}-year horizon.</b> {}<br><small>The horizon is an input you control (<code>--crqc-years</code>), not a prediction.</small></div>",
        if s.mosca.at_risk { "warn" } else { "good" }, s.mosca.shelf_life_years, s.mosca.migration_years, s.mosca.sum_years, s.mosca.crqc_years, esc(&s.mosca.note)
    ));

    // forecast
    h.push_str("<h2>Certificate renewal burden (CA/Browser Forum schedule)</h2>");
    h.push_str(&format!("<p>{} certificate(s) in scope. {}</p>", b.forecast.in_scope_certificates, esc(&b.forecast.scope_note)));
    h.push_str("<div class=\"wrap\"><table><tr><th>Effective</th><th>Max validity</th><th>Renewals / cert / year</th><th>Total renewals / year</th></tr>");
    for p in &b.forecast.schedule {
        h.push_str(&format!("<tr><td>{}</td><td>{} days</td><td>{}</td><td>{}</td></tr>", esc(&p.effective_from), p.max_validity_days, esc(&p.renewals_per_cert_per_year), esc(&p.total_renewals_per_year)));
    }
    h.push_str("</table></div>");
    if !b.forecast.certificates.is_empty() {
        h.push_str("<div class=\"wrap\"><table><tr><th>Certificate</th><th>Expires</th><th>Days left</th><th>Scope</th><th>Cap at next renewal</th></tr>");
        for c in &b.forecast.certificates {
            h.push_str(&format!("<tr><td>{}<br><code>{}</code></td><td>{}</td><td>{}</td><td>{:?}</td><td>{} days</td></tr>", esc(&c.subject), esc(&c.asset_id), esc(&c.not_after), c.days_to_expiry, c.trust_scope, c.cap_days_at_next_renewal));
        }
        h.push_str("</table></div>");
    }

    // findings
    h.push_str("<h2>Findings (highest severity first)</h2>");
    let actionable: Vec<&Finding> = b.findings.iter().filter(|f| f.severity != Severity::Info).collect();
    if actionable.is_empty() {
        h.push_str("<p>No findings above informational level.</p>");
    } else {
        h.push_str("<div class=\"wrap\"><table><tr><th>Severity</th><th>Finding</th><th>Asset</th><th>Why / fix</th></tr>");
        for f in actionable {
            let refs = if f.references.is_empty() { String::new() } else { format!("<br><small>Refs: {}</small>", esc(&f.references.join("; "))) };
            h.push_str(&format!(
                "<tr><td>{}</td><td><b>{}</b><br><code>{}</code></td><td><code>{}</code></td><td>{}<br><i>Fix:</i> {}{}</td></tr>",
                tag(f.severity), esc(&f.title), esc(&f.rule_id), esc(&f.asset_id), esc(&f.detail), esc(&f.remediation), refs
            ));
        }
        h.push_str("</table></div>");
    }

    // inventory
    h.push_str("<h2>Inventory</h2><div class=\"wrap\"><table><tr><th>Id</th><th>Kind</th><th>Algorithm</th><th>Quantum</th><th>Subject / location</th><th>Expires</th></tr>");
    for a in &b.assets {
        h.push_str(&format!(
            "<tr><td><code>{}</code></td><td>{:?}</td><td>{}</td><td>{:?}</td><td>{}</td><td>{}</td></tr>",
            esc(&a.id), a.kind, esc(&a.algorithm), a.quantum_status,
            esc(&a.subject.clone().unwrap_or_else(|| a.locations.first().cloned().unwrap_or_default())),
            esc(a.not_after.as_deref().unwrap_or("—"))
        ));
    }
    h.push_str("</table></div>");

    // coverage
    h.push_str(&format!("<h2>Coverage</h2><p>{} files examined, {} items parsed, {} skipped, {} TLS probes.</p>", b.coverage.files_examined, b.coverage.items_parsed, b.coverage.skipped.len(), b.coverage.tls_probes.len()));
    if !b.coverage.skipped.is_empty() {
        h.push_str("<div class=\"wrap\"><table><tr><th>Skipped</th><th>Reason</th></tr>");
        for s in &b.coverage.skipped {
            h.push_str(&format!("<tr><td><code>{}</code></td><td>{}</td></tr>", esc(&s.path), esc(&s.reason)));
        }
        h.push_str("</table></div>");
    }
    for p in &b.coverage.tls_probes {
        h.push_str(&format!("<p>TLS probe <code>{}</code>: {}</p>", esc(&p.target), if p.ok { "ok".to_string() } else { esc(p.error.as_deref().unwrap_or("failed")) }));
    }

    // assumptions
    h.push_str("<h2>Assumptions</h2><ul>");
    for a in &b.assumptions {
        h.push_str(&format!("<li>{}</li>", esc(a)));
    }
    h.push_str("</ul><h2>Limitations</h2><ul>");
    for l in &b.limitations {
        h.push_str(&format!("<li>{}</li>", esc(l)));
    }
    h.push_str("</ul>");

    // attestation
    h.push_str("<h2>Attestation</h2>");
    match att {
        Some(a) => h.push_str(&format!(
            "<div class=\"note good\">Signed with {} (key id <code>{}</code>). Verify offline:<br><code>vericrypt verify report.json --key-id {}</code><br><small>The signature covers the canonical report body, including the Merkle root that commits every asset and finding.</small></div>",
            esc(&a.alg), esc(&a.key_id), esc(&a.key_id)
        )),
        None => h.push_str("<div class=\"note warn\">This report is <b>unsigned</b>. Re-run with <code>--sign-key</code> to attach a post-quantum signature.</div>"),
    }
    h.push_str("</main></body></html>");
    h
}
