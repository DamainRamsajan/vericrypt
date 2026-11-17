// Contract tests: one test per rule in arc42 5.4 (PlaceOrder), 5.5 (MarkPaid) and 5.6 (HandleWebhook).
import test from 'node:test';
import assert from 'node:assert/strict';
import { startApp } from '../support/helpers.js';

test('PlaceOrder_same_key_twice_creates_one_order', async () => {
  const t = await startApp();
  try {
    const key = t.key();
    const a = await t.placeOrder({}, key), b = await t.placeOrder({}, key);
    assert.equal(a.status, 201); assert.equal(b.status, 200);
    assert.equal(b.body.duplicate, true);
    assert.equal(a.body.orderId, b.body.orderId);                                   // DuplicateRequest returns the original order
    assert.equal(t.count('SELECT COUNT(*) n FROM orders'), 1);
    assert.equal(t.count('SELECT COUNT(*) n FROM payments'), 1);
    assert.equal(a.body.paymentUrl, b.body.paymentUrl);
  } finally { await t.stop(); }
});

test('PlaceOrder_error_modes_VendorClosed_and_ItemUnavailable', async () => {
  const t = await startApp();
  try {
    await t.call('POST', '/api/vendor/stock', { body: { itemId: 'ria-d1', quantity: 1 }, headers: t.vendorH() });
    const short = await t.placeOrder({ items: [{ itemId: 'ria-d1', qty: 2 }] });
    assert.equal(short.status, 409); assert.equal(short.body.error, 'ItemUnavailable');
    assert.equal(t.count('SELECT COUNT(*) n FROM orders'), 0);                       // nothing stored
    assert.equal(t.count("SELECT stock n FROM items WHERE id='ria-d1'"), 1);         // stock untouched
    await t.call('POST', '/api/vendor/open', { body: { open: false }, headers: t.vendorH() });
    const closed = await t.placeOrder({ items: [{ itemId: 'ria-d2', qty: 1 }] });
    assert.equal(closed.status, 409); assert.equal(closed.body.error, 'VendorClosed');
  } finally { await t.stop(); }
});

test('PlaceOrder_rejects_empty_cart_and_missing_key', async () => {
  const t = await startApp();
  try {
    assert.equal((await t.placeOrder({ items: [] })).status, 400);
    const noKey = await t.call('POST', '/api/orders', { body: t.cart() });
    assert.equal(noKey.status, 400); assert.equal(noKey.body.error, 'InvalidRequest');
  } finally { await t.stop(); }
});

test('PlaceOrder_failed_validation_does_not_consume_the_key', async () => {
  const t = await startApp();
  try {
    const key = t.key();
    assert.equal((await t.placeOrder({ items: [] }, key)).status, 400);
    assert.equal((await t.placeOrder({}, key)).status, 201);
  } finally { await t.stop(); }
});

test('PlaceOrder_order_ids_are_never_reused', async () => {
  const t = await startApp();
  try {
    const ids = new Set();
    for (let i = 0; i < 40; i++) ids.add((await t.placeOrder({ items: [{ itemId: 'ria-d2', qty: 1 }] })).body.orderId);
    assert.equal(ids.size, 40);
  } finally { await t.stop(); }
});

test('MarkPaid_called_twice_changes_state_once', async () => {
  const t = await startApp();
  try {
    const o = await t.placeOrder(); await t.pay(o); await t.settle();
    assert.equal(await t.status(o.body.orderId), 'Paid');
    const again = t.app.lifecycle.markPaid(o.body.orderId, t.refOf(o), 'test');
    assert.deepEqual({ changed: again.changed, note: again.note }, { changed: false, note: 'AlreadyPaid' });
    assert.equal(t.count("SELECT COUNT(*) n FROM events WHERE type='order.paid'"), 1);
  } finally { await t.stop(); }
});

test('MarkPaid_never_marks_a_failed_order_paid', async () => {
  const t = await startApp();
  try {
    const o = await t.placeOrder(); await t.pay(o, 'failed'); await t.settle();
    assert.equal(await t.status(o.body.orderId), 'Failed');
    assert.throws(() => t.app.lifecycle.markPaid(o.body.orderId, t.refOf(o), 'test'), { code: 'PaymentMismatch' });
    assert.equal(await t.status(o.body.orderId), 'Failed');
  } finally { await t.stop(); }
});

test('MarkPaid_error_modes_UnknownOrder_and_PaymentMismatch', async () => {
  const t = await startApp();
  try {
    const a = await t.placeOrder(), b = await t.placeOrder();
    assert.throws(() => t.app.lifecycle.markPaid('no-such-order', t.refOf(a), 'test'), { code: 'UnknownOrder' });
    assert.throws(() => t.app.lifecycle.markPaid(a.body.orderId, t.refOf(b), 'test'), { code: 'PaymentMismatch' });
    assert.equal(await t.status(a.body.orderId), 'PendingPayment');
  } finally { await t.stop(); }
});

test('HandleWebhook_replayed_event_changes_state_once', async () => {
  const t = await startApp();
  try {
    await t.mode('duplicate');                                                      // the provider sends every event twice
    const o = await t.placeOrder(); await t.pay(o); await t.settle();
    assert.equal(await t.status(o.body.orderId), 'Paid');
    assert.equal(t.count("SELECT COUNT(*) n FROM jobs WHERE kind='confirm'"), 1);    // the replay was recognised at the door
    assert.equal(t.count("SELECT COUNT(*) n FROM events WHERE type='order.paid'"), 1);
  } finally { await t.stop(); }
});

test('HandleWebhook_different_event_ids_for_one_payment_still_change_state_once', async () => {
  const t = await startApp();
  try {
    const o = await t.placeOrder(); const ref = t.refOf(o);
    await t.webhook({ id: 'evt_a', type: 'payment.succeeded', paymentRef: ref });
    await t.webhook({ id: 'evt_b', type: 'payment.succeeded', paymentRef: ref });
    await t.settle();
    assert.equal(await t.status(o.body.orderId), 'Paid');
    assert.equal(t.count("SELECT COUNT(*) n FROM events WHERE type='order.paid'"), 1);
  } finally { await t.stop(); }
});

test('HandleWebhook_invalid_signature_is_rejected', async () => {
  const t = await startApp();
  try {
    const o = await t.placeOrder();
    const r = await t.webhook({ id: 'evt_x', type: 'payment.succeeded', paymentRef: t.refOf(o) }, { badSig: true });
    assert.equal(r.status, 401); assert.equal(r.body.error, 'InvalidSignature');
    assert.equal(t.count('SELECT COUNT(*) n FROM jobs'), 1);                         // only the reconciliation check, nothing from the bad event
    await t.settle(); assert.equal(await t.status(o.body.orderId), 'PendingPayment');
  } finally { await t.stop(); }
});

test('HandleWebhook_unknown_reference_is_parked', async () => {
  const t = await startApp();
  try {
    const r = await t.webhook({ id: 'evt_y', type: 'payment.succeeded', paymentRef: 'pay_does_not_exist' });
    assert.equal(r.status, 202); assert.equal(r.body.status, 'parked');
    assert.deepEqual(t.app.repo.parkedJobs().map((p) => p.error), ['UnknownReference']);
  } finally { await t.stop(); }
});

test('HandleWebhook_unknown_result_never_marks_failed', async () => {
  const t = await startApp();
  try {
    const o = await t.placeOrder();
    const r = await t.webhook({ id: 'evt_z', type: 'payment.pending', paymentRef: t.refOf(o) });
    assert.equal(r.status, 200); assert.equal(r.body.status, 'ignored');
    await t.settle();
    assert.equal(await t.status(o.body.orderId), 'PendingPayment');                  // not Failed
  } finally { await t.stop(); }
});
