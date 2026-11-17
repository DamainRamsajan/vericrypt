#!/usr/bin/env node
// Scaffold a product site from templates/product-site.
// Usage: node tools/new-product.mjs <id> [--out=products/<id>/site]
// Values come from templates/prefill/<id>.json (name, accent, tagline, audience, highlights, ...).

import fs from 'node:fs';
import path from 'node:path';
import { execFileSync } from 'node:child_process';

const id = process.argv[2];
if (!id) { console.error('Usage: node tools/new-product.mjs <id> [--out=dir] [--force]'); process.exit(1); }
const outArg = process.argv.find((a) => a.startsWith('--out='));
const out = path.resolve(outArg ? outArg.slice(6) : path.join('products', id, 'site'));
const prefillFile = path.join('templates', 'prefill', `${id}.json`);
if (!fs.existsSync(prefillFile)) { console.error(`No prefill file: ${prefillFile}`); process.exit(1); }

if (fs.existsSync(path.join(out, 'site.json')) && !process.argv.includes('--force')) {
  console.error(`${out} already has a site.json. Refusing to overwrite finished work. Re-run with --force to regenerate it.`);
  process.exit(1);
}
fs.mkdirSync(out, { recursive: true });
fs.cpSync(path.join('templates', 'product-site'), out, { recursive: true });
fs.copyFileSync(path.join('site', 'assets', 'favicon.svg'), path.join(out, 'assets', 'favicon.svg'));
fs.copyFileSync(path.join('site', '_headers'), path.join(out, '_headers'));

const base = JSON.parse(fs.readFileSync(path.join(out, 'site.json'), 'utf8'));
const prefill = JSON.parse(fs.readFileSync(prefillFile, 'utf8'));
const merged = { ...base, ...prefill, id };
fs.writeFileSync(path.join(out, 'site.json'), JSON.stringify(merged, null, 2) + '\n');

// product sites do not have a /legacy section: drop that rule from their copy of _headers
const headers = fs.readFileSync(path.join(out, '_headers'), 'utf8').split('\n\n').filter((b) => !b.trim().startsWith('/legacy/*')).join('\n\n');
fs.writeFileSync(path.join(out, '_headers'), headers.endsWith('\n') ? headers : headers + '\n');

const docsOverride = path.join('templates', 'prefill-docs', id);
if (fs.existsSync(docsOverride)) fs.cpSync(docsOverride, out, { recursive: true });

execFileSync('node', ['tools/sync-kit.mjs', out], { stdio: 'inherit' });
execFileSync('node', ['tools/prerender.mjs', out], { stdio: 'inherit' });
execFileSync('node', ['tools/manifest.mjs', out], { stdio: 'inherit' });
console.log(`Scaffolded ${merged.name} -> ${path.relative(process.cwd(), out)}`);
