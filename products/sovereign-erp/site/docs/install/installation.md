# Installation

## What you need

- Rust, current stable, with `cargo`
- A GitHub Codespace works well; the browser editor alone cannot run `cargo`

## Get and test

```
git clone https://github.com/intellica-ai-llc/Sovereign-ERP
cd Sovereign-ERP/code
cargo test --workspace
```

If `cargo` is not found in a Codespace, run "Codespaces: Rebuild Container" from the Command Palette.

To confirm the formal results independently, run `formal/PeriodClose.tla` and `formal/Ledger.tla` with TLC. The TLA+ was not re-run in the sandbox that produced the workspace, and the Lean files were not recompiled there either.
