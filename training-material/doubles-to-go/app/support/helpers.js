// Test helpers: start the real app on a free port with a controllable clock, and drive it over HTTP.
import crypto from 'node:crypto';
import { loadConfig } from '../src/config.js';
import { createApp } from '../src/server.js';
import { testClock } from '../src/clock.js';

export async function startApp(overrides = {}) {
  const lines = []; const clock = testClock();
  const config = { ...loadConfig({}), backoffBaseMs: 1000, reconcileAfterMs: 15000, ...overrides };
  const app = createApp({ config, clock, logSink: (l) => lines.push(l), manualTicks: true });
  const base = await app.start(0);
  let n = 0;
  const call = async (method, path, { body, headers = {} } = {}) => {
    const r = await fetch(base + path, { method, headers: { 'content-type': 'application/json', ...headers }, body: body ? JSON.stringify(body) : undefined });
    let j = null; try { j = await r.json(); } catch { /* no body */ }
    return { status: r.status, body: j, headers: r.headers };
  };
  const h = {
    app, base, clock, lines, call, config,
    vendorH: (id = 'ria') => ({ authorization: 'Bearer ' + config.vendorToken, 'x-vendor-id': id }),
    supportH: { authorization: 'Bearer ' + config.supportToken },
    cart: (o = {}) => ({ vendorId: 'ria', items: [{ itemId: 'ria-d1', qty: 2 }], customerName: 'Test Customer', phone: '868-555-0142', ...o }),
    key: () => `test-key-${++n}-${crypto.randomBytes(4).toString('hex')}`,
    async placeOrder(o = {}, key = h.key(), extraHeaders = {}) { const r = await call('POST', '/api/orders', { body: h.cart(o), headers: { 'idempotency-key': key, ...extraHeaders } }); return { ...r, key }; },
    pay: (order, outcome = 'paid') => call('POST', order.body.paymentUrl + '/complete', { body: { outcome, card: '4242 4242 4242 4242' } }),
    refOf: (order) => order.body.paymentUrl.split('/').pop(),
    status: async (id) => (await call('GET', '/api/orders/' + id)).body.status,
    settle: async () => { let total = 0, k; while ((k = await app.tick()) > 0) total += k; return total; },
    advance: async (ms) => { clock.advance(ms); return h.settle(); },
    mode: (m) => call('POST', '/fake-provider/admin/mode', { body: { mode: m } }),
    smsUp: (up) => call('POST', '/fake-sms/admin/mode', { body: { up } }),
    count: (sql, ...p) => app.db.prepare(sql).get(...p).n,
    webhook: (event, { secret = config.webhookSecret, badSig = false } = {}) => {
      const raw = JSON.stringify(event); const sig = badSig ? 'bad' : crypto.createHmac('sha256', secret).update(raw).digest('hex');
      return fetch(base + '/webhooks/payments', { method: 'POST', headers: { 'x-provider-signature': sig }, body: raw }).then(async (r) => ({ status: r.status, body: await r.json() }));
    },
    stop: () => app.stop(),
  };
  return h;
}
