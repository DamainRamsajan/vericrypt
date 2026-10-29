#![allow(warnings)]
use clap::Parser;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};
use vericrypt::cli::{Cli, Commands};

fn main() -> Result<(), i32> {
    let filter =
        EnvFilter::try_from_env("VERICRYPT_LOG_LEVEL").unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::registry()
        .with(fmt::layer().json().with_writer(std::io::stderr))
        .with(filter)
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Activate(args) => vericrypt::cli::run_activate(args).map_err(|e| {
            tracing::error!(error = %e, "License activation failed");
            2
        }),
        Commands::Analyze(a) => engine_exit(vericrypt::engine::run_analyze(a)),
        Commands::Reevaluate(a) => engine_exit(vericrypt::engine::run_reevaluate(a)),
        Commands::Keygen(a) => engine_exit(vericrypt::engine::run_keygen(a)),
        Commands::Verify(a) => engine_exit(vericrypt::engine::run_verify(a)),
        Commands::Diff(a) => engine_exit(vericrypt::engine::run_diff(a)),
        Commands::Prove(a) => engine_exit(vericrypt::engine::run_prove(a)),
        Commands::VerifyProof(a) => engine_exit(vericrypt::engine::run_verify_proof(a)),
    }
}

fn engine_exit(r: vericrypt::engine::CliResult) -> Result<(), i32> {
    if let Err((code, msg)) = r {
        eprintln!("{}", msg);
        // Exit with the real code (CI gates depend on it); returning Err from main would collapse it to 1.
        std::process::exit(code);
    }
    Ok(())
}
