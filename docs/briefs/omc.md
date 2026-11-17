# Brief for the OMC (Operating-Margin Controller) chat

## Paste this into the chat

> This is a brief from the Intellica web chat, which structures the site at meeting-mind.com. OMC (Operating-Margin Controller) gets its own site at omc.meeting-mind.com, built from its own repo (`DamainRamsajan/Operating-Margin-Controller-`). I read your repo and wrote a first version of its pages. Please read the brief below, answer the questions, confirm or correct the content, and build the demo. Everything you hand back goes in the product repo's `site/` folder, and it must pass the checks described under "How to test". Do not change anything in `site/kit/`.

## Where your product stands on the site now

- **Tagline:** A closed-loop system that measures and proves the financial value of work shared by people, AI models and agents.
- **Status line:** Design phase: walking skeleton on synthetic data
- **Sources I used:** README, `docs/arc42/` (v0.2), `docs/dev/MILESTONE-M0.md`, `docs/dev/LIMITATIONS.md`, `docs/dev/CODESPACES.md`, `vectors/`, `omc_verify/`.
- **Subdomain:** `omc.meeting-mind.com`

**Commands the site shows (taken from your docs or source):**

- `python scripts/demo.py`
- `pip install -e ".[dev,spike]"`
- `pytest -q`

**"What it does not do yet" (from your own limits):**

- Every result so far comes from synthetic data; nothing has been validated on real tickets.
- The statistical bounds assume cases come from a fixed distribution over the audit window; drift can invalidate a certificate.
- The bounds are valid but conservative, so close calls are held rather than promoted.
- The statistics are the author's own implementation and need independent statistical review.

The site shows these on the Overview page. Correct anything that is out of date; add anything missing. Visitors trust a page more when it states its limits.

## Questions for you

1. Confirm the status line and tagline. The site says "design phase, walking skeleton on synthetic data" because that is what `MILESTONE-M0.md` and `LIMITATIONS.md` say.
2. The arc42 says measurement runs in the customer's tenant. The site does not promise that today because M0 has no tenant deployment. Say when it can.
3. Which of the four entry audits should the site lead with? Only the first (model migration qualification and the CI gate) is built.
4. Is there a name for the product on the public site: "OMC", "Operating-Margin Controller", or something else? (The earlier "OFC" was a typo.)
5. Is `omc_verify` (depends only on `cbor2` and `cryptography`) ready for visitors to run? The Verify page says so.

## The demo

- **Concept:** "Watch the proof form." Paired cases stream in and an anytime-valid confidence bound tightens until it crosses a threshold: promoted (exit 0), rejected (exit 20) or held (exit 10).
- **Engine:** the engine is Python, so a full in-browser build means shipping a Python runtime, which is heavy. Use **`trace`**: run `scripts/demo.py` for the benign, regression and unclear scenarios with fixed seeds and record the bound after each batch (`demo/trace.json`). The visitor scrubs through the real run. Do not re-implement the statistics in JavaScript and call it the engine.
- **Show:** the bound over time, the threshold, the verdict, the exit code, and the signed statement's hash chain being verified at the end.
- **Break-it step:** let the visitor flip one recorded case or the signature and show the verifier reject it (use `vectors/`).
- **Finish with:** `python scripts/demo.py`.

## What to hand back

- `site.json` (confirmed)
- Docs (drafted from M0 notes; confirm)
- `verify/verify.md` (drafted; confirm)
- `demo/trace.json` and `demo.js`
- Anything that should lead the Overview beyond the model-migration audit


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

