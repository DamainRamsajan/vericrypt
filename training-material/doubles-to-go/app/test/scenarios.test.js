// Scenario tests: one test per quality scenario in arc42 section 10 that can be tested automatically.
import test from 'node:test';
import assert from 'node:assert/strict';
import { startApp } from '../support/helpers.js';

// A card-number scanner for Q3: 13 to 19 digits, a known card prefix, and a valid Luhn check.
export function findCardNumbers(text) {
  const luhn = (d) => { let s = 0, alt = false; for (let i = d.length - 1; i >= 0; i--) { let n = +d[i]; if (alt) { n *= 2; if (n > 9) n -= 9; } s += n; alt = !alt; } return s % 10 === 0; };
  const hits = [];
  for (const m of text.matchAll(/(?<!\d)(?:\d[ -]?){12,18}\d(?!\d)/g)) {
    const d = m[0].replace(/\D/g, '');
    if (d.length >= 13 && d.length <= 19 && /^[3-6]/.test(d) && luhn(d)) hits.push(d);
  }
  return hits;
}

test('Q3_scanner_detects_a_known_card_number', () => {
  assert.deepEqual(findCardNumbers('card: 4242 4242 4242 4242 ok'), ['4242424242424242']);
  assert.deepEqual(findCardNumbers('order 1800000000123 at 868-555-0142'), []);
});

test('Q3_scan_finds_no_stored_card_numbers', async () => {
  const t = await startApp();
  try {
    const o = await t.placeOrder(); await t.pay(o); await t.settle();                 // the customer types a real-looking test card on the provider's page
    assert.equal(t.app.provider().payments.get(t.refOf(o)).last4, '4242');            // the provider saw it...
    let all = t.lines.join('\n');
    for (const { name } of t.app.db.prepare("SELECT name FROM sqlite_master WHERE type='table'").all())
      all += '\n' + JSON.stringify(t.app.db.prepare(`SELECT * FROM ${name}`).all());
    assert.deepEqual(findCardNumbers(all), []);                                       // ...but our database and logs never did
  } finally { await t.stop(); }
});

test('Q1_connection_drops_during_payment_leave_one_order_and_one_charge', async () => {
  const t = await startApp();
  try {
    await t.mode('duplicate');                                                        // webhooks also arrive twice
    const N = 100; const paid = [];
    for (let i = 0; i < N; i++) {
      const key = t.key(), tries = 1 + (i % 3);                                       // the client never sees some responses and retries
      const calls = Array.from({ length: tries }, () => t.placeOrder({ items: [{ itemId: 'ria-d2', qty: 1 }] }, key));
      const rs = i % 5 === 0 ? await Promise.all(calls) : [await calls[0], ...(await Promise.all(calls.slice(1)))];  // some retries overlap
      assert.equal(new Set(rs.map((r) => r.body.orderId)).size, 1);
      await t.pay(rs[0]); if (i % 4 === 0) await t.pay(rs[0]);                        // some customers press Pay twice
      paid.push(rs[0].body.orderId);
    }
    await t.settle();
    assert.equal(t.count('SELECT COUNT(*) n FROM orders'), N);
    assert.equal(t.count('SELECT COUNT(*) n FROM (SELECT order_id FROM payments GROUP BY order_id HAVING COUNT(*)>1)'), 0);
    assert.equal(t.count("SELECT COUNT(*) n FROM (SELECT order_id FROM events WHERE type='order.paid' GROUP BY order_id HAVING COUNT(*)>1)"), 0);
    assert.equal(t.count("SELECT COUNT(*) n FROM orders WHERE status='Paid'"), N);
    assert.equal(t.count("SELECT stock n FROM items WHERE id='ria-d2'"), 100 - N);   // stock reduced exactly once per order
  } finally { await t.stop(); }
});

test('Q6_unknown_result_is_settled_by_reconciliation_and_never_marked_failed', async () => {
  const t = await startApp();
  try {
    await t.mode('drop');                                                             // the webhook is lost
    const o = await t.placeOrder(); await t.pay(o); await t.settle();
    assert.equal(await t.status(o.body.orderId), 'PendingPayment');
    await t.advance(t.config.reconcileAfterMs + 1);                                   // the scheduled check runs
    assert.equal(await t.status(o.body.orderId), 'Paid');
  } finally { await t.stop(); }
});

test('Q6_provider_outage_leaves_the_order_pending_then_parks_it_for_review', async () => {
  const t = await startApp();
  try {
    await t.mode('drop');
    const o = await t.placeOrder(); await t.pay(o);
    await t.mode('outage');                                                           // every check now comes back unknown
    await t.advance(t.config.reconcileAfterMs + 1);
    for (const wait of [1000, 2000, 4000, 8000]) await t.advance(wait + 1);           // backoff: 1, 2, 4, 8 seconds
    assert.equal(await t.status(o.body.orderId), 'PendingPayment');                   // never Failed
    assert.equal(t.app.repo.parkedJobs().length, 1);                                  // parked for manual reconciliation (R1)
  } finally { await t.stop(); }
});

test('Q6_a_short_outage_is_survived_by_backoff', async () => {
  const t = await startApp();
  try {
    await t.mode('drop');
    const o = await t.placeOrder(); await t.pay(o);
    await t.mode('outage'); await t.advance(t.config.reconcileAfterMs + 1);
    assert.equal(await t.status(o.body.orderId), 'PendingPayment');
    await t.mode('normal'); await t.advance(1001);
    assert.equal(await t.status(o.body.orderId), 'Paid');
    assert.equal(t.app.repo.parkedJobs().length, 0);
  } finally { await t.stop(); }
});

test('Q8_sms_gateway_down_does_not_block_the_ready_status', async () => {
  const t = await startApp();
  try {
    await t.smsUp(false);
    const o = await t.placeOrder(); await t.pay(o); await t.settle();
    const id = o.body.orderId;
    const ready = await t.call('POST', `/api/vendor/orders/${id}/ready`, { headers: t.vendorH() });
    assert.equal(ready.body.status, 'Ready');
    assert.equal(await t.status(id), 'Ready');                                        // visible at once
    await t.settle();
    assert.equal((await t.call('GET', '/fake-sms/outbox')).body.messages.length, 0);  // the SMS could not be sent yet
    assert.equal(await t.status(id), 'Ready');                                        // and that changed nothing
    await t.smsUp(true); await t.advance(1001);
    assert.equal((await t.call('GET', '/fake-sms/outbox')).body.messages.length, 1);  // sent after recovery
  } finally { await t.stop(); }
});

test('Q8_ready_sms_is_sent_once_even_if_ready_is_pressed_twice', async () => {
  const t = await startApp();
  try {
    const o = await t.placeOrder(); await t.pay(o); await t.settle();
    for (let i = 0; i < 2; i++) await t.call('POST', `/api/vendor/orders/${o.body.orderId}/ready`, { headers: t.vendorH() });
    await t.settle();
    assert.equal((await t.call('GET', '/fake-sms/outbox')).body.messages.length, 1);
  } finally { await t.stop(); }
});

test('Q9_correlation_id_links_the_request_the_logs_and_the_order_timeline', async () => {
  const t = await startApp();
  try {
    const o = await t.placeOrder({}, t.key(), { 'x-correlation-id': 'cid-test-123' });
    assert.equal(o.headers.get('x-correlation-id'), 'cid-test-123');
    assert.ok(t.lines.some((l) => JSON.parse(l).cid === 'cid-test-123' && JSON.parse(l).event === 'order.placed'));
    const s = await t.call('GET', `/api/support/orders/${o.body.orderId}`, { headers: t.supportH });
    assert.equal(s.status, 200);
    assert.equal(s.body.timeline[0].cid, 'cid-test-123');
  } finally { await t.stop(); }
});

test('Privacy_logs_and_support_view_never_contain_names_or_phone_numbers', async () => {
  const t = await startApp();
  try {
    const o = await t.placeOrder(); await t.pay(o); await t.settle();
    await t.call('POST', `/api/vendor/orders/${o.body.orderId}/ready`, { headers: t.vendorH() }); await t.settle();
    const s = await t.call('GET', `/api/support/orders/${o.body.orderId}`, { headers: t.supportH });
    const seen = t.lines.join('\n') + JSON.stringify(s.body);
    assert.ok(!seen.includes('555-0142') && !seen.includes('5550142') && !seen.includes('Test Customer'));
  } finally { await t.stop(); }
});

test('Q10_menu_changes_are_visible_within_60_seconds', async () => {
  const t = await startApp();
  try {
    await t.call('POST', '/api/vendor/menu', { body: { itemId: 'ria-d1', priceCents: 650 }, headers: t.vendorH() });
    const r = await t.call('GET', '/api/vendors');
    assert.equal(r.body.vendors.find((v) => v.id === 'ria').items.find((i) => i.id === 'ria-d1').priceCents, 650);
    assert.ok(Number(/max-age=(\d+)/.exec(r.headers.get('cache-control'))[1]) <= 60);   // caches may hold a menu for at most 30 s
  } finally { await t.stop(); }
});

test('Failed_payment_releases_the_reserved_stock', async () => {
  const t = await startApp();
  try {
    const before = t.count("SELECT stock n FROM items WHERE id='ria-d1'");
    const o = await t.placeOrder(); assert.equal(t.count("SELECT stock n FROM items WHERE id='ria-d1'"), before - 2);
    await t.pay(o, 'failed'); await t.settle();
    assert.equal(await t.status(o.body.orderId), 'Failed');
    assert.equal(t.count("SELECT stock n FROM items WHERE id='ria-d1'"), before);
  } finally { await t.stop(); }
});

test('Roles_a_vendor_cannot_touch_another_vendors_order_and_support_needs_its_own_token', async () => {
  const t = await startApp();
  try {
    const o = await t.placeOrder(); await t.pay(o); await t.settle();
    const other = await t.call('POST', `/api/vendor/orders/${o.body.orderId}/ready`, { headers: t.vendorH('ali') });
    assert.equal(other.status, 404);
    assert.equal((await t.call('GET', `/api/support/orders/${o.body.orderId}`, { headers: t.vendorH() })).status, 401);
    assert.equal((await t.call('GET', '/api/vendor/orders')).status, 401);
  } finally { await t.stop(); }
});
