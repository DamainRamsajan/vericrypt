use crate::errors::VeriCryptError;
use clap::{Parser, Subcommand};

/// VeriCrypt — Post-Quantum Cryptographic Compliance Engine
#[derive(Parser)]
#[command(name = "vericrypt")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "Cryptographic inventory, post-quantum exposure findings and offline-verifiable evidence")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Activate a license key for signed report generation
    Activate(ActivateArgs),
    /// v2 engine: analyse an inventory and write report.json, cbom.json and report.html
    Analyze(crate::engine::AnalyzeArgs),
    /// Generate a post-quantum (SLH-DSA, FIPS 205) report-signing key pair
    Keygen(crate::engine::KeygenArgs),
    /// Verify a report's Merkle commitment and post-quantum signature, offline
    Verify(crate::engine::VerifyArgs),
    /// Compare two reports and show new/resolved assets and findings
    Diff(crate::engine::DiffArgs),
    /// Produce a Merkle inclusion proof for one asset or finding
    Prove(crate::engine::ProveArgs),
    /// Check a Merkle inclusion proof against a trusted root
    VerifyProof(crate::engine::VerifyProofArgs),
    /// Sign any file (for example a release binary) with your post-quantum key
    SignFile(crate::engine::SignFileArgs),
    /// Check a file against its signature file, offline
    VerifyFile(crate::engine::VerifyFileArgs),
}

#[derive(clap::Args)]
pub struct ActivateArgs {
    /// License key (PASETO v4 token)
    #[arg(long)]
    pub key: String,
}

pub fn run_activate(args: ActivateArgs) -> Result<(), VeriCryptError> {
    crate::license::activate(&args.key)
}
