#!/usr/bin/env node
// Dev server for a static site folder. Applies the folder's _headers file, so
// Content-Security-Policy violations show up in your browser console during development.
// Usage: node tools/serve.mjs [siteDir=site] [port=8080]

import http from 'node:http';
import fs from 'node:fs/promises';
import path from 'node:path';

const root = path.resolve(process.argv[2] || 'site');
const port = Number(process.argv[3] || process.env.PORT || 8080);

const MIME = {
  '.html': 'text/html; charset=utf-8', '.css': 'text/css; charset=utf-8',
  '.js': 'text/javascript; charset=utf-8', '.mjs': 'text/javascript; charset=utf-8',
  '.json': 'application/json; charset=utf-8', '.md': 'text/markdown; charset=utf-8',
  '.txt': 'text/plain; charset=utf-8', '.svg': 'image/svg+xml', '.png': 'image/png',
  '.jpg': 'image/jpeg', '.jpeg': 'image/jpeg', '.ico': 'image/x-icon', '.webp': 'image/webp',
  '.wasm': 'application/wasm', '.pdf': 'application/pdf', '.woff2': 'font/woff2', '.zip': 'application/zip',
};

export function parseHeaders(text) {
  const rules = [];
  let current = null;
  for (const raw of text.split('\n')) {
    if (!raw.trim() || raw.trim().startsWith('#')) continue;
    if (!/^\s/.test(raw)) { current = { pattern: raw.trim(), headers: [] }; rules.push(current); continue; }
    const m = raw.trim().match(/^([^:]+):\s*(.*)$/);
    if (m && current) current.headers.push([m[1].trim(), m[2].trim()]);
  }
  return rules;
}

export function matches(pattern, pathname) {
  const re = new RegExp('^' + pattern.replace(/[.+?^${}()|[\]\\]/g, '\\$&').replace(/\*/g, '.*') + '$');
  return re.test(pathname);
}

async function loadRules() {
  try { return parseHeaders(await fs.readFile(path.join(root, '_headers'), 'utf8')); } catch { return []; }
}

async function resolveFile(pathname) {
  const clean = path.normalize(decodeURIComponent(pathname)).replace(/^(\.\.[/\\])+/, '');
  let file = path.join(root, clean);
  if (!file.startsWith(root)) return null;
  try {
    const st = await fs.stat(file);
    if (st.isDirectory()) file = path.join(file, 'index.html');
    await fs.access(file);
    return file;
  } catch { return null; }
}

http.createServer(async (req, res) => {
  const url = new URL(req.url, 'http://localhost');
  const rules = await loadRules();
  const headers = { 'Cache-Control': 'no-store' };
  for (const rule of rules) if (matches(rule.pattern, url.pathname)) for (const [k, v] of rule.headers) headers[k] = v;

  let file = await resolveFile(url.pathname);
  let status = 200;
  if (!file) {
    status = 404;
    file = await resolveFile('/404.html');
  }
  if (!file) { res.writeHead(404, { 'Content-Type': 'text/plain' }); res.end('Not found'); return; }
  headers['Content-Type'] = headers['Content-Type'] && file.endsWith('.wasm') ? headers['Content-Type'] : (MIME[path.extname(file).toLowerCase()] || 'application/octet-stream');
  res.writeHead(status, headers);
  res.end(await fs.readFile(file));
}).listen(port, '0.0.0.0', () => {
  console.log(`Serving ${root}`);
  console.log(`Open http://localhost:${port}/  (in Codespaces, use the forwarded port)`);
});
