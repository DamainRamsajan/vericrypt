# Installation

VeriChain is built from source today.

## What you need

- Rust, current stable, with `cargo` (install from rustup.rs)
- git

## Build

```
git clone https://github.com/intellica-ai-llc/verichain
cd verichain
cargo build --release -p vcd -p vc-verify
```

The programs land in `target/release/`: `vcd` (log daemon and demo) and `vcv` (verifier).

## Run the tests

```
cargo test -p vc-verify
```

## Hosting a pilot server

The repository includes a pilot runbook (`docs/PILOT_RUNBOOK.md`) for running `vc-server` on free resources, with HTTPS, backups and monitoring, and notes on what free tiers cannot promise. It is for testing, not for customers with an SLA.

## Not available yet

Packaged binaries, a hosted service and the browser verifier are not published. This page will list them when they are.
