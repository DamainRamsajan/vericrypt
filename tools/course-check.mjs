// Checks an access code against the paid lesson files, locally or on the live site.
//   npm run course:check -- "<code>"                          (the files in this repo)
//   npm run course:check -- "<code>" https://meeting-mind.com (the deployed site)
// It prints which lessons the code opens. If none open, the code was made with a different key
// from the one that encrypted the deployed lessons.
import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';

const [code, base] = process.argv.slice(2);
if (!code) { console.error('Usage: npm run course:check -- "<code>" [https://your-site]'); process.exit(1); }
let obj;
try { obj = JSON.parse(Buffer.from(code.trim(), 'base64url').toString('utf8')); } catch { obj = null; }
if (!obj || obj.v !== 1 || typeof obj.k !== 'string') { console.error('NOT A CODE: it did not decode. Paste the whole code, with no spaces or line breaks.'); process.exit(1); }
const key = Buffer.from(obj.k, 'base64url');
const dir = path.resolve('site/training/arc42/course/data');
const ids = fs.readdirSync(dir).filter((f) => f.endsWith('.json')).map((f) => f.slice(0, -5));
let open = 0, locked = 0;
for (const id of ids) {
  let enc;
  if (base) {
    const r = await fetch(`${base.replace(/\/$/, '')}/training/arc42/course/data/${id}.json`);
    if (!r.ok) { console.log(`  ${id}: not found on the site (HTTP ${r.status})`); continue; }
    enc = await r.json();
  } else enc = JSON.parse(fs.readFileSync(path.join(dir, `${id}.json`), 'utf8'));
  if (!enc.ct) continue; // free lessons are not encrypted
  try {
    const raw = Buffer.from(enc.ct, 'base64url');
    const d = crypto.createDecipheriv('aes-256-gcm', key, Buffer.from(enc.iv, 'base64url'));
    d.setAAD(Buffer.from(id)); d.setAuthTag(raw.subarray(raw.length - 16));
    Buffer.concat([d.update(raw.subarray(0, raw.length - 16)), d.final()]); open++;
  } catch { locked++; console.log(`  ${id}: this code does NOT open it`); }
}
console.log(`\n${open} paid lessons open, ${locked} do not.${base ? ' (live site)' : ' (local files)'}`);
if (locked && !open) console.log('The code was made with a different key from the one that encrypted these lessons. Run: npm run course:code -- "Name"  to make a matching code.');
process.exit(locked ? 2 : 0);
