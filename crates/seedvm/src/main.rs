//! AGENT-SEED v15.2 virtual machine CLI — `seedvm`.
//!
//! Executes `.aslb` bytecode modules produced by `seedc`.
//!
//! EXAMPLES:
//!   seedvm run hello.aslb
//!   seedvm run hello.aslb --seed 12345
//!   seedvm trace hello.aslb
//!   seedvm prove hello.aslb

use clap::{Parser, Subcommand};
use miette::{IntoDiagnostic, WrapErr};
use std::path::PathBuf;
use tracing_subscriber::EnvFilter;

// ═══════════════════════════════════════════════════════════════
// CLI
// ═══════════════════════════════════════════════════════════════

#[derive(Parser, Debug)]
#[command(
    name = "seedvm",
    version,
    about = "AGENT-SEED v15.2 virtual machine",
    long_about = "Deterministic bytecode interpreter for .aslb modules.",
    disable_help_subcommand = true
)]
struct Cli {
    /// Verbosity: -v (info), -vv (debug), -vvv (trace)
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Run a compiled .aslb module
    Run(RunArgs),
    /// Execute with full instruction tracing
    Trace(RunArgs),
    /// Generate an execution proof from a trace
    Prove(ProveArgs),
}

#[derive(clap::Args, Debug)]
struct RunArgs {
    /// Path to the .aslb bytecode file
    #[arg(value_name = "MODULE")]
    module: PathBuf,

    /// Deterministic seed for the PRNG (default: 0)
    #[arg(short, long, default_value = "0")]
    seed: u64,

    /// Maximum stack depth (default: 4096)
    #[arg(long, default_value = "4096")]
    max_stack: usize,
}

#[derive(clap::Args, Debug)]
struct ProveArgs {
    /// Path to the .aslb bytecode file
    #[arg(value_name = "MODULE")]
    module: PathBuf,

    /// Deterministic seed for the PRNG
    #[arg(short, long, default_value = "0")]
    seed: u64,

    /// Path to write the proof artifact (stdout if omitted)
    #[arg(short = 'o', long)]
    output: Option<PathBuf>,
}

// ═══════════════════════════════════════════════════════════════
// Entry point
// ═══════════════════════════════════════════════════════════════

fn main() -> miette::Result<()> {
    let cli = Cli::parse();

    let log_level = match cli.verbose {
        0 => "warn",
        1 => "info",
        2 => "debug",
        _ => "trace",
    };
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new(log_level))
        .with_writer(std::io::stderr)
        .init();

    match cli.command {
        Commands::Run(args) => cmd_run(args, false),
        Commands::Trace(args) => cmd_run(args, true),
        Commands::Prove(args) => cmd_prove(args),
    }
}

fn cmd_run(args: RunArgs, trace: bool) -> miette::Result<()> {
    let state = seedvm::run_file(&args.module, args.seed)
        .wrap_err_with(|| format!("VM execution failed for `{}`", args.module.display()))?;

    if trace {
        println!("Schedule trace:\n{}", state.schedule_trace);
    }

    eprintln!(
        "Execution complete — {} provenance events, {} schedule steps",
        state.provenance_log.len(),
        state.schedule_trace.len(),
    );

    Ok(())
}

fn cmd_prove(args: ProveArgs) -> miette::Result<()> {
    let state = seedvm::run_file(&args.module, args.seed)
        .wrap_err_with(|| format!("VM execution failed for `{}`", args.module.display()))?;

    // Build a proof artifact from the execution
    let trace_text = format!("{}", state.schedule_trace);
    let trace_hash = hex::encode(blake3::hash(trace_text.as_bytes()).as_bytes());

    let proof = serde_json::json!({
        "trace_hash": trace_hash,
        "provenance_events": state.provenance_log.len(),
        "schedule_steps": state.schedule_trace.len(),
        "exit_code": state.exit_code,
    });

    let proof_str = serde_json::to_string_pretty(&proof).into_diagnostic()?;

    if let Some(path) = args.output {
        std::fs::write(&path, &proof_str)
            .into_diagnostic()
            .wrap_err_with(|| format!("failed to write proof to `{}`", path.display()))?;
    } else {
        println!("{}", proof_str);
    }

    Ok(())
}
