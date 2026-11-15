use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let mut embed_code = String::new();

    // Embed the PASETO public key used for offline licence verification.
    // Search order: VERICRYPT_PUBKEY_FILE, then the crate folder, then the workspace root.
    // (Cargo runs this script with the crate folder as working directory, so the key file that lives at the
    // repository root must be looked up as ../../paseto_public_key.hex. An older version only looked in the crate folder,
    // which silently embedded an EMPTY key and made every licence activation fail.)
    println!("cargo:rerun-if-env-changed=VERICRYPT_PUBKEY_FILE");
    println!("cargo:rerun-if-changed=../../paseto_public_key.hex");
    let mut candidates: Vec<String> = Vec::new();
    if let Ok(f) = env::var("VERICRYPT_PUBKEY_FILE") {
        println!("cargo:rerun-if-changed={}", f);
        candidates.push(f);
    }
    candidates.push("paseto_public_key.hex".to_string());
    candidates.push("../../paseto_public_key.hex".to_string());
    let public_key_hex = candidates
        .iter()
        .filter_map(|c| fs::read_to_string(c).ok())
        .map(|s| s.trim().to_string())
        .find(|s| !s.is_empty())
        .unwrap_or_default();
    let key_ok = public_key_hex.len() == 64 && public_key_hex.chars().all(|c| c.is_ascii_hexdigit());
    if !key_ok {
        if env::var("CARGO_FEATURE_LICENSE_GATE").is_ok() {
            panic!("licence-gate build needs a 64-hex-character PASETO public key: put it in paseto_public_key.hex at the repository root or set VERICRYPT_PUBKEY_FILE");
        }
        println!("cargo:warning=No valid PASETO public key found: licence activation will fail in this build");
    }
    let public_key_hex = if key_ok { public_key_hex } else { String::new() };
    embed_code.push_str(&format!(
        "pub fn get_paseto_public_key() -> Vec<u8> {{\n    hex::decode(\"{}\").unwrap_or_default()\n}}\n",
        public_key_hex
    ));
    fs::write(out_dir.join("embedded_key.rs"), embed_code.as_bytes()).unwrap();
}
