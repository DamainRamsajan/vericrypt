# Checkpoints

## Checkpoint 14 (current): v14
Includes 13. New logo (I-beam tile with a cyan checkable half), the original interactive twelve-section card on the workshop page (aligned with the headline), a visible "You are in" banner after a course code is pasted, `npm run course:check`, course price TT$299 (regular TT$399, shown crossed out), proof strip, "How it works" and a sales band on the home page, WhatsApp-only purchase.

## Checkpoint 13 (superseded): professional identity
Includes checkpoint 12. Kit 0.6.0.
- No personal name or email address is displayed anywhere. Contact is a WhatsApp button labelled "Contact sales" (icon plus label; the number is only in the link). Facilitator is described by role.
- "Intellica family" is now "Systems".
- New I-beam logo mark and favicon; IBM Plex Sans and IBM Plex Mono (run `npm run fonts` again after unzipping).
- Square-cornered engineering style, mono labels, drafting grid, new footer.
- Home: animated system map of the five systems and a live panel in which the page re-hashes its own files against the public manifest.
- Motion (scroll reveal, page transitions) only where the browser supports it and never when reduced motion is requested.
- Remaining: security.txt and the booking mailto keep the company address in link targets (not displayed).

## Checkpoint 12 (superseded by 13): fonts
Includes checkpoint 11. Adds `npm run fonts`, which self-hosts IBM Plex Sans and Bricolage Grotesque (the typefaces of the original workshop page). Run it once in the Codespace; `node tools/install-fonts.mjs --remove` undoes it.

## Checkpoint 11 (superseded by 12): the online course, made visible and rebuilt
Includes checkpoint 10. The course is now promoted on Home, Training and the workshop page; the course page has a live slide preview, four syllabus modules, a price and access panel and the instructor; lessons have an outline sidebar and progress bar.

## Checkpoint 10 (superseded by 11): the online course
Includes checkpoint 9. Upload this one.
- **Online course** at `/training/arc42/course/`: the workshop as 19 lessons built from the slide deck (breaks and lunch dropped, appendices merged). Narrated slide player with transcript, read-aloud, optional recorded audio, keyboard control; free recall cards at the end; review at 1, 3, 7, 21 days; progress saved in the browser. Two lessons are a free preview; the rest are encrypted and open with an access code you issue (`npm run course:code -- "Buyer name"`).
- **Worked example** page `/training/arc42/example/`: Doubles-to-Go arc42 plus the working app (33 tests pass) as a download; source in `training-material/doubles-to-go/` (`npm run example:pack` rebuilds the zip).
- **MP4 export** per lesson: `npm run course:video -- <lesson-id>` (needs ffmpeg).
- Slides (7) are byte-identical to the deck already in checkpoint 8, so nothing changed there.
- Payments: Stripe is not available in Trinidad and Tobago, so access codes are issued by you after you receive payment. See ADR-29 and INSTALL.md.
- Also fixed: numbered steps on the register page rendered as broken columns.
- Browser check now has 37 checks. Kit unchanged (0.5.0).

## Checkpoint 9 (superseded by 10, may be uploaded): deploy from the Codespace
Includes checkpoint 8. Adds `npm run deploy -- <home|verichain|vericrypt|erp|omc|agentseed|all>`: syncs, runs the gate, creates the Pages project if needed and uploads with Wrangler (direct upload, no GitHub integration). `tools/deploy.config.json` holds the project names. INSTALL.md section 6 rewritten for this route.

## Checkpoint 8 (superseded by 9, may be uploaded): the Training section
Includes everything from checkpoints 6 and 7 (neither was confirmed uploaded). Upload this one.
- **`/training/`** index and **`/training/arc42/`** workshop (arc42 in a Day, **Saturday 30 January 2027**, UWI St Augustine), with **Student guide** and **Register** pages. Linked from a Training item in the home header, a Training block on the home page and the family menu on every product site.
- Built from one data file, `site/training/training.json`, into real HTML (works without JavaScript; Event structured data; in the sitemap). `training.js` only adds the deadline countdown, saved checklists, copy buttons and the email form.
- **Early bird TT$999 until 30 November 2026**; the text counts down and ends on its own. The standard price after that is not set: add `price.standard` in `training.json` when decided.
- **Your slides** (210) are a private presenter page at an unlisted address (`site/training/arc42/deck-<token>/`): not linked, not in the sitemap, manifest or robots, `noindex`, strict CSP, microphone and screen capture allowed for its recording feature. Converted without changing a slide. See INSTALL.md for real access control.
- Registration uses the company address (`dpramsajan@meeting-mind.com`), not a personal mailbox.
- Copy corrected from the original page: it said "fifteen short parts" but the programme has thirteen; the count is now computed from the data.
- Header no longer overflows at 320 px. Kit 0.5.0. Browser audit: 27 checks.

## Checkpoint 7 (superseded by 8, not uploaded): faster, safer headers, better docs
- **Found and fixed: `/verify-site/` had no Content-Security-Policy** (the rule list missed it). Added it, plus Cross-Origin-Opener-Policy and Strict-Transport-Security. A new test fetches every HTML page of every site and fails if any lacks a strict policy.
- **Found and fixed: layout shift.** Pages jumped while the shell built the header (CLS up to 0.18 on some pages; the limit is 0.1). The body and footer now wait for the shell; with a 2.5 s and 4 s time limit so nothing stays blank. Browser audit now enforces CLS, LCP and a 150 KB page budget on every product page.
- **Docs:** search across the whole manual (built in the browser), copy buttons on code blocks. Fixed a broken table cell in the VeriCrypt install page and added a test for stray backticks.
- Structured data (JSON-LD) on the home and every product overview.
- 320 px reflow test on every page (WCAG 1.4.10).
- Kit 0.4.0. Browser audit: 20 checks.

## Checkpoint 6 (superseded by 7, not uploaded): the site verifies itself; briefs for the product chats
- **Verify this site** on every site: published `site-manifest.json` (SHA-256 of every file plus a root hash) and a page that re-hashes everything in the visitor's browser. Footer link on every page.
- **What it does not do yet** section on every product Overview, from each repo's own limits.
- Briefs for the five product chats in `docs/briefs/` (paste-ready, with questions, demo concept, hand-back list and the shared contract) and `docs/CONTENT-SCHEMAS.md`.
- Checker guards: status and limits required; demo engine label must match the demo's code.
- `npm run scaffold` no longer overwrites finished work without `--force`.
- Browser audit: 15 checks.
- arc42: ADR-13 to 17, new scenarios and risks.
- Kit 0.3.0.

## Housekeeping after checkpoint 5
- Found in the uploaded repo: `node_modules/` was committed (no `.gitignore`) and a stale `INSTALL-CODESPACES.md` from checkpoint 1 was still present. Added `.gitignore`, removed the stale file. Verified a clean clone builds, scaffolds and passes the gate.

## Checkpoint 5 (uploaded): discoverable, accessible, useful when empty
- Every page now carries its own title, description, canonical link and link-preview tags, baked in without JavaScript (`tools/prerender.mjs`, run by `npm run sync`). Added sitemap.xml, robots.txt and security.txt per site.
- Download tab: when there are no binaries it shows build-from-source commands with a copy button instead of an empty table.
- Support tab: known problems (FAQ) per product, from each repo's own troubleshooting notes, plus a security note.
- Markdown headings are demoted so each page has exactly one h1.
- Print stylesheet.
- Home copy no longer implies the demos run in-browser today.
- Browser check now has 13 checks, including an accessibility audit (landmarks, heading order, names, WCAG AA contrast in light and dark on every page of every site) and a no-JavaScript metadata check.
- Kit 0.2.0.

## Checkpoint 4 (superseded by 5): five products, real content
- Fifth product: AgentSeed (ASL toolchain).
- Every product site now has a real user manual, installation guide, reference, architecture overview and Verify page, written from that product's own repository (README, arc42, install guides, CLAIMS.md). No more "Nothing is published here yet" in the docs.
- Status line per product, taken from each repo's own status.
- VeriCrypt content follows its `CLAIMS.md` registry (see CONTENT-TODO).
- Browser check now loads every tab and docs page of all five products (9 checks).
- Verified commands: vcv verify (VeriChain), vericrypt analyze/verify (VeriCrypt), seed check/run/build (AgentSeed), cargo test (Sovereign ERP), python scripts/demo.py (OMC).

## Checkpoint 3
Dependencies and browser check: `npm run setup`, `npm run verify`, devcontainer and CI.

## Checkpoint 2
Fresh `IntellicaWeb` repo layout; `npm run legacy -- <path>`.

## Checkpoint 1
Kit 0.1.0, home, template, scaffolds, tools, arc42 1 to 12. Docs sidebar fix.
