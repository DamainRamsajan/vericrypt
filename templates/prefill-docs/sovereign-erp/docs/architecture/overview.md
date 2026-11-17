# Architecture overview

Sovereign ERP is organised around a small kernel whose rules are checked, not just tested.

- **Period Controller.** A pure state machine that decides whether a period can close, reopen, or take an adjustment, and evolves state from the result. Its invariants are written as functions so they can be checked exhaustively.
- **Ledger Port.** An interface that hides the ledger engine, with an in-memory reference implementation and a conformance suite. A new engine adapter is accepted by passing the suite.
- **Formal specifications.** TLA+ models of period close and the ledger, and Lean files about closed periods and ledger conservation, kept in `formal/`.
- **Determinism.** Kernel code takes time and randomness only from `sim`, so runs are reproducible.

The full architecture document (`arc42-v*.md`) lives in the repository root and is not included in the code drop this page was built from.
