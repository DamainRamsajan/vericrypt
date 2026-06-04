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
        Commands::Scan(args) => vericrypt::cli::run_scan(args).map_err(|e| {
            tracing::error!(error = %e, "Scan failed");
            1
        }),
        Commands::Activate(args) => vericrypt::cli::run_activate(args).map_err(|e| {
            tracing::error!(error = %e, "License activation failed");
            2
        }),
    }
}
