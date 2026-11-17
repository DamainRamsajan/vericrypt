# 11. Risks and technical debt

| ID | Item | Mitigation |
| --- | --- | --- |
| R1 | Kit copies drift between repos | Manifest and `npm run check`; keep versions in `VERSION` |
| R2 | Placeholder demos left in place | Label is visible; `CONTENT-TODO.md` tracks each product |
| R3 | Sovereign ERP has no arc42 yet | Page shows only what is defined; fill when arc42 exists |
| R4 | Legacy pages need a loose CSP | Confined to `/legacy/*`; rebuild pages over time |
| R5 | Domain moves can break email | Keep MX/TXT records; change only web records |
| R6 | Support chat has no owner or backend | Support tab is a contact page until decided |
| R7 | Binary hosting undecided | Download tab links to wherever files live; checksums required |
| R8 | Products' verify commands not confirmed | Marked in `CONTENT-TODO.md` |
| R9 | Hand-written Markdown renderer | Tested; escapes first; keep scope small |
| R10 | A product's marketing text runs ahead of its claims registry (VeriCrypt) | Site follows `CLAIMS.md`; revisit when the registry changes |
| R11 | Product statuses go stale | Status is one field per product in `family.json` and `site.json`; update at each checkpoint |
| R12 | Cloudflare features rewrite pages and inject scripts | Turn off Email Address Obfuscation and Rocket Loader (and Auto Minify where it exists); Verify-this-site shows it if they are on |
| R13 | Manifest and files drift if someone edits without syncing | The gate runs sync first; the browser audit checks every site's manifest |
| R14 | The same-origin manifest cannot expose a compromised host | Stated on the page; compare the root hash with the repository copy. A signed manifest (SLH-DSA, via VeriCrypt's tooling) is a future option |
| R15 | Briefs go stale | Regenerate each checkpoint; product chats are authoritative |
| R16 | A path missing from `_headers` is served without a Content-Security-Policy (as `/verify-site/` was) | Browser audit checks every HTML page of every site for a strict policy |
| R17 | The slide deck, which holds all course content, is reachable by anyone who learns its unlisted address | Put `/training/arc42/deck-*` behind Cloudflare Access (free for up to 50 users) or do not deploy it; the course gates only the course pages |
| R18 | A buyer shares their access code | Name shown on every lesson; re-encrypt with a new key and re-issue codes if it spreads (needs `COURSE_KEY` rotation) |
| R19 | The course key is lost, or changed by accident | Back it up when first created; changing it locks out every existing buyer. Stored in `.course-key` (git-ignored) or the `COURSE_KEY` secret |
| R20 | Browser read-aloud voices are poor on some devices | Offer recorded audio per slide (`course/audio/<lesson>/NN.mp3`); the transcript is always there |
