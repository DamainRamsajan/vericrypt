# Brief for the VeriChain chat

## Paste this into the chat

> This is a brief from the Intellica web chat, which structures the site at meeting-mind.com. VeriChain gets its own site at verichain.meeting-mind.com, built from its own repo (`intellica-ai-llc/verichain`). I read your repo and wrote a first version of its pages. Please read the brief below, answer the questions, confirm or correct the content, and build the demo. Everything you hand back goes in the product repo's `site/` folder, and it must pass the checks described under "How to test". Do not change anything in `site/kit/`.

## Where your product stands on the site now

- **Tagline:** A platform where autonomous AI agents run, act and transact, with every consequential action recorded in a log anyone can verify.
- **Status line:** Pre-release: specification frozen, core log and verifier built
- **Sources I used:** README, ROADMAP, HANDOFF, `docs/arc42/VERICHAIN_ARC42_v9.md`, `crates/vc-verify/README.md`, `crates/vcd/README.md`, `docs/PILOT_RUNBOOK.md`, `spec/vectors/`.
- **Subdomain:** `verichain.meeting-mind.com`

**Commands the site shows (taken from your docs or source):**

- `vcv verify pack.json --trust trust.json --policy policy.json --require-seals`
- `vcd demo ~/vc-demo`, `vcd serve ~/vc-demo`, `vcd verify ~/vc-demo`
- `cargo build --release -p vcd -p vc-verify`

**"What it does not do yet" (from your own limits):**

- There is no packaged download, hosted service or in-browser verifier yet; you build from source.
- Assurance level A3 (zero-knowledge proof) is research-gated and not available.
- The verifier's WebAssembly build has not been verified, and an independent adversarial review is still pending.
- Completeness depends on how an agent runs: the strongest guarantee (C3) applies to agents hosted on VeriChain; agents running beside it are observed, not guaranteed.

The site shows these on the Overview page. Correct anything that is out of date; add anything missing. Visitors trust a page more when it states its limits.

## Questions for you

1. Is the status line and tagline above right? Change them in your own words.
2. The site says the verifier has no in-browser build yet because `vc-verify`'s WebAssembly build is listed as unverified. Has that changed? If it has, say so and `demo.engine` can become `wasm`.
3. Which scenario should the demo show? Recommended: **S5, a third party audits an evidence pack**, with a "break it" step (edit one statement, or remove the latest entry) that fails a named check. S4 (split-view alarm) is the stronger second demo.
4. The arc42 lists SDKs for Rust, Python and TypeScript. The site does not mention them because it cannot confirm they exist today. Which exist? Give the install command for each that does.
5. Where will binaries live (GitHub Releases on `intellica-ai-llc/verichain`)? When there is a release, give the file URLs and SHA-256 values.
6. Is the pilot server (`docs/PILOT_RUNBOOK.md`) something visitors may use? If so, what is the address and what are the limits?

## The demo

- **Concept:** "Audit an agent without trusting its operator." Load a valid evidence pack, show the checks passing one by one (signature, inclusion, witness quorum, chain rules), then let the visitor tamper (edit a statement, hide the newest entry) and watch the specific check fail.
- **Inputs already in your repo:** `spec/vectors/` has `chain-valid.jsonl`, `chain-broken.jsonl`, `tampered.cose`, `test-key-other.pub.pem` and an `EXPECTED.json` of results.
- **Engine:** the fastest honest route is `trace`: run the real `vcv verify` over the valid and each tampered vector, record the JSON reports it prints, and replay them (`demo/trace.json`). When `vc-verify`'s WebAssembly build is verified, switch to `wasm` and let the visitor edit bytes live.
- **Show:** the real check names (`cose_signature`, `receipt_inclusion`, `checkpoint_witness_quorum`, `partial_coverage` and the rest) and the completeness and assurance levels reached.
- **Finish with:** `vcv verify pack.json --trust trust.json --policy policy.json --require-seals` and the explanation of why the visitor passes their own trust file.

## What to hand back

- `site.json` (confirmed status, tagline, highlights, limits, demo block)
- Docs: user manual (add the agent-connection walkthrough, scenario S2, when an SDK exists), installation, reference, architecture
- `verify/verify.md` (already drafted; confirm)
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

