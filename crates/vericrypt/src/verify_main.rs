//! VeriCrypt Verify — Offline .pqc report verification tool.
//!
//! Standalone binary distributed freely to regulators. Verifies SLH-DSA signatures,
//! Merkle proofs, and optional TEE attestation quotes against embedded trust roots.

use std::path::PathBuf;
use std::process;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: vericrypt-verify <report.pqc>");
        process::exit(1);
    }

    let report_path = PathBuf::from(&args[1]);
    match crate::report::verify_file(&report_path) {
        Ok(summary) => {
            println!("VERIFIED — {}", summary);
            process::exit(0);
        }
        Err(e) => {
            eprintln!("VERIFICATION FAILED — {}", e);
            process::exit(1);
        }
    }
}
