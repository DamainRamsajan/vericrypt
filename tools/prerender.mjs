#!/usr/bin/env node
// Bakes the facts that must exist without JavaScript into each page: <title>, description,
// canonical URL, social tags, a <noscript> summary, plus sitemap.xml, robots.txt and security.txt.
// Search engines and link previews do not run the shell, so this is what they see.
// Safe to run repeatedly. Usage: node tools/prerender.mjs [siteDir ...]  (default: site + products/*/site)

import fs from 'node:fs';
import path from 'node:path';

const args = process.argv.slice(2);
let dirs = args;
if (!dirs.length) {
  dirs = ['site'];
  if (fs.existsSync('products')) for (const p of fs.readdirSync('products').sort()) if (fs.existsSync(path.join('products', p, 'site'))) dirs.push(path.join('products', p, 'site'));
}

const esc = (s) => String(s).replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');
const family = JSON.parse(fs.readFileSync('site/family.json', 'utf8'));
const TAB_LABEL = { verifysite: 'Verify this site', overview: 'Overview', demo: 'Demo', download: 'Download', docs: 'Docs', verify: 'Verify', support: 'Support' };
const TAB_PATH = { verifysite: '/verify-site/', overview: '/', demo: '/demo/', download: '/download/', docs: '/docs/', verify: '/verify/', support: '/support/' };

function siteUrl(site) {
  if (site.id === 'intellica') return family.home.replace(/\/$/, '');
  const p = family.products.find((x) => x.id === site.id);
  return p ? p.url.replace(/\/$/, '') : '';
}

function describe(site, tab) {
  const n = site.name;
  switch (tab) {
    case 'home': return site.description || site.tagline || `${n}: systems you can verify without trusting us.`;
    case 'overview': return site.tagline;
    case 'demo': return `Try ${n} in your browser. ${site.demo && site.demo.summary ? site.demo.summary : ''}`.trim();
    case 'download': return `Download ${n}, or build it from source, and check the SHA-256 of what you get.`;
    case 'docs': return `${n} documentation: user manual, installation, reference and architecture.`;
    case 'verify': return `How to check ${n}'s results yourself, without trusting Intellica.`;
    case 'verifysite': return `Check in your browser that every file ${n} served you matches the published manifest.`;
    case 'support': return `Get help with ${n}: contact, known problems and where to report issues.`;
    default: return site.tagline || n;
  }
}

function titleFor(site, tab) {
  if (tab === 'home') return `${site.name}: systems you can verify without trusting us`;
  if (tab === 'overview') return `${site.name} | Intellica`;
  return `${TAB_LABEL[tab] || tab} | ${site.name}`;
}

function summaryHtml(site, tab, base) {
  const ul = (items) => `<ul>${items.map((i) => `<li>${i}</li>`).join('')}</ul>`;
  if (tab === 'overview') {
    const parts = [`<h1>${esc(site.name)}</h1>`, `<p>${esc(site.tagline)}</p>`];
    if (site.status) parts.push(`<p><strong>Status:</strong> ${esc(site.status)}</p>`);
    if ((site.highlights || []).length) parts.push(ul(site.highlights.map((h) => `<strong>${esc(h.title)}.</strong> ${esc(h.text)}`)));
    if (site.verifyIt) parts.push(`<p><strong>Check it yourself:</strong> ${esc(site.verifyIt)}</p>`);
    parts.push(ul((site.tabs || []).filter((t) => t !== 'overview').map((t) => `<a href="${TAB_PATH[t]}">${TAB_LABEL[t]}</a>`)));
    return parts.join('');
  }
  if (tab === 'docs') return `<p>The documentation viewer needs JavaScript. The pages are plain Markdown files:</p>${ul(['use/user-manual.md', 'install/installation.md', 'reference/reference.md', 'architecture/overview.md'].map((f) => `<a href="/docs/${f}">${f}</a>`))}`;
  if (tab === 'verify') return `<p>This page needs JavaScript. The text is at <a href="/verify/verify.md">verify.md</a>.</p>`;
  if (tab === 'demo') return `<p>The demo needs JavaScript.${site.demo && site.demo.command ? ` To check it yourself on your own machine, run <code>${esc(site.demo.command)}</code>.` : ''}</p>`;
  if (tab === 'download') return `<p>This page needs JavaScript. The file list is at <a href="/download/downloads.json">downloads.json</a>.</p>`;
  if (tab === 'support') return '';
  return '';
}

// Structured data for search engines. Only states what the page itself says.
function jsonLd(site, tab, base, url) {
  if (tab === 'home') return { '@context': 'https://schema.org', '@type': 'Organization', name: 'Intellica', url: base + '/', description: site.description || site.tagline, email: site.contact || family.contact };
  if (tab === 'overview') {
    const d = { '@context': 'https://schema.org', '@type': 'SoftwareApplication', name: site.name, description: site.tagline, url, publisher: { '@type': 'Organization', name: 'Intellica', url: family.home } };
    if (site.repo) d.codeRepository = site.repo;
    return d;
  }
  return null;
}

function render(file, site, base) {
  let html = fs.readFileSync(file, 'utf8');
  if (/data-generated="/.test(html)) return false; // /training/ pages carry their own metadata (tools/build-training.mjs)
  const m = html.match(/<body[^>]*data-tab="([a-z]+)"/);
  if (!m) return false;
  const tab = m[1];
  const is404 = /404\.html$/.test(file);
  const title = is404 ? `Page not found | ${site.name}` : titleFor(site, tab);
  const desc = is404 ? '' : describe(site, tab);
  const rel = path.relative(dirs.find((d) => file.startsWith(path.resolve(d))) || '', file);
  const pagePath = TAB_PATH[tab] || '/';
  const canonical = base && !is404 ? `${base}${tab === 'home' ? '/' : pagePath}` : '';

  // Preload the two text weights when the fonts are installed, so the page paints once in its final font.
  const siteRoot = path.resolve(dirs.find((d) => file.startsWith(path.resolve(d))) || '.');
  const preload = ['ibm-plex-sans-latin-400-normal', 'ibm-plex-sans-latin-600-normal']
    .filter((n) => fs.existsSync(path.join(siteRoot, 'assets', 'fonts', `${n}.woff2`)))
    .map((n) => `<link rel="preload" href="/assets/fonts/${n}.woff2" as="font" type="font/woff2" crossorigin>`);

  const meta = [
    ...preload,
    `<meta name="description" content="${esc(desc)}">`,
    is404 ? '<meta name="robots" content="noindex">' : null,
    `<meta name="theme-color" content="${esc(site.accent || '#00C8FF')}">`,
    canonical ? `<link rel="canonical" href="${canonical}">` : null,
    is404 ? null : `<meta property="og:site_name" content="Intellica">`,
    is404 ? null : `<meta property="og:type" content="website">`,
    is404 ? null : `<meta property="og:title" content="${esc(title)}">`,
    is404 ? null : `<meta property="og:description" content="${esc(desc)}">`,
    canonical ? `<meta property="og:url" content="${canonical}">` : null,
    is404 ? null : `<meta name="twitter:card" content="summary">`,
  ].filter(Boolean).join('\n  ');

  html = html.replace(/<title>[\s\S]*?<\/title>/, `<title>${esc(title)}</title>`);
  html = html.replace(/<!-- meta:start -->[\s\S]*?<!-- meta:end -->/, '<meta name="description" content="">');
  html = html.replace(/<meta name="description"[^>]*>/, `<!-- meta:start -->\n  ${meta}\n  <!-- meta:end -->`);

  html = html.replace(/\n?\s*<script type="application\/ld\+json">[\s\S]*?<\/script>/, '');
  const ld = is404 ? null : jsonLd(site, tab, base, canonical);
  if (ld) html = html.replace('<!-- meta:end -->', `<!-- meta:end -->\n  <script type="application/ld+json">${JSON.stringify(ld).replace(/</g, '\\u003c')}</script>`);

  const summary = summaryHtml(site, tab, base);
  if (summary) {
    html = html.replace(/<noscript[^>]*>[\s\S]*?<\/noscript>/, `<noscript>${summary}</noscript>`);
  }
  fs.writeFileSync(file, html);
  return true;
}

for (const dir of dirs) {
  const root = path.resolve(dir);
  const siteFile = path.join(root, 'site.json');
  if (!fs.existsSync(siteFile)) { console.log(`skip ${dir} (no site.json)`); continue; }
  const site = JSON.parse(fs.readFileSync(siteFile, 'utf8'));
  const base = siteUrl(site);
  const pages = [];
  const visit = (d) => {
    for (const e of fs.readdirSync(d, { withFileTypes: true })) {
      const p = path.join(d, e.name);
      if (e.isDirectory()) { if (!['kit', 'legacy', 'assets', 'node_modules'].includes(e.name) && !e.name.startsWith('deck-')) visit(p); }
      else if (e.name.endsWith('.html')) pages.push(p);
    }
  };
  visit(root);
  let n = 0;
  for (const f of pages) if (render(f, site, base)) n++;

  const tabs = site.id === 'intellica' ? ['home'] : (site.tabs || Object.keys(TAB_PATH));
  const urls = tabs.map((t) => `${base}${t === 'home' ? '/' : TAB_PATH[t]}`);
  // generated pages (the /training/ section) are listed too; unlisted folders (deck-*) never are
  for (const f of pages) {
    if (!/data-generated="/.test(fs.readFileSync(f, 'utf8'))) continue;
    const rel = path.relative(root, path.dirname(f)).split(path.sep).join('/');
    urls.push(`${base}/${rel}${rel ? '/' : ''}`);
  }
  fs.writeFileSync(path.join(root, 'sitemap.xml'),
    `<?xml version="1.0" encoding="UTF-8"?>\n<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">\n${urls.map((u) => `  <url><loc>${u}</loc></url>`).join('\n')}\n</urlset>\n`);
  fs.writeFileSync(path.join(root, 'robots.txt'), `User-agent: *\nAllow: /\n\nSitemap: ${base}/sitemap.xml\n`);
  const wk = path.join(root, '.well-known');
  fs.mkdirSync(wk, { recursive: true });
  const sec = path.join(wk, 'security.txt');
  if (!fs.existsSync(sec)) {
    const contact = site.contact || family.contact;
    const exp = new Date(Date.now() + 365 * 86400000).toISOString().replace(/\.\d+Z$/, 'Z');
    fs.writeFileSync(sec, `Contact: mailto:${contact}\nExpires: ${exp}\nPreferred-Languages: en\nCanonical: ${base}/.well-known/security.txt\n`);
  }
  console.log(`prerendered ${n} page(s) in ${dir}`);
}
