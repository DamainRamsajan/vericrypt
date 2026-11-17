# 1. Introduction and goals

## 1.1 Requirements overview

The Intellica web system is the public face of five products: **VeriChain**, **VeriCrypt**, **Sovereign ERP**, **OMC** and **AgentSeed**. It is one company home plus one site per product, all served under a single domain, `meeting-mind.com`.

Each product site works like a small company page. It has six tabs:

| Tab | Purpose |
| --- | --- |
| Overview | What the product is, who it is for, how to verify a claim yourself |
| Demo | A browser-run demo of the product (engine supplied by the product's own repo) |
| Download | Binaries with checksums, driven by `downloads.json` |
| Docs | User manual, installation manual, reference, architecture (Diátaxis groups) |
| Verify | How to reproduce a result without trusting Intellica |
| Support | Contact and support entry point |

The company home at the apex routes visitors by role and lists the five products.

## 1.2 Quality goals

| Priority | Goal | Meaning |
| --- | --- | --- |
| 1 | Verifiability | Every page says how to check a claim yourself; the site itself loads nothing from third parties |
| 2 | Independence of products | Each product repo, site and release cycle stands alone |
| 3 | Consistency | One shell, one set of tokens; products differ by accent and content only |
| 4 | Accessibility | WCAG 2.2 AA, keyboard-first, reduced motion respected |
| 5 | Speed | LCP ≤ 2.5 s, INP ≤ 200 ms, CLS ≤ 0.1 on a mid-range phone |
| 6 | Low cost | Static hosting, one domain, no paid services required |

## 1.3 Stakeholders

| Stakeholder | Expectation |
| --- | --- |
| Visitors (CISO, CFO, auditor, agent builder, partner) | Find their product in one click, try it, verify it |
| Owner (Intellica) | One place to keep consistent; no extra domains |
| Product chats / repos | A clear contract for demo, downloads, docs |
| Regulators and auditors | Reproducible verification steps |
