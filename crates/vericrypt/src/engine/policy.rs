//! Deterministic policy engine. Every rule reads the real inventory and emits findings that carry
//! the evidence (asset fields) it was derived from. No rule outcome is a constant.
use super::classify::{self, ir8547};
use super::discover::parse_ts;
use super::lifecycle;
use super::model::*;
use chrono::{DateTime, Duration, TimeZone, Utc};
use std::collections::BTreeMap;

const DORA7: &str = "Commission Delegated Regulation (EU) 2024/1774 (DORA RTS), Art. 7";
const DORA6: &str = "Commission Delegated Regulation (EU) 2024/1774 (DORA RTS), Art. 6";
const IR8547: &str = "NIST IR 8547 (initial public draft, Nov 2024)";
const CABF: &str = "CA/Browser Forum TLS Baseline Requirements (maximum validity schedule)";

fn ev(a: &Asset, extra: &[(&str, String)]) -> BTreeMap<String, String> {
    let mut m = BTreeMap::new();
    m.insert("algorithm".into(), a.algorithm.clone());
    if let Some(b) = a.key_bits {
        m.insert("key_bits".into(), b.to_string());
    }
    if let Some(b) = a.classical_bits {
        m.insert("classical_security_bits".into(), b.to_string());
    }
    if let Some(s) = &a.subject {
        m.insert("subject".into(), s.clone());
    }
    if let Some(s) = &a.not_after {
        m.insert("not_after".into(), s.clone());
    }
    m.insert("sha256".into(), a.sha256.clone());
    m.insert("locations".into(), a.locations.join("; "));
    for (k, v) in extra {
        m.insert(k.to_string(), v.clone());
    }
    m
}

fn mk(rule: &str, pack: &str, sev: Severity, a: &Asset, title: &str, detail: String, fix: &str, refs: &[&str], extra: &[(&str, String)]) -> Finding {
    Finding {
        id: format!("{}:{}", rule, a.id),
        rule_id: rule.to_string(),
        pack: pack.to_string(),
        severity: sev,
        asset_id: a.id.clone(),
        title: title.to_string(),
        detail,
        remediation: fix.to_string(),
        references: refs.iter().map(|s| s.to_string()).collect(),
        evidence: ev(a, extra),
    }
}

pub fn apply_scope(assets: &mut [Asset], public: &[String], private: &[String]) {
    for a in assets.iter_mut() {
        if a.kind != AssetKind::Certificate {
            continue;
        }
        let issuer = a.issuer.clone().unwrap_or_default().to_ascii_lowercase();
        a.trust_scope = if a.self_signed || private.iter().any(|p| issuer.contains(&p.to_ascii_lowercase())) {
            TrustScope::Private
        } else if public.iter().any(|p| issuer.contains(&p.to_ascii_lowercase())) {
            TrustScope::Public
        } else {
            TrustScope::Unknown
        };
    }
}

pub fn evaluate(assets: &[Asset], now: DateTime<Utc>) -> Vec<Finding> {
    let mut out = Vec::new();
    let y2035_end = Utc.with_ymd_and_hms(2035, 12, 31, 23, 59, 59).unwrap();
    for a in assets {
        // ---------- hygiene ----------
        if a.kind == AssetKind::Certificate {
            if let Some(na) = a.not_after.as_deref().and_then(parse_ts) {
                let days = (na - now).num_days();
                if na < now {
                    out.push(mk("HYG-EXPIRED", "hygiene", Severity::High, a, "Certificate has expired",
                        format!("Expired {} day(s) before the scan date.", -days),
                        "Replace the certificate and remove the expired one from service.",
                        &[DORA7], &[("days_to_expiry", days.to_string())]));
                } else if days <= 7 {
                    out.push(mk("HYG-EXPIRING-7D", "hygiene", Severity::High, a, "Certificate expires within 7 days",
                        format!("Expires in {} day(s).", days),
                        "Renew immediately and verify the new certificate is deployed.",
                        &[DORA7], &[("days_to_expiry", days.to_string())]));
                } else if days <= 30 {
                    out.push(mk("HYG-EXPIRING-30D", "hygiene", Severity::Medium, a, "Certificate expires within 30 days",
                        format!("Expires in {} days.", days),
                        "Schedule renewal; automate issuance where the issuer allows it.",
                        &[DORA7], &[("days_to_expiry", days.to_string())]));
                }
            }
            if let Some(sig_oid) = a.signature_oid.as_deref() {
                let (name, weak) = classify::signature_algorithm(sig_oid);
                if weak && !(a.self_signed && a.is_ca) {
                    out.push(mk("HYG-WEAK-SIGNATURE", "hygiene", Severity::High, a, "Certificate is signed with a broken hash",
                        format!("Signature algorithm {} uses MD5 or SHA-1.", name),
                        "Reissue the certificate with a SHA-256 or stronger signature.",
                        &[], &[("signature_algorithm", name)]));
                }
            }
        }
        if let Some(bits) = a.classical_bits {
            if bits < 112 && a.quantum_status == QuantumStatus::Vulnerable {
                out.push(mk("HYG-WEAK-KEY", "hygiene", Severity::Critical, a, "Key is below 112-bit classical security",
                    format!("{} provides about {} bits of classical security, below the 112-bit minimum used by current NIST guidance.", a.algorithm, bits),
                    "Replace with at least RSA-3072 / P-256 now, and plan the post-quantum migration.",
                    &[], &[]));
            }
        }
        if a.kind == AssetKind::PrivateKey && !a.encrypted {
            out.push(mk("HYG-PRIVKEY-PLAINTEXT", "hygiene", Severity::High, a, "Unencrypted private key found on disk",
                "A private key is stored without passphrase protection in the scanned location.".to_string(),
                "Move the key into an HSM or secrets manager, or encrypt it; restrict file permissions; rotate if the file was ever exposed.",
                &[DORA7], &[]));
        }

        // ---------- CA/B Forum lifetime ----------
        if lifecycle::is_tls_candidate(a) {
            if let (Some(nb), Some(vd)) = (a.not_before.as_deref().and_then(parse_ts), a.validity_days) {
                let cap = lifecycle::cap_days_at(nb) as i64;
                if vd > cap {
                    let (sev, note) = match a.trust_scope {
                        TrustScope::Public => (Severity::High, "The issuer is declared publicly trusted, so this lifetime should not have been issued."),
                        TrustScope::Unknown => (Severity::Info, "Applies only if the issuer is publicly trusted; declare it with --public-issuers or --private-issuers."),
                        TrustScope::Private => continue_private(),
                    };
                    if a.trust_scope != TrustScope::Private {
                        out.push(mk("CABF-LIFETIME", "cabf", sev, a, "Validity exceeds the maximum for its issue date",
                            format!("Valid for {} days; the cap when it was issued was {} days. {}", vd, cap, note),
                            "Reissue within the current maximum validity and automate renewal.",
                            &[CABF], &[("validity_days", vd.to_string()), ("cap_days_at_issue", cap.to_string())]));
                    }
                }
            }
        }

        // ---------- post-quantum ----------
        match a.quantum_status {
            QuantumStatus::Vulnerable => {
                let (dep, dis) = ir8547(a.classical_bits);
                let (sev, tl) = if dep.is_some() {
                    (Severity::High, format!("deprecated after {} and disallowed after {}", dep.unwrap(), dis))
                } else {
                    (Severity::Medium, format!("disallowed after {}", dis))
                };
                let agree = a.family == Family::Dh;
                out.push(mk("PQC-IR8547", "pqc", sev, a, "Quantum-vulnerable public-key algorithm",
                    format!("{} can be broken by a large quantum computer. Under the NIST IR 8547 draft timeline it is {}.", a.algorithm, tl),
                    classify::replacement(a.family, agree),
                    &[IR8547, DORA6], &[("ir8547_disallow_after", dis.to_string())]));
                if a.kind == AssetKind::Certificate {
                    if let Some(na) = a.not_after.as_deref().and_then(parse_ts) {
                        if na > y2035_end {
                            out.push(mk("PQC-OUTLIVES-2035", "pqc", Severity::High, a, "Certificate outlives the 2035 disallow date",
                                "The certificate stays valid after the date NIST proposes to disallow its algorithm.".to_string(),
                                "Shorten the lifetime or reissue with a post-quantum or hybrid algorithm before 2035.",
                                &[IR8547], &[]));
                        }
                    }
                }
            }
            QuantumStatus::Unknown => {
                if !a.encrypted {
                    out.push(mk("PQC-UNKNOWN-ALGORITHM", "pqc", Severity::Low, a, "Algorithm not recognised",
                        format!("The algorithm {} is not in the registry, so its quantum status is unknown.", a.algorithm),
                        "Review manually and report the OID so it can be added to the registry.",
                        &[], &[("oid", a.oid.clone())]));
                }
            }
            QuantumStatus::Resistant => {
                out.push(mk("PQC-READY", "pqc", Severity::Info, a, "Post-quantum algorithm in use",
                    format!("{} is a NIST post-quantum algorithm.", a.algorithm),
                    "No action.", &[], &[]));
            }
        }
    }
    out.sort_by(|x, y| y.severity.cmp(&x.severity).then(x.rule_id.cmp(&y.rule_id)).then(x.asset_id.cmp(&y.asset_id)));
    out
}

// Helper to keep the match above exhaustive without emitting anything for private-CA certificates.
fn continue_private() -> (Severity, &'static str) {
    (Severity::Info, "")
}
