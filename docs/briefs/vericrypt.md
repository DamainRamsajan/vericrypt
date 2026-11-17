# Brief for the VeriCrypt chat

## Paste this into the chat

> This is a brief from the Intellica web chat, which structures the site at meeting-mind.com. VeriCrypt gets its own site at vericrypt.meeting-mind.com, built from its own repo (`DamainRamsajan/vericrypt`). I read your repo and wrote a first version of its pages. Please read the brief below, answer the questions, confirm or correct the content, and build the demo. Everything you hand back goes in the product repo's `site/` folder, and it must pass the checks described under "How to test". Do not change anything in `site/kit/`.

## Where your product stands on the site now

- **Tagline:** An engine that inventories the cryptography in an estate, flags quantum-vulnerable assets, and produces signed reports anyone can verify offline.
- **Status line:** v2 evidence engine built and tested; later versions in design
- **Sources I used:** README, `docs/INSTALL_V2.md`, `docs/PROJECT_STATUS.md`, `CLAIMS.md`, `docs/sample/` (a signed report, CBOM and public key), `docs/architecture/ARCHITECTURE_V4_arc42.md` and V5.
- **Subdomain:** `vericrypt.meeting-mind.com`

**Commands the site shows (taken from your docs or source):**

- `vericrypt keygen --out my-key.json`
- `vericrypt analyze -i <dir> --out out --sign-key my-key.json`
- `vericrypt verify report.json --pubkey my-key.pub.json`
- `vericrypt diff old.json new.json --fail-on high`

**"What it does not do yet" (from your own limits):**

- It produces an observation-based inventory and findings. It is not a compliance certification.
- Not implemented yet: formal proofs, hardware attestation, revocation checking, certificate chain building, TLS version and cipher enumeration, SSH, code scanning, hybrid signatures and trusted timestamps.
- Live TLS probing reads the leaf certificate only.
- Only Ubuntu 24.04 and WSL2 have been tested; macOS, native Windows and ARM are untested.

The site shows these on the Overview page. Correct anything that is out of date; add anything missing. Visitors trust a page more when it states its limits.

## Questions for you

1. **Decision needed.** Your README and arc42 describe `.pqc` reports, Lean 4 proofs, hardware (TEE) attestation, a regulator portal and an air-gapped single binary. Your own `CLAIMS.md` says several of these are not implemented and must not be claimed. The site follows `CLAIMS.md` (reports are `report.json`, commands are `analyze` and `verify`). Is that right? Which wording should the site use as each capability ships?
2. Confirm the status line and tagline above.
3. Is there a release plan? Linux x86_64 only, or more? Give file URLs and SHA-256 values when a release exists.
4. Which of the v4 or v5 capabilities, if any, are ready to appear on the site today? The site shows only what `CLAIMS.md` lists as covered.
5. Is the CBOM validation script (`scripts/validate_cbom.py`) something visitors should be told to run? It is mentioned on the Verify page.

## The demo

- **Concept:** "Check a report you did not make." Load the sample signed report, show `VERIFIED` with the findings and the Merkle root, let the visitor edit a finding or swap the public key, and show verification fail.
- **Inputs already in your repo:** `docs/sample/report.json`, `docs/sample/test-signing-key.pub.json`, `docs/sample/cbom.json`, `docs/sample/report.html`.
- **Engine:** `trace` is available today: record the real `vericrypt verify` output for (1) the untouched sample, (2) an edited finding, (3) the wrong key, (4) the signature removed. For `wasm`, the SLH-DSA verifier and the Merkle recomputation must both run in the page; a page that only recomputes the Merkle root must not claim signature verification (set the label to `trace` and say so in the summary).
- **Show:** the exit code (0, 1), `RESULT: VERIFIED` or the failure reason, and what was pinned.
- **Second demo, if you want one:** `diff` between two scans with `--fail-on high`, showing the exit code 3 a pipeline would act on.
- **Finish with:** `vericrypt verify report.json --pubkey my-key.pub.json`.

## What to hand back

- `site.json` (confirmed, in line with `CLAIMS.md`)
- Docs (drafted from `INSTALL_V2.md`; confirm and extend)
- `verify/verify.md` (drafted; confirm)
- `demo/` files
- `downloads.json` when releases exist


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

