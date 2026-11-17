// Payment Coordinator (arc42 5.3): starts payments, applies confirmations, settles unknown results.
export class PaymentCoordinator {
  constructor({ repo, adapter, queue, lifecycle, clock, config, logger }) { Object.assign(this, { repo, adapter, queue, lifecycle, clock, config, logger }); }

  // Idempotent: one payment per order. Returns the payment page URL, or null if the provider is unavailable.
  async startPayment(orderId, cid) {
    const existing = this.repo.getPaymentByOrder(orderId);
    if (existing) return existing.payUrl;
    const order = this.repo.getOrder(orderId);
    if (!order || order.status !== 'PendingPayment') return null;
    try {
      const r = await this.adapter.startPayment({ orderId, amountCents: order.totalCents, cid });
      this.repo.insertPayment({ ref: r.ref, orderId, payUrl: r.payUrl, now: this.clock.now() });
      const p = this.repo.getPaymentByOrder(orderId);
      this.queue.enqueue('check', 'check:' + p.ref, { paymentRef: p.ref }, this.config.reconcileAfterMs);   // settle it later if no result arrives
      this.logger.log(cid, 'payment.started', { orderId, paymentRef: p.ref });
      return p.payUrl;
    } catch (e) {
      this.logger.log(cid, 'payment.start_failed', { orderId, error: e.message });
      return null;
    }
  }

  // The port the Payment Adapter uses to talk to the Order Service without touching the Database (arc42 8, architecture test).
  get port() {
    return {
      knowsPayment: (ref) => !!this.repo.getPaymentByRef(ref),
      enqueueConfirmation: (eventId, paymentRef, outcome) => this.queue.enqueue('confirm', 'evt:' + eventId, { eventId, paymentRef, outcome }),
      parkConfirmation: (eventId, paymentRef, reason) => this.queue.park('confirm', 'evt:' + eventId, { eventId, paymentRef }, reason),
    };
  }

  #apply(p, outcome, cid) {
    if (outcome === 'paid') return this.lifecycle.markPaid(p.orderId, p.ref, cid);
    if (outcome === 'failed') return this.lifecycle.markFailed(p.orderId, p.ref, cid);
  }

  // Queue handlers
  async handleConfirm({ paymentRef, outcome }, job) {
    const p = this.repo.getPaymentByRef(paymentRef);
    if (!p) throw new Error('UnknownReference');
    this.#apply(p, outcome, `job-${job.id}`);
    return {};
  }
  async handleCheck({ paymentRef }, job) {
    const p = this.repo.getPaymentByRef(paymentRef);
    if (!p || p.status !== 'initiated') return {};                     // already settled
    const outcome = await this.adapter.checkPayment(paymentRef);
    if (outcome === 'unknown') return { retry: true, error: 'payment result still unknown' };   // never treated as a failure
    this.#apply(p, outcome, `job-${job.id}`);
    return {};
  }
}
