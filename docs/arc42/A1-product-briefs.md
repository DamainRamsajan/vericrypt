# Appendix A. Product briefs

Facts here come from each product's arc42; product chats own and correct them.

| Product | Accent | One line | Verify command |
| --- | --- | --- | --- |
| VeriChain | #35D47A | A platform where autonomous AI agents run, act and transact, with every consequential action recorded in a log anyone can verify | `vcv verify --min-completeness C2 --min-assurance A1 --witnesses 2 <pack>` |
| VeriCrypt | #8B3A44 | An engine that inventories the cryptography in an estate, flags quantum-vulnerable assets, and produces signed reports anyone can verify offline | `vericrypt verify report.json --pubkey key.pub.json` |
| Sovereign ERP | #E8A617 | A ledger and period-close kernel whose rules are checked by model checking and formal specifications | `cargo test --workspace` |
| OMC | #7C6CF0 | A closed-loop system that measures and proves the financial value of work shared by people, AI models and agents | `python scripts/demo.py` |
| AgentSeed | #FF7A59 (proposed) | A programming language for autonomous agents, where safety rules are checked at compile time | `seed check hello.seed` |

Each product chat hands back: `site.json`, demo engine (`demo.js` or WASM), `downloads.json`, docs Markdown, `verify.md`.

## AgentSeed notes

- Source: the AgentSeed landing page, the `agentseed` repository (README, CLI source, `ASL_V_0_2_0_ARC42.md`). The product is the ASL toolchain: `seedc` compiler and `seedvm` VM, MIT licensed, v0.1.0.
- Commands confirmed from the CLI source: `build`, `check`, `run`, `emit-ir`, `emit-grammar`, `prove`.
- Audience (from the landing page): agent engineers, AI safety practitioners, programming-language researchers.
- The repository's own playground says in-browser compilation to WebAssembly is "coming soon", so the Demo tab shows the labeled placeholder until that exists.
- The accent is a proposal: the project's own pages use a green close to VeriChain's, so the family site uses coral to keep products distinguishable.
- Its arc42 names VeriChain swarm developers as a stakeholder, so the two products are related. The family site does not claim more than that until the product chats say so.

## Status and sources per product (checkpoint 4)

| Product | Status on the site | Source |
| --- | --- | --- |
| VeriChain | Pre-release: specification frozen, core log and verifier built | README, ROADMAP, HANDOFF, `vc-verify` and `vcd` READMEs |
| VeriCrypt | v2 evidence engine built and tested; later versions in design | README, `docs/INSTALL_V2.md`, `docs/PROJECT_STATUS.md`, `CLAIMS.md` |
| Sovereign ERP | Walking skeleton, model-checked | code README and GETTING-STARTED (no arc42 in the drop) |
| OMC | Design phase, walking skeleton on synthetic data | README, `docs/dev/MILESTONE-M0.md`, `LIMITATIONS.md`, arc42 |
| AgentSeed | v0.1.0 | landing page, README, CLI source, arc42 blueprint |

## Rule for content

If a product repo has a claims registry (VeriCrypt's `CLAIMS.md`), the site claims only what that registry allows. Otherwise the site states only what the repo's own docs state, and gives a command only if it appears in the repo's docs or source.
