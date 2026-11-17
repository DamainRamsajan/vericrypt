# Verify it yourself

1. Check out the [repository](https://github.com/intellica-ai-llc/Sovereign-ERP) and run `cargo test --workspace` in `code/`.
2. Run `cargo test -p conformance -- --nocapture` and compare the printed state counts with those recorded in the architecture appendix.
3. Run TLC on `formal/PeriodClose.tla` and `formal/Ledger.tla` and compare the results.
4. Compile the Lean files in `formal/`.

Steps 3 and 4 have not been re-run outside the author's environment; they are the independent check.
