# Reference

## What M0 built

| Capability | Where |
| --- | --- |
| Anytime-valid confidence bounds | `omc/stats/cs.py` |
| Paired audit, Tier 0 (no labels) and Tier 1 (labels on a random subset of disagreements) | `omc/audit/paired.py` |
| CI promotion gate with signed verdict and exit codes | `omc/audit/gate.py`, `omc gate` |
| Evidence Statements as signed COSE_Sign1 | `omc/vault/statement.py` |
| Append-only hash chain | `omc/vault/chain.py` |
| Standalone verifier and public test vectors | `omc_verify/`, `vectors/` |
| Model exposure inventory and retirement calendar | `omc/exposure/inventory.py` |
| Leak report: cost per accepted outcome | `omc/report/leaks.py` |
| Replay privacy gate: redaction, leakage bound, provider terms | `omc/privacy/gate.py` |

Commands also include `omc check-data`, `omc pseudonymize` and `omc redact`.

## Limitations (from the repository)

1. Synthetic data only; nothing validated on real tickets.
2. The bounds assume cases come from a fixed distribution over the audit window; drift can invalidate a certificate.
3. Covers losses decided by a gold label; free-text quality is out of scope until a classifier exists.
4. Conservative by design: valid but wide bounds, so close calls are held.
5. Labels are sampled at a uniform rate; gold labels are assumed correct.
6. Signatures are Ed25519 only, with no post-quantum signing, transparency registration or timestamps yet.
7. Redaction is rule-based and catches common patterns only.
8. The statistics are the author's own implementation and need independent statistical review before customer-facing claims.

## Next in the backlog

Real-data spike, material-difference classifier, SCITT registration and post-quantum signatures, helpdesk connectors, outcome billing estimator, human-access audit checks, review workbench.
