// Customer Web App (arc42 5.1, ADR-0001): plain HTML and a small script, no framework, so it loads on 3G.
const $ = (s) => document.querySelector(s);
const money = (c) => 'TT$' + (c / 100).toFixed(2);
const uuid = () => (crypto.randomUUID ? crypto.randomUUID() : ([1e7] + -1e3 + -4e3 + -8e3 + -1e11).replace(/[018]/g, (c) => (c ^ crypto.getRandomValues(new Uint8Array(1))[0] & 15 >> c / 4).toString(16)));
function h(tag, attrs, ...kids) { const e = document.createElement(tag); Object.assign(e, attrs || {}); for (const k of kids.flat()) e.append(k); return e; }
let vendors = [], cart = {}, cartVendor = null, note = '';

async function load() {
  try { vendors = (await (await fetch('/api/vendors')).json()).vendors; render(); }
  catch { $('#app').textContent = 'Could not load menus. Check your connection and refresh.'; }
}
function setQty(vendor, item, d) {
  if (cartVendor && cartVendor !== vendor.id && Object.values(cart).some(Boolean)) { cart = {}; note = 'Your cart holds one vendor at a time, so it was cleared.'; }
  cartVendor = vendor.id; cart[item.id] = Math.max(0, Math.min(Math.min(20, item.stock), (cart[item.id] || 0) + d));
  sessionStorage.removeItem('idk');            // a changed cart is a new order, so it gets a new key
  render();
}
function render() {
  const app = $('#app'); const keep = { n: $('#name')?.value || '', p: $('#phone')?.value || '' };
  app.replaceChildren();
  for (const v of vendors) {
    app.append(h('h2', {}, v.name, ' ', v.open ? '' : h('span', { className: 'tag bad', textContent: 'Closed' })));
    const box = h('div', { className: 'card' });
    for (const i of v.items) {
      const q = (cartVendor === v.id && cart[i.id]) || 0;
      box.append(h('div', { className: 'row' },
        h('div', {}, i.name, h('div', { className: 'mut' }, money(i.priceCents), i.stock === 0 ? ' · sold out' : i.stock <= 10 ? ` · only ${i.stock} left` : '')),
        h('div', { className: 'qty' },
          h('button', { className: 'alt', textContent: '−', disabled: !q, onclick: () => setQty(v, i, -1), ariaLabel: 'Remove one ' + i.name }),
          h('b', { textContent: q }),
          h('button', { className: 'alt', textContent: '+', disabled: !v.open || i.stock === 0, onclick: () => setQty(v, i, 1), ariaLabel: 'Add one ' + i.name }))));
    }
    app.append(box);
  }
  const v = vendors.find((x) => x.id === cartVendor);
  const total = v ? v.items.reduce((s, i) => s + i.priceCents * (cart[i.id] || 0), 0) : 0;
  app.append(h('div', { className: 'card' },
    h('b', {}, 'Your order: ' + money(total)),
    h('label', {}, 'Your name', h('input', { id: 'name', value: keep.n, autocomplete: 'name', maxLength: 60 })),
    h('label', {}, 'Phone number (for the "ready" SMS)', h('input', { id: 'phone', value: keep.p, type: 'tel', autocomplete: 'tel' })),
    h('p', { id: 'msg', textContent: note }),
    h('button', { className: 'big', id: 'go', textContent: 'Order and pay', disabled: total === 0, onclick: place })));
}
async function place() {
  const msg = (t) => { $('#msg').textContent = t; };
  const body = { vendorId: cartVendor, customerName: $('#name').value, phone: $('#phone').value,
    items: Object.entries(cart).filter(([, q]) => q > 0).map(([itemId, qty]) => ({ itemId, qty })) };
  let key = sessionStorage.getItem('idk'); if (!key) { key = uuid(); sessionStorage.setItem('idk', key); }   // reused on retry: one key, at most one order
  $('#go').disabled = true; msg('Placing your order…');
  try {
    const r = await fetch('/api/orders', { method: 'POST', headers: { 'content-type': 'application/json', 'idempotency-key': key }, body: JSON.stringify(body) });
    const j = await r.json();
    if (!r.ok) { note = j.message || 'Something went wrong.'; render(); return; }
    sessionStorage.removeItem('idk');
    location.href = j.paymentUrl || '/order.html?id=' + j.orderId;      // pay on the provider's hosted page (ADR-0002)
  } catch { msg('Connection lost. Tap the button again. You will not be charged twice.'); $('#go').disabled = false; }
}
load();
