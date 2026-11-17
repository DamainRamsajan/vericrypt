# 4. Solution strategy

| Goal | Approach |
| --- | --- |
| Consistency across five repos | A shared **kit** (CSS tokens, shell, helpers) copied into each repo by a sync script, with a hash manifest that flags hand edits |
| Independence | Each product is a standalone static site; it works with only its own files |
| Safety | No inline script or style; strict CSP; JSON data is inserted as text; Markdown is escaped before rendering |
| Content without code | `site.json`, `downloads.json`, `docs.json` and Markdown files drive every page |
| Demos from product chats | A **mount contract**: `export async function mount(root, ctx)` returns `{ unmount() }`. The shell supplies the frame |
| Honesty before engines exist | A labeled placeholder demo so pages are never empty or misleading |
| Verifiability | A Verify tab per product and a "Run it yourself" command panel in each demo |
| Speed | Static files, no framework, system fonts, wasm loaded only on `/demo/*` |

Key decisions are recorded in chapter 9.
