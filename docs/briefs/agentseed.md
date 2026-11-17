# Brief for the AgentSeed chat

## Paste this into the chat

> This is a brief from the Intellica web chat, which structures the site at meeting-mind.com. AgentSeed gets its own site at agentseed.meeting-mind.com, built from its own repo (`agentseedlanguage-cpu/agentseed`). I read your repo and wrote a first version of its pages. Please read the brief below, answer the questions, confirm or correct the content, and build the demo. Everything you hand back goes in the product repo's `site/` folder, and it must pass the checks described under "How to test". Do not change anything in `site/kit/`.

## Where your product stands on the site now

- **Tagline:** A programming language for autonomous agents, where safety rules are checked at compile time.
- **Status line:** v0.1.0
- **Sources I used:** The landing page, README, `ASL_V_0_2_0_ARC42.md`, `seedc-cli/src/main.rs`, `docs/book/ASL_MANUAL.md`, `docs/playground/index.html`, `examples/`.
- **Subdomain:** `agentseed.meeting-mind.com`

**Commands the site shows (taken from your docs or source):**

- `seed check hello.seed`
- `seed run hello.seed`
- `seed build hello.seed`
- `cargo build --release -p seedc-cli`

**"What it does not do yet" (from your own limits):**

- Its guarantees apply to programs written in ASL, not to existing Python or JavaScript code.
- It is version 0.1.0, and the in-browser compiler is not built yet.
- Packaged installs are still being confirmed; today you build from source.

The site shows these on the Overview page. Correct anything that is out of date; add anything missing. Visitors trust a page more when it states its limits.

## Questions for you

1. **Install name.** The landing page and manual say `npm install -g @agentseed/cli`; `cli/npm/package.json` is named `@asl-lang/cli`; the manual also cites an install script at `agentseed.org`. Which is real today? Until you say, the site gives only the build-from-source command.
2. Is there a GitHub release with binaries? If so give the URLs and SHA-256 values.
3. The repo is under the `agentseedlanguage-cpu` account. Cloudflare Pages must be able to read it. Move or fork it under your account, or grant access.
4. Your own playground says in-browser compilation is "coming soon". Is a WebAssembly build of `seedc` and `seedvm` planned? That decides whether the demo can be `wasm`.
5. Accent colour: your pages use a green close to VeriChain's, so the family site proposes coral `#FF7A59`. Accept or propose another.
6. The VeriChain arc42 names ASL developers as a stakeholder and VeriCrypt mentions an ASL VM. Say in one or two sentences how AgentSeed relates to the other products, and the site will say exactly that and no more.
7. The landing page links a placeholder PDF for the paper. What is the final link?

## The demo

- **Concept:** "The compiler says no." Show a short agent program, let the visitor change one line so it breaks a rule (use a value from an uncertain source without handling it, call an effect without discharging it, exceed a capability), and show the compiler's exact error.
- **Engine:** `seedc` is Rust, so compiling it to `wasm32` is the natural route and would make this a true `wasm` demo (the `check` command needs only parse and semantic analysis, not the VM). Until then use `trace`: record the real `seed check` output for three prepared programs (accepted, rejected for an effect, rejected for taint) from `examples/`.
- **Show:** the source, the diagnostic with its code and line, and the same program passing after the fix.
- **Finish with:** `seed check hello.seed`.

## What to hand back

- `site.json` (accent, relationship to the other products, confirmed status)
- Install command and release URLs
- Docs (drafted; confirm install)
- `demo/` files
- Final paper link


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

