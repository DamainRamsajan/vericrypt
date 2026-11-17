# 9. Architecture decisions

| ID | Decision | Why | Consequence |
| --- | --- | --- | --- |
| ADR-01 | One domain, product subdomains | Only one domain owned | Each product needs its own Pages project |
| ADR-02 | One repo and Pages project per product | Independent releases; a domain attaches to one project | Shared code must be copied |
| ADR-03 | Static HTML per tab, no framework | Speed, safety, no build | Shell code is plain modules |
| ADR-04 | Shared kit copied with manifest | Works offline per repo; drift is detectable | Needs a sync step |
| ADR-05 | Strict CSP, no third parties | Verifiability story must be true of the site itself | No analytics or CDN fonts |
| ADR-06 | Content in JSON and Markdown | Product chats edit content, not code | Schemas must stay stable |
| ADR-07 | Demo mount contract | Product chats build engines independently | Shell owns the frame and labels |
| ADR-08 | Labeled placeholder demo | No empty or misleading pages | Must be replaced per product |
| ADR-09 | Legacy pages under `/legacy/` | Keep sector pages and decks alive | Looser CSP on that path only |
| ADR-10 | Docs follow Diátaxis (Use, Install, Reference, Architecture) | Readers arrive with different needs | Four groups per product |
| ADR-11 | Six fixed tabs | Predictable across products | Products hide tabs they lack |
| ADR-12 | System fonts | No third-party requests | Less brand typography |
| ADR-13 | The site verifies itself: published hash manifest plus an in-browser check | Premise of the company is verifiability; the website should meet it first | Needs a Cloudflare setting audit; same-origin limitation is stated on the page |
| ADR-14 | Every product states what it does not do yet | Credibility; matches the products' own claims registries | Chats must keep limits current |
| ADR-15 | Demo engine label is checked against the demo's code | A placeholder must never be presented as a product's engine | Chats must set `demo.engine` when they replace the placeholder |
| ADR-16 | Per-product briefs, self-contained and pasteable | Product chats cannot read this repo | Briefs can drift from the repo; always use the newest checkpoint |
| ADR-17 | Metadata baked in at sync time, not by the browser | Crawlers and link previews do not run the shell | Run `npm run sync` after content changes (the gate does) |
| ADR-18 | Hide body until the shell header exists, rather than reserving fixed header heights | Header height varies with status text and screen width; any fixed guess still shifts | Pages are blank for the first ~100 ms; fallback timers prevent a stuck blank page |
| ADR-19 | Docs search is built in the browser | No build step, no stale index | Fetches every page on first use (small manuals only) |
| ADR-20 | Training lives on the apex at `/training/`, not a subdomain or its own repo | It is an Intellica offering, not a product; one home for the company's teaching | Shares the home project's deploy |
| ADR-21 | Training pages are generated from `training.json` into real HTML (`tools/build-training.mjs`) | Content works without JavaScript, crawlers read it, one source for pages, structured data and home block | Edit the data, never the HTML; run `npm run sync` |
| ADR-22 | Training pages are an editorial design (system serif display, ruled sections, one warm accent scoped to the training pages) | Distinct from product-site and competitor templates without downloading fonts or adding script | Serif face differs per operating system |
| ADR-23 | The presenter slides are an unlisted page (`deck-<token>/`), excluded from nav, sitemap, manifest and robots, `noindex`, strict CSP | The facilitator controls who gets the link | Unlisted is not secret; real access control needs Cloudflare Access (INSTALL.md) |
| ADR-24 | Registration is a composed email to the company address; no backend, no stored data | No personal data held, nothing to secure | Manual confirmation; move to a form endpoint when volume needs it |
| ADR-25 | Deadlines are real and self-expiring (`data-until`), no seat counts or testimonials unless true | Honest urgency; the site's premise is verifiability | Needs real numbers and real quotes supplied by the owner |
| ADR-26 | The online course is generated from the same slide deck as the live workshop (`tools/build-course.mjs`), one lesson per deck Part | One source: the live day and the course cannot drift apart | Lesson length follows the deck; an awkward Part makes an awkward lesson |
| ADR-27 | Lessons are narrated slide players (HTML, text transcript, browser read-aloud, optional recorded audio), not hosted video; an MP4 can be exported per lesson | Zero budget: Cloudflare Stream has no free tier, static files are free and unlimited, and slides plus transcript are accessible and tiny | Browser voices vary in quality; an MP4 needs ffmpeg and a free host such as unlisted YouTube |
| ADR-28 | Paid lessons are AES-256-GCM files on static hosting; the buyer's access code carries the key | No server, no database, no monthly cost, works offline of any payment provider | A shared code shares access: mitigated by the buyer's name on every lesson; revoking means re-encrypting and re-issuing codes. It is not DRM |
| ADR-29 | Payment is arranged directly with the buyer (email or WhatsApp, then bank, PayPal or a local acquirer); the code is issued by the owner | Stripe does not operate in Trinidad and Tobago; a payments layer cannot be tested or trusted from here | Manual, about one working day per sale. An automatic route (a Pages Function plus a PayPal webhook) is documented, not built |
| ADR-30 | Recall and spacing are built in: free-recall cards end each lesson, reviews return at 1, 3, 7 and 21 days, progress lives in the browser only | Retrieval practice and spacing are among the best-supported learning techniques | Evidence on spacing varies by course (a 2024 nine-course study found about two points on average); claims on the page say so. Progress does not follow the learner between devices |
| ADR-31 | The home page proves itself: it recomputes the manifest root and re-hashes six files in the visitor's browser | An engineering-evidence company should show its own method, not describe it | The manifest comes from the same server, so this does not defeat a server that changes both; the page links to the full check and the public repository |
| ADR-32 | Contact is WhatsApp only, shown as an icon and "Contact sales" with no number or personal name on the page | Professional presentation and less scraping of a personal mailbox | The number remains in the link target, so it is not secret |
| ADR-33 | Motion uses CSS scroll-driven animations and cross-document view transitions as progressive enhancement | Supported in current Chromium, Safari and (scroll-driven) Firefox; no script, no third-party code | Unsupported browsers and reduced-motion users see a static page |

