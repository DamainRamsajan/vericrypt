# VeriCrypt Regulatory Axiom Mapping

## DORA Article-to-Theorem Mapping

| DORA Article | Requirement | ASL Axiom | Verification |
|---|---|---|---|
| Art. 5 | ICT governance | `ict_governance(system)` | Inventory completeness + policy documentation |
| Art. 9 | Protection of ICT systems | `ict_protection(system)` | Algorithm classification + migration path validation |
| Art. 10 | Detection | `ict_detection(system)` | Continuous monitoring capability |
| Art. 12 | Crypto-agility | `crypto_agility(system)` | All quantum-vulnerable assets have NIST FIPS 204/205 migration paths |
| Art. 13 | ICT incident management | `ict_incident_mgmt(system)` | Incident response plan evidence |
| Art. 14 | Reporting | `ict_reporting(system)` | Report generation capability |

## SEC PQFIF Mapping

| PQFIF Requirement | Verification |
|---|---|
| Cryptographic inventory completeness | Visibility score ≥ 0.80 |
| PQC migration timeline | Phase 1/2/3 assignments with regulatory milestones |
| Multi-jurisdictional compliance | DORA + NCSC + NIST cross-mapping |

## NCSC Phase Mapping

| NCSC Phase | Timeline | Verification |
|---|---|---|
| Phase 1 | Discovery | All critical systems inventoried |
| Phase 2 | Migration planning | Roadmap with NIST PQC replacements |
| Phase 3 | Execution | Migration completion evidence |

## Evidence Retention

- .pqc compliance reports: Minimum 7 years
- CBOM artifacts: Same retention period as parent .pqc report
- Migration roadmaps: Retained until superseded by subsequent scan
- Cryptographic survivability through 2055+ under NIST PQC assumptions
