use crate::errors::VeriCryptError;
use pasetors::claims::ClaimsValidationRules;
use pasetors::keys::AsymmetricPublicKey;
use pasetors::token::UntrustedToken;
use pasetors::version4::V4;
use std::fs;
use std::path::PathBuf;

mod embedded {
    include!(concat!(env!("OUT_DIR"), "/embedded_axioms.rs"));
}

static mut LICENSE_ACTIVE: bool = false;
static mut LICENSE_EXPIRY: Option<chrono::DateTime<chrono::Utc>> = None;
static mut LICENSE_TIER: Option<String> = None;

pub fn activate(token: &str) -> Result<(), VeriCryptError> {
    if token.is_empty() {
        return Err(VeriCryptError::ParseError("Empty license key".into()));
    }

    let untrusted = UntrustedToken::<pasetors::token::Public, V4>::try_from(token)
        .map_err(|e| VeriCryptError::ParseError(format!("Invalid PASETO token: {}", e)))?;

    let public_key_bytes = embedded::get_paseto_public_key();
    let public_key = AsymmetricPublicKey::<V4>::from(&public_key_bytes)
        .map_err(|e| VeriCryptError::ParseError(format!("Invalid public key: {}", e)))?;

    let validation_rules = ClaimsValidationRules::new();
    let trusted = pasetors::public::verify(
        &public_key,
        &untrusted,
        &validation_rules,
        None,
        None,
    )
    .map_err(|e| VeriCryptError::ParseError(format!("Signature verification failed: {}", e)))?;

    let payload_str = trusted.payload();
    let claims: serde_json::Value = serde_json::from_str(payload_str)
        .map_err(|e| VeriCryptError::ParseError(format!("Invalid token claims: {}", e)))?;

    let token_binary_hash = claims["binary_hash"].as_str().unwrap_or("unknown");
    let our_binary_hash = option_env!("VERICRYPT_BINARY_HASH").unwrap_or(env!("CARGO_PKG_VERSION"));
    if token_binary_hash != our_binary_hash {
        return Err(VeriCryptError::ParseError(format!(
            "License not valid for this binary version. Token scope: {}, binary: {}",
            token_binary_hash, our_binary_hash
        )));
    }

    let tier = claims["tier"].as_str().unwrap_or("standard").to_string();
    let exp_str = claims["exp"].as_str().unwrap_or("unknown");
    let expiry = chrono::DateTime::parse_from_rfc3339(exp_str)
        .map_err(|e| VeriCryptError::ParseError(format!("Invalid expiry: {}", e)))?;
    let now = chrono::Utc::now();

    unsafe {
        LICENSE_ACTIVE = true;
        LICENSE_EXPIRY = Some(expiry.with_timezone(&chrono::Utc));
        LICENSE_TIER = Some(tier.clone());
    }

    let license_dir = get_license_dir()?;
    fs::create_dir_all(&license_dir).map_err(|e| VeriCryptError::Io(e))?;
    fs::write(
        license_dir.join("license.json"),
        serde_json::to_string(&serde_json::json!({
            "token": token,
            "tier": tier,
            "expiry": exp_str,
            "activated_at": now.to_rfc3339(),
        }))
        .map_err(|e| VeriCryptError::ParseError(format!("Failed to serialize: {}", e)))?,
    )
    .map_err(|e| VeriCryptError::Io(e))?;

    tracing::info!(tier = tier, expiry = %expiry, "License activated and persisted");
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