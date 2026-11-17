# Doubles-to-Go: the working app

A small, real implementation of the architecture in [`../arc42.md`](../arc42.md). It exists so students can see each arc42 section turn into running code, and watch the design cope when things go wrong.

- **No dependencies.** It uses only what ships with Node: `http`, `fetch`, `node:sqlite`, `node:test`.
- **Needs Node 22.13 or newer.**
- **One deployable service** (ADR-0004). The payment provider and the SMS gateway are faked inside the same process so the demo works offline.

```bash
cd app
npm start            # http://localhost:3000   (customers)  /vendor/  /demo/
npm test             # 33 tests: contracts, scenarios, architecture rules
npm run simulate     # Q1: 1,000 customers, dropped connections, duplicate webhooks
npm run docs:check   # the arc42 checks its own consistency (needs python3)
```

Useful settings (environment variables): `PORT`, `DB_FILE` (a file path keeps data between runs), `RECONCILE_AFTER_MS` (default 15000; use 5000 in class), `BACKOFF_BASE_MS` (default 1000), `NODE_ENV=production` (switches the demo panel and fakes off).

## Where each arc42 block lives

| arc42 block (section 5) | Code |
|---|---|
| Customer Web App | `public/index.html`, `app.js`, `order.html`, `style.css` (about 8 KB in total) |
| Vendor Console (and the support role) | `public/vendor/` |
| Order Service: Order API | `src/order-service/order-api.js` |
| Order Service: Cart Validator | `src/order-service/cart-validator.js` |
| Order Service: Order Lifecycle | `src/order-service/order-lifecycle.js` |
| Order Service: Idempotency Guard | `src/order-service/idempotency-guard.js` |
| Order Service: Payment Coordinator | `src/order-service/payment-coordinator.js` |
| Order Service: Confirmation Queue | `src/order-service/confirmation-queue.js` |
| Order Service: Order Repository | `src/order-service/order-repository.js` |
| Payment Adapter | `src/payment-adapter/payment-adapter.js` |
| Notification Adapter | `src/notification-adapter/notification-adapter.js` |
| Database | `src/db.js` (SQLite) |
| Payment provider, SMS gateway (external) | `src/fake-provider/`, `src/fake-sms/` (stand-ins) |
| Wiring (section 7) | `src/server.js`, `src/main.js` |

| arc42 rule | Where it is enforced |
|---|---|
| Contracts 5.4, 5.5, 5.6 | `test/contracts.test.js` (one test per rule, named in the arc42) |
| Quality scenarios Q1, Q3, Q6, Q8, Q9, Q10 | `test/scenarios.test.js`, `scripts/simulate-drops.js` |
| Block boundaries, no card data, page budget, docs stay true | `test/architecture.test.js` (fitness functions) |

## A classroom demo script (about 20 minutes)

Open three windows: the customer app, the vendor console and the demo panel (`/demo/`).

1. **Happy path (6.1).** Order two doubles, pay on the provider's page, watch the status page reach Paid. In the vendor console press "Mark ready", and see the SMS in the demo panel's outbox.
2. **One key, one order (ADR-0005, Q1).** Send the same request twice and show that only one order exists:
   ```bash
   KEY=demo-key-0001
   BODY='{"vendorId":"ria","items":[{"itemId":"ria-d1","qty":1}],"customerName":"Demo","phone":"868-555-0100"}'
   for i in 1 2; do curl -s -X POST localhost:3000/api/orders -H "content-type: application/json" -H "idempotency-key: $KEY" -d "$BODY"; echo; done
   ```
   The second answer says `"duplicate":true` with the same order id.
3. **A webhook arrives twice (ADR-0003).** In the demo panel choose `duplicate`, place and pay for an order. It is paid once, and the queue shows one confirmation.
4. **A webhook is lost (6.2, Q6).** Choose `drop`, then pay. The order stays pending ("never treated as a failure"). After the reconciliation delay it settles to Paid by itself.
5. **The provider is down (R1).** Choose `outage`, then order. Payment cannot start, so the status page offers "Start payment". Nothing is marked Failed. Switch back to `normal` and retry.
6. **The SMS gateway is down (6.3, Q8).** Switch it off in the panel, mark an order ready: the customer sees Ready at once. Switch the gateway back on and the SMS is sent.
7. **Support in minutes (Q9).** In the vendor console's Support tab, paste an order id and read its timeline, with correlation ids and no names or phone numbers.
8. **Break the architecture on purpose.** In `src/payment-adapter/payment-adapter.js` add `import { openDb } from '../db.js';` and run `npm test`. The architecture test fails and names the rule. Remove the line again.
9. **Break a contract on purpose.** In `idempotency-guard.js` make `lookup()` return `null`. The contract tests and the Q1 scenario fail. Restore it.

## Exercises for students

- **Q5, a change scenario.** Add a second payment provider behind the Payment Adapter. The arc42 budget is five developer-days: how long does it take you?
- **A missing test.** `MarkReady` has no contract in the arc42. Write one (pre-conditions, post-conditions, invariants, error modes), then write the tests.
- **A new decision.** Replace the database-backed queue with a managed queue service, and record it as ADR-0009 superseding ADR-0003. Update section 5.3 and technical debt TD1.
- **Fix the gap.** Add job claiming with a lease to the queue (risk R8), then add a test that runs two workers.
- **Find the lie.** Ask an AI assistant to write section 6 from this code, then red-team it with the three lenses.

## Where the code is deliberately simpler than the arc42

Be open about these with students: each one is a good question to ask of any real system.

- **Authentication** uses fixed demo tokens (`vendor-demo`, `support-demo`). Real roles, sessions and rate limiting are not built.
- **Two instances (R8).** The arc42 deploys two app instances, but the queue worker is not safe to run twice. Payments and orders are protected by idempotent handlers, but a ready SMS could be sent twice. Add job claiming first.
- **The payout report** is computed when requested, not generated daily (TD2).
- **Data retention.** The arc42 says the retention period is for the business owner to confirm. No deletion job exists.
- **The front ends** are served by the app itself. The arc42 puts them on a CDN.
- **HTTPS** is assumed to be handled by the platform in front of the app.
- **The provider and SMS gateway** are fakes. A real provider needs its own signature scheme, retries and a test mode.
- **The sample catalogue** (three fictional vendors) is in `src/db.js`.

## Notes on the tests

- Tests start the real server on a free port and a controllable clock, so retries and reconciliation run in milliseconds without waiting.
- The tests were checked by sabotage: eleven deliberate bugs (an ignored key, an unchecked signature, an unknown result treated as failed, an adapter reaching into the database, a card column, a phone number in the log, and so on) were each caught by at least one test.
