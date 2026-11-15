//! VENDOR-ONLY licence tool. Build with `--features issuer`. Never ship this binary to customers.
//!   vericrypt-license keygen --out DIR
//!   vericrypt-license issue  --private-key FILE --org "Pilot Bank" --email ciso@x.com [--tier B] [--days 90] [--max-assets 100000]
//!                            [--features signed_reports,html_report,drift_diff] [--public-key-file paseto_public_key.hex]
//!   vericrypt-license inspect --public-key-file FILE TOKEN
use clap::{Parser, Subcommand};
use pasetors::claims::Claims;
use pasetors::keys::{AsymmetricKeyPair, AsymmetricPublicKey, AsymmetricSecretKey, Generate};
use pasetors::version4::V4;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "vericrypt-license", about = "Issue and inspect VeriCrypt licence keys (vendor only)")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Create a new signing key pair (keep the private key secret)
    Keygen {
        #[arg(long, default_value = ".")]
        out: PathBuf,
    },
    /// Issue a licence key (prints the token)
    Issue {
        #[arg(long)]
        private_key: PathBuf,
        #[arg(long)]
        org: String,
        #[arg(long)]
        email: String,
        #[arg(long, default_value = "B")]
        tier: String,
        #[arg(long, default_value_t = 90)]
        days: i64,
        #[arg(long, default_value_t = 100000)]
        max_assets: u64,
        #[arg(long, value_delimiter = ',', default_value = "signed_reports,html_report,drift_diff")]
        features: Vec<String>,
        /// If given, refuse to issue unless this public key matches the private key
        #[arg(long)]
        public_key_file: Option<PathBuf>,
    },
    /// Verify a token and print its claims
    Inspect {
        #[arg(long)]
        public_key_file: PathBuf,
        token: String,
    },
}

fn read_hex(p: &PathBuf) -> Result<Vec<u8>, String> {
    let s = std::fs::read_to_string(p).map_err(|e| format!("{}: {}", p.display(), e))?;
    hex::decode(s.trim()).map_err(|e| format!("{}: not valid hex ({})", p.display(), e))
}

/// Accepts a 64-byte secret (seed||public) or a 32-byte seed.
fn secret_from(bytes: &[u8]) -> Result<(AsymmetricSecretKey<V4>, Vec<u8>), String> {
    let full: Vec<u8> = match bytes.len() {
        64 => bytes.to_vec(),
        32 => {
            let kp = ed25519_compact::KeyPair::from_seed(ed25519_compact::Seed::from_slice(bytes).map_err(|e| e.to_string())?);
            kp.sk.to_vec()
        }
        n => return Err(format!("private key must be 32 or 64 bytes, got {}", n)),
    };
    let public = full[32..].to_vec();
    let sk = AsymmetricSecretKey::<V4>::from(&full).map_err(|e| format!("invalid private key: {}", e))?;
    Ok((sk, public))
}

fn main() {
    let cli = Cli::parse();
    let r: Result<(), String> = (|| match cli.cmd {
        Cmd::Keygen { out } => {
            let kp = AsymmetricKeyPair::<V4>::generate().map_err(|e| e.to_string())?;
            std::fs::create_dir_all(&out).map_err(|e| e.to_string())?;
            let sk_path = out.join("paseto_private_key.hex");
            std::fs::write(&sk_path, hex::encode(kp.secret.as_bytes())).map_err(|e| e.to_string())?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                std::fs::set_permissions(&sk_path, std::fs::Permissions::from_mode(0o600)).map_err(|e| e.to_string())?;
            }
            std::fs::write(out.join("paseto_public_key.hex"), hex::encode(kp.public.as_bytes())).map_err(|e| e.to_string())?;
            println!("Wrote {} (SECRET, mode 0600) and paseto_public_key.hex in {}", sk_path.display(), out.display());
            println!("Put paseto_public_key.hex in the repository root, then rebuild. Never commit the private key.");
            Ok(())
        }
        Cmd::Issue { private_key, org, email, tier, days, max_assets, features, public_key_file } => {
            let (sk, public) = secret_from(&read_hex(&private_key)?)?;
            if let Some(pf) = public_key_file {
                let expected = read_hex(&pf)?;
                if expected != public {
                    return Err(format!(
                        "this private key does NOT match {}: binaries built with that public key would reject the licence",
                        pf.display()
                    ));
                }
            }
            let expires = chrono::Utc::now() + chrono::Duration::days(days);
            let mut c = Claims::new().map_err(|e| e.to_string())?;
            let mut add = |k: &str, v: &str| c.add_additional(k, v).map_err(|e| e.to_string());
            add("org", &org)?;
            add("email", &email)?;
            add("tier", &tier)?;
            c.expiration(&expires.to_rfc3339()).map_err(|e| e.to_string())?; // registered claim: dedicated setter
            c.add_additional("max_assets", max_assets).map_err(|e| e.to_string())?;
            c.add_additional("features", serde_json::json!(features)).map_err(|e| e.to_string())?;
            let token = pasetors::public::sign(&sk, &c, None, None).map_err(|e| e.to_string())?;
            eprintln!("Issued for {} <{}>, tier {}, expires {}", org, email, tier, expires.to_rfc3339());
            println!("{}", token);
            Ok(())
        }
        Cmd::Inspect { public_key_file, token } => {
            let pk = read_hex(&public_key_file)?;
            let _ = AsymmetricPublicKey::<V4>::from(&pk).map_err(|e| e.to_string())?;
            let c = vericrypt::license::verify_token(&token, &pk).map_err(|e| e.to_string())?;
            println!("VALID  org={}  email={}  tier={}  expires={}  max_assets={:?}  features={:?}", c.org, c.email, c.tier, c.expires, c.max_assets, c.features);
            Ok(())
        }
    })();
    if let Err(e) = r {
        eprintln!("error: {}", e);
        std::process::exit(1);
    }
}
