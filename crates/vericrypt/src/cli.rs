use crate::confidence;
use crate::errors::VeriCryptError;
use crate::types::{ComplianceConfidence, InventoryConfidence, StageTiming};
use clap::{Parser, Subcommand, ValueEnum};

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

    /// Deployment mode (Addendum 2 §5.4)
    #[arg(long, default_value = "shadow")]
    pub mode: DeploymentMode,

    /// Path to signed theorem pack for custom regulatory axioms (ADR-017)
    #[arg(long)]
    pub load_theorems: Option<String>,

    /// Export Signed Tree Head for VeriChain anchoring
    #[arg(long)]
    pub publish_sth: bool,
    /// Path to custom ASL axiom source files for Mode 2/3 compliance (EAC FR-001)
    #[arg(long)]
    pub axiom_source: Option<String>,
}

#[derive(clap::Args)]
pub struct ActivateArgs {
    /// License key (PASETO v4 token)
    #[arg(long)]
    pub key: String,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum DeploymentMode {
    /// Phase 1: Reports generated but not submitted to regulators
    Shadow,
    /// Phase 2: Reports submitted alongside traditional documentation
    Parallel,
    /// Phase 3: .pqc files are primary compliance evidence
    Primary,
}

pub fn run_scan(args: ScanArgs) -> Result<(), VeriCryptError> {
    let mode_label = match args.mode {
        DeploymentMode::Shadow => "SHADOW (Phase 1)",
        DeploymentMode::Parallel => "PARALLEL (Phase 2)",
        DeploymentMode::Primary => "PRIMARY (Phase 3)",
    };

    tracing::info!(mode = mode_label, "Starting scan");

    let mut stage_timings: Vec<StageTiming> = Vec::new();

    // Stage 1: Ingestion
    let t0 = std::time::Instant::now();
    let ingestion = crate::ingest::discover_all(&args)?;
    let assets = ingestion.assets;
    stage_timings.push(StageTiming {
        stage_name: "ingestion".into(),
        elapsed_ms: t0.elapsed().as_millis() as u64,
        complexity: "O(n)".into(),
        item_count: assets.len() as u64,
    });

    let inventory = confidence::compute_inventory_confidence(assets.len() as u64, ingestion.unreachable_assets, &ingestion.unsupported_formats, ingestion.encrypted_uninspectable);

    // Stage 2: Knowledge graph
    let t1 = std::time::Instant::now();
    let graph = crate::graph::build_graph(assets)?;
    stage_timings.push(StageTiming {
        stage_name: "graph_building".into(),
        elapsed_ms: t1.elapsed().as_millis() as u64,
        complexity: "O(n log n)".into(),
        item_count: graph.node_count() as u64,
    });

    // Stage 3: Exposure analysis
    let t2 = std::time::Instant::now();
    let exposure = crate::exposure::analyze(&graph)?;
    stage_timings.push(StageTiming {
        stage_name: "exposure_analysis".into(),
        elapsed_ms: t2.elapsed().as_millis() as u64,
        complexity: "O(n²) exact / O(n) Monte Carlo".into(),
        item_count: graph.node_count() as u64,
    });

    // Stage 4: ASL VM compliance verification
    let t3 = std::time::Instant::now();
    let theorems = if let Some(source_dir) = &args.axiom_source {
        // Mode 2/3: Compile custom axioms at scan time (EAC FR-001, FR-002)
        let mut custom_theorems = Vec::new();
        let runtime = crate::compliance::asl_runtime::AslRuntime::new();
        for entry in std::fs::read_dir(source_dir).map_err(|e| VeriCryptError::Io(e))? {
            let entry = entry.map_err(|e| VeriCryptError::Io(e))?;
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "asl") {
                let source = std::fs::read_to_string(&path).map_err(|e| VeriCryptError::Io(e))?;
                let bytecode = seedc::compile(&source).map_err(|e| VeriCryptError::ParseError(format!("Axiom compilation failed: {:?}", e)))?;
                let framework = path.file_stem().unwrap().to_string_lossy().to_uppercase();
                let inventory_hash = blake3::hash(b"custom-inventory").as_bytes().to_vec();
                let (_, theorem) = runtime.execute_framework(&framework, &bytecode)?;
                custom_theorems.push(theorem);
            }
        }
        custom_theorems
    } else if let Some(bytecode_path) = &args.load_theorems {
        let bytecode = std::fs::read(bytecode_path).map_err(|e| VeriCryptError::Io(e))?;
        let runtime = crate::compliance::asl_runtime::AslRuntime::new();
        let framework = "CUSTOM";
        let (_, theorem) = runtime.execute_framework(framework, &bytecode)?;
        vec![theorem]
    } else {
        crate::compliance::prove_compliance(&graph)?
    };
    stage_timings.push(StageTiming {
        stage_name: "asl_compliance".into(),
        elapsed_ms: t3.elapsed().as_millis() as u64,
        complexity: "VM execution".into(),
        item_count: theorems.len() as u64,
    });

    let compliance_conf = confidence::compute_compliance_confidence(&theorems, &inventory);

    // Stage 5: Prioritization
    let t4 = std::time::Instant::now();
    let roadmap = crate::prioritize::generate_roadmap(&exposure, &graph)?;
    stage_timings.push(StageTiming {
        stage_name: "prioritization".into(),
        elapsed_ms: t4.elapsed().as_millis() as u64,
        complexity: "O(n log n)".into(),
        item_count: roadmap.len() as u64,
    });

    // Stage 6: CBOM
    let t5 = std::time::Instant::now();
    let cbom = crate::cbom::generate_cbom(&graph)?;
    stage_timings.push(StageTiming {
        stage_name: "cbom".into(),
        elapsed_ms: t5.elapsed().as_millis() as u64,
        complexity: "O(n)".into(),
        item_count: graph.node_count() as u64,
    });

    // Stage 7: Report
    let t6 = std::time::Instant::now();
    let report = crate::report::assemble_report(&args.output, cbom, theorems, roadmap)?;
    stage_timings.push(StageTiming {
        stage_name: "report".into(),
        elapsed_ms: t6.elapsed().as_millis() as u64,
        complexity: "O(n) + O(1) signing".into(),
        item_count: 1,
    });

    // STH export
    if args.publish_sth {
        let sth = crate::report::verichain::SignedTreeHead::new(
            hex::decode(&report.cbom_merkle_root).unwrap_or_default(),
            1,
        );
        let sth_path = std::path::Path::new(&args.output).join("sth.json");
        std::fs::write(&sth_path, sth.export_for_anchoring()).map_err(|e| VeriCryptError::Io(e))?;
        tracing::info!("STH exported for VeriChain anchoring");
    }

    eprintln!("  Stage timings:");
    for t in &stage_timings {
        eprintln!("    {} ({}, {} items): {}ms", t.stage_name, t.complexity, t.item_count, t.elapsed_ms);
    }
    eprintln!();
    eprintln!("=== VERICRYPT SCAN COMPLETE ===");
    eprintln!("  Mode: {}", mode_label);
    if args.axiom_source.is_some() {
        eprintln!("  Axiom mode: BANK_CUSTOM (Mode 2)");
    } else if args.load_theorems.is_some() {
        eprintln!("  Axiom mode: THIRD_PARTY (Mode 3)");
    } else {
        eprintln!("  Axiom mode: STANDARD (Mode 1)");
    }
    eprintln!("  Assets discovered: {}", report.total_assets);
    eprintln!("  Quantum-vulnerable: {}", report.quantum_vulnerable_count);
    eprintln!("  Compliance violations: {}", report.violations_found);
    eprintln!(
        "  Compliance confidence: {:.2} (proof={:.2} × inventory={:.2} × axiom={:.2})",
        compliance_conf.composite_confidence,
        compliance_conf.proof_confidence,
        compliance_conf.inventory_confidence,
        compliance_conf.regulatory_axiom_confidence,
    );
    eprintln!(
        "  Inventory confidence: {:?} ({:.0}%)",
        inventory.confidence_level,
        inventory.visibility_score * 100.0,
    );
    eprintln!("  Report: {}/report.pqc", args.output);

    if matches!(args.mode, DeploymentMode::Shadow) {
        eprintln!();
        eprintln!("  NOTE: Shadow mode — this report is NOT for regulatory submission.");
    }

    Ok(())
}

pub fn run_activate(args: ActivateArgs) -> Result<(), VeriCryptError> {
    crate::license::activate(&args.key)
}
