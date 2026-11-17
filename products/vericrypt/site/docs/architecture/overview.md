# Architecture overview

VeriCrypt is a Rust engine that turns an inventory of cryptographic assets into signed evidence.

1. **Ingest.** Certificates and keys are read from folders, and TLS endpoints are probed (leaf certificate only in v2).
2. **Classify and evaluate.** Each key is classified for quantum exposure, and rules fire on real properties. Lifetimes and transition dates are checked against published schedules.
3. **Assemble evidence.** The results are canonicalised, hashed into a Merkle tree, and signed with SLH-DSA-SHAKE-128f.
4. **Verify.** A separate verifier checks the signature against a pinned key and recomputes the Merkle root.

## Versions

The v2 evidence engine is built and tested. The v4 architecture ("Trustworthy core") is approved and v5 is a draft, but code for them has not started by agreement: the architecture is being finalised first.

Full documents in the repository: `VERICRYPT_ARC42.md`, `docs/architecture/ARCHITECTURE_V4_arc42.md`, `docs/architecture/ARCHITECTURE_V5_arc42.md`, `docs/PROJECT_STATUS.md`, `CLAIMS.md`.
