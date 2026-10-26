//! Real post-quantum attestation: SLH-DSA-SHAKE-128f (NIST FIPS 205) via the RustCrypto `slh-dsa` crate.
//! The signature covers the canonical JSON of the report body, under a fixed context string.
//! NOTE: the `slh-dsa` crate has not been independently audited at the time of writing.
use serde::{Deserialize, Serialize};
use slh_dsa::{Shake128f, Signature, SigningKey, VerifyingKey};

pub const ALG: &str = "SLH-DSA-SHAKE-128f (FIPS 205)";
pub const CONTEXT: &[u8] = b"vericrypt-report-v2";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyFile {
    pub alg: String,
    pub key_id: String,
    pub public_key: String,
    pub secret_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attestation {
    pub alg: String,
    pub context: String,
    pub key_id: String,
    pub public_key: String,
    pub signature: String,
}

pub fn key_id(public_key: &[u8]) -> String {
    hex::encode(&blake3::hash(public_key).as_bytes()[..16])
}

pub fn keygen() -> KeyFile {
    let mut rng = rand::rngs::OsRng;
    let sk = SigningKey::<Shake128f>::new(&mut rng);
    let vk: &VerifyingKey<Shake128f> = sk.as_ref();
    let pk = vk.to_vec();
    KeyFile {
        alg: ALG.to_string(),
        key_id: key_id(&pk),
        public_key: hex::encode(&pk),
        secret_key: hex::encode(sk.to_vec()),
    }
}

pub fn sign(message: &[u8], key: &KeyFile) -> Result<Attestation, String> {
    if key.alg != ALG {
        return Err(format!("key algorithm {} is not supported (expected {})", key.alg, ALG));
    }
    let sk_bytes = hex::decode(&key.secret_key).map_err(|e| format!("secret key: {}", e))?;
    let sk = SigningKey::<Shake128f>::try_from(sk_bytes.as_slice()).map_err(|_| "secret key has the wrong length".to_string())?;
    let mut rnd = [0u8; 16];
    rand::RngCore::fill_bytes(&mut rand::rngs::OsRng, &mut rnd);
    let sig = sk.try_sign_with_context(message, CONTEXT, Some(&rnd)).map_err(|_| "signing failed".to_string())?;
    let sig_bytes: Vec<u8> = Vec::from(&sig);
    let pk = hex::decode(&key.public_key).map_err(|e| format!("public key: {}", e))?;
    Ok(Attestation {
        alg: ALG.to_string(),
        context: String::from_utf8_lossy(CONTEXT).to_string(),
        key_id: key_id(&pk),
        public_key: key.public_key.clone(),
        signature: hex::encode(sig_bytes),
    })
}

pub fn verify(message: &[u8], att: &Attestation) -> Result<(), String> {
    if att.alg != ALG {
        return Err(format!("unsupported attestation algorithm {}", att.alg));
    }
    if att.context.as_bytes() != CONTEXT {
        return Err("unexpected signing context".into());
    }
    let pk = hex::decode(&att.public_key).map_err(|e| format!("public key: {}", e))?;
    if key_id(&pk) != att.key_id {
        return Err("key_id does not match the embedded public key".into());
    }
    let vk = VerifyingKey::<Shake128f>::try_from(pk.as_slice()).map_err(|_| "public key has the wrong length".to_string())?;
    let sig_bytes = hex::decode(&att.signature).map_err(|e| format!("signature: {}", e))?;
    let sig = Signature::<Shake128f>::try_from(sig_bytes.as_slice()).map_err(|_| "signature has the wrong length".to_string())?;
    vk.try_verify_with_context(message, CONTEXT, &sig).map_err(|_| "signature does not verify".to_string())
}
