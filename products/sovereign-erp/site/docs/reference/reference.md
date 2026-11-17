# Reference

## Workspace layout

| Path | What it is |
| --- | --- |
| `crates/sim` | Deterministic random numbers and clock; no wall clock or OS entropy in kernel code |
| `crates/period-controller` | Pure decide and evolve state machine for period close, reopen and adjustments, with invariants as checkable functions |
| `crates/conformance` | Exhaustive model check, lockstep check against a transcription of the TLA+ spec, mutation tests, random walks |
| `crates/ledger-port` | Engine-neutral Ledger Port trait, in-memory reference ledger, invariants I1 to I4, a conformance suite any adapter must pass |
| `formal/` | TLA+ specs (`PeriodClose`, `Ledger`) and Lean files (`ClosedPeriod`, `LedgerConservation`) |

## Not covered yet

The checks are bounded and small-constant. Multi-currency, consolidation, sub-ledger detail, time and concurrency are not modelled.

## Next steps (from the repository)

1. A TigerBeetle adapter for the Ledger Port, run through the conformance suite.
2. Extend the ledger model to linked transfers, multi-currency and period close, then wire the controller to the ledger.
3. An event store on PostgreSQL with explicit valid and recorded time.
4. Wire the controller to the event store, then the first slice of the governed semantic layer.
