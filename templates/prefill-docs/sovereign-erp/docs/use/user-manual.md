# User manual

Sovereign ERP is a ledger and period-close kernel. The repository today is a **walking skeleton**: a Rust workspace with the Period Controller and the Ledger Port, checked against formal specifications. There is no production code yet.

## Run everything

```
cd code
cargo test --workspace                       # about 30 s in debug
cargo test -p conformance -- --nocapture     # prints mutation results and state counts
```

## What the tests establish

- The spec transcription, explored with the recorded TLC bounds (3 periods, max amount 2, max total 3), reaches 4,160 distinct and 18,833 generated states, the same counts recorded in the architecture appendix.
- The real controller, explored through its public API, reaches the same state space with no violation of the period and adjustment invariants.
- On all 4,160 reachable states and every action, the controller and the spec agree on enabledness and next state (about 150,000 comparisons).
- Six seeded defects are each caught, with shortest counterexample traces.
- 300 seeded random walks of 3,000 steps hold all invariants, rejected commands never change state, and replaying the event log reproduces state exactly.
- The in-memory ledger, explored exhaustively with the recorded constants, reaches 21,433 distinct states at depth 7 with no violation of the four ledger invariants.
