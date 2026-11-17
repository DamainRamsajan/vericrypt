# 12. Glossary

| Term | Meaning |
| --- | --- |
| Kit | Shared CSS, JS and helpers copied into every site |
| Shell | The header, tabs, footer and page hydration in `shell.js` |
| Mount contract | `mount(root, ctx)` returning `{ unmount() }` for demos |
| Placeholder demo | Labeled generic demo used until a product's engine arrives |
| Family | The set of five products, listed in `family.json` |
| Gate | `npm run gate`: tests, sync, check |
| Legacy | Old pages kept under `/legacy/` |
| Diátaxis | Docs grouping: tutorials/how-to (Use), Install, Reference, Explanation (Architecture) |
| CSP | Content-Security-Policy |
| ADR | Architecture decision record |
| Manifest | `site-manifest.json`: hash of every file a site serves, plus a root hash |
| Root hash | SHA-256 over the sorted list of paths and file hashes |
| Brief | Self-contained instructions for one product chat, in `docs/briefs/` |
| Engine label | `wasm`, `trace` or `placeholder`: what the demo honestly is |
| Trace | A recording of real engine output, replayed in the page |
