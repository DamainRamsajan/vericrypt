// Small DOM helpers shared by the shell and by demos.
// Rule: data from JSON is only ever inserted with textContent or setAttribute,
// never as HTML. The only HTML we inject is output of markdown.js, which escapes.

export const $ = (selector, root = document) => root.querySelector(selector);
export const $$ = (selector, root = document) => [...root.querySelectorAll(selector)];

const SAFE_HREF = /^(https?:\/\/|mailto:|\/|\.\/|\.\.\/|\?|#|[A-Za-z0-9_-]+[/.?#]?)/;
const BAD_SCHEME = /^\s*(javascript|data|vbscript):/i;

export function safeHref(url) {
  const u = String(url ?? '');
  return SAFE_HREF.test(u) && !BAD_SCHEME.test(u) ? u : '#';
}

export function el(tag, attrs = {}, ...kids) {
  const node = document.createElement(tag);
  for (const [key, value] of Object.entries(attrs || {})) {
    if (value === null || value === undefined || value === false) continue;
    if (key === 'class') node.className = value;
    else if (key === 'text') node.textContent = value;
    else if (key === 'href') node.setAttribute('href', safeHref(value));
    else node.setAttribute(key, value === true ? '' : String(value));
  }
  for (const kid of kids.flat(Infinity)) {
    if (kid === null || kid === undefined || kid === false) continue;
    node.append(kid instanceof Node ? kid : document.createTextNode(String(kid)));
  }
  return node;
}

export async function loadJSON(url) {
  const res = await fetch(url, { headers: { accept: 'application/json' } });
  if (!res.ok) throw new Error(`${url}: ${res.status}`);
  return res.json();
}

export async function loadText(url) {
  const res = await fetch(url);
  if (!res.ok) throw new Error(`${url}: ${res.status}`);
  return res.text();
}

export function copyText(text) {
  try {
    return navigator.clipboard.writeText(text).then(() => true, () => false);
  } catch {
    return Promise.resolve(false);
  }
}

export const storage = {
  get(key) { try { return localStorage.getItem(key); } catch { return null; } },
  set(key, value) { try { localStorage.setItem(key, value); } catch { /* ignore */ } },
};
