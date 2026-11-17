#!/usr/bin/env node
// Regenerates site/kit/MANIFEST.json, then copies the kit and family.json into product sites.
// Usage: node tools/sync-kit.mjs                 (manifest only, plus every products/*/site)
//        node tools/sync-kit.mjs <siteDir> ...   (also sync the given site folders, e.g. a product repo's site/)

import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';

const kitSrc = path.resolve('site/kit');
const familySrc = path.resolve('site/family.json');
const sha = (file) => crypto.createHash('sha256').update(fs.readFileSync(file)).digest('hex');

function writeManifest(kitDir) {
  const files = {};
  for (const f of fs.readdirSync(kitDir).filter((n) => n !== 'MANIFEST.json').sort()) files[f] = sha(path.join(kitDir, f));
  const version = fs.readFileSync(path.join(kitDir, 'VERSION'), 'utf8').trim();
  fs.writeFileSync(path.join(kitDir, 'MANIFEST.json'), JSON.stringify({ version, files }, null, 2) + '\n');
}

writeManifest(kitSrc);
console.log('kit manifest updated');

const targets = process.argv.slice(2);
if (!targets.length && fs.existsSync('products')) {
  for (const p of fs.readdirSync('products').sort()) if (fs.existsSync(path.join('products', p, 'site'))) targets.push(path.join('products', p, 'site'));
}

for (const target of targets) {
  const kitDest = path.join(path.resolve(target), 'kit');
  fs.rmSync(kitDest, { recursive: true, force: true });
  fs.mkdirSync(kitDest, { recursive: true });
  for (const f of fs.readdirSync(kitSrc)) fs.copyFileSync(path.join(kitSrc, f), path.join(kitDest, f));
  fs.copyFileSync(familySrc, path.join(path.resolve(target), 'family.json'));
  console.log(`synced kit and family.json -> ${target}`);
}
