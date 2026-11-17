# 8. Cross-cutting concepts

## 8.1 Security

- CSP: `default-src 'self'`; no inline scripts or styles; `wasm-unsafe-eval` only on `/demo/*`.
- `/legacy/*` has a looser policy (inline, Google Fonts, Calendly) because the old pages need it.
- JSON values are inserted as text or attributes, never as HTML.
- Markdown: all HTML escaped, `javascript:`/`data:` links dropped, external images dropped.
- `safeHref` allows http(s), mailto, relative paths and query-only links.

## 8.2 Theming

Dark neutral base from the Intellica navy palette, plus a light theme. The theme button cycles auto, light, dark and saves the choice. Each product sets `--accent`; `--accent-ink` is derived with `color-mix`, and `--on-accent` is chosen by luminance in JS so text on accent stays readable.

## 8.3 Accessibility

WCAG 2.2 AA. Skip link first in tab order, visible focus, 44px targets, `aria-current` on active tabs, reduced motion respected, status never conveyed by color alone.

## 8.4 Performance

No framework, system fonts, wasm loaded only on the demo, `/kit/*` cached.

## 8.5 Honesty of content

Missing content shows "Nothing is published here yet." The placeholder demo is labeled as a placeholder, not as any product's engine. Product claims come from product arc42s.

## 8.6 Errors

Failed loads show an in-page message with the reason; the 404 page links back to the product home.

## 8.7 Testing

`npm run gate` runs the Markdown tests, syncs the kit and runs the checker. `npm run verify` adds `tools/browser-test.mjs`, a Playwright check of home, demo, docs, tabs, mobile overflow, theme and keyboard focus. Playwright is the only dependency (dev only); the site itself has none.

## 8.8 Self-verification of the site

The site holds itself to its own premise. Every site publishes a manifest of file hashes and a page that re-checks it in the visitor's browser. The footer links to it on every page. Cloudflare features that rewrite pages (Email Address Obfuscation, Rocket Loader, and Auto Minify where it still exists) must be off, or the check fails and scripts the Content-Security-Policy blocks get injected.

## 8.9 Stating limits

Each product's Overview carries "What it does not do yet", taken from the product's own limits. The checker warns when a product has no status or no limits, and fails when the demo's engine label does not match the demo's code (a placeholder must say it is a placeholder).

## 8.10 Progressive enhancement and discovery

Every page carries its own title, description, canonical URL and link-preview tags without JavaScript, plus a plain-text summary in `<noscript>`. Each site has a sitemap, robots.txt and security.txt.

## 8.11 Layout stability and performance budget

The shell builds the header in the browser, which would move the page. Until the header exists the body is not painted (`html[data-shell]`), and the footer waits for hydration (`html[data-ready]`); both have a time-limited fallback so a failure never leaves a blank page. The browser audit enforces CLS < 0.1, LCP < 2.5 s and < 150 KB per page.

## 8.12 Docs search

Built in the browser from the same Markdown files the docs show. Nothing is sent anywhere and there is no index file to keep in sync.

## 8.10 Online course: learning design and access
- **Learning design.** Short learner-paced lessons (segmenting), a finished example before practice with support fading (as in the live day), free recall at the end of each lesson, and spaced review at 1, 3, 7 and 21 days. Every slide has a transcript. The site states that spacing effects vary: a 2024 study across nine STEM courses found a small average gain of about two percentage points, not in every course (Bego et al., International Journal of STEM Education).
- **Access.** Free lessons are plain JSON. Paid lessons are AES-256-GCM files (additional authenticated data = lesson id), so a wrong or tampered file fails to open. The key is not in the repository or the site: it is in `.course-key` or `COURSE_KEY`. The access code is `base64url({v:1, b:name, k:key})`, made by `npm run course:code`, and is kept in the buyer's browser only.
- **Strict CSP is kept.** No inline script or style; slide content is inserted from our own build output and, for paid lessons, only after authenticated decryption.
- **Limits, stated plainly.** This protects against casual copying, not a determined buyer who saves the lessons, and not the unlisted slide deck (R17).
