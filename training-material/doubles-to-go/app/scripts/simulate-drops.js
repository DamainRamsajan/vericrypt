// Scenario Q1 (arc42 10.2): "The customer's connection drops during payment. On reconnect, exactly one order and one charge exist."
// Measure: 100% of 1,000 simulated drops.  Run:  npm run simulate   (or: node --no-warnings scripts/simulate-drops.js 1000)
import { startApp } from '../support/helpers.js';

const N = Number(process.argv[2]) || 1000;
const t = await startApp();
await t.call('POST', '/api/vendor/stock', { body: { itemId: 'ria-d2', quantity: 10000 }, headers: t.vendorH() });   // plenty of stock for the whole run
await t.mode('duplicate');                                      // the provider also sends every webhook twice
const started = Date.now();
let drops = 0;
for (let i = 0; i < N; i++) {
  const key = t.key();
  const tries = 1 + (i % 4);                                    // the client resends up to three more times
  drops += tries - 1;
  const calls = Array.from({ length: tries }, () => t.placeOrder({ items: [{ itemId: 'ria-d2', qty: 1 }] }, key));
  const rs = await Promise.all(calls);                          // the retries overlap, as they do on a flaky network
  await t.pay(rs[0]); if (i % 3 === 0) await t.pay(rs[0]);      // some customers also press Pay twice
}
await t.settle();

const checks = [
  ['Orders created', t.count('SELECT COUNT(*) n FROM orders'), N],
  ['Orders with more than one payment', t.count('SELECT COUNT(*) n FROM (SELECT order_id FROM payments GROUP BY order_id HAVING COUNT(*)>1)'), 0],
  ['Orders paid more than once', t.count("SELECT COUNT(*) n FROM (SELECT order_id FROM events WHERE type='order.paid' GROUP BY order_id HAVING COUNT(*)>1)"), 0],
  ['Orders ending Paid', t.count("SELECT COUNT(*) n FROM orders WHERE status='Paid'"), N],
  ['Stock sold (exactly once per order)', 10000 - t.count("SELECT stock n FROM items WHERE id='ria-d2'"), N],
];
console.log(`\n${N} customers, ${drops} dropped responses retried, duplicate webhooks on, ${((Date.now() - started) / 1000).toFixed(1)} s\n`);
let ok = true;
for (const [name, got, want] of checks) {
  const pass = got === want; ok &&= pass;
  console.log(`${pass ? 'PASS' : 'FAIL'}  ${name}: ${got} (expected ${want})`);
}
console.log(ok ? `\nQ1 holds: exactly one order and one charge for every one of ${N} customers.` : '\nQ1 VIOLATED');
await t.stop();
process.exit(ok ? 0 : 1);
