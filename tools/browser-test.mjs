#!/usr/bin/env node
// Real-browser check. Starts the dev servers, drives Chromium, asserts, exits non-zero on failure.
// Needs: npm run setup   (installs Playwright and Chromium)
import { spawn } from 'node:child_process';
import { createRequire } from 'node:module';
import assert from 'node:assert/strict';

let chromium;
try { ({ chromium } = createRequire(import.meta.url)('playwright')); }
catch { console.error('Playwright is not installed. Run: npm run setup'); process.exit(2); }

const PRODUCTS = [['verichain', 8081], ['vericrypt', 8082], ['sovereign-erp', 8083], ['omc', 8084], ['agentseed', 8085]];
const servers = [['site', 8080], ...PRODUCTS.map(([id, port]) => [`products/${id}/site`, port])].map(([dir, port]) =>
  spawn('node', ['tools/serve.mjs', dir, String(port)], { stdio: 'ignore' }));
const stop = () => servers.forEach((s) => s.kill());
process.on('exit', stop);
await new Promise((r) => setTimeout(r, 800));

const HOME = 'http://localhost:8080', VC = 'http://localhost:8082';
let failed = 0;
const step = async (name, fn) => { try { await fn(); console.log('ok   ' + name); } catch (e) { failed++; console.log('FAIL ' + name + '\n     ' + e.message); } };

const browser = await chromium.launch();
async function open(url, opts = { viewport: { width: 1280, height: 900 } }) {
  const ctx = await browser.newContext(opts);
  const page = await ctx.newPage();
  page.problems = [];
  page.on('console', (m) => { if (m.type() === 'error') page.problems.push(m.text()); });
  page.on('pageerror', (e) => page.problems.push(e.message));
  await page.goto(url);
  await page.waitForSelector('html[data-ready="true"]', { timeout: 8000 });
  return page;
}

await step('home renders roles and products, no console errors', async () => {
  const p = await open(HOME);
  assert.equal(await p.locator('#roles-grid .card').count(), 6);
  assert.equal(await p.locator('#family-cards .card').count(), 5);
  assert.deepEqual(p.problems, []);
});
await step('home has no horizontal scroll at 390px', async () => {
  const p = await open(HOME, { viewport: { width: 390, height: 844 } });
  assert.equal(await p.evaluate(() => document.documentElement.scrollWidth > document.documentElement.clientWidth), false);
});
await step('theme button switches theme', async () => {
  const p = await open(HOME);
  const before = await p.evaluate(() => document.documentElement.getAttribute('data-theme'));
  await p.locator('.theme-btn').click();
  assert.notEqual(await p.evaluate(() => document.documentElement.getAttribute('data-theme')), before);
});
await step('keyboard: skip link is first focus', async () => {
  const p = await open(VC);
  await p.keyboard.press('Tab');
  assert.match(await p.evaluate(() => document.activeElement.textContent.trim()), /skip to content/i);
});
await step('demo: untouched matches, tampered does not, reset restores', async () => {
  const p = await open(VC + '/demo/');
  await p.waitForSelector('.chain-row');
  await p.click('text=Verify against the sealed chain');
  assert.match(await p.locator('.demo-result').innerText(), /^Match/);
  await p.fill('#entry-1', 'Authority skipped');
  await p.click('text=Verify against the sealed chain');
  assert.match(await p.locator('.demo-result').innerText(), /^No match/);
  await p.click('text=Reset the demo');
  await p.waitForSelector('.chain-row');
  assert.equal(await p.inputValue('#entry-1'), 'Authority checked');
});
await step('docs: sidebar navigation changes the page', async () => {
  const p = await open(VC + '/docs/');
  const first = await p.locator('article.prose h2').innerText();
  await p.locator('.docs-nav a', { hasText: 'Reference' }).click();
  await p.waitForSelector('article.prose h2');
  assert.notEqual(await p.locator('article.prose h2').innerText(), first);
  assert.match(p.url(), /\?d=/);
});
await step('download, verify and support tabs render without errors', async () => {
  for (const path of ['/download/', '/verify/', '/support/']) {
    const p = await open(VC + path);
    assert.deepEqual(p.problems, [], path);
  }
});

await step('every product: all tabs and every docs page load without errors', async () => {
  for (const [id, port] of PRODUCTS) {
    const base = `http://localhost:${port}`;
    const index = await (await fetch(base + '/docs/docs.json')).json();
    for (const item of index.groups.flatMap((g) => g.items)) {
      const p = await open(`${base}/docs/?d=${encodeURIComponent(item.md)}`);
      const text = await p.locator('article.prose').innerText();
      assert.ok(text.length > 40, `${id} ${item.md} is empty`);
      assert.ok(!/could not be loaded/.test(text), `${id} ${item.md} failed to load`);
      assert.ok(!text.includes('`'), `${id} ${item.md} shows a stray backtick (broken code span or table cell)`);
      assert.deepEqual(p.problems, [], `${id} ${item.md}`);
    }
    for (const tab of ['/', '/demo/', '/download/', '/verify/', '/support/']) {
      const p = await open(base + tab);
      assert.deepEqual(p.problems, [], `${id}${tab}`);
    }
  }
});
await step('no product page shows an unpublished placeholder in its main docs', async () => {
  for (const [id, port] of PRODUCTS) {
    const p = await open(`http://localhost:${port}/docs/`);
    assert.ok(!/Nothing is published here yet/.test(await p.locator('#docs-root').innerText()), id);
  }
});


// ---- accessibility and content checks (WCAG 2.2 AA basics) ----
const AUDIT = () => {
  const out = [];
  const parse = (c) => { const m = c.match(/rgba?\(([^)]+)\)/); if (!m) return null; const [r, g, b, a = 1] = m[1].split(/[ ,\/]+/).filter(Boolean).map(Number); return { r, g, b, a }; };
  const lum = ({ r, g, b }) => { const f = (v) => { v /= 255; return v <= 0.03928 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4; }; return 0.2126 * f(r) + 0.7152 * f(g) + 0.0722 * f(b); };
  const blend = (top, bot) => ({ r: top.r * top.a + bot.r * (1 - top.a), g: top.g * top.a + bot.g * (1 - top.a), b: top.b * top.a + bot.b * (1 - top.a), a: 1 });
  const bgOf = (el) => { const stack = []; for (let n = el; n; n = n.parentElement) { const c = parse(getComputedStyle(n).backgroundColor); if (c && c.a > 0) { stack.push(c); if (c.a === 1) break; } } let base = { r: 255, g: 255, b: 255, a: 1 }; if (!stack.length || stack[stack.length - 1].a < 1) { const c = parse(getComputedStyle(document.body).backgroundColor); if (c) base = { ...c, a: 1 }; } for (const c of stack.reverse()) base = blend(c, base); return base; };
  const main = document.querySelectorAll('main');
  if (main.length !== 1) out.push(`expected 1 <main>, found ${main.length}`);
  if (document.querySelectorAll('h1').length !== 1) out.push(`expected 1 <h1>, found ${document.querySelectorAll('h1').length}`);
  if (!document.documentElement.lang) out.push('missing lang');
  let prev = 0;
  for (const h of document.querySelectorAll('h1,h2,h3,h4')) { const l = Number(h.tagName[1]); if (prev && l > prev + 1) out.push(`heading jumps h${prev} to h${l}: ${h.textContent.trim().slice(0, 30)}`); prev = l; }
  for (const el of document.querySelectorAll('a[href],button')) { const name = (el.getAttribute('aria-label') || el.textContent || '').trim(); if (!name) out.push(`unnamed ${el.tagName.toLowerCase()} ${el.getAttribute('href') || ''}`); }
  for (const img of document.querySelectorAll('img')) if (!img.hasAttribute('alt')) out.push('img without alt');
  const seen = new Set();
  for (const el of document.querySelectorAll('main *, header *, footer *')) {
    if (!el.childNodes.length || ![...el.childNodes].some((n) => n.nodeType === 3 && n.textContent.trim())) continue;
    const cs = getComputedStyle(el); if (cs.visibility === 'hidden' || cs.display === 'none') continue;
    const fg = parse(cs.color); if (!fg) continue;
    const bg = bgOf(el); const f = blend(fg, bg);
    const L1 = lum(f), L2 = lum(bg); const ratio = (Math.max(L1, L2) + 0.05) / (Math.min(L1, L2) + 0.05);
    const size = parseFloat(cs.fontSize), bold = Number(cs.fontWeight) >= 700;
    const need = size >= 24 || (size >= 18.66 && bold) ? 3 : 4.5;
    if (ratio < need) { const k = el.tagName + el.className; if (!seen.has(k)) { seen.add(k); out.push(`low contrast ${ratio.toFixed(2)} (< ${need}) on <${el.tagName.toLowerCase()} class="${el.className}"> "${el.textContent.trim().slice(0, 24)}"`); } }
  }
  return out;
};
await step('accessibility: landmarks, headings, names, contrast, light and dark, every page', async () => {
  const bad = [];
  const sites = [['home', 8080, ['/', '/verify-site/', '/training/', '/training/arc42/', '/training/arc42/guide/', '/training/arc42/register/', '/training/arc42/example/', '/training/arc42/course/', '/training/arc42/course/part-0-opening/', '/training/arc42/course/part-7-6-runtime/']], ...PRODUCTS.map(([id, port]) => [id, port, ['/', '/demo/', '/download/', '/docs/', '/verify/', '/support/', '/verify-site/']])];
  for (const scheme of ['light', 'dark']) {
    for (const [id, port, paths] of sites) for (const path of paths) {
      const p = await open(`http://localhost:${port}${path}`, { viewport: { width: 1280, height: 900 }, colorScheme: scheme });
      for (const issue of await p.evaluate(AUDIT)) bad.push(`${scheme} ${id}${path}: ${issue}`);
      await p.context().close();
    }
  }
  assert.deepEqual(bad.slice(0, 12), [], `${bad.length} issue(s)`);
});
await step('every page has its own title, description and canonical link, with no JavaScript needed', async () => {
  const ctx = await browser.newContext({ javaScriptEnabled: false });
  const page = await ctx.newPage();
  const titles = new Set();
  for (const [id, port] of PRODUCTS) for (const path of ['/', '/demo/', '/download/', '/docs/', '/verify/', '/support/']) {
    await page.goto(`http://localhost:${port}${path}`);
    const title = await page.title();
    assert.ok(title && title !== 'Overview' && !titles.has(id + title), `${id}${path} title: ${title}`);
    titles.add(id + title);
    assert.ok((await page.locator('meta[name=description]').getAttribute('content')).length > 20, `${id}${path} description`);
    assert.match(await page.locator('link[rel=canonical]').getAttribute('href'), /^https:\/\/[a-z-]*\.?meeting-mind\.com\//, `${id}${path} canonical`);
  }
  await page.goto('http://localhost:8082/');
  assert.match(await page.locator('noscript').first().innerText(), /VeriCrypt/);
  await ctx.close();
});
await step('download tab with no binaries shows build-from-source; support shows known problems', async () => {
  for (const [id, port] of PRODUCTS) {
    let p = await open(`http://localhost:${port}/download/`);
    assert.match(await p.locator('#downloads-root').innerText(), /no packaged downloads/i, id);
    assert.ok(await p.locator('#downloads-root pre').count() >= 1, id + ' build steps');
    p = await open(`http://localhost:${port}/support/`);
    assert.ok(await p.locator('#support-faq details').count() >= 3, id + ' faq');
  }
});
await step('sitemap, robots and security.txt exist for every site', async () => {
  for (const port of [8080, ...PRODUCTS.map(([, x]) => x)]) {
    for (const f of ['/sitemap.xml', '/robots.txt', '/.well-known/security.txt']) {
      const r = await fetch(`http://localhost:${port}${f}`);
      assert.equal(r.status, 200, `${port}${f}`);
    }
  }
});

await step('verify this site: every file matches, and a tampered file is caught', async () => {
  for (const [id, port] of [['home', 8080], ...PRODUCTS]) {
    const p = await open(`http://localhost:${port}/verify-site/`);
    await p.waitForSelector('.callout[role=status]', { timeout: 15000 });
    assert.match(await p.locator('.callout[role=status]').innerText(), /^All \d+ files match/, id);
    assert.ok(!/\bnull\b|undefined/.test(await p.locator('main').innerText()), id + ' shows a stray null/undefined');
  }
  const ctx = await browser.newContext();
  const page = await ctx.newPage();
  await page.route('**/kit/dom.js', async (route) => { const r = await route.fetch(); await route.fulfill({ response: r, body: (await r.text()) + '\n// tampered' }); });
  await page.goto('http://localhost:8080/verify-site/');
  await page.waitForSelector('.callout[role=status]', { timeout: 15000 });
  assert.match(await page.locator('.callout[role=status]').innerText(), /do not match/);
  assert.match(await page.locator('table').innerText(), /kit\/dom\.js/);
  await ctx.close();
});
await step('every product overview lists honest limits', async () => {
  for (const [id, port] of PRODUCTS) {
    const p = await open(`http://localhost:${port}/`);
    assert.ok(await p.locator('ul.limits li').count() >= 3, id);
  }
});

await step('every HTML page is served with a Content-Security-Policy (strict, except legacy)', async () => {
  for (const [id, port] of [['home', 8080], ...PRODUCTS]) {
    const manifest = await (await fetch(`http://localhost:${port}/site-manifest.json`)).json();
    for (const file of Object.keys(manifest.files).filter((f) => f.endsWith('.html') && !f.startsWith('/legacy/'))) {
      const res = await fetch(`http://localhost:${port}${file}`);
      const csp = res.headers.get('content-security-policy') || '';
      assert.match(csp, /default-src 'self'/, `${id}${file} has no strict CSP`);
      assert.ok(!/unsafe-inline|(?<!wasm-)unsafe-eval/.test(csp), `${id}${file} CSP is loose`);
      assert.equal(res.headers.get('x-content-type-options'), 'nosniff', `${id}${file} nosniff`);
      if (!file.startsWith('/demo/')) assert.ok(!/wasm-unsafe-eval/.test(csp), `${id}${file} allows wasm outside /demo/`);
    }
  }
});

await step('docs: search finds pages, code blocks have copy buttons', async () => {
  const p = await open(VC + '/docs/');
  await p.fill('#docs-search', 'exit code');
  await p.waitForSelector('.docs-results li');
  assert.ok(await p.locator('.docs-results li').count() >= 1);
  await p.locator('.docs-results a').first().click();
  await p.waitForSelector('article.prose');
  assert.ok(await p.locator('.code-copy').count() >= 1, 'no copy button');
  await p.fill('#docs-search', 'zzzqqq');
  await p.waitForSelector('.docs-results .muted');
  assert.match(await p.locator('.docs-results').innerText(), /No matches/);
});
await step('structured data: valid JSON-LD on home and on every product overview', async () => {
  const check = async (url, type) => {
    const res = await fetch(url); const html = await res.text();
    const m = html.match(/<script type="application\/ld\+json">([\s\S]*?)<\/script>/);
    assert.ok(m, `${url} has no JSON-LD`);
    assert.equal(JSON.parse(m[1])['@type'], type, url);
  };
  await check(HOME + '/', 'Organization');
  for (const [, port] of PRODUCTS) await check(`http://localhost:${port}/`, 'SoftwareApplication');
});
await step('reflow: no horizontal scroll at 320px on any page (WCAG 1.4.10)', async () => {
  const bad = [];
  const sites = [['home', 8080, ['/', '/verify-site/', '/training/', '/training/arc42/', '/training/arc42/guide/', '/training/arc42/register/']], ...PRODUCTS.map(([id, port]) => [id, port, ['/', '/demo/', '/download/', '/docs/', '/verify/', '/support/']])];
  for (const [id, port, paths] of sites) for (const path of paths) {
    const p = await open(`http://localhost:${port}${path}`, { viewport: { width: 320, height: 640 } });
    if (await p.evaluate(() => document.documentElement.scrollWidth > document.documentElement.clientWidth)) bad.push(`${id}${path}`);
    await p.context().close();
  }
  assert.deepEqual(bad, []);
});
await step('performance budget: CLS under 0.1, LCP under 2.5 s, under 150 KB per page plus at most 100 KB of cached fonts (local, uncached)', async () => {
  const problems = [];
  const sites = [['home', 8080, ['/', '/training/', '/training/arc42/', '/training/arc42/guide/', '/training/arc42/register/']], ...PRODUCTS.map(([id, port]) => [id, port, ['/', '/demo/', '/docs/', '/download/', '/verify/', '/support/']])];
  for (const [id, port, paths] of sites) for (const path of paths) {
    const ctx = await browser.newContext({ viewport: { width: 412, height: 800 } });
    const page = await ctx.newPage();
    await page.addInitScript(() => {
      window.__cls = 0; window.__lcp = 0;
      new PerformanceObserver((l) => { for (const e of l.getEntries()) if (!e.hadRecentInput) window.__cls += e.value; }).observe({ type: 'layout-shift', buffered: true });
      new PerformanceObserver((l) => { for (const e of l.getEntries()) window.__lcp = e.startTime; }).observe({ type: 'largest-contentful-paint', buffered: true });
    });
    await page.goto(`http://localhost:${port}${path}`);
    await page.waitForSelector('html[data-ready="true"]');
    await page.waitForTimeout(400);
    const m = await page.evaluate(() => ({ cls: window.__cls, lcp: window.__lcp, bytes: performance.getEntriesByType('resource').filter((r) => !/\.woff2/.test(r.name)).reduce((n, r) => n + (r.transferSize || r.encodedBodySize || 0), 0), fontBytes: performance.getEntriesByType('resource').filter((r) => /\.woff2/.test(r.name)).reduce((n, r) => n + (r.transferSize || r.encodedBodySize || 0), 0) }));
    if (m.fontBytes > 100000) problems.push(`${id}${path} fonts ${Math.round(m.fontBytes / 1000)} KB`);
    if (m.cls > 0.1) problems.push(`${id}${path} CLS ${m.cls.toFixed(3)}`);
    if (m.lcp > 2500) problems.push(`${id}${path} LCP ${Math.round(m.lcp)} ms`);
    if (m.bytes > 150000) problems.push(`${id}${path} ${Math.round(m.bytes / 1000)} KB`);
    await ctx.close();
  }
  assert.deepEqual(problems, []);
});

// ---- training section ----
import fs from 'node:fs';
const WS = '/training/arc42/';
await step('training: home links to it, pages render with content and no console errors', async () => {
  let p = await open(HOME);
  assert.ok(await p.locator('.bar-nav a[href="/training/"]').count() === 1, 'no Training link in the header');
  assert.ok(await p.locator('#training a[href="/training/"]').count() === 1, 'no Training block on home');
  p = await open(HOME + '/training/');
  assert.ok(await p.locator('.ws-card a[href="/training/arc42/"]').count() >= 1, 'training index does not link to the workshop');
  assert.deepEqual(p.problems, []);
  p = await open(HOME + WS);
  assert.equal(await p.locator('ol.twelve li').count(), 12);
  assert.ok(await p.locator('ol.day > li:not(.grp):not(.brk)').count() >= 10);
  assert.match(await p.locator('.t-hero').innerText(), /30 January 2027/);
  assert.deepEqual(p.problems, []);
  p = await open('http://localhost:8082/');
  await p.locator('.family summary').click();
  assert.ok(await p.locator('.family a', { hasText: 'Training' }).count() === 1, 'product family menu has no Training link');
});
await step('training: the content is in the HTML itself (no JavaScript needed)', async () => {
  const ctx = await browser.newContext({ javaScriptEnabled: false });
  const page = await ctx.newPage();
  await page.goto(HOME + WS);
  assert.match(await page.locator('h1').innerText(), /architecture docs people and AI can act on/);
  assert.ok(await page.locator('ol.day li').count() >= 15);
  await page.goto(HOME + WS + 'register/');
  assert.match(await page.locator('a[href^="mailto:"]').first().getAttribute('href'), /^mailto:dpramsajan@meeting-mind\.com\?subject=/);
  await ctx.close();
});
await step('training: early bird counts down, then ends on its own', async () => {
  for (const [now, re] of [['2026-11-21T12:00:00Z', /10 days left/], ['2026-12-02T12:00:00Z', /early-bird price has ended/i]]) {
    const ctx = await browser.newContext();
    const page = await ctx.newPage();
    await page.clock.install({ time: new Date(now) });
    await page.goto(HOME + WS);
    await page.waitForSelector('html[data-ready="true"]');
    assert.match(await page.locator('.t-pill').innerText(), re, now);
    await ctx.close();
  }
});
await step('training: Event structured data is valid and matches the page', async () => {
  const html = await (await fetch(HOME + WS)).text();
  const ld = JSON.parse(html.match(/<script type="application\/ld\+json">([\s\S]*?)<\/script>/)[1]);
  assert.equal(ld['@type'], 'EducationEvent');
  assert.match(ld.startDate, /^2027-01-30T09:00/);
  assert.equal(ld.offers.priceCurrency, 'TTD');
  assert.ok(new Date(ld.offers.validThrough) < new Date(ld.startDate));
  const sitemap = await (await fetch(HOME + '/sitemap.xml')).text();
  for (const u of ['/training/', '/training/arc42/', '/training/arc42/guide/', '/training/arc42/register/']) assert.ok(sitemap.includes(`https://meeting-mind.com${u}<`), u);
});
await step('training: guide ticks are remembered, copy button works', async () => {
  const ctx = await browser.newContext({ permissions: ['clipboard-read', 'clipboard-write'] });
  const page = await ctx.newPage();
  await page.goto(HOME + WS + 'guide/');
  await page.waitForSelector('html[data-ready="true"]');
  await page.locator('input[data-check="setup-1"]').check();
  await page.reload();
  await page.waitForSelector('html[data-ready="true"]');
  assert.equal(await page.locator('input[data-check="setup-1"]').isChecked(), true);
  await page.locator('button[data-copy]').click();
  assert.match(await page.evaluate(() => navigator.clipboard.readText()), /## 12\. Glossary/);
  await ctx.close();
});
await step('training: registration form builds an email to the company address and shows no personal mailbox', async () => {
  const p = await open(HOME + WS + 'register/');
  assert.equal(await p.locator('#reserve').isVisible(), true);
  assert.equal(await p.locator('#reserve').getAttribute('data-to'), 'dpramsajan@meeting-mind.com');
  assert.ok(!/gmail\.com/.test(await p.content()), 'a personal mailbox is in the page');
});
await step('training: the slides are private (unlisted, noindex, strict CSP) and run', async () => {
  const dirs = fs.readdirSync('site/training/arc42').filter((d) => d.startsWith('deck-'));
  assert.equal(dirs.length, 1, 'expected exactly one deck-* folder');
  const url = `${HOME}${WS}${dirs[0]}/`;
  const manifest = await (await fetch(HOME + '/site-manifest.json')).text();
  const sitemap = await (await fetch(HOME + '/sitemap.xml')).text();
  const robots = await (await fetch(HOME + '/robots.txt')).text();
  for (const [name, text] of [['manifest', manifest], ['sitemap', sitemap], ['robots.txt', robots]]) assert.ok(!text.includes('deck-'), `the slides path leaks into ${name}`);
  for (const f of ['site/index.html', 'site/training/index.html', 'site/training/arc42/index.html', 'site/training/arc42/guide/index.html', 'site/training/arc42/register/index.html', 'site/training/training.json']) assert.ok(!fs.readFileSync(f, 'utf8').includes('deck-'), `${f} links to the slides`);
  const res = await fetch(url);
  assert.match(res.headers.get('x-robots-tag') || '', /noindex/);
  const csp = res.headers.get('content-security-policy') || '';
  assert.match(csp, /default-src 'self'/);
  assert.ok(!/unsafe-inline|unsafe-eval/.test(csp));
  const ctx = await browser.newContext({ viewport: { width: 1280, height: 720 } });
  const page = await ctx.newPage();
  const bad = [];
  page.on('console', (m) => { if (m.type() === 'error' || /Content Security Policy/i.test(m.text())) bad.push(m.text()); });
  page.on('pageerror', (e) => bad.push(e.message));
  await page.goto(url);
  await page.waitForSelector('#stage .s');
  assert.ok(await page.locator('#ovg button').count() > 100, 'slides did not load');
  const k = await page.evaluate(() => SL.findIndex((s) => /@@/.test(s.h)));
  await page.goto(`${url}#${k + 1}`); await page.reload(); await page.waitForTimeout(300);
  assert.ok(await page.locator('#stage svg text.nt').count() > 0, 'diagram slide did not render');
  assert.deepEqual(bad, []);
  await ctx.close();
});

/* ---------- online course ---------- */
const CRS = HOME + WS + 'course/';
const { loadKey } = await import('./build-course.mjs');
const courseKey = loadKey({ create: false });
const mkCode = (name, keyBuf = courseKey) => Buffer.from(JSON.stringify({ v: 1, b: name, k: keyBuf.toString('base64url') })).toString('base64url');
await step('course: index lists the lessons, two free, and loads without errors or CSP violations', async () => {
  assert.ok(courseKey, 'no course key: run npm run course first');
  const p = await open(CRS);
  assert.equal(await p.locator('li.lesson').count(), 19);
  assert.equal(await p.locator('li.lesson.is-free').count(), 2);
  assert.ok(await p.locator('form[data-unlock]').isVisible());
  assert.deepEqual(p.problems, []);
  await p.context().close();
});
await step('course: a free lesson plays without a code (next, back, slide changes)', async () => {
  const p = await open(CRS + 'part-1-foundations/');
  await p.waitForSelector('[data-stage] .s');
  const first = await p.locator('[data-stage]').innerText();
  await p.click('[data-next]');
  assert.notEqual(await p.locator('[data-stage]').innerText(), first);
  assert.match(await p.locator('[data-pos]').innerText(), /^2 \/ 15$/);
  await p.keyboard.press('ArrowLeft');
  assert.match(await p.locator('[data-pos]').innerText(), /^1 \/ 15$/);
  const box = await p.locator('[data-frame]').boundingBox();
  assert.ok(box.width > 300 && Math.abs(box.height / box.width - 9 / 16) < 0.01, 'stage is not 16:9');
  assert.deepEqual(p.problems, []);
  await p.context().close();
});
await step('course: a paid lesson is locked without a code and its file holds no readable content', async () => {
  const p = await open(CRS + 'part-7-6-runtime/');
  await p.waitForSelector('[data-locked]:not([hidden])');
  assert.equal(await p.locator('[data-stage-wrap]').isHidden(), true);
  const enc = await (await fetch(CRS + 'data/part-7-6-runtime.json')).json();
  assert.deepEqual(Object.keys(enc).sort(), ['ct', 'id', 'iv', 'v']);
  assert.ok(!/<h2|Runtime|scenario/i.test(Buffer.from(enc.ct, 'base64url').toString('latin1')), 'the paid lesson file contains readable text');
  assert.ok(!(await p.content()).includes('Facilitator note'), 'paid text is in the page');
  await p.context().close();
});
await step('course: pasting a code on the course page shows a clear "you are in" banner with a start button', async () => {
  const ctx = await browser.newContext();
  const page = await ctx.newPage();
  await page.goto(CRS);
  await page.waitForSelector('form[data-unlock]:not([hidden])');
  await page.fill('form[data-unlock] input', mkCode('Banner Buyer'));
  await page.click('form[data-unlock] [type=submit]');
  await page.waitForSelector('.unlock-banner');
  const t = await page.locator('.unlock-banner').innerText();
  assert.match(t, /You are in, Banner Buyer/);
  assert.match(t, /Start lesson 1/);
  assert.ok(await page.locator('.unlock-banner a.btn').isVisible());
  await ctx.close();
});
await step('workshop page: the twelve-section card shows the question for the section you tap, and lines up with the headline', async () => {
  const page = await open(HOME + WS);
  await page.locator('.sheet .cell').nth(2).click();
  assert.match(await page.locator('[data-sheet-q]').innerText(), /Who do we talk to/);
  const h = await page.locator('.t-pill').boundingBox(), c = await page.locator('.sheet').boundingBox();
  assert.ok(Math.abs(h.y - c.y) < 40, `card top ${c.y} vs pill top ${h.y}`);
  await page.context().close();
});
await step('course: a wrong code is refused, the right code unlocks every lesson and shows the buyer name', async () => {
  const ctx = await browser.newContext();
  const page = await ctx.newPage();
  const errs = []; page.on('pageerror', (e) => errs.push(e.message));
  await page.goto(CRS + 'part-7-6-runtime/');
  await page.waitForSelector('[data-locked]:not([hidden])');
  await page.fill('[data-locked] input', 'not-a-code');
  await page.click('[data-locked] [type=submit]');
  assert.match(await page.locator('[data-locked] [data-unlock-msg]').innerText(), /does not look like/);
  await page.fill('[data-locked] input', mkCode('Wrong Key', Buffer.alloc(32, 7)));
  await page.click('[data-locked] [type=submit]');
  await page.waitForFunction(() => /does not unlock/.test(document.querySelector('[data-locked] [data-unlock-msg]').textContent));
  await page.fill('[data-locked] input', mkCode('Test Buyer'));
  await page.click('[data-locked] [type=submit]');
  await page.waitForSelector('[data-stage] .s');
  assert.match(await page.locator('[data-licence]').innerText(), /Test Buyer/);
  await page.goto(CRS + 'part-14-agents-as-readers/');
  await page.waitForSelector('[data-stage] .s');
  assert.match(await page.locator('[data-pos]').innerText(), /^1 \/ 16$/);
  // a diagram slide renders in a paid lesson
  await page.goto(CRS + 'part-9b-see-it-run/');
  await page.waitForSelector('[data-stage] .s');
  await page.click('[data-next]');
  assert.ok(await page.locator('[data-stage] svg text').count() > 0, 'diagram did not render');
  // forgetting the code locks it again
  await page.goto(CRS);
  await page.click('[data-forget]');
  await page.waitForSelector('html[data-ready="true"]');
  await page.goto(CRS + 'part-7-6-runtime/');
  await page.waitForSelector('[data-locked]:not([hidden])');
  assert.deepEqual(errs, []);
  await ctx.close();
});
await step('course: a tampered lesson file is rejected, not shown', async () => {
  const ctx = await browser.newContext();
  const page = await ctx.newPage();
  await page.route('**/course/data/part-7-6-runtime.json', async (route) => {
    const r = await route.fetch(); const j = await r.json();
    j.ct = (j.ct[0] === 'A' ? 'B' : 'A') + j.ct.slice(1);
    await route.fulfill({ json: j });
  });
  await page.addInitScript((c) => localStorage.setItem('iw-course-code', c), mkCode('Test Buyer'));
  await page.goto(CRS + 'part-7-6-runtime/');
  await page.waitForSelector('[data-locked]:not([hidden])');
  assert.equal(await page.locator('[data-stage-wrap]').isHidden(), true);
  await ctx.close();
});
await step('course: finishing a lesson offers recall cards, and a missed card returns for review the next day', async () => {
  const ctx = await browser.newContext();
  const page = await ctx.newPage();
  await page.clock.install({ time: new Date('2026-12-10T12:00:00Z') });
  await page.goto(CRS + 'part-0-opening/');
  await page.waitForSelector('[data-stage] .s');
  const n = Number((await page.locator('[data-pos]').innerText()).split('/')[1]);
  for (let i = 0; i < n; i++) await page.click('[data-next]');
  await page.waitForSelector('[data-recall]:not([hidden])');
  assert.ok(await page.locator('.rcard').count() >= 3, 'no recall cards');
  await page.locator('.rcard').first().locator('button', { hasText: 'Show the answer' }).click();
  await page.locator('.rcard').first().locator('button', { hasText: 'Not yet' }).click();
  await page.goto(CRS);
  assert.equal(await page.locator('[data-due]').isHidden(), true, 'a card due tomorrow is due today');
  await page.clock.setFixedTime(new Date('2026-12-12T12:00:00Z'));
  await page.reload();
  await page.waitForSelector('html[data-ready="true"]');
  assert.equal(await page.locator('[data-due]').isVisible(), true);
  assert.match(await page.locator('li.lesson[data-lesson="part-0-opening"] .badge').innerText(), /done/i);
  await ctx.close();
});
await step('course: lessons stay inside the screen at 320px', async () => {
  const p = await open(CRS + 'part-1-foundations/', { viewport: { width: 320, height: 700 } });
  await p.waitForSelector('[data-stage] .s');
  assert.equal(await p.evaluate(() => document.documentElement.scrollWidth > document.documentElement.clientWidth), false);
  await p.context().close();
});
await step('course: the slides folder stays private and the course pages do not link to it', async () => {
  for (const f of fs.readdirSync('site/training/arc42/course', { recursive: true })) {
    const full = 'site/training/arc42/course/' + f;
    if (fs.statSync(full).isFile() && /\.(html|js|css)$/.test(f)) assert.ok(!fs.readFileSync(full, 'utf8').includes('deck-'), `${full} links to the slides`);
  }
});
await step('worked example: page, zip and arc42 are served and the zip matches the source', async () => {
  const p = await open(HOME + WS + 'example/');
  assert.ok(await p.locator('a[href$="doubles-to-go.zip"]').count());
  const z = await fetch(HOME + WS + 'example/doubles-to-go.zip');
  assert.equal(z.status, 200);
  assert.ok((await z.arrayBuffer()).byteLength > 20000);
  assert.match(await (await fetch(HOME + WS + 'example/arc42.md')).text(), /^# Doubles-to-Go/);
  await p.context().close();
});

await browser.close();
stop();
console.log(failed ? `\n${failed} failed` : '\nall browser checks passed');
process.exit(failed ? 1 : 0);
