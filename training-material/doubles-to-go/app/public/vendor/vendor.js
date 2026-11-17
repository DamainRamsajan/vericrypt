// Vendor Console (arc42 5.1). The tokens below are demo values; real deployments set VENDOR_TOKEN and SUPPORT_TOKEN.
const VENDOR_TOKEN = 'vendor-demo', SUPPORT_TOKEN = 'support-demo';
const $ = (s) => document.querySelector(s);
const money = (c) => 'TT$' + (c / 100).toFixed(2);
function h(tag, attrs, ...kids) { const e = document.createElement(tag); Object.assign(e, attrs || {}); for (const k of kids.flat()) e.append(k); return e; }
const say = (t) => { $('#msg').textContent = t; };
const vid = () => $('#vendor').value;
async function api(method, path, body, token = VENDOR_TOKEN) {
  const r = await fetch(path, { method, headers: { 'content-type': 'application/json', authorization: 'Bearer ' + token, 'x-vendor-id': vid() }, body: body ? JSON.stringify(body) : undefined });
  const j = await r.json().catch(() => ({})); if (!r.ok) throw new Error(j.message || j.error || 'Request failed'); return j;
}
let vendors = [], tab = 'orders';
async function init() {
  vendors = (await (await fetch('/api/vendors')).json()).vendors;
  $('#vendor').replaceChildren(...vendors.map((v) => h('option', { value: v.id, textContent: v.name })));
  $('#vendor').onchange = refresh;
  $('#tabs').onclick = (e) => { if (!e.target.dataset.t) return; tab = e.target.dataset.t; for (const b of document.querySelectorAll('#tabs button')) b.classList.toggle('on', b === e.target); for (const s of ['orders', 'menu', 'payouts', 'support']) $('#' + s).hidden = s !== tab; refresh(); };
  refresh(); setInterval(() => { if (tab === 'orders') refresh(); }, 4000);
}
async function refresh() {
  try { say(''); ({ orders, menu, payouts, support })[tab](); } catch (e) { say(e.message); }
}
const orders = async () => {
  const { orders } = await api('GET', '/api/vendor/orders');
  const t = h('table', {}, h('tr', {}, h('th', { textContent: 'Order' }), h('th', { textContent: 'Status' }), h('th', { textContent: 'Total' }), h('th')));
  for (const o of orders) {
    const act = o.status === 'Paid' ? h('button', { textContent: 'Mark ready', onclick: () => act2(o.id, 'ready') })
      : o.status === 'Ready' ? h('button', { className: 'alt', textContent: 'Collected', onclick: () => act2(o.id, 'collected') }) : '';
    t.append(h('tr', {}, h('td', { textContent: o.id.slice(0, 8) }), h('td', {}, h('span', { className: 'tag ' + (o.status === 'Failed' ? 'bad' : 'ok'), textContent: o.status })), h('td', { textContent: money(o.totalCents) }), h('td', {}, act)));
  }
  $('#orders').replaceChildren(orders.length ? t : h('p', { className: 'mut', textContent: 'No orders yet.' }));
};
async function act2(id, what) { try { await api('POST', `/api/vendor/orders/${id}/${what}`); refresh(); } catch (e) { say(e.message); } }
const menu = async () => {
  const v = (await (await fetch('/api/vendors', { cache: 'no-store' })).json()).vendors.find((x) => x.id === vid());
  const box = h('div', { className: 'card' }, h('label', {}, h('input', { type: 'checkbox', checked: v.open, style: 'width:auto', onchange: async (e) => { await api('POST', '/api/vendor/open', { open: e.target.checked }); say(e.target.checked ? 'Open for orders' : 'Closed'); } }), ' Open for orders'));
  for (const i of v.items) {
    const stock = h('input', { type: 'number', min: 0, value: i.stock, style: 'width:90px' }), price = h('input', { type: 'number', min: 0, step: '0.01', value: (i.priceCents / 100).toFixed(2), style: 'width:90px' });
    box.append(h('div', { className: 'row' }, h('div', {}, i.name), h('div', { className: 'qty' }, 'Stock', stock, 'TT$', price,
      h('button', { textContent: 'Save', onclick: async () => { await api('POST', '/api/vendor/stock', { itemId: i.id, quantity: +stock.value }); await api('POST', '/api/vendor/menu', { itemId: i.id, priceCents: Math.round(price.value * 100) }); say('Saved. Customers see it within 30 seconds.'); } }))));
  }
  $('#menu').replaceChildren(box);
};
const payouts = async () => {
  const date = h('input', { type: 'date', value: new Date(Date.now() - 4 * 3600e3).toISOString().slice(0, 10) }), out = h('p');
  const go = async () => { const r = await api('GET', '/api/vendor/payouts?date=' + date.value); out.textContent = `${r.orders} paid orders, ${money(r.totalCents)}`; };
  date.onchange = go; $('#payouts').replaceChildren(h('label', {}, 'Day ', date), out); go();
};
const support = async () => {
  const id = h('input', { placeholder: 'Order id' }), out = h('div', { className: 'card', hidden: true });
  const go = h('button', { textContent: 'Look up', onclick: async () => {
    try { const r = await api('GET', '/api/support/orders/' + encodeURIComponent(id.value.trim()), null, SUPPORT_TOKEN); out.hidden = false;
      out.replaceChildren(h('b', { textContent: `${r.status} · ${r.vendor} · ${money(r.totalCents)}` }), h('p', { className: 'mut', textContent: r.payment ? `Payment ${r.payment.ref}: ${r.payment.status}` : 'No payment started' }),
        ...r.timeline.map((e) => h('div', { className: 'row' }, h('span', { textContent: new Date(e.at).toLocaleTimeString() + ' ' + e.type }), h('code', { className: 'mut', textContent: e.cid || '' })))); } catch (e) { say(e.message); } } });
  $('#support').replaceChildren(h('p', { className: 'mut', textContent: 'Staff role: find a failed order by its id and read its timeline. Customer names and phone numbers are not shown.' }), id, go, out);
};
init();
