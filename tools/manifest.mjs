#!/usr/bin/env node
// Writes site-manifest.json: the SHA-256 of every file a site serves, plus one root hash.
// The "Verify this site" page re-hashes each file in the visitor's browser and compares.
// Usage: node tools/manifest.mjs [siteDir ...]   (default: site + products/*/site)
import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';

let dirs = process.argv.slice(2);
if (!dirs.length) {
  dirs = ['site'];
  if (fs.existsSync('products')) for (const p of fs.readdirSync('products').sort()) if (fs.existsSync(path.join('products', p, 'site'))) dirs.push(path.join('products', p, 'site'));
}
const NOT_SERVED = new Set(['_headers', '_redirects', 'site-manifest.json', '.DS_Store']);
const sha = (buf) => crypto.createHash('sha256').update(buf).digest('hex');

for (const dir of dirs) {
  const root = path.resolve(dir);
  const files = {};
  const visit = (d) => {
    for (const e of fs.readdirSync(d, { withFileTypes: true })) {
      const p = path.join(d, e.name);
      if (e.isDirectory()) { if (e.name !== 'node_modules' && !e.name.startsWith('deck-')) visit(p); continue; } // deck-* folders are unlisted on purpose: the manifest is public
      if (NOT_SERVED.has(e.name)) continue;
      const rel = '/' + path.relative(root, p).split(path.sep).join('/');
      files[rel] = sha(fs.readFileSync(p));
    }
  };
  visit(root);
  const sorted = Object.fromEntries(Object.entries(files).sort(([a], [b]) => (a < b ? -1 : 1)));
  const rootHash = sha(Buffer.from(Object.entries(sorted).map(([f, h]) => `${f}\t${h}\n`).join('')));
  const kit = fs.existsSync(path.join(root, 'kit', 'VERSION')) ? fs.readFileSync(path.join(root, 'kit', 'VERSION'), 'utf8').trim() : '';
  fs.writeFileSync(path.join(root, 'site-manifest.json'), JSON.stringify({ format: 1, kit, root: rootHash, files: sorted }, null, 2) + '\n');
  console.log(`manifest ${dir}: ${Object.keys(sorted).length} files, root ${rootHash.slice(0, 16)}…`);
}
