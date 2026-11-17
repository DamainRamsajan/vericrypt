#!/usr/bin/env node
// Copies the existing pages from the old web/ folder into site/legacy/ so the sector pages,
// decks and data rooms keep working at meeting-mind.com/legacy/... after cutover.
// Usage: npm run legacy -- <path-to-old-web-folder>

import fs from 'node:fs';
import path from 'node:path';

if (!process.argv[2]) { console.error('Usage: npm run legacy -- <path-to-old-web-folder>'); process.exit(1); }
const src = path.resolve(process.argv[2]);
const dest = path.resolve('site/legacy');
if (!fs.existsSync(src)) { console.error(`Not found: ${src}`); process.exit(1); }

const SKIP = new Set(['apply-recording-fix.py', 'slidesbroken.html']);
fs.mkdirSync(dest, { recursive: true });
let n = 0;
for (const entry of fs.readdirSync(src, { withFileTypes: true })) {
  if (SKIP.has(entry.name) || entry.name.endsWith('.py') || entry.name.startsWith('.')) continue;
  fs.cpSync(path.join(src, entry.name), path.join(dest, entry.name), { recursive: true });
  n++;
}
console.log(`Copied ${n} item(s) from ${src} to ${dest}`);
console.log('Legacy pages are served with a looser Content-Security-Policy (see site/_headers).');
console.log('Their links to intellica-15c.pages.dev still work; update them as pages are rebuilt.');
