# 2. Architecture constraints

## 2.1 Technical

- **One domain.** Only `meeting-mind.com` is owned. Home on the apex; products on `verichain.`, `vericrypt.`, `erp.`, `omc.` subdomains.
- **Cloudflare Pages**, one project per repo. A custom domain can attach to only one Pages project at a time.
- **Static output.** Plain HTML, CSS and ES modules. No framework, no build step for content. Output directory `site`.
- **No third-party loads** on the new pages (strict CSP). Fonts are system fonts.
- **Email DNS records** (MX, TXT) for the company email address must be preserved.
- **Zero-cost tooling.** Node 20+ for local tools only; Playwright is a dev dependency for the browser check; nothing is needed at deploy time.

## 2.2 Organisational

- Each product lives in its own repo. Product chats own the product's engine, demo, binaries and docs content.
- This repo owns the shell contract, the kit, the home page, the template and the tools.
- The old MeetingMind repo and system are retired.

## 2.3 Conventions

- Tabs and paths are fixed (see chapter 5).
- Product facts come from each product's own arc42. This system does not restate or audit them.
- Content gaps are shown honestly as "Nothing is published here yet", never invented.
