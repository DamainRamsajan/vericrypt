# Reference

## vcv, the verifier

| Command | What it does |
| --- | --- |
| `vcv verify <pack.json or ->` | Verify an evidence pack. Options: `--trust t.json`, `--policy p.json`, `--require-seals` |
| `vcv verify-receipt <bundle.json or ->` | Verify one receipt bundle |
| `vcv version` | Print the version |

Exit codes: 0 all checks passed, 1 a check failed (a JSON report is printed), 2 usage or I/O error.

## Checks in a report

`pack_format`, `trust_anchors`, `cose_signature`, `statement_schema`, `receipt_inclusion`, `checkpoint_operator`, `checkpoint_witness_quorum`, `checkpoint_agreement`, `chain_rules`, `partial_coverage`, `completeness_ceiling`, `policy_min_completeness`, `policy_min_assurance`, `open_intents`.

## Input limits

Strict JSON, no unknown fields, and bounds on size (32 MiB input, 10,000 entries, 64 KiB per statement, 64 witnesses). The verifier is built not to panic on any input, and an empty pack does not pass.

## Levels

| Level | Meaning |
| --- | --- |
| C3 | Hosted agent: no external effect without a prior recorded intent |
| C2 | Chokepoint-enforced, for example a venue that accepts only signer-signed orders |
| C1 | Gateway-observed |
| A0 to A3 | Signed, replay-verified, TEE-attested, zero-knowledge proof (A3 is research-gated) |

## Sources in the repository

`spec/contracts/L0.md`, `spec/statement.schema.json`, `spec/vectors/`, `docs/THREAT_MODEL.md`, `docs/arc42/VERICHAIN_ARC42_v9.md`.
