use crate::types::TeeStatus;
pub fn collect_attestation() -> TeeStatus {
    if std::path::Path::new("/dev/tdx_guest").exists() {
        match std::fs::read("/dev/tdx_guest") {
            Ok(q) => { let m = hex::encode(&q[..32.min(q.len())]); return TeeStatus::Attested{quote_bytes:q,measurement:m,tee_type:"Intel TDX".into()}; }
            Err(e) => return TeeStatus::Unavailable{reason:format!("TDX: {}",e)},
        }
    }
    if std::path::Path::new("/dev/sev-guest").exists() {
        match std::fs::read("/dev/sev-guest") {
            Ok(q) => { let m = hex::encode(&q[..32.min(q.len())]); return TeeStatus::Attested{quote_bytes:q,measurement:m,tee_type:"AMD SEV-SNP".into()}; }
            Err(e) => return TeeStatus::Unavailable{reason:format!("SEV: {}",e)},
        }
    }
    TeeStatus::Unavailable{reason:"No TEE detected".into()}
}
