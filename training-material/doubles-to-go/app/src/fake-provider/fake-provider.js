// A stand-in for the external payment provider (arc42 3.2). It lives in the same process only to make the demo easy.
// It behaves like a real provider: hosted payment page, status API, signed webhooks, and failure modes you can switch on.
import crypto from 'node:crypto';
import { sendJson, sendText, parseJson, readBody } from '../router.js';

export function registerFakeProvider(router, { config, webhookUrl, logger }) {
  const state = { mode: 'normal', lateMs: 2000, payments: new Map(), byKey: new Map() };
  const authed = (req) => req.headers['x-api-key'] === config.providerApiKey;
  const sign = (body) => crypto.createHmac('sha256', config.webhookSecret).update(body).digest('hex');

  async function post(body) {
    try {
      await fetch(webhookUrl(), { method: 'POST', headers: { 'content-type': 'application/json', 'x-provider-signature': sign(body) }, body, signal: AbortSignal.timeout(config.requestTimeoutMs) });
    } catch (e) { logger.log(null, 'fake_provider.webhook_failed', { error: e.message }); }
  }
  async function deliver(p) {
    const event = { id: 'evt_' + crypto.randomUUID(), type: p.status === 'paid' ? 'payment.succeeded' : 'payment.failed', paymentRef: p.ref, createdAt: new Date().toISOString() };
    const body = JSON.stringify(event);
    if (state.mode === 'drop') return;                                   // the webhook is lost
    if (state.mode === 'late') { setTimeout(() => post(body), state.lateMs).unref?.(); return; }
    await post(body);
    if (state.mode === 'duplicate') await post(body);                     // the same event arrives twice
  }

  router.post('/fake-provider/api/payments', async ({ req, res, raw }) => {
    if (!authed(req)) return sendJson(res, 401, { error: 'bad api key' });
    if (state.mode === 'outage') return sendJson(res, 503, { error: 'provider outage' });
    const { orderId, amountCents } = parseJson(raw);
    const key = req.headers['idempotency-key'];
    if (key && state.byKey.has(key)) { const p = state.byKey.get(key); return sendJson(res, 200, { ref: p.ref, payUrl: `/fake-provider/pay/${p.ref}` }); }
    const p = { ref: 'pay_' + crypto.randomUUID().slice(0, 12), orderId, amountCents, status: 'pending', last4: null };
    state.payments.set(p.ref, p); if (key) state.byKey.set(key, p);
    sendJson(res, 201, { ref: p.ref, payUrl: `/fake-provider/pay/${p.ref}` });
  });
  router.get('/fake-provider/api/payments/:ref', ({ req, res, params }) => {
    if (!authed(req)) return sendJson(res, 401, { error: 'bad api key' });
    if (state.mode === 'outage') return sendJson(res, 503, { error: 'provider outage' });
    const p = state.payments.get(params.ref);
    if (!p) return sendJson(res, 404, { error: 'unknown payment' });
    sendJson(res, 200, { ref: p.ref, status: p.status });
  });
  // The hosted payment page: the customer types card details here, on the provider's side, never on ours (ADR-0002, C1).
  router.get('/fake-provider/pay/:ref', ({ res, params }) => {
    const p = state.payments.get(params.ref);
    if (!p) return sendText(res, 404, '<h1>Unknown payment</h1>');
    sendText(res, 200, `<!doctype html><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>Test payment page</title>
<style>body{font:16px system-ui;max-width:420px;margin:2rem auto;padding:0 1rem}input,button{font:inherit;padding:.6rem;width:100%;margin:.3rem 0;box-sizing:border-box}button{cursor:pointer}.x{background:#eee}</style>
<h2>Test payment page</h2><p>This page belongs to the (fake) payment provider. Amount: <b>TT$${(p.amountCents / 100).toFixed(2)}</b></p>
<label>Card number (test card)<input id="c" value="4242 4242 4242 4242" inputmode="numeric"></label>
<button id="pay">Pay now</button><button id="no" class="x">Decline</button><p id="m"></p>
<script>
async function go(o){document.getElementById('m').textContent='Working…';
const r=await fetch(location.pathname+'/complete',{method:'POST',headers:{'content-type':'application/json'},body:JSON.stringify({outcome:o,card:document.getElementById('c').value})});
const j=await r.json();location.href=j.returnUrl}
pay.onclick=()=>go('paid');no.onclick=()=>go('failed')</script>`);
  });
  router.post('/fake-provider/pay/:ref/complete', async ({ res, params, raw }) => {
    const p = state.payments.get(params.ref);
    if (!p) return sendJson(res, 404, { error: 'unknown payment' });
    const { outcome, card } = parseJson(raw);
    if (p.status === 'pending') {
      p.status = outcome === 'paid' ? 'paid' : 'failed';
      p.last4 = String(card || '').replace(/\D/g, '').slice(-4);          // a provider keeps only the last four digits
      await deliver(p);
    }
    sendJson(res, 200, { returnUrl: `/order.html?id=${p.orderId}` });
  });
  router.post('/fake-provider/admin/mode', ({ res, raw }) => {
    const { mode, lateMs } = parseJson(raw);
    if (!['normal', 'duplicate', 'drop', 'late', 'outage'].includes(mode)) return sendJson(res, 400, { error: 'mode must be normal, duplicate, drop, late or outage' });
    state.mode = mode; if (lateMs) state.lateMs = lateMs;
    sendJson(res, 200, { mode: state.mode, lateMs: state.lateMs });
  });
  router.get('/fake-provider/admin/state', ({ res }) => sendJson(res, 200, { mode: state.mode, lateMs: state.lateMs, payments: state.payments.size }));
  return state;
}
