# Brief for the Sovereign ERP chat

## Paste this into the chat

> This is a brief from the Intellica web chat, which structures the site at meeting-mind.com. Sovereign ERP gets its own site at erp.meeting-mind.com, built from its own repo (`intellica-ai-llc/Sovereign-ERP`). I read your repo and wrote a first version of its pages. Please read the brief below, answer the questions, confirm or correct the content, and build the demo. Everything you hand back goes in the product repo's `site/` folder, and it must pass the checks described under "How to test". Do not change anything in `site/kit/`.

## Where your product stands on the site now

- **Tagline:** A ledger and period-close kernel whose rules are checked by model checking and formal specifications.
- **Status line:** Walking skeleton: period controller and ledger port, model-checked
- **Sources I used:** The code drop `sovereign-erp-code-v2.zip` (README, GETTING-STARTED, crate docs, `formal/`). **No arc42 was in it**, and the repository `intellica-ai-llc/Sovereign-ERP` was empty when last looked at.
- **Subdomain:** `erp.meeting-mind.com`

**Commands the site shows (taken from your docs or source):**

- `cargo test --workspace` (in `code/`)
- `cargo test -p conformance -- --nocapture`

**"What it does not do yet" (from your own limits):**

- It is a walking skeleton: the Period Controller and the Ledger Port. There is no production code and no ERP to run yet.
- The model checks are bounded and use small constants; multi-currency, consolidation, sub-ledger detail, time and concurrency are not modelled.
- The TLA+ and Lean files have not been re-run outside the author's environment.

The site shows these on the Overview page. Correct anything that is out of date; add anything missing. Visitors trust a page more when it states its limits.

## Questions for you

1. **Please share `arc42-v*.md`** from the repo root. The page is thinner than the others because the code drop had none.
2. The GitHub repository was empty. Push the code so the "Source repository", clone commands and Verify steps on the site work.
3. Who is it for? `audience` is empty on purpose: no audience statement exists in the material I have. Give two to four roles in your words.
4. Confirm the tagline and status. Is "walking skeleton" the right public description?
5. The TLA+ and Lean files were not re-run outside your environment, and the site says so. If you have run them since, give the TLC and Lean results so the Verify page can state them.

## The demo

- **Concept:** "Try to break the books." A Period Controller explorer: buttons for close, reopen and post-adjustment on a few periods; the controller accepts or refuses each move and a panel shows the invariant that refused it.
- **Engine:** the `period-controller` crate is a pure, deterministic state machine with no I/O, which makes it the best candidate in the family for a real **`wasm`** demo: compile it to `wasm32-unknown-unknown` and call `decide` and `evolve` directly. That is the strongest possible demo of "rules checked, not just tested".
- **Show:** the state, the command, the decision (accepted or refused) and which invariant fired.
- **Seeded defect toggle:** your mutation tests catch six seeded defects with shortest counterexamples. Let the visitor switch a defect on and see the counterexample trace the suite found.
- **Finish with:** `cargo test --workspace` and the state counts the suite prints.

## What to hand back

- `arc42-v*.md` (so I can ground the page)
- `site.json` (audience, confirmed status and tagline)
- Docs and `verify/verify.md` (drafted from the code README; confirm)
- `demo/` files
- Push the code to the repo


---

# Shared contract (applies to every product)

## What the site is

Each product has its own static site in its own repo, served from the `site/` folder on its own subdomain. It has six tabs: Overview, Demo, Download, Docs, Verify, Support. The Intellica web chat owns the shell (header, tabs, theme, docs viewer, download table, support page). **You own the content and the demo.** Do not edit anything in `site/kit/`; it is copied in and checked for hand edits.

## What you hand back

| File (under `site/`) | What |
| --- | --- |
| `site.json` | Name, tagline, accent, status, audience, highlights, limits, verifyIt, demo settings, build-from-source steps, FAQ |
| `docs/*.md` and `docs/docs.json` | User manual, installation manual, reference, architecture overview (add more pages freely) |
| `verify/verify.md` | How a stranger checks a result on their own machine |
| `download/downloads.json` | Binaries with SHA-256, once releases exist |
| `demo/demo.js` plus its files | The demo (see below) |

Field-by-field schemas are in `docs/CONTENT-SCHEMAS.md`.

## Rules for content

1. **Say only what is true today, in the product's own words.** The status line says where the product stands. `limits` lists what it does not do yet. Research on two-sided messages (Eisend, 2006 meta-analysis; Int. J. Consumer Studies, 2022) finds that stating limits raises credibility; our whole premise is that visitors should not have to take our word for anything.
2. **Commands must be real.** Give a command only if it appears in the repo's docs or source and has been run.
3. **Respect your claims registry.** If the product has one (VeriCrypt's `CLAIMS.md`), the site claims only what it allows.
4. **Verification steps are for a stranger.** They must work without our help, on a machine we do not control.

## The demo contract

```js
// demo/demo.js
import { el } from '/kit/dom.js';

export async function mount(root, ctx) {
  // ctx = { base, theme, reducedMotion }
  //   base          URL of the demo folder (load engine.wasm, samples and demo.css from here)
  //   theme         'light' or 'dark'
  //   reducedMotion true when the visitor asked for less motion: do not animate
  const css = el('link', { rel: 'stylesheet', href: new URL('demo.css', ctx.base).href });
  document.head.append(css);

  const { instance } = await WebAssembly.instantiateStreaming(
    fetch(new URL('engine.wasm', ctx.base)), {});          // served as application/wasm

  root.replaceChildren(/* build the UI with el(); use textContent, never innerHTML with data */);

  return { unmount() { css.remove(); /* remove listeners, terminate workers */ } };
}
```

The shell draws the frame around your demo: the engine label, the sample-input line, "runs entirely in your browser", the **Reset** button (calls `unmount()` then `mount()` again) and the **Run it yourself** command panel from `site.json`. You draw only what is inside the frame.

### Technical limits

- No network requests except to files in the same site. No third-party scripts, fonts or images.
- No inline `<script>`, `<style>` or `style=""` attributes (strict Content-Security-Policy). Put styles in `demo.css` and use classes.
- WebAssembly is allowed only under `/demo/`. Heavy work belongs in a Worker (`new Worker(new URL('worker.js', ctx.base))`) so the page stays responsive (target: interaction response under 200 ms).
- Keep the first load small. Show the sample and the main button immediately; load the engine on first interaction or when the page is idle. Aim for under 2 MB transferred before the first interaction.
- Colours: use the kit's CSS variables (`--bg`, `--card`, `--text`, `--muted`, `--line`, `--accent`, `--good`, `--bad`, `--warn`, `--font-mono`, spacing `--s1` to `--s8`) so the light and dark themes both work.

### Design rules for a demo that convinces

1. **One demo, one claim.** Pick the single strongest claim from your arc42 and show only that.
2. **Valid, break, caught.** Start from a sample that passes, let the visitor break it (edit a byte, drop an entry, change a number), and show the check fail and say exactly why.
3. **No empty state.** A sample is loaded on arrival; the first action is one click.
4. **Show the evidence, not a score.** Display the real output of the engine (hashes, proofs, bounds, error messages), not an invented percentage.
5. **Finish on the real thing.** End with the command from `site.json` (`demo.command`) so the visitor can reproduce the result on their own machine.
6. **Keyboard and screen reader.** Every control reachable by keyboard, results announced in an element with `role="status"`, no meaning carried by colour alone, 44 px targets, and no motion when `reducedMotion` is true.
7. **Be honest about the engine.** Set `demo.engine` to `wasm` only if the real engine runs in the page, `trace` if the page replays output recorded from the real engine, and leave `placeholder` until then.

## How to test before handing back

Your product repo holds only the `site/` folder, so the checker lives in the IntellicaWeb repo. Two ways:

1. **Hand it to the Intellica web chat** (paste the files or give the commit). I run the checks and send back anything that fails.
2. **Check it yourself** in a Codespace of the IntellicaWeb repo: copy your files over `products/<product-id>/site/` (keep `kit/`), then run `npm run verify`.

`npm run verify` runs the checker (schemas, CSP rules, missing files, engine-label honesty) and a real-browser audit: every tab and docs page loads, accessibility (landmarks, headings, names, contrast in light and dark), page metadata, and a "verify this site" hash check. A hand-back is ready when it passes.

Note: do not run `npm run scaffold` over finished work; it refuses to overwrite unless you add `--force`.

## How to deliver

Copy your `site/` contents into your product repo's `site/` folder, commit, and tell the Intellica web chat the commit. Cloudflare Pages settings for the repo: framework preset None, build command empty, output directory `site`. Also switch **off** Email Address Obfuscation, Rocket Loader and Auto Minify (if your dashboard still shows it) for the zone (Scrape Shield and Speed settings): they rewrite pages, inject scripts the Content-Security-Policy blocks, and make the site's "Verify this site" check fail. Then add the subdomain in that Pages project's Custom domains.

