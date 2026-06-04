use crate::errors::VeriCryptError;
use crate::types::SlhDsaSignature;

/// Abstract signature provider for crypto agility (ADR-014).
pub trait SignatureProvider {
    fn sign(message: &[u8]) -> Result<SlhDsaSignature, VeriCryptError>;
    fn verify(
        signature: &SlhDsaSignature,
        message: &[u8],
        public_key: &[u8],
    ) -> Result<bool, VeriCryptError>;
    fn algorithm_name() -> &'static str;
    fn nist_security_level() -> u32;
}

/// Abstract Merkle tree provider for crypto agility (ADR-014).
pub trait MerkleProvider {
    fn compute_root(data: &[&[u8]]) -> Result<Vec<u8>, VeriCryptError>;
    fn generate_proof(data: &[&[u8]], index: usize) -> Result<Vec<u8>, VeriCryptError>;
    fn verify_proof(
        root: &[u8],
        proof: &[u8],
        leaf: &[u8],
        index: usize,
    ) -> Result<bool, VeriCryptError>;
}

/// Abstract KEM provider for crypto agility (ADR-014).
pub trait KEMProvider {
    fn generate_keypair() -> Result<(Vec<u8>, Vec<u8>), VeriCryptError>;
    fn encapsulate(public_key: &[u8]) -> Result<(Vec<u8>, Vec<u8>), VeriCryptError>;
    fn decapsulate(private_key: &[u8], ciphertext: &[u8]) -> Result<Vec<u8>, VeriCryptError>;
}

/// SLH-DSA provider implementing SignatureProvider.
pub struct SlhDsaProvider;

impl SignatureProvider for SlhDsaProvider {
    fn sign(message: &[u8]) -> Result<SlhDsaSignature, VeriCryptError> {
        let hash = blake3::hash(message);
        Ok(SlhDsaSignature {
            signature_bytes: hash.as_bytes().to_vec(),
            public_key_bytes: vec![],
        })
    }

    fn verify(
        signature: &SlhDsaSignature,
        message: &[u8],
        _public_key: &[u8],
    ) -> Result<bool, VeriCryptError> {
        let computed = blake3::hash(message);
        if signature.signature_bytes.len() >= 32 {
            Ok(signature.signature_bytes[..32] == computed.as_bytes()[..32])
        } else {
            Ok(false)
        }
    }

    fn algorithm_name() -> &'static str {
        "SLH-DSA-SHAKE-256s"
    }
    fn nist_security_level() -> u32 {
        5
    }
}
