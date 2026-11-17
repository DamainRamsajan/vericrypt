# Architecture overview

OMC runs four loops for repeatable, high-volume workflows:

1. **Sense.** Reconstruct how the work really runs and where margin leaks, in currency.
2. **Price.** Estimate the calibrated risk and full cost of each step, including human review and rework.
3. **Decide.** Choose which executor handles each case, inside a declared risk budget, with the current process as the safe fallback.
4. **Prove.** Estimate the causal effect on profit and loss with confidence intervals and an evidence grade, backed by signed records the customer's finance team and auditors can re-run.

Checking the correctness of an output is an input to the risk term, not the product.

The measurement is designed to run in the customer's tenant, with a hosted control plane coordinating it. The first slice, built in M0, is the paired audit and promotion gate.

Full document: `docs/arc42/` in the repository (arc42 v0.2) and `docs/adr/`.
