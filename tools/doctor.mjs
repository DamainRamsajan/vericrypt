#!/usr/bin/env node
// Checks the machine can run this repo. Run by `npm run setup`.
const major = Number(process.versions.node.split('.')[0]);
if (major < 20) { console.error(`Node ${process.versions.node} is too old. Need Node 20 or newer (nvm install 20).`); process.exit(1); }
console.log(`Node ${process.versions.node} ok`);
