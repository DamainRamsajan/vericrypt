// Intellica shell: builds the header, tab bar, status line and footer around each page,
// and hydrates the page body from JSON/Markdown files that live in the same site.
// Loaded as <script type="module" src="/kit/shell.js"> on every page.

import { $, el, loadJSON, loadText, copyText, safeHref, storage } from './dom.js';
import { renderMarkdown, slugify } from './markdown.js';

export const KIT_VERSION = '0.6.0';

const TABS = {
  overview: { label: 'Overview', path: '/' },
  demo: { label: 'Demo', path: '/demo/' },
  download: { label: 'Download', path: '/download/' },
  docs: { label: 'Docs', path: '/docs/' },
  verify: { label: 'Verify', path: '/verify/' },
  support: { label: 'Support', path: '/support/' },
};

/* ---------- brand and theme ---------- */

function contrastOn(hex) {
  const m = /^#?([0-9a-f]{6})$/i.exec(hex || '');
  if (!m) return '#04121a';
  const n = parseInt(m[1], 16);
  const [r, g, b] = [(n >> 16) & 255, (n >> 8) & 255, n & 255].map((v) => {
    v /= 255;
    return v <= 0.03928 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4;
  });
  const L = 0.2126 * r + 0.7152 * g + 0.0722 * b;
  return (L + 0.05) / 0.056 > 1.05 / (L + 0.05) ? '#04121a' : '#ffffff';
}

function applyBrand(site) {
  const root = document.documentElement;
  if (site.accent) {
    root.style.setProperty('--accent', site.accent);
    root.style.setProperty('--on-accent', contrastOn(site.accent));
  }
  const staticTitle = document.title || '';
  document.title = !staticTitle || staticTitle === site.name ? site.name : `${site.name} · ${staticTitle}`;
  const desc = $('meta[name="description"]');
  if (desc && site.tagline) desc.setAttribute('content', site.tagline);
}

const THEMES = ['auto', 'light', 'dark'];

function themeButton() {
  let mode = storage.get('iw-theme');
  mode = THEMES.includes(mode) ? mode : 'auto';
  const btn = el('button', { type: 'button', class: 'btn btn-ghost theme-btn' }, svg('svg', { viewBox: '0 0 24 24', 'aria-hidden': 'true', focusable: 'false', class: 'theme-icon' }, svg('circle', { cx: 12, cy: 12, r: 8, fill: 'none', stroke: 'currentColor', 'stroke-width': 1.8 }), svg('path', { d: 'M12 4a8 8 0 0 1 0 16z', fill: 'currentColor' })));
  const paint = () => { btn.setAttribute('data-mode', mode); btn.setAttribute('title', `Colour theme: ${mode}`); btn.setAttribute('aria-label', `Colour theme: ${mode}. Activate to change.`); };
  btn.addEventListener('click', () => {
    mode = THEMES[(THEMES.indexOf(mode) + 1) % THEMES.length];
    if (mode === 'auto') { document.documentElement.removeAttribute('data-theme'); storage.set('iw-theme', 'auto'); }
    else { document.documentElement.setAttribute('data-theme', mode); storage.set('iw-theme', mode); }
    paint();
  });
  paint();
  return btn;
}


/* ---------- brand, icons, contact ---------- */

const SVGNS = 'http://www.w3.org/2000/svg';
function svg(tag, attrs = {}, ...kids) {
  const n = document.createElementNS(SVGNS, tag);
  for (const [k, v] of Object.entries(attrs)) if (v !== null && v !== undefined && v !== false) n.setAttribute(k, String(v));
  for (const kid of kids.flat()) if (kid) n.append(kid instanceof Node ? kid : document.createTextNode(String(kid)));
  return n;
}

// The mark is an I-beam: the letter I, and the member that carries load in structural engineering.
// The cyan half of the web is the part you can check.
function brandMark() {
  return svg('svg', { viewBox: '0 0 64 64', class: 'brand-mark', 'aria-hidden': 'true', focusable: 'false' },
    svg('rect', { width: 64, height: 64, rx: 8, class: 'bm-tile' }),
    svg('path', { d: 'M12 12h40v10H38v20h14v10H12V42h14V22H12z', class: 'bm-beam' }),
    svg('rect', { x: 38, y: 22, width: 14, height: 20, class: 'bm-node' }));
}

function waIcon() {
  return svg('svg', { viewBox: '0 0 24 24', class: 'wa-icon', 'aria-hidden': 'true', focusable: 'false' },
    svg('path', { d: 'M12 2.5a9.5 9.5 0 0 0-8.2 14.3L2.5 21.5l4.8-1.2A9.5 9.5 0 1 0 12 2.5z', fill: 'none', stroke: 'currentColor', 'stroke-width': 1.8, 'stroke-linejoin': 'round' }),
    svg('path', { d: 'M8.9 7.6c-.3 0-.7.2-.9.6-.3.7-.2 1.5.6 2.8 1 1.6 2.3 2.9 3.9 3.6 1 .4 1.7.5 2.3.1.4-.2.6-.6.6-1l-.1-.4-1.6-.8c-.3-.1-.5 0-.7.2l-.5.6c-.7-.2-1.5-.9-2.2-1.9l-.3-.5.5-.5c.2-.2.2-.4.1-.7L9.5 7.9c-.1-.2-.3-.3-.6-.3z', fill: 'currentColor' }));
}

function contactMessage(site) {
  return `Hello, I’d like to speak to sales about ${site && site.id && site.id !== 'intellica' ? site.name : 'Intellica'}.`;
}

// Contact is WhatsApp only: an icon and "Contact sales". The number is in the link, not on the page.
function contactButton(site, family, cls = 'btn btn-primary contact-btn') {
  const num = (site && site.whatsapp) || (family && family.whatsapp);
  if (!num) return null;
  return el('a', { class: cls, href: `https://wa.me/${num}?text=${encodeURIComponent(contactMessage(site))}`, rel: 'noopener', 'aria-label': 'Contact sales on WhatsApp' },
    waIcon(), el('span', { class: 'contact-label' }, 'Contact sales'));
}

/* ---------- chrome ---------- */

function familyMenu(site, family) {
  if (!family) return null;
  const items = [{ id: 'intellica', name: 'Intellica', url: family.home }, ...family.products, ...(family.links || [])];
  return el('details', { class: 'family' },
    el('summary', {}, 'Systems'),
    el('ul', {}, items.map((p) =>
      el('li', {}, el('a', { href: p.url, 'aria-current': p.id === site.id ? 'page' : null }, p.name)))));
}

function primaryNav(site) {
  const links = site.nav || [];
  if (!links.length) return null;
  return el('nav', { class: 'bar-nav', 'aria-label': 'Primary' }, el('ul', {}, links.map((l) =>
    el('li', {}, el('a', { href: l.href, 'aria-current': location.pathname.startsWith(l.href) ? 'page' : null }, l.label)))));
}

function renderHeader(site, family, page) {
  const host = $('#shell-header');
  if (!host) return;
  const parts = [];
  parts.push(el('header', { class: 'shell-header' },
    el('div', { class: 'container bar' },
      el('a', { class: 'brand', href: '/', 'aria-label': `${site.name}, home` }, brandMark(), el('span', { class: 'brand-name' }, site.name)),
      el('div', { class: 'bar-right' }, primaryNav(site), familyMenu(site, family), themeButton(), contactButton(site, family)))));

  const tabs = (site.tabs || []).filter((t) => TABS[t]);
  if (tabs.length) {
    parts.push(el('nav', { class: 'tabs', 'aria-label': 'Sections' },
      el('div', { class: 'container' },
        el('ul', {}, tabs.map((t) =>
          el('li', {}, el('a', { href: TABS[t].path, 'aria-current': t === page ? 'page' : null }, TABS[t].label)))))));
  }

  if (site.status || site.version) {
    parts.push(el('div', { class: 'status-strip' },
      el('div', { class: 'container' },
        site.status ? el('span', { class: 'badge' }, site.status) : null,
        site.version ? el('span', { class: 'muted' }, `Version ${site.version}`) : null)));
  }
  host.replaceChildren(...parts.filter(Boolean));
}

function renderFooter(site, family) {
  const host = $('#shell-footer');
  if (!host) return;
  const systems = family ? family.products : [];
  host.replaceChildren(el('footer', { class: 'shell-footer' },
    el('div', { class: 'container foot' },
      el('div', { class: 'foot-grid' },
        el('div', { class: 'foot-brand' },
          el('a', { class: 'brand', href: family ? family.home : '/' }, brandMark(), el('span', { class: 'brand-name' }, 'Intellica')),
          el('p', { class: 'muted' }, 'Systems whose results you can check yourself.')),
        systems.length ? el('nav', { 'aria-label': 'Systems' }, el('h2', { class: 'foot-h' }, 'Systems'),
          el('ul', {}, systems.map((p) => el('li', {}, el('a', { href: p.url }, p.name))))) : null,
        el('nav', { 'aria-label': 'Company' }, el('h2', { class: 'foot-h' }, 'Company'),
          el('ul', {},
            el('li', {}, el('a', { href: '/training/' }, 'Training')),
            el('li', {}, el('a', { href: '/verify-site/' }, 'Verify this site')))),
        el('div', {}, el('h2', { class: 'foot-h' }, 'Talk to us'),
          el('p', { class: 'muted' }, 'Pilots, partnerships and questions.'),
          contactButton(site, family))),
      el('p', { class: 'muted foot-note' }, `Self-hosted: this site loads nothing from third parties. Kit ${KIT_VERSION}.`))));
}

/* ---------- page: home (Intellica) ---------- */

const productUrl = (product, tab) => product.url.replace(/\/$/, '') + (TABS[tab] ? TABS[tab].path : '/');

function hydrateHome(site, family) {
  if (!family) return;
  const byId = Object.fromEntries(family.products.map((p) => [p.id, p]));

  const roles = $('#roles-grid');
  if (roles) {
    roles.replaceChildren(...family.roles.map((r) =>
      el('article', { class: 'card role' },
        el('h3', {}, r.title),
        el('p', { class: 'muted' }, r.blurb),
        el('ul', { class: 'link-list' }, r.links
          .filter((l) => l.href || byId[l.product])
          .map((l) => el('li', {}, el('a', { href: l.href || productUrl(byId[l.product], l.tab) }, l.label)))))));
  }

  const cards = $('#family-cards');
  if (cards) {
    cards.replaceChildren(...family.products.map((p) => {
      const card = el('article', { class: 'card product' },
        p.short ? el('p', { class: 'kicker-m' }, p.short) : null,
        el('h3', {}, el('a', { href: p.url }, p.name)),
        el('p', {}, p.tagline),
        p.status ? el('p', {}, el('span', { class: 'badge' }, p.status)) : null,
        el('ul', { class: 'chip-links' }, ['overview', 'demo', 'download', 'docs'].map((t) =>
          el('li', {}, el('a', { href: productUrl(p, t) }, TABS[t].label)))));
      card.style.setProperty('--card-accent', p.accent);
      return card;
    }));
  }

  const sectors = $('#sector-links');
  if (sectors && family.sectors && family.sectors.length) {
    sectors.replaceChildren(...family.sectors.map((s) => el('li', {}, el('a', { href: s.path }, s.title))));
  } else if (sectors) {
    sectors.closest('section')?.setAttribute('hidden', '');
  }

  const standards = $('#standards-list');
  if (standards && family.standards) {
    standards.replaceChildren(...family.standards.map((s) => el('li', {}, s)));
  }

  const contact = $('#contact-link');
  if (contact) {
    const btn = contactButton(site, family);
    if (btn) contact.replaceWith(btn);
  }
  drawSystemMap(family);
  runProof();
}

/* The five systems around one idea: every result comes with a way to check it. Drawn from family.json. */
function drawSystemMap(family) {
  const host = $('#system-map');
  if (!host) return;
  const W = 560, H = 430;
  const spine = { x: 130, y: 178, w: 300, h: 74 };
  const place = [
    { id: 'verichain', x: 0, y: 14 }, { id: 'agentseed', x: 200, y: 14 }, { id: 'vericrypt', x: 400, y: 14 },
    { id: 'sovereign-erp', x: 50, y: 344 }, { id: 'omc', x: 350, y: 344 },
  ];
  const NW = 160, NH = 62;
  const root = svg('svg', { viewBox: `0 0 ${W} ${H}`, class: 'sysmap-svg', role: 'group', 'aria-label': 'The five Intellica systems, each producing evidence that can be checked' });
  const edges = svg('g', { class: 'sm-edges' });
  const nodes = svg('g', {});
  const byId = Object.fromEntries(family.products.map((p) => [p.id, p]));
  place.forEach((pl, i) => {
    const p = byId[pl.id];
    if (!p) return;
    const cx = pl.x + NW / 2;
    const top = pl.y < 100;
    const y1 = top ? pl.y + NH : pl.y;
    const y2 = top ? spine.y : spine.y + spine.h;
    const sx = spine.x + 40 + (i * (spine.w - 80)) / (place.length - 1);
    const mid = (y1 + y2) / 2;
    const d = `M${cx} ${y1} V${mid} H${sx} V${y2}`;
    edges.append(svg('path', { d, class: 'sm-edge' }), svg('path', { d, class: 'sm-flow', style: null }));
    const a = svg('a', { href: p.url, class: 'sm-node', 'aria-label': `${p.name}: ${p.short || ''}` });
    a.append(
      svg('rect', { x: pl.x, y: pl.y, width: NW, height: NH, class: 'sm-box' }),
      svg('rect', { x: pl.x, y: pl.y, width: 4, height: NH, fill: p.accent }),
      svg('text', { x: pl.x + 16, y: pl.y + 27, class: 'sm-name' }, p.name),
      svg('text', { x: pl.x + 16, y: pl.y + 46, class: 'sm-sub' }, p.short || ''));
    nodes.append(a);
  });
  const core = svg('g', { class: 'sm-core' },
    svg('rect', { x: spine.x, y: spine.y, width: spine.w, height: spine.h, class: 'sm-spine' }),
    svg('text', { x: spine.x + spine.w / 2, y: spine.y + 31, 'text-anchor': 'middle', class: 'sm-core-name' }, 'Checkable evidence'),
    svg('text', { x: spine.x + spine.w / 2, y: spine.y + 54, 'text-anchor': 'middle', class: 'sm-sub' }, 'Every result ships with a way to verify it'));
  root.append(edges, core, nodes);
  host.replaceChildren(root);
}

/* Live proof: the page checks itself. It recomputes the manifest root and re-hashes a few key files in your browser. */
async function runProof() {
  const host = $('#proof');
  if (!host) return;
  const out = $('#proof-out', host), summary = $('#proof-summary', host);
  let manifest;
  try { manifest = await loadJSON('/site-manifest.json'); }
  catch { summary.textContent = 'The manifest could not be loaded, so the check did not run. Open Verify this site to try again.'; return; }
  const entries = Object.entries(manifest.files);
  const root = await sha256Hex(new TextEncoder().encode(entries.map(([p, h]) => `${p}\t${h}\n`).join('')));
  const spot = ['/index.html', '/kit/tokens.css', '/kit/dom.js', '/kit/theme-init.js', '/site.json', '/family.json'].filter((p) => manifest.files[p]);
  let ok = root === manifest.root;
  const rows = spot.map((p) => {
    const mark = el('span', { class: 'pr-mark', 'aria-hidden': 'true' }, '·');
    const hash = el('span', { class: 'pr-hash' }, 'hashing');
    const row = el('li', { class: 'pr-row' }, mark, el('span', { class: 'pr-path' }, p), hash);
    out.append(row);
    return { p, mark, hash, row };
  });
  for (const r of rows) {
    try {
      const res = await fetch(r.p, { cache: 'no-store' });
      const got = await sha256Hex(await res.arrayBuffer());
      const good = got === manifest.files[r.p];
      ok = ok && good;
      r.mark.textContent = good ? '✓' : '✗';
      r.hash.textContent = got.slice(0, 16);
      r.row.setAttribute('data-ok', String(good));
    } catch { ok = false; r.mark.textContent = '✗'; r.hash.textContent = 'unreachable'; r.row.setAttribute('data-ok', 'false'); }
    await new Promise((res) => setTimeout(res, 90));
  }
  host.setAttribute('data-state', ok ? 'ok' : 'fail');
  summary.replaceChildren(
    ok ? `Manifest root recomputed and ${spot.length} files re-hashed: all match. ` : 'A file did not match its manifest. ',
    el('code', {}, manifest.root.slice(0, 24) + '\u2026'), ` · ${entries.length} files listed. `,
    el('a', { href: '/verify-site/' }, `Verify all ${entries.length}`));
}

/* ---------- page: overview (product) ---------- */

function section(title, ...kids) {
  return el('section', { class: 'block' }, el('h2', {}, title), ...kids);
}

function hydrateOverview(site) {
  const root = $('#overview-root');
  if (!root) return;
  const tabs = site.tabs || [];
  const actions = [];
  if (tabs.includes('demo')) actions.push(el('a', { class: 'btn btn-primary', href: '/demo/' }, 'Try the demo'));
  if (tabs.includes('download')) actions.push(el('a', { class: 'btn btn-ghost', href: '/download/' }, 'Download'));
  if (tabs.includes('docs')) actions.push(el('a', { class: 'btn btn-ghost', href: '/docs/' }, 'Read the docs'));

  const parts = [el('section', { class: 'hero' },
    el('p', { class: 'eyebrow' }, 'Intellica'),
    el('h1', {}, site.name),
    el('p', { class: 'lead' }, site.tagline),
    el('div', { class: 'actions' }, actions))];

  if (site.audience && site.audience.length) {
    parts.push(section('Who it is for', el('div', { class: 'grid grid-3' }, site.audience.map((a) =>
      el('article', { class: 'card' }, el('h3', {}, a.role), el('p', { class: 'muted' }, a.concern))))));
  }
  if (site.highlights && site.highlights.length) {
    parts.push(section('What it does', el('div', { class: 'grid grid-2' }, site.highlights.map((h) =>
      el('article', { class: 'card' }, el('h3', {}, h.title), el('p', { class: 'muted' }, h.text))))));
  }
  if (site.limits && site.limits.length) {
    parts.push(section('What it does not do yet', el('ul', { class: 'limits' }, site.limits.map((t) => el('li', {}, t)))));
  }
  if (site.verifyIt) {
    const next = tabs.includes('demo') ? el('a', { class: 'btn btn-primary', href: '/demo/' }, 'Open the demo')
      : tabs.includes('verify') ? el('a', { class: 'btn btn-primary', href: '/verify/' }, 'Go to Verify') : null;
    parts.push(el('section', { class: 'callout verify' },
      el('h2', {}, 'Verify it yourself'), el('p', {}, site.verifyIt), next ? el('div', { class: 'actions' }, next) : null));
  }
  root.replaceChildren(...parts.filter(Boolean));
}

/* ---------- page: demo ---------- */

const ENGINE_LABELS = {
  wasm: 'Real engine, compiled to WebAssembly',
  trace: 'Recorded trace made by the real engine',
  placeholder: 'Placeholder: a generic hash-chain demo, not this product’s engine',
};

async function hydrateDemo(site) {
  const root = $('#demo-root');
  if (!root) return;
  const cfg = site.demo || {};
  const engine = ENGINE_LABELS[cfg.engine] ? cfg.engine : 'placeholder';

  const mountArea = el('div', { id: 'demo-mount', class: 'demo-mount' });
  const reset = el('button', { type: 'button', class: 'btn btn-ghost' }, 'Reset the demo');
  let handle = null;

  async function start() {
    try { handle && handle.unmount && handle.unmount(); } catch { /* ignore */ }
    mountArea.replaceChildren(el('p', { class: 'muted' }, 'Loading the demo…'));
    try {
      const mod = await import(new URL('./demo.js', location.href).href);
      const theme = document.documentElement.getAttribute('data-theme') ||
        (matchMedia('(prefers-color-scheme: light)').matches ? 'light' : 'dark');
      handle = await mod.mount(mountArea, {
        base: new URL('./', location.href).href,
        theme,
        reducedMotion: matchMedia('(prefers-reduced-motion: reduce)').matches,
      });
    } catch (err) {
      mountArea.replaceChildren(el('p', { class: 'callout bad' }, `The demo could not start: ${err.message}`));
    }
  }
  reset.addEventListener('click', start);

  const parts = [
    el('h1', {}, cfg.title || `${site.name} demo`),
    cfg.summary ? el('p', { class: 'lead' }, cfg.summary) : null,
    el('div', { class: 'demo-strip' },
      el('span', { class: 'badge', 'data-engine': engine }, ENGINE_LABELS[engine]),
      cfg.sample ? el('span', { class: 'muted' }, `Sample input: ${cfg.sample}`) : null,
      el('span', { class: 'muted' }, 'Runs entirely in your browser. Nothing is sent anywhere.')),
    el('div', { class: 'demo-frame' }, mountArea, el('div', { class: 'actions' }, reset)),
  ];

  if (cfg.command) {
    const copy = el('button', { type: 'button', class: 'btn btn-ghost' }, 'Copy command');
    copy.addEventListener('click', async () => { copy.textContent = (await copyText(cfg.command)) ? 'Copied' : 'Copy failed'; });
    parts.push(section('Run it yourself',
      el('p', {}, 'The same check with the downloaded binary:'),
      el('pre', {}, el('code', {}, cfg.command)),
      cfg.expected ? [el('p', {}, 'Expected result:'), el('pre', {}, el('code', {}, cfg.expected))] : null,
      el('div', { class: 'actions' }, copy)));
  }
  root.replaceChildren(...parts.filter(Boolean));
  await start();
}

/* ---------- page: download ---------- */

function formatBytes(n) {
  if (!Number.isFinite(n)) return '';
  const units = ['B', 'KB', 'MB', 'GB'];
  let i = 0;
  while (n >= 1024 && i < units.length - 1) { n /= 1024; i++; }
  return `${n.toFixed(i ? 1 : 0)} ${units[i]}`;
}

async function hydrateDownload(site) {
  const root = $('#downloads-root');
  if (!root) return;
  let data;
  try { data = await loadJSON('./downloads.json'); } catch { data = null; }
  const items = (data && data.items) || [];
  const parts = [];

  if (data && data.licence) parts.push(el('p', { class: 'callout' }, data.licence));

  if (!items.length) {
    parts.push(el('p', { class: 'muted' }, 'There are no packaged downloads yet.'));
    const b = site && site.buildFromSource;
    if (b) {
      const copy = el('button', { type: 'button', class: 'btn btn-ghost btn-small' }, 'Copy commands');
      copy.addEventListener('click', async () => { copy.textContent = (await copyText(b.steps)) ? 'Copied' : 'Failed'; });
      parts.push(section(b.title || 'Build from source',
        b.intro ? el('p', {}, b.intro) : null,
        el('pre', {}, el('code', {}, b.steps)),
        copy,
        b.note ? el('p', { class: 'muted' }, b.note) : null,
        site.repo ? el('p', {}, el('a', { href: safeHref(site.repo) }, 'Source repository')) : null));
    }
  } else {
    const rows = items.map((it) => {
      const copy = el('button', { type: 'button', class: 'btn btn-ghost btn-small' }, 'Copy');
      copy.addEventListener('click', async () => { copy.textContent = (await copyText(it.sha256)) ? 'Copied' : 'Failed'; });
      return el('tr', {},
        el('td', {}, it.file ? el('a', { href: it.file, download: '' }, it.name) : it.name),
        el('td', {}, it.version || ''),
        el('td', {}, it.platform || ''),
        el('td', {}, formatBytes(it.bytes)),
        el('td', {}, el('code', { class: 'hash' }, it.sha256 || ''), ' ', it.sha256 ? copy : null),
        el('td', {}, it.signature ? el('a', { href: it.signature }, 'Signature') : ''),
        el('td', {}, it.notes || ''));
    });
    parts.push(el('div', { class: 'table-wrap' }, el('table', {},
      el('thead', {}, el('tr', {}, ['Name', 'Version', 'Platform', 'Size', 'SHA-256', 'Signature', 'Notes'].map((h) => el('th', {}, h)))),
      el('tbody', {}, rows))));
  }

  if (items.length) {
    parts.push(section('Check what you downloaded',
      el('p', {}, 'Compare the SHA-256 of the file you received with the value in the table.'),
      el('pre', {}, el('code', {}, 'macOS:    shasum -a 256 <file>\nLinux:    sha256sum <file>\nWindows:  Get-FileHash <file> -Algorithm SHA256'))));
  }
  root.replaceChildren(...parts.filter(Boolean));
}

/* ---------- page: docs ---------- */

function docsRelative(href, docDir) {
  const base = new URL('./', location.href);
  const target = new URL(href, new URL(docDir, base));
  return { rel: target.pathname.slice(base.pathname.length), hash: target.hash };
}

async function hydrateDocs(site) {
  const root = $('#docs-root');
  if (!root) return;
  let index;
  try { index = await loadJSON('./docs.json'); } catch { root.replaceChildren(el('p', { class: 'muted' }, 'Nothing is published here yet.')); return; }
  const flat = index.groups.flatMap((g) => g.items.map((i) => ({ ...i, group: g.title })));
  if (!flat.length) { root.replaceChildren(el('p', { class: 'muted' }, 'Nothing is published here yet.')); return; }

  const wanted = new URLSearchParams(location.search).get('d');
  const entry = flat.find((i) => i.md === wanted) || flat[0];

  const nav = el('nav', { class: 'docs-nav', 'aria-label': 'Documentation' }, index.groups.map((g) =>
    el('div', { class: 'docs-group' }, el('h2', {}, g.title),
      el('ul', {}, g.items.map((i) => el('li', {}, el('a', { href: `?d=${encodeURIComponent(i.md)}`, 'aria-current': i.md === entry.md ? 'page' : null }, i.title)))))));

  const article = el('article', { class: 'prose' });
  const searchBox = el('div', { class: 'docs-searchbox' });
  nav.prepend(searchBox);
  buildSearch(flat, searchBox);
  root.replaceChildren(el('div', { class: 'docs-layout' }, nav, article));
  document.title = `${entry.title} · ${site.name} docs`;

  try {
    article.innerHTML = renderMarkdown(await loadText(`./${entry.md}`), { demote: 1 });
  } catch (err) {
    article.replaceChildren(el('p', { class: 'callout bad' }, `This page could not be loaded: ${err.message}`));
    return;
  }
  const docDir = `./${entry.md.split('/').slice(0, -1).join('/')}${entry.md.includes('/') ? '/' : ''}`;
  article.querySelectorAll('a[href]').forEach((a) => {
    const href = a.getAttribute('href');
    if (/^([a-z]+:|\/|#)/i.test(href)) return;
    const { rel, hash } = docsRelative(href.split('#')[0] || '.', docDir);
    a.setAttribute('href', rel.endsWith('.md') ? `?d=${encodeURIComponent(rel)}${hash || (href.includes('#') ? '#' + href.split('#')[1] : '')}` : `./${rel}`);
  });
  article.querySelectorAll('img[src]').forEach((img) => {
    const { rel } = docsRelative(img.getAttribute('src'), docDir);
    img.setAttribute('src', `./${rel}`);
  });
  addCopyButtons(article);
  const heads = [...article.querySelectorAll('h2, h3')];
  if (heads.length > 3) {
    article.prepend(el('details', { class: 'toc' }, el('summary', {}, 'On this page'),
      el('ul', {}, heads.map((h) => el('li', { class: h.tagName === 'H3' ? 'sub' : '' }, el('a', { href: `#${h.id}` }, h.textContent))))));
  }
}

/* docs: copy buttons on code blocks */
function addCopyButtons(article) {
  article.querySelectorAll('pre').forEach((pre) => {
    const wrap = el('div', { class: 'code-block' });
    pre.replaceWith(wrap);
    const btn = el('button', { type: 'button', class: 'code-copy', 'aria-label': 'Copy code' }, 'Copy');
    btn.addEventListener('click', async () => {
      btn.textContent = (await copyText(pre.textContent)) ? 'Copied' : 'Failed';
      setTimeout(() => { btn.textContent = 'Copy'; }, 1800);
    });
    wrap.append(pre, btn);
  });
}

/* docs: search across every page of the manual (built in the browser, nothing is sent anywhere) */
function sectionsOf(file, md) {
  const out = [];
  const seen = new Map();
  let cur = { heading: '', id: '', text: [] };
  let inFence = false;
  const push = () => { if (cur.heading || cur.text.length) out.push({ file, heading: cur.heading, id: cur.id, text: cur.text.join(' ') }); };
  for (const line of md.split('\n')) {
    if (/^\s*```/.test(line)) { inFence = !inFence; cur.text.push(''); continue; }
    const h = !inFence && line.match(/^(#{1,6})\s+(.*?)\s*#*\s*$/);
    if (h) {
      push();
      const base = slugify(h[2]);
      const n = seen.get(base) || 0; seen.set(base, n + 1);
      cur = { heading: h[2], id: n ? `${base}-${n}` : base, text: [] };
    } else cur.text.push(line.replace(/[`*_>#]/g, '').replace(/\|/g, ' ').replace(/\s+/g, ' ').trim());
  }
  push();
  return out;
}

function buildSearch(flat, host) {
  const input = el('input', { type: 'search', id: 'docs-search', class: 'docs-search', placeholder: 'Search the docs', autocomplete: 'off' });
  const label = el('label', { for: 'docs-search', class: 'sr-only' }, 'Search the docs');
  const out = el('div', { class: 'docs-results', role: 'region', 'aria-label': 'Search results', 'aria-live': 'polite' });
  host.append(label, input, out);
  let sections = null;
  const load = async () => {
    if (sections) return sections;
    sections = [];
    await Promise.all(flat.map(async (e) => {
      try { sections.push(...sectionsOf(e.md, await loadText(`./${e.md}`)).map((s) => ({ ...s, title: e.title }))); } catch { /* skip */ }
    }));
    return sections;
  };
  const run = async () => {
    const q = input.value.trim().toLowerCase();
    if (q.length < 2) { out.replaceChildren(); return; }
    const words = q.split(/\s+/);
    const hits = (await load())
      .map((s) => { const hay = `${s.heading} ${s.text}`.toLowerCase(); if (!words.every((w) => hay.includes(w))) return null; return { s, score: words.reduce((n, w) => n + (s.heading.toLowerCase().includes(w) ? 5 : 0) + (hay.split(w).length - 1), 0) }; })
      .filter(Boolean).sort((a, b) => b.score - a.score).slice(0, 8);
    if (!hits.length) { out.replaceChildren(el('p', { class: 'muted' }, 'No matches.')); return; }
    out.replaceChildren(el('ul', {}, hits.map(({ s }) => {
      const i = s.text.toLowerCase().indexOf(words[0]);
      let snippet = '';
      if (i >= 0) {
        let a = Math.max(0, i - 40);
        if (a > 0) { const sp = s.text.indexOf(' ', a); a = sp > -1 && sp < i ? sp + 1 : a; }
        let b = Math.min(s.text.length, i + 100);
        if (b < s.text.length) { const sp = s.text.lastIndexOf(' ', b); b = sp > i ? sp : b; }
        snippet = (a > 0 ? '… ' : '') + s.text.slice(a, b).trim() + (b < s.text.length ? ' …' : '');
      } else snippet = s.text.slice(0, 120).trim();
      return el('li', {}, el('a', { href: `?d=${encodeURIComponent(s.file)}${s.id ? '#' + s.id : ''}` }, `${s.title}${s.heading ? ': ' + s.heading : ''}`), el('span', { class: 'muted' }, ` ${snippet}`));
    })));
  };
  let timer;
  input.addEventListener('input', () => { clearTimeout(timer); timer = setTimeout(run, 120); });
  input.addEventListener('focus', load, { once: true });
}

/* ---------- page: verify (markdown) ---------- */

async function hydrateMarkdownPage() {
  const root = $('#md-root');
  if (!root) return;
  try {
    root.innerHTML = renderMarkdown(await loadText(root.getAttribute('data-src') || './page.md'), { demote: 1 });
  } catch (err) {
    root.replaceChildren(el('p', { class: 'muted' }, 'Nothing is published here yet.'));
  }
}

/* ---------- page: support ---------- */

function hydrateSupport(site, family) {
  const mail = $('#support-contact');
  if (mail) { const btn = contactButton(site, family); if (btn) mail.replaceWith(btn); }
  const faq = $('#support-faq');
  if (faq) {
    const items = site.faq || [];
    if (!items.length) faq.closest('section')?.setAttribute('hidden', '');
    else faq.replaceChildren(...items.map((it) => el('details', { class: 'faq-item' }, el('summary', {}, it.q), el('p', {}, it.a))));
  }
  const issues = $('#support-issues');
  if (issues && site.repo) { issues.setAttribute('href', safeHref(`${site.repo.replace(/\/$/, '')}/issues`)); }
  else if (issues) { issues.closest('li')?.setAttribute('hidden', ''); }
}


/* ---------- page: verify this site ---------- */

async function sha256Hex(buf) {
  return [...new Uint8Array(await crypto.subtle.digest('SHA-256', buf))].map((b) => b.toString(16).padStart(2, '0')).join('');
}

async function hydrateVerifySite() {
  const root = $('#verify-site-root');
  if (!root) return;
  const status = el('p', { role: 'status', class: 'muted' }, 'Loading the manifest…');
  const results = el('div', {});
  root.replaceChildren(status, results);
  let manifest;
  try { manifest = await loadJSON('/site-manifest.json'); }
  catch (err) { status.replaceWith(el('p', { class: 'callout bad' }, `The manifest could not be loaded: ${err.message}`)); return; }

  const files = Object.entries(manifest.files);
  const bad = [];
  let done = 0;
  const rootInput = files.map(([path, hash]) => `${path}\t${hash}\n`).join('');
  const computedRoot = await sha256Hex(new TextEncoder().encode(rootInput));
  const rootOk = computedRoot === manifest.root;

  await Promise.all(files.map(async ([path, expected]) => {
    try {
      const res = await fetch(path, { cache: 'no-store' });
      if (!res.ok) throw new Error(`HTTP ${res.status}`);
      const actual = await sha256Hex(await res.arrayBuffer());
      if (actual !== expected) bad.push({ path, expected, actual });
    } catch (err) { bad.push({ path, expected, actual: `could not fetch (${err.message})` }); }
    done++;
    status.textContent = `Checked ${done} of ${files.length} files…`;
  }));

  const ok = !bad.length && rootOk;
  status.replaceWith(el('p', { role: 'status', class: ok ? 'callout good' : 'callout bad' },
    ok ? `All ${files.length} files match the manifest.` : `${bad.length} file(s) do not match the manifest${rootOk ? '' : ', and the manifest root does not match its own list'}.`));
  const rows = bad.map((b) => el('tr', {}, el('td', {}, el('code', {}, b.path)), el('td', {}, el('code', {}, b.expected.slice(0, 16) + '…')), el('td', {}, el('code', {}, String(b.actual).slice(0, 40)))));
  results.replaceChildren(...[
    bad.length ? el('div', { class: 'table-wrap' }, el('table', {}, el('thead', {}, el('tr', {}, el('th', {}, 'File'), el('th', {}, 'Expected'), el('th', {}, 'Received'))), el('tbody', {}, rows))) : null,
    section('Manifest root',
      el('p', {}, 'One SHA-256 over every path and hash, in sorted order:'),
      el('pre', {}, el('code', {}, manifest.root)),
      el('p', { class: 'muted' }, `${files.length} files. Kit ${manifest.kit}.`)),
    section('What this proves, and what it does not',
      el('p', {}, 'Your browser fetched each file from this server and hashed it itself. A match means the files you received are the files the manifest lists, which catches corruption, partial updates and a stale cache.'),
      el('p', {}, 'The manifest comes from the same server, so on its own it cannot expose a server that changes both. For that, compare the root above with the root in the public repository, in ', el('code', {}, 'site-manifest.json'), ' at the same commit.'))].filter(Boolean));
}

/* ---------- boot ---------- */

const HYDRATE = {
  home: hydrateHome,
  overview: hydrateOverview,
  demo: hydrateDemo,
  download: hydrateDownload,
  docs: hydrateDocs,
  verify: hydrateMarkdownPage,
  support: hydrateSupport,
  verifysite: hydrateVerifySite,
};

async function main() {
  const page = document.body.getAttribute('data-tab') || 'overview';
  try {
    const [site, family] = await Promise.all([loadJSON('/site.json'), loadJSON('/family.json').catch(() => null)]);
    applyBrand(site);
    renderHeader(site, family, page);
    renderFooter(site, family);
    document.documentElement.setAttribute('data-shell', 'true');
    if (HYDRATE[page]) await HYDRATE[page](site, family);
    document.documentElement.setAttribute('data-ready', 'true');
  } catch (err) {
    document.documentElement.setAttribute('data-shell', 'true');
    document.documentElement.setAttribute('data-ready', 'true');
    const main = $('#main');
    if (main) main.prepend(el('p', { class: 'callout bad' }, `This page could not finish loading: ${err.message}`));
  }
}

main();
