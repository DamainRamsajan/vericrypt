//! VeriCrypt — Post-Quantum Cryptographic Compliance Engine
//!
//! Single air-gapped binary. Ingests certificate inventories, classifies quantum
//! vulnerability, proves regulatory compliance via Lean 4 theorem extraction,
//! and outputs cryptographically signed .pqc compliance artifacts.

mod ingest;
mod graph;
mod exposure;
mod compliance;
mod prioritize;
mod cbom;
mod report;
mod tee;
mod license;
mod cli;
mod types;
mod errors;

use clap::Parser;
use cli::{Cli, Commands};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

fn main() -> Result<(), i32> {
    // Initialize structured JSON logging to stderr
    let filter = EnvFilter::try_from_env("VERICRYPT_LOG_LEVEL")
        .unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::registry()
        .with(fmt::layer().json().with_writer(std::io::stderr))
        .with(filter)
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Scan(args) => {
            cli::run_scan(args).map_err(|e| {
                tracing::error!(error = %e, "Scan failed");
                1
            })
        }
        Commands::Activate(args) => {
            cli::run_activate(args).map_err(|e| {
                tracing::error!(error = %e, "License activation failed");
                2
            })
        }
    }
}
