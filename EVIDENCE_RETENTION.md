# VeriCrypt Evidence Retention Policy

## Retention Periods

- `.pqc` compliance reports: Minimum 7 years (aligned with standard financial record retention)
- CBOM artifacts: Same retention period as parent `.pqc` report
- Migration roadmaps: Retained until superseded by subsequent scan
- Regulatory correspondence referencing VeriCrypt reports: Per applicable regulatory retention requirements

## Cryptographic Survivability

All reports are designed for cryptographic survivability through 2055+ under current NIST PQC assumptions:
- SLH-DSA (NIST FIPS 205): 256-bit classical security, 128-bit quantum security (Security Level 5)
- BLAKE3 (256-bit output): 128-bit effective quantum security via Grover's algorithm
- No classical-only cryptographic primitives used in the evidence chain

## Hash Migration Policy

If BLAKE3 is deprecated:
1. Reports can be re-hashed with successor algorithm, producing new Merkle root
2. Original signature remains valid over original root
3. New signature applied over new root + migration attestation

## Verification Horizon

`vericrypt-verify` shall continue to verify any `.pqc` report produced by any historically-valid VeriCrypt version. Root public keys are archived and published for all historical root keys.
