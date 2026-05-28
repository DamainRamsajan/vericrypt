use clap::{Parser, Subcommand};
use crate::errors::VeriCryptError;

/// VeriCrypt — Post-Quantum Cryptographic Compliance Engine
#[derive(Parser)]
#[command(name = "vericrypt")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "Scan cryptographic inventory and produce signed .pqc compliance reports", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Scan cryptographic inventory and produce a .pqc report
    Scan(ScanArgs),
    /// Activate a license key for signed report generation
    Activate(ActivateArgs),
}

#[derive(clap::Args)]
pub struct ScanArgs {
    /// Directory containing certificates to scan
    #[arg(long)]
    pub cert_dir: Option<String>,

    /// Network CIDR range to probe for TLS endpoints
    #[arg(long)]
    pub network: Option<String>,

    /// Output directory for .pqc report and CBOM
    #[arg(long, default_value = "./report/")]
    pub output: String,
}

#[derive(clap::Args)]
pub struct ActivateArgs {
    /// License key (PASETO v4 token)
    #[arg(long)]
    pub key: String,
}

pub fn run_scan(args: ScanArgs) -> Result<(), VeriCryptError> {
    tracing::info!(?args, "Starting scan");
    // Pipeline: Ingest → Graph → Exposure → Compliance → Prioritize → CBOM → Report
    let assets = crate::ingest::discover_all(&args)?;
    let graph = crate::graph::build_graph(assets)?;
    let exposure = crate::exposure::analyze(&graph)?;
    let theorems = crate::compliance::prove_compliance(&graph)?;
    let roadmap = crate::prioritize::generate_roadmap(&exposure, &graph)?;
    let cbom = crate::cbom::generate_cbom(&graph)?;
    let report = crate::report::assemble_report(&args.output, cbom, theorems, roadmap, exposure)?;
    tracing::info!(
        total_assets = report.total_assets,
        violations = report.violations_found,
        "Scan complete"
    );
    Ok(())
}

pub fn run_activate(args: ActivateArgs) -> Result<(), VeriCryptError> {
    crate::license::activate(&args.key)
}
