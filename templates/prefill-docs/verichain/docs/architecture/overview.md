# Architecture overview

VeriChain has two halves. The **evidence foundation (L0)** is a witnessed, append-only, post-quantum-ready transparency log of signed agent statements, exported in SCITT form. The **platform above it (L1 to L6)** adds identity and delegated authority, a runtime that hosts agents, assurance tiers, multi-operator ordering, settlement, and governance.

| Layer | What it adds |
| --- | --- |
| L0 Evidence | Submit statement, get receipt, verify inclusion and consistency offline, witness cosigning, trusted-time anchoring, evidence packs |
| L1 Identity and authority | Agent and operator identity, delegation chains with attenuation and revocation, key custody, hybrid post-quantum signing |
| L2 Runtime | Deploy and sandbox agents, mediated tool and network I/O, replay, kill switch, a gateway for external agents |
| L3 Assurance | Policy decisions, replay verification (A1), TEE attestation (A2), zero-knowledge proofs (A3, research-gated) |
| L4 Ordering and storage | Single sequencer first, then replicated ordering across operators and federation |
| L5 Economy | Agent payments, job escrow, reputation derived from evidence (legal-gated parts) |
| L6 Governance and compliance | Compliance mappings, regulator and auditor views, operator console |

Agents can run **on** VeriChain (strongest evidence) or **beside** it through a gateway or signer.

## Where it stands

The roadmap works in gates. G0.5 (the engineering factory and core crates `vc-core`, `vc-log`, `vc-cose`) is done, and G1 (spec freeze: schema, vectors, requirements, threat model) is frozen. The next gates are the tile log and sequencer, witnesses, anchors and the verifier with browser support.

Full specification: `docs/arc42/VERICHAIN_ARC42_v9.md` and `ROADMAP.md` in the repository.
