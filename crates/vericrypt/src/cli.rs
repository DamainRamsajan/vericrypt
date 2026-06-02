use clap::{Parser, Subcommand};
use crate::errors::VeriCryptError;

/// VeriCrypt — Post-Quantum Cryptographic Compliance Engine
#[derive(Parser)]
#[command(name = "vericrypt")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "Scan cryptographic inventory and produce signed .pqc compliance reports")]
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

#[derive(Debug, clap::Args)]
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

    let assets = crate::ingest::discover_all(&args)?;
    tracing::info!(count = assets.len(), "Ingestion complete");

    let graph = crate::graph::build_graph(assets)?;
    tracing::info!(nodes = graph.node_count(), "Graph built");

    let exposure = crate::exposure::analyze(&graph)?;
    tracing::info!(total = exposure.total_hndl_exposure, "Exposure analyzed");

    let theorems = crate::compliance::prove_compliance(&graph)?;
    tracing::info!(count = theorems.len(), "Compliance checked");

    let roadmap = crate::prioritize::generate_roadmap(&exposure, &graph)?;
    tracing::info!(phases = roadmap.len(), "Roadmap generated");

    let cbom = crate::cbom::generate_cbom(&graph)?;
    tracing::info!("CBOM generated");

    let report = crate::report::assemble_report(&args.output, cbom, theorems, roadmap)?;
    tracing::info!(id = %report.report_id, assets = report.total_assets, "Scan complete");

    eprintln!();
    eprintln!("=== VERICRYPT SCAN COMPLETE ===");
    eprintln!("  Assets discovered: {}", report.total_assets);
    eprintln!("  Quantum-vulnerable: {}", report.quantum_vulnerable_count);
    eprintln!("  Compliance violations: {}", report.violations_found);
    eprintln!("  Report: {}/report.pqc", args.output);

    Ok(())
}

pub fn run_activate(args: ActivateArgs) -> Result<(), VeriCryptError> {
    crate::license::activate(&args.key)
}
