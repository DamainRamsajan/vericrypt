use crate::errors::VeriCryptError;
use pasetors::claims::ClaimsValidationRules;
use pasetors::keys::AsymmetricPublicKey;
use pasetors::token::UntrustedToken;
use pasetors::version4::V4;
use std::fs;
use std::path::PathBuf;

mod embedded {
    include!(concat!(env!("OUT_DIR"), "/embedded_key.rs"));
}

static mut LICENSE_ACTIVE: bool = false;
static mut LICENSE_EXPIRY: Option<chrono::DateTime<chrono::Utc>> = None;
static mut LICENSE_TIER: Option<String> = None;

/// What a verified licence says. `binary_hash` is intentionally NOT enforced: a binary cannot contain its own hash,
/// so tying a licence to it made every licence unusable. A licence is tied to the product, the customer and an expiry.
#[derive(Debug, Clone, PartialEq)]
pub struct LicenseClaims {
    pub org: String,
    pub email: String,
    pub tier: String,
    pub expires: chrono::DateTime<chrono::Utc>,
    pub max_assets: Option<u64>,
    pub features: Vec<String>,
}

/// Verify a PASETO v4.public licence token against a public key. Pure: no files, no global state.
/// Rejects: wrong format, wrong key, tampering, expired, missing or malformed expiry.
pub fn verify_token(token: &str, public_key_bytes: &[u8]) -> Result<LicenseClaims, VeriCryptError> {
    if token.trim().is_empty() {
        return Err(VeriCryptError::ParseError("Empty license key".into()));
    }
    if public_key_bytes.is_empty() {
        return Err(VeriCryptError::ParseError(
            "This build has no licence public key embedded. Rebuild with paseto_public_key.hex in the repository root.".into(),
        ));
    }
    let untrusted = UntrustedToken::<pasetors::token::Public, V4>::try_from(token.trim())
        .map_err(|e| VeriCryptError::ParseError(format!("Invalid licence key format: {}", e)))?;
    let public_key = AsymmetricPublicKey::<V4>::from(public_key_bytes)
        .map_err(|e| VeriCryptError::ParseError(format!("Invalid public key: {}", e)))?;
    // pasetors also checks the standard `exp` claim against the current time.
    let trusted = pasetors::public::verify(&public_key, &untrusted, &ClaimsValidationRules::new(), None, None)
        .map_err(|e| VeriCryptError::ParseError(format!("Licence verification failed (wrong key, tampered or expired): {}", e)))?;
    let claims: serde_json::Value = serde_json::from_str(trusted.payload())
        .map_err(|e| VeriCryptError::ParseError(format!("Invalid licence claims: {}", e)))?;
    let exp_str = claims["exp"]
        .as_str()
        .ok_or_else(|| VeriCryptError::ParseError("Licence has no expiry".into()))?;
    let expires = chrono::DateTime::parse_from_rfc3339(exp_str)
        .map_err(|e| VeriCryptError::ParseError(format!("Invalid expiry: {}", e)))?
        .with_timezone(&chrono::Utc);
    if expires <= chrono::Utc::now() {
        return Err(VeriCryptError::ParseError(format!("Licence expired on {}", exp_str)));
    }
    Ok(LicenseClaims {
        org: claims["org"].as_str().unwrap_or("").to_string(),
        email: claims["email"].as_str().unwrap_or("").to_string(),
        tier: claims["tier"].as_str().unwrap_or("standard").to_string(),
        expires,
        max_assets: claims["max_assets"].as_u64(),
        features: claims["features"]
            .as_array()
            .map(|a| a.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
            .unwrap_or_default(),
    })
}

pub fn activate(token: &str) -> Result<(), VeriCryptError> {
    let public_key_bytes = embedded::get_paseto_public_key();
    let claims = verify_token(token, &public_key_bytes)?;
    let exp_str = claims.expires.to_rfc3339();
    let tier = claims.tier.clone();
    let now = chrono::Utc::now();

    unsafe {
        LICENSE_ACTIVE = true;
        LICENSE_EXPIRY = Some(claims.expires);
        LICENSE_TIER = Some(tier.clone());
    }

    let license_dir = get_license_dir()?;
    fs::create_dir_all(&license_dir).map_err(VeriCryptError::Io)?;
    fs::write(
        license_dir.join("license.json"),
        serde_json::to_string(&serde_json::json!({
            "token": token.trim(),
            "tier": tier,
            "org": claims.org,
            "expiry": exp_str,
            "activated_at": now.to_rfc3339(),
        }))
        .map_err(|e| VeriCryptError::ParseError(format!("Failed to serialize: {}", e)))?,
    )
    .map_err(VeriCryptError::Io)?;

    tracing::info!(tier = tier, expiry = %claims.expires, "License activated and persisted");
    Ok(())
}

pub fn is_licensed() -> bool {
    if unsafe { LICENSE_ACTIVE } {
        if let Some(expiry) = unsafe { LICENSE_EXPIRY.as_ref() } {
            if *expiry > chrono::Utc::now() {
                return true;
            }
        }
    }
    if let Ok(license_dir) = get_license_dir() {
        let path = license_dir.join("license.json");
        if path.exists() {
            if let Ok(data) = fs::read_to_string(&path) {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&data) {
                    if let Some(exp_str) = json["expiry"].as_str() {
                        if let Ok(expiry) = chrono::DateTime::parse_from_rfc3339(exp_str) {
                            if expiry.with_timezone(&chrono::Utc) > chrono::Utc::now() {
                                unsafe {
                                    LICENSE_ACTIVE = true;
                                    LICENSE_EXPIRY = Some(expiry.with_timezone(&chrono::Utc));
                                    LICENSE_TIER = json["tier"].as_str().map(|s| s.to_string());
                                }
                                return true;
                            }
                        }
                    }
                }
            }
        }
    }
    false
}

pub fn license_tier() -> Option<String> {
    if is_licensed() {
        unsafe { LICENSE_TIER.clone() }
    } else {
        None
    }
}

fn get_license_dir() -> Result<PathBuf, VeriCryptError> {
    let base = std::env::var("VERICRYPT_DATA_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            dirs::data_dir()
                .unwrap_or_else(|| PathBuf::from("/var/lib/vericrypt"))
                .join("vericrypt")
        });
    Ok(base.join("license"))
}
/// Token persisted by `activate`, if any. Used by feature gating, which re-verifies the token
/// signature on every use instead of trusting the expiry written to disk.
pub fn stored_token() -> Option<String> {
    let dir = get_license_dir().ok()?;
    let data = fs::read_to_string(dir.join("license.json")).ok()?;
    let v: serde_json::Value = serde_json::from_str(&data).ok()?;
    v["token"].as_str().map(|s| s.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use pasetors::claims::Claims;
    use pasetors::keys::{AsymmetricKeyPair, Generate};

    fn token_with(kp: &AsymmetricKeyPair<V4>, exp: &str, extra: &[(&str, &str)]) -> String {
        let mut c = Claims::new().unwrap();
        c.add_additional("org", "Pilot Bank").unwrap();
        c.add_additional("email", "ciso@pilot.example").unwrap();
        c.add_additional("tier", "B").unwrap();
        c.expiration(exp).unwrap(); // "exp" is a registered claim: use the dedicated setter
        for (k, v) in extra {
            c.add_additional(k, *v).unwrap();
        }
        pasetors::public::sign(&kp.secret, &c, None, None).unwrap()
    }
    fn far_future() -> String {
        (chrono::Utc::now() + chrono::Duration::days(90)).to_rfc3339()
    }

    #[test]
    fn valid_token_verifies_and_exposes_claims() {
        let kp = AsymmetricKeyPair::<V4>::generate().unwrap();
        let t = token_with(&kp, &far_future(), &[]);
        let c = verify_token(&t, kp.public.as_bytes()).unwrap();
        assert_eq!(c.org, "Pilot Bank");
        assert_eq!(c.tier, "B");
    }
    #[test]
    fn binary_hash_claim_does_not_block_activation() {
        let kp = AsymmetricKeyPair::<V4>::generate().unwrap();
        let t = token_with(&kp, &far_future(), &[("binary_hash", "latest")]);
        assert!(verify_token(&t, kp.public.as_bytes()).is_ok());
    }
    #[test]
    fn expired_token_is_refused() {
        let kp = AsymmetricKeyPair::<V4>::generate().unwrap();
        let t = token_with(&kp, "2020-01-01T00:00:00+00:00", &[]);
        assert!(verify_token(&t, kp.public.as_bytes()).is_err());
    }
    #[test]
    fn token_signed_by_another_key_is_refused() {
        let kp = AsymmetricKeyPair::<V4>::generate().unwrap();
        let other = AsymmetricKeyPair::<V4>::generate().unwrap();
        let t = token_with(&other, &far_future(), &[]);
        assert!(verify_token(&t, kp.public.as_bytes()).is_err());
    }
    #[test]
    fn tampered_token_is_refused() {
        let kp = AsymmetricKeyPair::<V4>::generate().unwrap();
        let mut t = token_with(&kp, &far_future(), &[]).into_bytes();
        let i = t.len() / 2;
        t[i] = if t[i] == b'A' { b'B' } else { b'A' };
        assert!(verify_token(&String::from_utf8(t).unwrap(), kp.public.as_bytes()).is_err());
    }
    #[test]
    fn empty_embedded_key_gives_a_clear_error() {
        let kp = AsymmetricKeyPair::<V4>::generate().unwrap();
        let t = token_with(&kp, &far_future(), &[]);
        let e = verify_token(&t, &[]).unwrap_err().to_string();
        assert!(e.contains("no licence public key"), "{}", e);
    }
    #[test]
    fn the_old_generator_format_is_rejected() {
        // v4.public.<header>.<payload>.<signature>.<footer>  (what the old Supabase function produced)
        let kp = AsymmetricKeyPair::<V4>::generate().unwrap();
        assert!(verify_token("v4.public.eyJhbGciOiJ2NC5wdWJsaWMifQ.eyJvcmciOiJYIn0.c2ln.", kp.public.as_bytes()).is_err());
    }
}
