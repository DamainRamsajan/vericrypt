// Notification Adapter (arc42 5.1): sends SMS through the gateway. Never touches the Database.
export class NotificationAdapter {
  constructor({ smsUrl, timeoutMs }) { Object.assign(this, { smsUrl, timeoutMs }); }
  async sendSms({ to, message, cid }) {
    try {
      const res = await fetch(this.smsUrl() + '/fake-sms/send', { method: 'POST', signal: AbortSignal.timeout(this.timeoutMs),
        headers: { 'content-type': 'application/json', 'x-correlation-id': cid || '' }, body: JSON.stringify({ to, message }) });
      return res.ok ? { ok: true } : { ok: false, error: 'gateway answered ' + res.status };
    } catch (e) { return { ok: false, error: e.message }; }
  }
}
