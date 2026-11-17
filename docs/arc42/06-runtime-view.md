# 6. Runtime view

## 6.1 Page load

1. Browser requests an HTML page; Cloudflare applies `_headers`.
2. `theme-init.js` sets the theme before paint.
3. `shell.js` loads `site.json` and `family.json`, builds header, tabs and footer, then hydrates the page by `body[data-tab]`.
4. `html[data-ready="true"]` is set when hydration ends.

## 6.2 Demo

1. The Demo page loads `demo/demo.js` and calls `mount(root, ctx)` with `ctx = { base, theme, reducedMotion }`.
2. The shell draws the frame: engine label (wasm, trace or placeholder), sample input, "runs entirely in your browser", Reset, and the command panel.
3. Reset calls `unmount()` and mounts again.
4. WASM modules load only here; `/demo/*` is the only path with `wasm-unsafe-eval`.

## 6.3 Docs

1. `docs.json` builds the sidebar.
2. The selected file is the `?d=` query parameter, defaulting to the first item.
3. Markdown is fetched, rendered with escaping, relative links are rewritten, and a table of contents is added when there are more than three headings.

## 6.4 Download

1. `downloads.json` renders a table.
2. Each row shows a SHA-256 with a copy button and a short "how to check" block for macOS, Linux and Windows.

## 6.5 Release of a kit change

Edit `site/kit/` here, run `npm run gate`, commit, then copy to product repos (via `npm run sync` or the product repo's copy of the kit) and commit there.

## 6.6 Verify this site

1. The page loads `/site-manifest.json` (written at sync time by `tools/manifest.mjs`).
2. The browser fetches every listed file with caching off and hashes it with Web Crypto.
3. It recomputes the root hash from the listed paths and hashes, compares each file, and reports matches or lists the files that differ.
4. What it proves: the files the visitor received are the files in the manifest (catches corruption, partial updates, stale cache, an edge that rewrites pages). What it does not prove: the manifest is served by the same origin, so a server that changes both is not exposed; the visitor compares the root hash with the one in the public repository.
