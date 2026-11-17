// Placeholder demo: a four-entry hash chain you can tamper with.
// It uses the browser's built-in SHA-256 (no WebAssembly, no network).
// It is NOT any product's engine. A product replaces demo/demo.js with its own mount().
//
// Demo mount contract:
//   export async function mount(root, ctx)  ->  { unmount() }
//   ctx = { base, theme, reducedMotion }

import { el } from './dom.js';

const enc = new TextEncoder();
const toHex = (buf) => [...new Uint8Array(buf)].map((b) => b.toString(16).padStart(2, '0')).join('');
const sha256 = async (text) => toHex(await crypto.subtle.digest('SHA-256', enc.encode(text)));

const INITIAL = [
  'Request received',
  'Authority checked',
  'Action recorded',
  'Outcome recorded',
];

async function chain(entries) {
  const hashes = [];
  let prev = '0'.repeat(64);
  for (const entry of entries) {
    prev = await sha256(prev + '\n' + entry);
    hashes.push(prev);
  }
  return hashes;
}

export async function mount(root) {
  let sealed = await chain(INITIAL);
  const inputs = [];
  const hashCells = [];

  const result = el('p', { class: 'demo-result', role: 'status', 'aria-live': 'polite' },
    'Edit any entry, then press Verify.');

  const rows = INITIAL.map((text, i) => {
    const input = el('input', { type: 'text', id: `entry-${i}`, value: text, class: 'chain-input', autocomplete: 'off', spellcheck: 'false' });
    const hash = el('code', { class: 'chain-hash', 'aria-label': `Hash of entry ${i + 1}` });
    inputs.push(input);
    hashCells.push(hash);
    return el('div', { class: 'chain-row' },
      el('label', { for: `entry-${i}`, class: 'chain-label' }, `Entry ${i + 1}`),
      input,
      hash);
  });

  async function refreshHashes() {
    const current = await chain(inputs.map((i) => i.value));
    current.forEach((h, i) => { hashCells[i].textContent = h.slice(0, 20) + '…'; hashCells[i].dataset.changed = String(h !== sealed[i]); });
    return current;
  }

  async function verify() {
    const current = await refreshHashes();
    const first = current.findIndex((h, i) => h !== sealed[i]);
    if (first === -1) {
      result.textContent = 'Match: every hash equals the sealed chain, so nothing has changed.';
      result.dataset.state = 'ok';
    } else {
      result.textContent = `No match: entry ${first + 1} changed, and because each hash includes the one before it, every hash from entry ${first + 1} onward changed too.`;
      result.dataset.state = 'fail';
    }
  }

  const verifyBtn = el('button', { type: 'button', class: 'btn btn-primary' }, 'Verify against the sealed chain');
  verifyBtn.addEventListener('click', verify);
  inputs.forEach((input) => input.addEventListener('input', refreshHashes));

  root.replaceChildren(
    el('p', { class: 'demo-intro' },
      'The chain below was sealed when this page loaded. Each entry’s hash covers its text and the hash before it. Change a word, then verify.'),
    el('div', { class: 'chain' }, rows),
    el('div', { class: 'actions' }, verifyBtn),
    result);

  await refreshHashes();
  return { unmount() { root.replaceChildren(); } };
}
