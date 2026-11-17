// Payment Adapter (arc42 5.1): hides the provider's API and handles its callbacks (ADR-0002, Q5).
// It never touches the Database. It reaches the Order Service only through the port it is given.
import crypto from 'node:crypto';

export class PaymentAdapter {
  constructor({ providerUrl, apiKey, webhookSecret, timeoutMs, logger }) { Object.assign(this, { providerUrl, apiKey, webhookSecret, timeoutMs, logger }); }

  async startPayment({ orderId, amountCents, cid }) {
    const res = await fetch(this.providerUrl() + '/fake-provider/api/payments', {
      method: 'POST', signal: AbortSignal.timeout(this.timeoutMs),
      headers: { 'content-type': 'application/json', 'x-api-key': this.apiKey, 'idempotency-key': 'order:' + orderId, 'x-correlation-id': cid || '' },
      body: JSON.stringify({ orderId, amountCents }),
    });
    if (!res.ok) throw new Error('provider answered ' + res.status);
    const j = await res.json();
    return { ref: j.ref, payUrl: j.payUrl };
  }

  // Returns 'paid', 'failed' or 'unknown'. Any trouble reaching the provider is 'unknown', never 'failed'.
  async checkPayment(ref) {
    try {
      const res = await fetch(this.providerUrl() + '/fake-provider/api/payments/' + encodeURIComponent(ref), { headers: { 'x-api-key': this.apiKey }, signal: AbortSignal.timeout(this.timeoutMs) });
      if (!res.ok) return 'unknown';
      const j = await res.json();
      return j.status === 'paid' ? 'paid' : j.status === 'failed' ? 'failed' : 'unknown';
    } catch { return 'unknown'; }
  }

  // Contract: HandleWebhook (arc42 5.6)
  handleWebhook(rawBody, signature, port, cid) {
    const expected = crypto.createHmac('sha256', this.webhookSecret).update(rawBody).digest('hex');
    const ok = typeof signature === 'string' && signature.length === expected.length && crypto.timingSafeEqual(Buffer.from(signature), Buffer.from(expected));
    if (!ok) { this.logger.log(cid, 'webhook.invalid_signature'); return { status: 401, body: { error: 'InvalidSignature' } }; }
    let ev; try { ev = JSON.parse(rawBody); } catch { return { status: 400, body: { error: 'InvalidRequest' } }; }
    if (!ev || !ev.id || !ev.paymentRef) return { status: 400, body: { error: 'InvalidRequest' } };
    if (!port.knowsPayment(ev.paymentRef)) {
      port.parkConfirmation(ev.id, ev.paymentRef, 'UnknownReference');
      this.logger.log(cid, 'webhook.unknown_reference', { eventId: ev.id });
      return { status: 202, body: { status: 'parked' } };
    }
    const outcome = ev.type === 'payment.succeeded' ? 'paid' : ev.type === 'payment.failed' ? 'failed' : 'unknown';
    if (outcome === 'unknown') { this.logger.log(cid, 'webhook.unclear_result_ignored', { eventId: ev.id }); return { status: 200, body: { status: 'ignored' } }; }  // unknown never becomes Failed
    try {
      const queued = port.enqueueConfirmation(ev.id, ev.paymentRef, outcome);
      this.logger.log(cid, queued ? 'webhook.queued' : 'webhook.replay_ignored', { eventId: ev.id, outcome });
      return { status: 200, body: { status: queued ? 'queued' : 'duplicate' } };
    } catch (e) {
      this.logger.log(cid, 'webhook.unavailable', { error: e.message });
      return { status: 503, body: { error: 'OrderServiceUnavailable' } };    // the provider will retry
    }
  }
}
