//! Certificate lifetime forecast against the CA/Browser Forum maximum-validity schedule for
//! publicly trusted TLS certificates: 398 days -> 200 (2026-03-15) -> 100 (2027-03-15) -> 47 (2029-03-15).
use super::model::*;
use chrono::{DateTime, Duration, NaiveDate, TimeZone, Utc};

/// (effective date, label, max validity days)
pub const SCHEDULE: [(i32, u32, u32, &str, u32); 4] = [
    (1970, 1, 1, "before 2026-03-15", 398),
    (2026, 3, 15, "2026-03-15", 200),
    (2027, 3, 15, "2027-03-15", 100),
    (2029, 3, 15, "2029-03-15", 47),
];

fn start_of(y: i32, m: u32, d: u32) -> DateTime<Utc> {
    Utc.from_utc_datetime(&NaiveDate::from_ymd_opt(y, m, d).unwrap().and_hms_opt(0, 0, 0).unwrap())
}

/// Maximum certificate validity (days) allowed for a publicly trusted TLS certificate issued on `date`.
pub fn cap_days_at(date: DateTime<Utc>) -> u32 {
    let mut cap = SCHEDULE[0].4;
    for (y, m, d, _, days) in SCHEDULE.iter() {
        if date >= start_of(*y, *m, *d) {
            cap = *days;
        }
    }
    cap
}

/// A certificate is a candidate for the TLS lifetime rules if it is a leaf that looks like a server certificate.
pub fn is_tls_candidate(a: &Asset) -> bool {
    a.kind == AssetKind::Certificate && !a.is_ca && (a.tls_server || !a.sans.is_empty())
}

pub fn build(assets: &[Asset], now: DateTime<Utc>) -> Forecast {
    let in_scope: Vec<&Asset> = assets
        .iter()
        .filter(|a| is_tls_candidate(a) && a.trust_scope != TrustScope::Private)
        .collect();
    let n = in_scope.len() as f64;
    let schedule = SCHEDULE
        .iter()
        .map(|(_, _, _, label, days)| {
            let per = 365.25 / (*days as f64);
            Phase {
                effective_from: label.to_string(),
                max_validity_days: *days,
                renewals_per_cert_per_year: format!("{:.2}", per),
                total_renewals_per_year: format!("{:.1}", per * n),
            }
        })
        .collect();
    let mut certificates: Vec<CertForecast> = in_scope
        .iter()
        .filter_map(|a| {
            let na = super::discover::parse_ts(a.not_after.as_deref()?)?;
            let renewal_date = std::cmp::max(now, na - Duration::days(30));
            Some(CertForecast {
                asset_id: a.id.clone(),
                subject: a.subject.clone().unwrap_or_default(),
                not_after: a.not_after.clone().unwrap_or_default(),
                days_to_expiry: (na - now).num_days(),
                trust_scope: a.trust_scope,
                cap_days_at_next_renewal: cap_days_at(renewal_date),
            })
        })
        .collect();
    certificates.sort_by(|a, b| a.days_to_expiry.cmp(&b.days_to_expiry).then(a.asset_id.cmp(&b.asset_id)));
    Forecast {
        in_scope_certificates: in_scope.len() as u64,
        scope_note: "Counts leaf certificates that look like TLS server certificates and are not declared private. \
The schedule binds publicly trusted certificates only; certificates from a private CA are outside it. \
Declare issuers with --public-issuers / --private-issuers to tighten the scope. Renewals per year assume each certificate \
is renewed at the maximum allowed lifetime; renewing earlier (common practice) increases the count."
            .to_string(),
        schedule,
        certificates,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn caps_follow_schedule() {
        assert_eq!(cap_days_at(start_of(2026, 3, 14)), 398);
        assert_eq!(cap_days_at(start_of(2026, 3, 15)), 200);
        assert_eq!(cap_days_at(start_of(2027, 3, 15)), 100);
        assert_eq!(cap_days_at(start_of(2029, 3, 15)), 47);
        assert_eq!(cap_days_at(start_of(2031, 1, 1)), 47);
    }
}
