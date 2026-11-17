#!/usr/bin/env node
// Checkpoint gate for one or more site folders.
// Usage: node tools/check.mjs [siteDir ...]     (default: site)
//        node tools/check.mjs --all             (site + every products/*/site)
//
// Fails (exit 1) when a page would break the contract: inline script/style, third-party
// resource loads, missing local files, malformed site.json / downloads.json / docs.json,
// or a kit that differs from its MANIFEST (someone hand-edited a copy).

import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';

const args = process.argv.slice(2);
let dirs = args.filter((a) => !a.startsWith('--'));
if (args.includes('--all')) {
  dirs = ['site'];
  if (fs.existsSync('products')) for (const p of fs.readdirSync('products').sort()) if (fs.existsSync(path.join('products', p, 'site'))) dirs.push(path.join('products', p, 'site'));
}
if (!dirs.length) dirs = ['site'];

let errors = 0;
let warnings = 0;
const fail = (dir, msg) => { errors++; console.log(`  ERROR  ${msg}`); };
const warn = (dir, msg) => { warnings++; console.log(`  warn   ${msg}`); };

const walk = (dir) => fs.readdirSync(dir, { withFileTypes: true }).flatMap((e) =>
  e.isDirectory() ? walk(path.join(dir, e.name)) : [path.join(dir, e.name)]);

const EXTERNAL = /^(https?:)?\/\//i;
const sha = (file) => crypto.createHash('sha256').update(fs.readFileSync(file)).digest('hex');

function resolveRef(siteDir, fromFile, ref) {
  const clean = ref.split('#')[0].split('?')[0];
  if (!clean) return null;
  let target = clean.startsWith('/') ? path.join(siteDir, clean) : path.join(path.dirname(fromFile), clean);
  if (clean.endsWith('/')) target = path.join(target, 'index.html');
  return target;
}

function checkHtml(siteDir, file) {
  const rel = path.relative(siteDir, file);
  const src = fs.readFileSync(file, 'utf8');
  if (/<style[\s>]/i.test(src)) fail(siteDir, `${rel}: inline <style> is not allowed (CSP). Move it to a .css file.`);
  for (const m of src.matchAll(/<script\b([^>]*)>([\s\S]*?)<\/script>/gi)) {
    if (!/\bsrc\s*=/.test(m[1]) && m[2].trim() && !/type\s*=\s*"application\/ld\+json"/i.test(m[1])) fail(siteDir, `${rel}: inline <script> is not allowed (CSP). Move it to a .js file.`);
  }
  if (/\son[a-z]+\s*=/i.test(src.replace(/<script[\s\S]*?<\/script>/gi, ''))) fail(siteDir, `${rel}: inline event handler attribute (onclick= etc.) is not allowed.`);
  if (/<[a-z][^>]*\sstyle\s*=/i.test(src)) fail(siteDir, `${rel}: inline style="" attribute is not allowed (CSP). Use a class.`);

  for (const m of src.matchAll(/<(script|link|img|source|iframe|video|audio)\b([^>]*)>/gi)) {
    if (m[1].toLowerCase() === 'link' && /\brel\s*=\s*"(canonical|alternate)"/i.test(m[2])) continue; // a pointer, not a load
    for (const a of m[2].matchAll(/\b(src|href)\s*=\s*"([^"]*)"/gi)) {
      const ref = a[2];
      if (EXTERNAL.test(ref)) { fail(siteDir, `${rel}: <${m[1]}> loads a third-party resource: ${ref}`); continue; }
      const target = resolveRef(siteDir, file, ref);
      if (target && !fs.existsSync(target)) fail(siteDir, `${rel}: <${m[1]}> points to a missing file: ${ref}`);
    }
  }
  for (const m of src.matchAll(/<a\b[^>]*\shref\s*=\s*"([^"#?][^"]*)"/gi)) {
    const ref = m[1];
    if (EXTERNAL.test(ref) || /^(mailto:|tel:)/i.test(ref)) continue;
    const target = resolveRef(siteDir, file, ref);
    if (target && !fs.existsSync(target) && !fs.existsSync(target + '.html')) warn(siteDir, `${rel}: link to a missing page: ${ref}`);
  }
}

function checkCss(siteDir, file) {
  const rel = path.relative(siteDir, file);
  const src = fs.readFileSync(file, 'utf8');
  if (/@import\s+(url\()?["']?(https?:)?\/\//i.test(src)) fail(siteDir, `${rel}: @import of a third-party stylesheet.`);
  for (const m of src.matchAll(/url\(\s*["']?([^"')]+)["']?\s*\)/gi)) {
    const ref = m[1];
    if (ref.startsWith('data:')) continue;
    if (EXTERNAL.test(ref)) { fail(siteDir, `${rel}: url() loads a third-party resource: ${ref}`); continue; }
    const target = resolveRef(siteDir, file, ref);
    if (target && !fs.existsSync(target)) fail(siteDir, `${rel}: url() points to a missing file: ${ref}`);
  }
}

function checkJs(siteDir, file) {
  const rel = path.relative(siteDir, file);
  const src = fs.readFileSync(file, 'utf8');
  for (const m of src.matchAll(/(?:from\s+|import\s*\(\s*)['"]([^'"]+)['"]/g)) {
    const ref = m[1];
    if (EXTERNAL.test(ref)) { fail(siteDir, `${rel}: imports from a third-party URL: ${ref}`); continue; }
    if (ref.startsWith('.') || ref.startsWith('/')) {
      const target = resolveRef(siteDir, file, ref);
      if (target && !fs.existsSync(target)) fail(siteDir, `${rel}: import points to a missing file: ${ref}`);
    }
  }
  if (/\beval\s*\(|new Function\s*\(/.test(src)) fail(siteDir, `${rel}: eval/new Function is not allowed.`);
}

function readJson(siteDir, file) {
  try { return JSON.parse(fs.readFileSync(file, 'utf8')); } catch (e) { fail(siteDir, `${path.relative(siteDir, file)}: invalid JSON (${e.message})`); return null; }
}

const TAB_NAMES = ['overview', 'demo', 'download', 'docs', 'verify', 'support'];

function checkSiteJson(siteDir) {
  const file = path.join(siteDir, 'site.json');
  if (!fs.existsSync(file)) return fail(siteDir, 'site.json is missing.');
  const s = readJson(siteDir, file);
  if (!s) return;
  for (const k of ['id', 'name', 'tagline', 'accent']) if (!s[k]) fail(siteDir, `site.json: "${k}" is required.`);
  if (s.accent && !/^#[0-9a-fA-F]{6}$/.test(s.accent)) fail(siteDir, 'site.json: "accent" must be a #RRGGBB colour.');
  if (!Array.isArray(s.tabs)) fail(siteDir, 'site.json: "tabs" must be an array.');
  else for (const t of s.tabs) {
    if (!TAB_NAMES.includes(t)) { fail(siteDir, `site.json: unknown tab "${t}".`); continue; }
    const page = t === 'overview' ? 'index.html' : path.join(t, 'index.html');
    if (!fs.existsSync(path.join(siteDir, page))) fail(siteDir, `site.json lists tab "${t}" but ${page} does not exist.`);
  }
  for (const k of ['repo', 'contact']) if (!s[k]) warn(siteDir, `site.json: "${k}" is empty.`);

  // Honesty guards for product sites.
  if (s.id !== 'intellica') {
    if (!s.status) warn(siteDir, 'site.json: "status" is empty; say where the product stands.');
    if (!Array.isArray(s.limits) || !s.limits.length) warn(siteDir, 'site.json: "limits" is empty; list what the product does not do yet.');
    else if (s.limits.some((x) => typeof x !== 'string' || !x.trim())) fail(siteDir, 'site.json: every "limits" entry must be a non-empty string.');
    const demoFile = path.join(siteDir, 'demo', 'demo.js');
    const engine = s.demo && s.demo.engine;
    if (engine && !['wasm', 'trace', 'placeholder'].includes(engine)) fail(siteDir, `site.json: demo.engine must be wasm, trace or placeholder (got "${engine}").`);
    if (fs.existsSync(demoFile)) {
      const usesPlaceholder = /placeholder-demo/.test(fs.readFileSync(demoFile, 'utf8'));
      if (usesPlaceholder && engine && engine !== 'placeholder') fail(siteDir, `demo/demo.js is still the placeholder but site.json says demo.engine is "${engine}". The page would misdescribe the demo.`);
      if (!usesPlaceholder && engine === 'placeholder') warn(siteDir, 'demo/demo.js is no longer the placeholder: set demo.engine to "wasm" or "trace" in site.json so the label is honest.');
    }
  }
}

function checkDownloads(siteDir) {
  const file = path.join(siteDir, 'download', 'downloads.json');
  if (!fs.existsSync(file)) return;
  const d = readJson(siteDir, file);
  if (!d) return;
  if (!Array.isArray(d.items)) return fail(siteDir, 'download/downloads.json: "items" must be an array.');
  d.items.forEach((it, i) => {
    for (const k of ['name', 'version', 'platform', 'file', 'sha256']) if (!it[k]) fail(siteDir, `download/downloads.json: item ${i + 1} is missing "${k}".`);
    if (it.sha256 && !/^[0-9a-f]{64}$/.test(it.sha256)) fail(siteDir, `download/downloads.json: item ${i + 1} sha256 must be 64 lowercase hex characters.`);
  });
}

function checkDocs(siteDir) {
  const file = path.join(siteDir, 'docs', 'docs.json');
  if (!fs.existsSync(file)) return;
  const d = readJson(siteDir, file);
  if (!d) return;
  if (!Array.isArray(d.groups)) return fail(siteDir, 'docs/docs.json: "groups" must be an array.');
  for (const g of d.groups) for (const it of g.items || []) {
    if (!it.title || !it.md) { fail(siteDir, `docs/docs.json: an item in "${g.title}" needs title and md.`); continue; }
    if (!fs.existsSync(path.join(siteDir, 'docs', it.md))) fail(siteDir, `docs/docs.json: ${it.md} does not exist.`);
  }
}

function checkKit(siteDir) {
  const manifestFile = path.join(siteDir, 'kit', 'MANIFEST.json');
  if (!fs.existsSync(manifestFile)) return warn(siteDir, 'kit/MANIFEST.json is missing. Run: npm run sync');
  const m = readJson(siteDir, manifestFile);
  if (!m) return;
  const kitDir = path.join(siteDir, 'kit');
  const actual = fs.readdirSync(kitDir).filter((f) => f !== 'MANIFEST.json').sort();
  const expected = Object.keys(m.files).sort();
  if (JSON.stringify(actual) !== JSON.stringify(expected)) fail(siteDir, `kit/ files differ from MANIFEST.json. Run: npm run sync`);
  for (const f of expected) if (fs.existsSync(path.join(kitDir, f)) && sha(path.join(kitDir, f)) !== m.files[f]) fail(siteDir, `kit/${f} was edited by hand or is stale. Edit it in the Intellica repo, then run: npm run sync`);
}

for (const dir of dirs) {
  console.log(`\nChecking ${dir}`);
  if (!fs.existsSync(dir)) { fail(dir, 'folder does not exist.'); continue; }
  const before = errors;
  const files = walk(dir).filter((f) => !f.includes(`${path.sep}legacy${path.sep}`));
  for (const f of files) {
    if (f.endsWith('.html')) checkHtml(dir, f);
    else if (f.endsWith('.css')) checkCss(dir, f);
    else if (f.endsWith('.js')) checkJs(dir, f);
  }
  checkSiteJson(dir);
  checkDownloads(dir);
  checkDocs(dir);
  checkKit(dir);
  console.log(errors === before ? '  ok' : '');
}

console.log(`\n${errors} error(s), ${warnings} warning(s)`);
process.exit(errors ? 1 : 0);
