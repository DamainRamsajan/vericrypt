// Order Lifecycle (arc42 5.3): moves orders through their statuses and enforces the contracts in arc42 5.4 and 5.5.
import { randomUUID } from 'node:crypto';
import { DomainError } from '../errors.js';
import { IdempotencyGuard } from './idempotency-guard.js';

const PAID_OR_LATER = ['Paid', 'Ready', 'Collected'];

export class OrderLifecycle {
  constructor({ repo, guard, validator, queue, notifier, clock, logger }) {
    Object.assign(this, { repo, guard, validator, queue, notifier, clock, logger });
  }
  #ev(orderId, type, cid, details) {
    this.repo.addEvent({ orderId, type, cid, at: this.clock.now(), details });
    this.logger.log(cid, type, { orderId, ...details });
  }

  // Contract: PlaceOrder (arc42 5.4)
  place(input, key, cid) {
    IdempotencyGuard.validate(key);
    const hit = this.guard.lookup(key);
    if (hit) { this.#ev(hit.orderId, 'order.duplicate_request', cid); return { ...hit, duplicate: true }; }  // DuplicateRequest returns the original order
    return this.repo.transaction(() => {
      const again = this.guard.lookup(key);
      if (again) return { ...again, duplicate: true };
      const v = this.validator.validate(input);
      for (const l of v.lines) if (!this.repo.decrementStock(l.itemId, l.qty)) throw new DomainError('ItemUnavailable', 'An item just sold out');
      const id = randomUUID(), now = this.clock.now();
      this.repo.insertOrder({ id, vendorId: v.vendor.id, customerName: v.customerName, phone: v.phone, totalCents: v.totalCents, lines: v.lines, now });
      this.#ev(id, 'order.placed', cid, { totalCents: v.totalCents });
      const result = { orderId: id, status: 'PendingPayment', totalCents: v.totalCents };
      this.guard.remember(key, 'PlaceOrder', result, now);
      return { ...result, duplicate: false };
    });
  }

  // Contract: MarkPaid (arc42 5.5). Called by the Payment Coordinator, never exposed to the public.
  markPaid(orderId, paymentRef, cid) {
    return this.repo.transaction(() => {
      const o = this.repo.getOrder(orderId);
      if (!o) throw new DomainError('UnknownOrder', 'No such order', 404);
      const p = this.repo.getPaymentByRef(paymentRef);
      if (!p || p.orderId !== orderId) throw new DomainError('PaymentMismatch', 'Payment does not belong to this order');
      if (PAID_OR_LATER.includes(o.status)) { this.#ev(orderId, 'order.paid_again_ignored', cid); return { ack: true, changed: false, note: 'AlreadyPaid' }; }
      if (o.status === 'Failed') throw new DomainError('PaymentMismatch', 'A failed order is never marked paid');
      this.repo.setStatus(orderId, ['PendingPayment'], 'Paid', this.clock.now());
      this.repo.setPaymentStatus(paymentRef, 'paid');
      this.#ev(orderId, 'order.paid', cid);
      return { ack: true, changed: true };
    });
  }

  markFailed(orderId, paymentRef, cid) {
    return this.repo.transaction(() => {
      const o = this.repo.getOrder(orderId);
      if (!o) throw new DomainError('UnknownOrder', 'No such order', 404);
      const p = this.repo.getPaymentByRef(paymentRef);
      if (!p || p.orderId !== orderId) throw new DomainError('PaymentMismatch', 'Payment does not belong to this order');
      if (o.status === 'Failed') return { ack: true, changed: false };
      if (PAID_OR_LATER.includes(o.status)) {            // conflicting news: never undo a paid order, flag it for review
        this.#ev(orderId, 'order.conflicting_failure_ignored', cid);
        throw new DomainError('PaymentMismatch', 'Failure reported for an order that is already paid');
      }
      this.repo.setStatus(orderId, ['PendingPayment'], 'Failed', this.clock.now());
      this.repo.setPaymentStatus(paymentRef, 'failed');
      this.repo.releaseStock(orderId);
      this.#ev(orderId, 'order.failed', cid);
      return { ack: true, changed: true };
    });
  }

  // The order is Ready as soon as this returns; the SMS is queued and can fail without affecting it (Q8).
  markReady(orderId, vendorId, cid) {
    return this.repo.transaction(() => {
      const o = this.repo.getOrder(orderId);
      if (!o || o.vendorId !== vendorId) throw new DomainError('UnknownOrder', 'No such order', 404);
      if (o.status === 'Ready' || o.status === 'Collected') return { ack: true, status: o.status };
      if (o.status !== 'Paid') throw new DomainError('NotPaid', 'Only a paid order can be marked ready');
      this.repo.setStatus(orderId, ['Paid'], 'Ready', this.clock.now());
      this.queue.enqueue('sms', `sms:${orderId}:ready`, { orderId });
      this.#ev(orderId, 'order.ready', cid);
      return { ack: true, status: 'Ready' };
    });
  }
  markCollected(orderId, vendorId, cid) {
    const o = this.repo.getOrder(orderId);
    if (!o || o.vendorId !== vendorId) throw new DomainError('UnknownOrder', 'No such order', 404);
    if (o.status === 'Collected') return { ack: true, status: 'Collected' };
    if (o.status !== 'Ready') throw new DomainError('NotReady', 'Only a ready order can be collected');
    this.repo.setStatus(orderId, ['Ready'], 'Collected', this.clock.now());
    this.#ev(orderId, 'order.collected', cid);
    return { ack: true, status: 'Collected' };
  }

  status(orderId) {
    const o = this.repo.getOrder(orderId);
    if (!o) throw new DomainError('UnknownOrder', 'No such order', 404);
    const p = this.repo.getPaymentByOrder(orderId);
    return { orderId: o.id, status: o.status, vendor: o.vendorName, totalCents: o.totalCents,
      items: o.items.map(i => ({ name: i.name, qty: i.qty })), paymentUrl: o.status === 'PendingPayment' && p ? p.payUrl : null };
  }

  // Queue handler for ready notices.
  async handleSms({ orderId }, job) {
    const o = this.repo.getOrder(orderId);
    if (!o) return {};
    const r = await this.notifier.sendSms({ to: o.phone, message: `Your order ${o.id.slice(0, 8)} from ${o.vendorName} is ready for pickup.`, cid: `job-${job.id}` });
    if (r.ok) { this.#ev(orderId, 'sms.sent', `job-${job.id}`); return {}; }
    return { retry: true, error: r.error };
  }
}
