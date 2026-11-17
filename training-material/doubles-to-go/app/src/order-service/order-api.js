// Order API (arc42 5.3 and 5.2): the public face of the Order Service. It checks who is calling, then hands over to the other inner blocks.
import { DomainError } from '../errors.js';
import { sendJson, parseJson } from '../router.js';

export function registerOrderApi(router, { lifecycle, coordinator, repo, config, clock }) {
  const bearer = (req) => (req.headers.authorization || '').replace(/^Bearer /, '');
  function vendorAuth(req) {
    if (bearer(req) !== config.vendorToken) throw new DomainError('Unauthorized', 'Vendor token required', 401);
    const vendorId = req.headers['x-vendor-id'];
    if (!repo.getVendor(vendorId)) throw new DomainError('Unauthorized', 'Unknown vendor', 401);
    return vendorId;
  }
  function supportAuth(req) { if (bearer(req) !== config.supportToken) throw new DomainError('Unauthorized', 'Support token required', 401); }

  // ---- Customer Web App
  router.get('/api/vendors', ({ res }) => sendJson(res, 200, { vendors: repo.listVendors() }, { 'cache-control': 'public, max-age=30' }));  // menus may be up to 30 s old (Q10)

  router.post('/api/orders', async ({ req, res, raw, cid }) => {
    const result = lifecycle.place(parseJson(raw), req.headers['idempotency-key'], cid);        // PlaceOrder
    const paymentUrl = await coordinator.startPayment(result.orderId, cid);                      // StartPayment, null if the provider is unavailable
    sendJson(res, result.duplicate ? 200 : 201, { orderId: result.orderId, status: result.status, totalCents: result.totalCents, paymentUrl, duplicate: result.duplicate });
  });
  router.post('/api/orders/:id/pay', async ({ res, params, cid }) => {                           // retry starting a payment
    const o = lifecycle.status(params.id);
    sendJson(res, 200, { orderId: o.orderId, status: o.status, paymentUrl: o.status === 'PendingPayment' ? await coordinator.startPayment(params.id, cid) : null });
  });
  router.get('/api/orders/:id', ({ res, params }) => sendJson(res, 200, lifecycle.status(params.id), { 'cache-control': 'no-store' }));   // GetOrderStatus

  // ---- Vendor Console
  router.get('/api/vendor/orders', ({ req, res }) => sendJson(res, 200, { orders: repo.ordersForVendor(vendorAuth(req)) }, { 'cache-control': 'no-store' }));
  router.post('/api/vendor/orders/:id/ready', ({ req, res, params, cid }) => sendJson(res, 200, lifecycle.markReady(params.id, vendorAuth(req), cid)));   // MarkReady
  router.post('/api/vendor/orders/:id/collected', ({ req, res, params, cid }) => sendJson(res, 200, lifecycle.markCollected(params.id, vendorAuth(req), cid)));
  router.post('/api/vendor/stock', ({ req, res, raw }) => {                                       // UpdateStock
    const vendorId = vendorAuth(req); const { itemId, quantity } = parseJson(raw);
    if (!Number.isInteger(quantity) || quantity < 0 || quantity > 10000) throw new DomainError('InvalidRequest', 'quantity must be a whole number from 0 to 10000', 400);
    if (!repo.setStock(itemId, vendorId, quantity)) throw new DomainError('UnknownItem', 'No such item', 404);
    sendJson(res, 200, { ack: true });
  });
  router.post('/api/vendor/menu', ({ req, res, raw }) => {                                        // UpdateMenu
    const vendorId = vendorAuth(req); const { itemId, name, priceCents } = parseJson(raw);
    if (priceCents != null && (!Number.isInteger(priceCents) || priceCents < 0)) throw new DomainError('InvalidRequest', 'priceCents must be a whole number', 400);
    if (!repo.setMenuItem(itemId, vendorId, { name, priceCents })) throw new DomainError('UnknownItem', 'No such item', 404);
    sendJson(res, 200, { ack: true });
  });
  router.post('/api/vendor/open', ({ req, res, raw }) => { repo.setOpen(vendorAuth(req), !!parseJson(raw).open); sendJson(res, 200, { ack: true }); });
  router.get('/api/vendor/payouts', ({ req, res, query }) => {                                    // GetPayoutReport (computed on request in this demo, see TD2)
    const vendorId = vendorAuth(req); const date = query.get('date') || new Date(clock.now() - 4 * 3600e3).toISOString().slice(0, 10);
    if (!/^\d{4}-\d{2}-\d{2}$/.test(date)) throw new DomainError('InvalidRequest', 'date must look like 2026-12-05', 400);
    const [y, m, d] = date.split('-').map(Number);
    const from = Date.UTC(y, m - 1, d, 4);                                                         // a Trinidad day starts at 04:00 UTC
    sendJson(res, 200, { vendorId, date, ...repo.payoutFor(vendorId, from, from + 86_400_000) });
  });

  // ---- Support role (Vendor Console, support role): diagnose a failed order from its correlation ids (Q9)
  router.get('/api/support/orders/:id', ({ req, res, params }) => {
    supportAuth(req);
    const o = repo.getOrder(params.id);
    if (!o) throw new DomainError('UnknownOrder', 'No such order', 404);
    const p = repo.getPaymentByOrder(params.id);
    sendJson(res, 200, { orderId: o.id, status: o.status, vendor: o.vendorName, totalCents: o.totalCents,
      payment: p ? { ref: p.ref, status: p.status } : null, timeline: repo.eventsFor(params.id) });   // no phone number, no name
  });
}
