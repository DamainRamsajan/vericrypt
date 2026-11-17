// Wiring (arc42 7): one deployable service containing the Order Service, both adapters and the Database access.
import http from 'node:http';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { randomUUID } from 'node:crypto';
import { DomainError } from './errors.js';
import { systemClock } from './clock.js';
import { createLogger } from './logger.js';
import { Router, readBody, sendJson, sendText } from './router.js';
import { openDb, seed } from './db.js';
import { OrderRepository } from './order-service/order-repository.js';
import { IdempotencyGuard } from './order-service/idempotency-guard.js';
import { CartValidator } from './order-service/cart-validator.js';
import { ConfirmationQueue } from './order-service/confirmation-queue.js';
import { OrderLifecycle } from './order-service/order-lifecycle.js';
import { PaymentCoordinator } from './order-service/payment-coordinator.js';
import { registerOrderApi } from './order-service/order-api.js';
import { PaymentAdapter } from './payment-adapter/payment-adapter.js';
import { NotificationAdapter } from './notification-adapter/notification-adapter.js';
import { registerFakeProvider } from './fake-provider/fake-provider.js';
import { registerFakeSms } from './fake-sms/fake-sms.js';

const PUBLIC = path.join(path.dirname(fileURLToPath(import.meta.url)), '..', 'public');
const TYPES = { '.html': 'text/html; charset=utf-8', '.js': 'text/javascript; charset=utf-8', '.css': 'text/css; charset=utf-8', '.json': 'application/json', '.svg': 'image/svg+xml' };

export function createApp({ config, clock = systemClock, logSink = (l) => console.log(l), manualTicks = false } = {}) {
  const logger = createLogger(logSink);
  const db = openDb(config.dbFile); seed(db);
  const repo = new OrderRepository(db);
  let baseUrl = null;
  const own = () => baseUrl;
  const providerUrl = () => config.providerUrl || own();
  const smsUrl = () => config.smsUrl || own();

  const queue = new ConfirmationQueue({ repo, clock, config, logger });
  const adapter = new PaymentAdapter({ providerUrl, apiKey: config.providerApiKey, webhookSecret: config.webhookSecret, timeoutMs: config.requestTimeoutMs, logger });
  const notifier = new NotificationAdapter({ smsUrl, timeoutMs: config.requestTimeoutMs });
  const lifecycle = new OrderLifecycle({ repo, guard: new IdempotencyGuard(repo), validator: new CartValidator(repo), queue, notifier, clock, logger });
  const coordinator = new PaymentCoordinator({ repo, adapter, queue, lifecycle, clock, config, logger });
  const handlers = { confirm: (p, j) => coordinator.handleConfirm(p, j), check: (p, j) => coordinator.handleCheck(p, j), sms: (p, j) => lifecycle.handleSms(p, j) };
  const tick = () => queue.runDue(handlers);

  const router = new Router();
  registerOrderApi(router, { lifecycle, coordinator, repo, config, clock });
  router.post('/webhooks/payments', ({ req, res, raw, cid }) => {            // Payment Adapter: HandleWebhook
    const r = adapter.handleWebhook(raw, req.headers['x-provider-signature'], coordinator.port, cid);
    sendJson(res, r.status, r.body);
  });
  let provider = null, sms = null;
  if (config.demo) {
    provider = registerFakeProvider(router, { config, webhookUrl: () => own() + '/webhooks/payments', logger });
    sms = registerFakeSms(router, { logger });
    router.get('/api/demo/state', ({ res }) => sendJson(res, 200, { providerMode: provider.mode, smsUp: sms.up, jobs: repo.jobCounts(), parked: repo.parkedJobs(), orders: repo.countOrders() }));
  }

  function serveStatic(pathname, res) {
    let p = path.normalize(decodeURIComponent(pathname));
    if (p.endsWith(path.sep) || p === '.') p = path.join(p, 'index.html');
    const file = path.join(PUBLIC, p);
    if (!file.startsWith(PUBLIC)) return sendJson(res, 403, { error: 'Forbidden' });
    fs.readFile(file, (err, data) => {
      if (err) return sendJson(res, 404, { error: 'NotFound' });
      res.writeHead(200, { 'content-type': TYPES[path.extname(file)] || 'application/octet-stream', 'cache-control': 'no-cache' });
      res.end(data);
    });
  }

  const server = http.createServer(async (req, res) => {
    const cid = req.headers['x-correlation-id'] || randomUUID();
    res.setHeader('x-correlation-id', cid);
    const url = new URL(req.url, 'http://localhost');
    res.on('finish', () => logger.log(cid, 'request', { method: req.method, path: url.pathname, status: res.statusCode }));
    try {
      const m = router.match(req.method, url.pathname);
      if (m) { const raw = req.method === 'GET' ? '' : await readBody(req); return await m.handler({ req, res, params: m.params, query: url.searchParams, raw, cid }); }
      if (req.method === 'GET') return serveStatic(url.pathname, res);
      sendJson(res, 404, { error: 'NotFound', correlationId: cid });
    } catch (e) {
      if (e instanceof DomainError) return sendJson(res, e.status, { error: e.code, message: e.message, correlationId: cid });
      logger.log(cid, 'error.unhandled', { error: e.message });
      sendJson(res, 500, { error: 'InternalError', correlationId: cid });
    }
  });

  let timer = null;
  return {
    db, repo, queue, lifecycle, coordinator, logger, tick, provider: () => provider, sms: () => sms,
    baseUrl: () => baseUrl,
    start: (port = 0) => new Promise((resolve) => server.listen(port, () => {
      baseUrl = `http://127.0.0.1:${server.address().port}`;
      if (!manualTicks) { timer = setInterval(() => tick().catch((e) => logger.log(null, 'worker.error', { error: e.message })), config.workerIntervalMs); timer.unref?.(); }
      resolve(baseUrl);
    })),
    stop: () => new Promise((resolve) => { clearInterval(timer); server.close(() => { db.close(); resolve(); }); server.closeAllConnections?.(); }),
  };
}
