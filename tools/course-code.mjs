#!/usr/bin/env node
// Makes an access code for one buyer. Usage: npm run course:code -- "Buyer name"
// The code carries the course key, so keep it private: send it only to the person who paid.
// The buyer's name is shown on every lesson they open, which discourages sharing.
import { loadKey } from './build-course.mjs';
const name = process.argv.slice(2).join(' ').trim();
if (!name) { console.error('Usage: npm run course:code -- "Buyer name"'); process.exit(1); }
const key = loadKey({ create: false });
if (!key) { console.error('No course key found. Run: npm run course   (it creates .course-key)'); process.exit(1); }
const code = Buffer.from(JSON.stringify({ v: 1, b: name.slice(0, 60), k: key.toString('base64url') })).toString('base64url');
console.log(`\nAccess code for ${name}:\n\n${code}\n\nEmail it to the buyer with the link https://meeting-mind.com/training/arc42/course/\n`);
