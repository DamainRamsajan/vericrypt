# 5. Building block view

## 5.1 Level 1: repositories

| Repo | Owns |
| --- | --- |
| `IntellicaWeb` (this system) | Kit, home, template, tools, arc42, legacy pages |
| `verichain`, `vericrypt`, `Sovereign-ERP`, `Operating-Margin-Controller-`, `agentseed` | Product site content, demo engine, binaries, docs |

## 5.2 Level 2: inside a site

```
site/
  index.html        Overview (or home)
  site.json         name, accent, tagline, tabs, audience, highlights
  demo/             index.html + demo.js (exports mount)
  download/         index.html + downloads.json
  docs/             index.html + docs.json + markdown (use/ install/ reference/ architecture/)
  verify/           index.html + verify.md
  support/          index.html
  kit/              shared, synced, do not edit
  family.json       product list, synced from the home
  _headers  404.html
```

## 5.3 The kit

| File | Responsibility |
| --- | --- |
| `tokens.css` | Colors, spacing, type; dark and light themes; per-product `--accent` |
| `shell.css` | Layout and components |
| `shell.js` | Header, family menu, tab bar, footer, theme button, hydrates each page by `body[data-tab]` |
| `dom.js` | `el()`, safe hrefs, JSON and text loading |
| `markdown.js` | Safe renderer for Docs and Verify |
| `placeholder-demo.js` | Labeled hash-chain demo used until a real engine exists |
| `theme-init.js` | Applies saved theme before paint |
| `VERSION`, `MANIFEST.json` | Kit version and file hashes for drift detection |

## 5.4 Tools

| Tool | Purpose |
| --- | --- |
| `serve.mjs` | Local server that applies `_headers` |
| `check.mjs` | Gate: no inline script/style, no third-party loads, no missing files, valid JSON, no kit drift |
| `sync-kit.mjs` | Regenerates the manifest and copies kit and `family.json` to targets |
| `new-product.mjs` | Scaffolds a product site from the template and a prefill |
| `import-legacy.mjs` | Copies old pages into `site/legacy/` |
| `test-markdown.mjs` | Tests for the renderer |
| `prerender.mjs` | Bakes title, description, canonical and link-preview tags and a no-JavaScript summary into every page; writes sitemap, robots and security.txt |
| `manifest.mjs` | Writes `site-manifest.json`: SHA-256 of every served file plus a root hash |
| `browser-test.mjs` | Real-browser audit (Playwright): tabs, demo, docs, accessibility, metadata, verify-this-site |

## 5.5 Content schemas

Full field list: `docs/CONTENT-SCHEMAS.md`. Per-product hand-back instructions: `docs/briefs/`.


- **site.json:** `id`, `name`, `accent`, `tagline`, `tabs`, `status`, `audience[]`, `highlights[]`, `verifyIt`, `demo{title,intro,sampleInput,command}`.
- **downloads.json:** items with `name`, `platform`, `version`, `url`, `sha256`, `size`.
- **docs.json:** `groups[]` of `{title, items[{title, md}]}`.
- **family.json:** home URL, contact, `products[]`, `roles[]`, `standards[]`, `sectors[]`.

## 5.x Training and the online course
| Block | Responsibility | Where |
| --- | --- | --- |
| Training generator | Pages for the index, workshop, guide, register and worked example from `training.json` | `tools/build-training.mjs` |
| Course builder | Reads the slide deck, makes one lesson per Part, writes the course pages, the scoped slide CSS and the plain or encrypted lesson files | `tools/build-course.mjs`, `site/training/course-layout.css` |
| Course player | Loads a lesson, decrypts it with the buyer's code, runs the slides, transcript, read-aloud, recall cards and review; stores progress in the browser | `site/training/arc42/course/course.js` |
| Access-code tool | Makes a code for a buyer from the course key | `tools/course-code.mjs` |
| Video exporter | Renders a lesson to MP4 with ffmpeg and Chromium | `tools/export-lesson-video.mjs` |
| Worked example | Doubles-to-Go arc42, app and tests; packed as a download | `training-material/doubles-to-go/`, `tools/pack-example.py` |
